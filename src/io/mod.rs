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

// We can't use "a" or "d" as register constraints, otherwise we could
// just use real generics for this. Instead, a trait keeps the magic
// alive, in theory.
pub trait port {
    fn outport(port: u16, val: Self);
    fn inport(port: u16) -> Self;
}

impl port for u8 {
    fn outport(port: u16, val: u8) {
        unsafe {
            asm!(" \
                mov $0, %dx; \
                mov $1, %al; \
                outb %al, %dx" :: "r" (port), "r" (val) : "eax", "edx");
        }
    }

    fn inport(port: u16) -> u8 {
        unsafe {
            let mut val: u8 = 0;
            asm!(" \
                mov $1, %dx;
                inb %dx, %al;
                mov %al, $0" : "=r" (val) : "r" (port) : "eax", "edx");
            val
        }
    }
}

impl port for u16 {
    fn outport(port: u16, val: u16) {
        unsafe {
            asm!(" \
                mov $0, %dx; \
                mov $1, %ax; \
                outw %ax, %dx" :: "r" (port), "r" (val) : "eax", "edx");
        }
    }

    fn inport(port: u16) -> u16 {
        unsafe {
            let mut val: u16 = 0;
            asm!(" \
                mov $1, %dx;
                inw %dx, %ax;
                mov %ax, $0" : "=r" (val) : "r" (port) : "eax", "edx");
            val
        }
    }
}

impl port for u32 {
    fn outport(port: u16, val: u32) {
        unsafe {
            asm!(" \
                mov $0, %dx; \
                mov $1, %eax; \
                outl %eax, %dx" :: "r" (port), "r" (val) : "eax", "edx");
        }
    }

    fn inport(port: u16) -> u32 {
        unsafe {
            let mut val: u32 = 0;
            asm!(" \
                mov $1, %dx;
                inl %dx, %eax;
                mov %eax, $0" : "=r" (val) : "r" (port) : "eax", "edx");
            val
        }
    }
}

