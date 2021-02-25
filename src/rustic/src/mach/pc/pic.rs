/*
 * Copyright (c) 2013 Matthew Iselin
 *
 * Permission to use, copy, modify, and distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
 * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 */

use core::sync::atomic;
use core::sync::atomic::Ordering;
use alloc::sync::Arc;
use alloc::boxed::Box;

use crate::util::sync::Spinlock;

use crate::arch::{Architecture, TrapHandler, ThreadSpawn, Threads};

use crate::mach::{IoPort, IrqController, IrqHandler, IrqRegister, Machine, Serial};

use crate::Kernel;

#[derive(Copy, Clone)]
struct PicIrqHandler {
    f: extern "Rust" fn(usize),
    level: bool,
}

pub static REMAP_BASE: usize = 0x20;

pub struct Pic {
    irqhandlers: [Option<PicIrqHandler>; 16],
    active_irqs: atomic::AtomicUsize
}

impl Pic {
    pub fn new() -> Pic {
        Pic{
            irqhandlers: [None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None],
            active_irqs: atomic::AtomicUsize::new(0)}
    }
}

impl IrqController for Kernel {
    fn init_irqs(&mut self) {
        self.outport(0x20, 0x11u8);
        self.outport(0xA0, 0x11u8);
        self.outport(0x21, REMAP_BASE as u8); // Remap to start at the remap base.
        self.outport(0xA1, (REMAP_BASE + 8) as u8);
        self.outport(0x21, 0x04u8);
        self.outport(0xA1, 0x02u8);
        self.outport(0x21, 0x01u8);
        self.outport(0xA1, 0x01u8);

        // Mask all, machine layer will call our enable() when an IRQ is registered.
        self.outport(0x21, 0xFFu8);
        self.outport(0xA1, 0xFFu8);

        // Spin up IRQ handling thread.
        // We have the IRQ handler set a flag and mask the IRQ - that's very
        // fast and easy to do in that context. Then this thread handles the
        // reading of that flag to trigger the actual IRQ handlers and unmask.
        self.spawn_thread(|| {
            loop {
                match Kernel::optional_kernel() {
                    Some(kernel_locked) => {
                        // Grab what we need with the lock held and then run the rest without.
                        let kernel = kernel_locked.lock().unwrap();
                        let active = kernel.mach.state.irq_ctlr.active_irqs.swap(0, Ordering::SeqCst);
                        let handlers = kernel.mach.state.irq_ctlr.irqhandlers.clone();
                        drop(kernel);

                        for irqnum in 0..16 {
                            if (active & (1 << irqnum)) == 0 {
                                continue;
                            }

                            // Call handlers - and run the EOI as well.
                            // We have to do this here now that we're actually actioning the IRQ.
                            for h in handlers.iter() {
                                match h {
                                    Some (handler) => {
                                        if handler.level == false {
                                            kernel_locked.lock().unwrap().eoi(irqnum);
                                        }

                                        (handler.f)(irqnum);

                                        if handler.level == true {
                                            kernel_locked.lock().unwrap().eoi(irqnum);
                                        }

                                        // Unmask the IRQ now that we've handled it.
                                        kernel_locked.lock().unwrap().enable_irq(irqnum);
                                    },
                                    None => {}
                                }
                            }
                        }

                        Kernel::reschedule(Arc::clone(&kernel_locked));
                    },
                    None => {
                        panic!("somehow managed to run a thread before the kernel is ready")
                    }
                };
            }
        });
    }

    fn enable_irq(&self, irq: usize) {
        if irq > 7 {
            let actual = irq - 8;
            let curr: u8 = self.inport(0xA1);
            let flag: u8 = 1 << actual;
            self.outport(0xA1, curr & !flag)
        } else {
            let curr: u8 = self.inport(0x21);
            let flag: u8 = 1 << irq;
            self.outport(0x21, curr & !flag)
        }
    }

    fn disable_irq(&self, irq: usize) {
        if irq > 7 {
            let actual = irq - 8;
            let curr: u8 = self.inport(0xA1);
            let flag: u8 = 1 << actual;
            self.outport(0xA1, curr | flag)
        } else {
            let curr: u8 = self.inport(0x21);
            let flag: u8 = 1 << irq;
            self.outport(0x21, curr | flag)
        }
    }

    fn eoi(&self, irq: usize) {
        if irq > 7 { self.outport(0xA0, 0x20u8); }
        self.outport(0x20, 0x20u8);
    }
}

impl IrqRegister for Kernel {
    fn register_irq(&mut self, irq: usize, handler: extern "Rust" fn(usize), level_trigger: bool) {
        let irqhandler = PicIrqHandler{f: handler, level: level_trigger};
        self.mach.state.irq_ctlr.irqhandlers[irq] = Some(irqhandler);

        self.register_trap(irq + REMAP_BASE, irq_stub);
        self.enable_irq(irq);
    }
}

impl TrapHandler for Kernel {
    fn trap(&mut self, num: usize) -> Option<extern "Rust" fn(usize)> {
        let irq_ctlr = &self.mach.state.irq_ctlr;

        let irqnum = num - REMAP_BASE;

        // Get status registers for master/slave
        self.outport(0x20, 0x0Bu8);
        self.outport(0xA0, 0x0Bu8);
        let slaveisr: u8 = self.inport(0xA0);
        let masterisr: u8 = self.inport(0x20);
        let isr: u16 = ((slaveisr as u16) << 8) | (masterisr as u16);

        // Spurious IRQ?
        if irqnum == 7 {
            if (isr & (1 << 7)) == 0 {
                self.serial_write("spurious IRQ 7\n");
                return None;
            }
        } else if irqnum == 15 {
            if (isr & (1 << 15)) == 0 {
                self.serial_write("spurious IRQ 15\n");
                self.eoi(7);
                return None;
            }
        }

        if (isr & (1 << irqnum)) == 0 {
            self.serial_write("IRQ stub called with no interrupt status\n");
            return None;
        }

        // Get the handler we need.
        // TODO: mark as active and let a thread handle the code so we don't
        // spend forever in the IRQ handler running code
        match irq_ctlr.irqhandlers[irqnum] {
            Some(_) => {
                irq_ctlr.active_irqs.fetch_or(1 << irqnum, Ordering::SeqCst);

                // Mask IRQ until we're done handling it.
                self.disable_irq(irqnum);
            },
            None => {
                // Unhandled IRQ, just send the EOI and hope all's well.
                self.serial_write("Unhandled IRQ");
                self.eoi(irqnum);
            }
        };

        None
    }
}

fn irq_stub(which: usize) {
    Kernel::kernel().lock().unwrap().trap(which);
}
