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

mod gdt;

// External variable in assembly code (not actually a function)
extern { fn tls_emul_segment(); }

pub fn init() {
    // Configure and load GDT
    let systemgdt = gdt::table::init();
    systemgdt.entry(0, 0, 0, 0, 0); // 0x00 - NULL
    systemgdt.entry(1, 0, 0xFFFFFFFF, 0x98, 0xCF); // 0x08 - Kernel Code
    systemgdt.entry(2, 0, 0xFFFFFFFF, 0x92, 0xCF); // 0x10 - Kernel Data
    systemgdt.entry(3, 0, 0xFFFFFFFF, 0xF8, 0xCF); // 0x18 - User Code
    systemgdt.entry(4, 0, 0xFFFFFFFF, 0xF2, 0xCF); // 0x20 - User Data
    systemgdt.entry(5, tls_emul_segment as uint, 0xFFFFFFFF, 0x92, 0xCF); // 0x28 - TLS emulation (for stack switching support)
    systemgdt.load(0x08, 0x10, 0x28);
}

pub fn setirqs(state: bool) {
    if(state == true) {
        unsafe { asm!("sti") }
    } else {
        unsafe { asm!("cli") }
    }
}

pub fn waitforinterrupt() {
    setirqs(true);
    unsafe { asm!("hlt") }
}
