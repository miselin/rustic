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

use crate::arch::{Architecture, TrapHandler};

use crate::mach;
use crate::mach::{IoPort, IrqController, Serial};

use crate::Kernel;

struct IrqHandler<'a> {
    f: &'a dyn mach::IrqHandler,
    level: bool,
}

impl<'a> IrqHandler<'a> {
    fn call(&'a self, irqnum: usize) {
        self.f.irq(irqnum);
    }
}

pub static RemapBase: usize = 0x20;

pub struct Pic<'a> {
    irqhandlers: [Option<IrqHandler<'a>>; 16],
}

impl<'a> Pic<'a> {
    pub fn new() -> Pic<'a> {
        Pic{irqhandlers: [None, None, None, None, None, None, None, None,
                          None, None, None, None, None, None, None, None]}
    }
}

impl<'a> IrqController<'a> for Kernel<'a> {
    fn init_irqs(&mut self) {
        self.outport(0x20, 0x11u8);
        self.outport(0xA0, 0x11u8);
        self.outport(0x21, RemapBase as u8); // Remap to start at the remap base.
        self.outport(0xA1, (RemapBase + 8) as u8);
        self.outport(0x21, 0x04u8);
        self.outport(0xA1, 0x02u8);
        self.outport(0x21, 0x01u8);
        self.outport(0xA1, 0x01u8);

        // Mask all, machine layer will call our enable() when an IRQ is registered.
        self.outport(0x21, 0xFFu8);
        self.outport(0xA1, 0xFFu8);
    }

    fn register_irq(&mut self, irq: usize, f: &'a dyn mach::IrqHandler, level_trigger: bool) {
        let irqhandler = IrqHandler{f: f, level: level_trigger};
        self.mach.state.irq_ctlr.irqhandlers[irq] = Some(irqhandler);

        self.register_trap(irq + RemapBase, irq_stub);
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

impl TrapHandler for Pic<'_> {
    fn trap(&mut self, num: usize) {
        /*
        let irqnum = num - RemapBase;

        // Get status registers for master/slave
        kernel().machine().outport(0x20, 0x0Bu8);
        kernel().machine().outport(0xA0, 0x0Bu8);
        let slaveisr: u8 = kernel().machine().inport(0xA0);
        let masterisr: u8 = kernel().machine().inport(0x20);
        let isr: u16 = ((slaveisr as u16) << 8) | (masterisr as u16);

        // Spurious IRQ?
        if irqnum == 7 {
            if (isr & (1 << 7)) == 0 {
                kernel().machine().serial_write("spurious IRQ 7\n");
                return;
            }
        } else if irqnum == 15 {
            if (isr & (1 << 15)) == 0 {
                kernel().machine().serial_write("spurious IRQ 15\n");
                self.eoi(7);
                return;
            }
        }

        if (isr & (1 << irqnum)) == 0 {
            kernel().machine().serial_write("IRQ stub called with no interrupt status");
            return;
        }

        // Get the handler we need.
        match self.irqhandlers[irqnum] {
            Some(ref handler) => {
                if handler.level == false {
                    self.eoi(irqnum);
                }

                handler.call(irqnum);

                if handler.level == true {
                    self.eoi(irqnum);
                }
            },
            None => {
                // Unhandled IRQ, just send the EOI and hope all's well.
                kernel().machine().serial_write("Unhandled IRQ");
                self.eoi(irqnum);
            }
        };
        */

        // todo
    }
}

fn irq_stub(which: usize) {
    // TODO: this happens in a different execution context and might be unsafe?
    // kernel_mut().machine_mut().state.irq_ctlr.trap(which)
}
