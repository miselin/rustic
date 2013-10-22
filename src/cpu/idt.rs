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
use core::intrinsics::size_of;

type idttable = [idtentry, ..256];

// One handler per interrupt line.
type handlers = [handler, ..256];

// Base for all our IRQ handling.
#[allow(ctypes)]
extern "C" { fn isrs_base(); fn set_isr_handler(f: uint); }

// Size of the interrupt stub, so we can create our initial IDT easily.
static ISR_STUB_LENGTH: uint = 10;

#[packed]
struct idtreg {
    limit: u16,
    addr: *idttable,
}

#[packed]
struct idtentry {
    handler_low: u16,
    selector: u16,
    always0: u8,
    flags: u8,
    handler_high: u16,
}

struct handler {
    f: extern "Rust" fn(n: uint),
    set: bool,
}

struct table {
    reg: *mut idtreg,
    table: *mut idttable,
    handlers: *mut handlers,
}

impl idtreg {
    pub fn new(idt: *idttable) -> idtreg {
        idtreg {
            addr: idt,
            limit: (size_of::<idttable>() + 1) as u16,
        }
    }
}

impl idtentry {
    pub fn new(handler: uint, sel: u16, flags: u8) -> idtentry {
        idtentry {
            handler_low: (handler & 0xFFFF) as u16,
            selector: sel,
            always0: 0,
            flags: flags | 0x60,
            handler_high: ((handler >> 16) & 0xFFFF) as u16,
        }
    }
}

static mut systemidt: table = table {
    table: 0 as *mut idttable,
    reg: 0 as *mut idtreg,
    handlers: 0 as *mut handlers,
};

fn entry(index: int, handler: uint, sel: u16, flags: u8) {
    unsafe {
        (*systemidt.table)[index] = idtentry::new(handler, sel, flags)
    }
}

pub fn register(index: int, handler: extern "Rust" fn(n: uint)) {
    unsafe {
        (*systemidt.handlers)[index].f = handler;
        (*systemidt.handlers)[index].set = true;
    }
}

#[fixed_stack_segment]
pub fn init() {
    unsafe {
        systemidt.table = core::libc::malloc(2048) as *mut idttable;
        systemidt.reg = core::libc::malloc(6) as *mut idtreg;
        systemidt.handlers = core::libc::malloc(2048) as *mut handlers;
        *systemidt.reg = idtreg::new(systemidt.table as *idttable);
    }

    // Load default IDT entries, that generally shouldn't ever be changed.
    let mut i = 0;
    let mut base = isrs_base as uint;
    while i < 256 {
        entry(i, base, 0x08u16, 0x8E);
        unsafe { (*systemidt.handlers)[i].set = false; }
        base += ISR_STUB_LENGTH;
        i += 1;
    }

    unsafe { set_isr_handler(isr_rustentry as uint) };
}

#[no_mangle]
#[fixed_stack_segment]
pub extern "C" fn isr_rustentry(which: uint) {
    // Entry point for IRQ - find if we have a handler configured or not.
    let x: handler = unsafe { (*systemidt.handlers)[which] };
    if x.set == true {
        let f = x.f;
        f(which);
    }
}

pub fn load() {
    unsafe { asm!("lidt ($0)" :: "r" (systemidt.reg)); }
}

