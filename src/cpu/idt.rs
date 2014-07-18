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

use util;

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
    f: extern "Rust" fn(n: uint),
    set: bool,
}

struct IdtTableMetadata {
    reg: *mut IdtRegister,
    table: *mut IdtTable,
    handlers: *mut InterruptHandlerList,
}

impl IdtRegister {
    pub fn new(idt: *const IdtTable) -> IdtRegister {
        IdtRegister {
            addr: idt,
            limit: (core::mem::size_of::<IdtTable>() + 1) as u16,
        }
    }
}

impl IdtEntry {
    pub fn new(handler: uint, sel: u16, flags: u8) -> IdtEntry {
        IdtEntry {
            handler_low: (handler & 0xFFFF) as u16,
            selector: sel,
            always0: 0,
            flags: flags | 0x60,
            handler_high: ((handler >> 16) & 0xFFFF) as u16,
        }
    }
}

static mut SystemIDT: IdtTableMetadata = IdtTableMetadata {
    table: 0 as *mut IdtTable,
    reg: 0 as *mut IdtRegister,
    handlers: 0 as *mut InterruptHandlerList,
};

fn entry(index: uint, handler: uint, sel: u16, flags: u8) {
    unsafe {
        (*SystemIDT.table)[index] = IdtEntry::new(handler, sel, flags)
    }
}

pub fn register(index: uint, handler: extern "Rust" fn(n: uint)) {
    unsafe {
        (*SystemIDT.handlers)[index].f = handler;
        (*SystemIDT.handlers)[index].set = true;
    }
}

pub fn init() {
    unsafe {
        SystemIDT.table = util::mem::allocate();
        SystemIDT.reg = util::mem::allocate();
        SystemIDT.handlers = util::mem::allocate();
        *SystemIDT.reg = IdtRegister::new(SystemIDT.table as *const IdtTable);
    }

    // Load default IDT entries, that generally shouldn't ever be changed.
    let mut i = 0;
    let mut base = isrs_base as uint;
    while i < 256 {
        entry(i, base, 0x08u16, 0x8E);
        unsafe { (*SystemIDT.handlers)[i].set = false; }
        base += ISR_STUB_LENGTH;
        i += 1;
    }

    unsafe { set_isr_handler(isr_rustentry as uint) };
}

#[no_mangle]
pub extern "C" fn isr_rustentry(which: uint) {
    // Entry point for IRQ - find if we have a handler configured or not.
    let x = unsafe { (*SystemIDT.handlers)[which] };
    if x.set == true {
        let f = x.f;
        f(which);
    }
}

pub fn load() {
    unsafe { asm!("lidt ($0)" :: "r" (SystemIDT.reg)); }
}

