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

use core;
use serial;

use io;
use cpu;

type handlers = [handler, ..16];

struct handler {
    f: extern "Rust" fn(),
    set: bool,
    level: bool,
}

pub static RemapBase: int = 0x20;

static mut irqhandlers: *mut handlers = 0 as *mut handlers;

#[fixed_stack_segment]
pub fn init() {
    io::outport(0x20, 0x11u8);
    io::outport(0xA0, 0x11u8);
    io::outport(0x21, RemapBase as u8); // Remap to start at the remap base.
    io::outport(0xA1, (RemapBase + 8) as u8);
    io::outport(0x21, 0x04u8);
    io::outport(0xA1, 0x02u8);
    io::outport(0x21, 0x01u8);
    io::outport(0xA1, 0x01u8);

    // Mask all, machine layer will call our enable() when an IRQ is registered.
    io::outport(0x21, 0xFFu8);
    io::outport(0xA1, 0xFFu8);

    // Allocate space for our handler list.
    unsafe { irqhandlers = core::libc::malloc(192) as *mut handlers; }

    // Set handlers, set IRQ entries on the CPU.
    let mut i = 0;
    while i < 16 {
        unsafe { (*irqhandlers)[i].set = false; }
        cpu::registertrap(i + RemapBase, irq);
        i += 1;
    }
}

pub fn register(irq: int, f: extern "Rust" fn()) {
    // TODO: expose level-trigger Boolean
    unsafe {
        (*irqhandlers)[irq].f = f;
        (*irqhandlers)[irq].set = true;
        (*irqhandlers)[irq].level = true;
    }
}

pub fn enable(line: int) {
    if line > 7 {
        let actual = line - 8;
        let curr: u8 = io::inport(0xA1);
        io::outport(0xA1, curr & !((1 << actual) as u8))
    } else {
        let curr: u8 = io::inport(0x21);
        io::outport(0x21, curr & !((1 << line) as u8))
    }
}

pub fn disable(line: int) {
    if line > 7 {
        let actual = line - 8;
        let curr: u8 = io::inport(0xA1);
        io::outport(0xA1, curr | ((1 << actual) as u8))
    } else {
        let curr: u8 = io::inport(0x21);
        io::outport(0x21, curr | ((1 << line) as u8))
    }
}

fn eoi(n: uint) {
    if n > 7 { io::outport(0xA0, 0x20u8); }
    io::outport(0x20, 0x20u8);
}

pub fn irq(n: uint) {
    let irqnum = n - RemapBase as uint;

    // Get status registers for master/slave
    io::outport(0x20, 0x0Bu8);
    io::outport(0xA0, 0x0Bu8);
    let slaveisr: u8 = io::inport(0xA0);
    let masterisr: u8 = io::inport(0x20);
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
            eoi(7);
            return;
        }
    }

    if (isr & (1 << irqnum)) == 0 {
        serial::write("IRQ stub called with no interrupt status");
        return;
    }

    // Get the handler we need.
    let h: handler = unsafe { (*irqhandlers)[irqnum] };
    if h.set == true {
        // Edge triggered IRQs need to be ACKed before the handler.
        if h.level == false {
            eoi(irqnum);
        }

        // Handle!
        let f = h.f;
        f();

        // ACK level triggered IRQ.
        if h.level == true {
            eoi(irqnum);
        }
    } else {
        // Unhandled IRQ, just send the EOI and hope all's well.
        serial::write("Unhandled IRQ");
        eoi(irqnum);
    }
}

