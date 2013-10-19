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

use io;
use vga;
use mach;

static mut x: uint = 0;
static mut y: uint = 1;

static mut shifted: bool = false;

static mut ledstate: u8 = 0;

// Scan code set #1
static ScanCodeMapping: &'static str = "\
\x00\x1B1234567890-=\x08\tqwertyuiop[]\n?asdfghjkl;'`?\\zxcvbnm,./?*? ?????????????789-456+1230.?????";
static ScanCodeMappingShifted: &'static str = "\
\x00\x1B!@#$%^&*()_+\x08\tQWERTYUIOP{}\n?ASDFGHJKL:\"~?|ZXCVBNM<>??*? ?????????????789-456+1230.?????";


pub fn init() {
    // Put the keyboard into scan code set 1, ready for our mapping.
    /*
    kbcmdwait();
    io::outport(0x60, 0xF0u8);
    kbcmdwait();
    io::outport(0x60, 1u8);
    */

    mach::registerirq(1, irq);
}

fn kbcmdwait() {
    loop {
        let status: u8 = io::inport(0x64);
        if status & 0x2 == 0 { break; }
    }
}

fn kbdatawait() {
    loop {
        let status: u8 = io::inport(0x64);
        if status & 0x1 != 0 { break; }
    }
}

pub fn leds(state: u8) {
    unsafe { ledstate ^= state; }

    kbcmdwait();
    io::outport(0x60, 0xEDu8);
    kbcmdwait();
    unsafe { io::outport(0x60, ledstate); }
}

fn gotkey(scancode: u8) {
    // Sanity.
    if scancode > 0x58u8 { return; }

    let c: u8 = unsafe {
        if shifted {
            ScanCodeMappingShifted[scancode] as u8
        } else {
            ScanCodeMapping[scancode] as u8
        }
    };
    let s: &str = unsafe { core::intrinsics::transmute((&c, 1)) };

    unsafe {
        let off = vga::write(s, x, y, vga::White, vga::Black);

        // Update x/y
        y = off / 80;
        x = off % 80;

        if y >= vga::ROWS {
            y = vga::ROWS - 1;
        }
    }
}

fn irq() {
    // Check status, make sure a key is actually pending.
    let status: u8 = io::inport(0x64);
    if status & 0x1 == 0 {
        return;
    }

    // Get scancode.
    let scancode: u8 = io::inport(0x60);

    // Top bit set means 'key up'
    if scancode & 0x80 != 0 {
        let code = scancode & !0x80u8;
        match code {
            0x2A | 0x36 => unsafe { shifted = false },
            0x3A => leds(0b100), // Caps lock
            0x45 => leds(0b10), // Number lock
            0x46 => leds(0b1), // Scroll lock
            _ => gotkey(code)
        }
    } else {
        match scancode {
            0x2A | 0x36 => unsafe { shifted = true },
            _ => {}
        }
    }
}

