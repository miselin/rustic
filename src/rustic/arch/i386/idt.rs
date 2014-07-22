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

use std;

use architecture;

type IdtTable = [IdtEntry, ..256];

// One handler per interrupt line.
type InterruptHandlerList = [InterruptHandler, ..256];

// Base for all our IRQ handling.
#[allow(ctypes)]
extern "C" { fn isrs_base(); fn set_isr_handler(f: uint); }

// Size of the interrupt stub, so we can create our initial IDT easily.
static ISR_STUB_LENGTH: uint = 10;

#[packed]
struct IdtRegister {
    limit: u16,
    addr: *const IdtTable,
}

#[packed]
struct IdtEntry {
    handler_low: u16,
    selector: u16,
    always0: u8,
    flags: u8,
    handler_high: u16,
}

struct InterruptHandler {
    f: extern "Rust" fn(uint),
}

pub struct Idt {
    table: [IdtEntry, ..256],
    handlers: [InterruptHandler, ..256],
    reg: IdtRegister,
}

impl Idt {
    pub fn new() -> Idt {
        Idt{table: [IdtEntry::new(), ..256],
            handlers: [InterruptHandler::new(default_trap), ..256],
            reg: IdtRegister::new(0 as *const IdtTable)}
    }

    pub fn init(&mut self) {
        let mut base = isrs_base as uint;
        for i in range(0, 256) {
            self.entry(i, base, 0x08u16, 0x8E);
            base += ISR_STUB_LENGTH;
        }

        self.reg = IdtRegister::new(&self.table as *const IdtTable);

        self.load();

        unsafe { set_isr_handler(isr_rustentry as uint) };
    }

    pub fn register(&mut self, index: uint, handler: extern "Rust" fn(uint)) {
        self.handlers[index] = InterruptHandler::new(handler);
    }

    fn load(&self) {
        unsafe { asm!("lidt ($0)" :: "r" (&self.reg)); }
    }

    fn entry(&mut self, index: uint, handler: uint, sel: u16, flags: u8) {
        self.table[index] = IdtEntry::create(handler, sel, flags)
    }

    fn trap(&self, which: uint) {
        let f = self.handlers[which].f;
        f(which);
    }
}

impl IdtRegister {
    pub fn new(idt: *const IdtTable) -> IdtRegister {
        IdtRegister {
            addr: idt,
            limit: (std::mem::size_of::<IdtTable>() + 1) as u16,
        }
    }
}

impl IdtEntry {
    fn new() -> IdtEntry {
        IdtEntry{handler_low: 0, selector: 0, always0: 0, flags: 0, handler_high: 0}
    }

    fn create(handler: uint, sel: u16, flags: u8) -> IdtEntry {
        IdtEntry {
            handler_low: (handler & 0xFFFF) as u16,
            selector: sel,
            always0: 0,
            flags: flags | 0x60,
            handler_high: ((handler >> 16) & 0xFFFF) as u16,
        }
    }
}

impl InterruptHandler {
    pub fn new(handler: extern "Rust" fn(uint)) -> InterruptHandler {
        InterruptHandler{f: handler}
    }
}

#[no_mangle]
pub extern "C" fn isr_rustentry(which: uint) {
    // Entry point for IRQ - find if we have a handler configured or not.
    architecture().state.idt.trap(which)
}

fn default_trap(_: uint) {
    // no-op
}
