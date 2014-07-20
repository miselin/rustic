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

use core::option::{Option, Some, None};

use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::RefCell;

use arch::{Architecture, TrapHandler};

use serial;

use mach;

use mach::IoPort;

use architecture;
use machine;

struct IrqHandler {
    f: Option<Rc<RefCell<Box<mach::IrqHandler>>>>,
    level: bool,
}

pub static RemapBase: uint = 0x20;

pub struct Pic {
    irqhandlers: [Option<IrqHandler>, ..16],
}

impl IrqHandler {
    fn new() -> IrqHandler {
        IrqHandler{f: None, level: false}
    }
}

impl Pic {
    pub fn new() -> Pic {
        Pic{irqhandlers: [None, None, None, None, None, None, None, None,
                          None, None, None, None, None, None, None, None]}
    }

    pub fn init() -> Pic {
        let result = Pic{irqhandlers: [None, None, None, None, None, None, None,
                                       None, None, None, None, None, None, None,
                                       None, None]};

        machine().outport(0x20, 0x11u8);
        machine().outport(0xA0, 0x11u8);
        machine().outport(0x21, RemapBase as u8); // Remap to start at the remap base.
        machine().outport(0xA1, (RemapBase + 8) as u8);
        machine().outport(0x21, 0x04u8);
        machine().outport(0xA1, 0x02u8);
        machine().outport(0x21, 0x01u8);
        machine().outport(0xA1, 0x01u8);

        // Mask all, machine layer will call our enable() when an IRQ is registered.
        machine().outport(0x21, 0xFFu8);
        machine().outport(0xA1, 0xFFu8);

        result
    }

    pub fn remap_base() -> uint {
        return RemapBase;
    }

    pub fn irq_count() -> uint {
        return 16;
    }

    pub fn register(&mut self, irq: uint, f: Rc<RefCell<Box<mach::IrqHandler>>>, level: bool) {
        let irqhandler = IrqHandler{f: Some(f), level: level};
        self.irqhandlers[irq] = Some(irqhandler);

        architecture().register_trap(irq + RemapBase, irq_stub);
    }

    pub fn enable(&self, line: uint) {
        if line > 7 {
            let actual = line - 8;
            let curr: u8 = machine().inport(0xA1);
            let flag: u8 = 1 << actual;
            machine().outport(0xA1, curr & !flag)
        } else {
            let curr: u8 = machine().inport(0x21);
            let flag: u8 = 1 << line;
            machine().outport(0x21, curr & !flag)
        }
    }

    pub fn disable(&self, line: uint) {
        if line > 7 {
            let actual = line - 8;
            let curr: u8 = machine().inport(0xA1);
            let flag: u8 = 1 << actual;
            machine().outport(0xA1, curr | flag)
        } else {
            let curr: u8 = machine().inport(0x21);
            let flag: u8 = 1 << line;
            machine().outport(0x21, curr | flag)
        }
    }

    fn eoi(&self, n: uint) {
        if n > 7 { machine().outport(0xA0, 0x20u8); }
        machine().outport(0x20, 0x20u8);
    }
}

impl TrapHandler for Pic {
    fn trap(&mut self, num: uint) {
        let irqnum = num - RemapBase;

        // Get status registers for master/slave
        machine().outport(0x20, 0x0Bu8);
        machine().outport(0xA0, 0x0Bu8);
        let slaveisr: u8 = machine().inport(0xA0);
        let masterisr: u8 = machine().inport(0x20);
        let isr: u16 = (slaveisr as u16 << 8) | masterisr as u16;

        // Spurious IRQ?
        if irqnum == 7 {
            if (isr & (1 << 7)) == 0 {
                serial::write("spurious IRQ 7\n");
                return;
            }
        } else if irqnum == 15 {
            if (isr & (1 << 15)) == 0 {
                serial::write("spurious IRQ 15\n");
                self.eoi(7);
                return;
            }
        }

        if (isr & (1 << irqnum)) == 0 {
            serial::write("IRQ stub called with no interrupt status");
            return;
        }

        // Get the handler we need.
        match self.irqhandlers[irqnum] {
            Some(ref handler) => {
                if handler.level == false {
                    self.eoi(irqnum);
                }

                match handler.f {
                    Some(ref f) => {
                        let handler = f.try_borrow_mut();
                        match handler {
                            Some(mut x) => x.irq(irqnum),
                            None => {
                                serial::write("!! dropping IRQ because borrowing handler out of RefCell failed!")
                            },
                        }
                    },
                    None => {},
                };

                if handler.level == true {
                    self.eoi(irqnum);
                }
            },
            None => {
                // Unhandled IRQ, just send the EOI and hope all's well.
                serial::write("Unhandled IRQ");
                self.eoi(irqnum);
            }
        };
    }
}

fn irq_stub(which: uint) {
    machine().state.irq_ctlr.trap(which)
}
