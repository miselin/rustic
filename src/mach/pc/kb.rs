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

use core::str::StrSlice;

use vga;
use mach;
use mach::IoPort;

use machine;

static KEYBOARD_IRQ: uint = 1;
static KEYBOARD_CMD: u16 = 0x60;
static KEYBOARD_DATA: u16 = 0x64;

// Scan code set #1
static ScanCodeMapping: &'static str = "\
\x00\x1B1234567890-=\x08\tqwertyuiop[]\n?asdfghjkl;'`?\\zxcvbnm,./?*? ?????????????789-456+1230.?????";
static ScanCodeMappingShifted: &'static str = "\
\x00\x1B!@#$%^&*()_+\x08\tQWERTYUIOP{}\n?ASDFGHJKL:\"~?|ZXCVBNM<>??*? ?????????????789-456+1230.?????";

pub struct PS2Keyboard {
    x: uint,
    y: uint,
    shifted: bool,
    ledstate: u8,
}

impl PS2Keyboard {
    pub fn new() -> PS2Keyboard {
        PS2Keyboard{x: 0, y: 1, shifted: false, ledstate: 0u8}
    }

    pub fn init() -> PS2Keyboard {
        let state = PS2Keyboard::new();

        // Put the keyboard into scan code set 1, ready for our mapping.
        /*
        kbcmdwait();
        machine().outport(0x60, 0xF0u8);
        kbcmdwait();
        machine().outport(0x60, 1u8);
        */

        state
    }

    pub fn irq_num() -> uint {
        KEYBOARD_IRQ
    }

    fn kbcmdwait(&self) {
        loop {
            let status: u8 = machine().inport(KEYBOARD_DATA);
            if status & 0x2 == 0 { break; }
        }
    }

    fn kbdatawait(&self) {
        loop {
            let status: u8 = machine().inport(KEYBOARD_DATA);
            if status & 0x1 != 0 { break; }
        }
    }

    pub fn leds(&mut self, state: u8) {
        self.ledstate ^= state;
        self.kbcmdwait();
        machine().outport(KEYBOARD_CMD, 0xEDu8);
        self.kbcmdwait();
        machine().outport(KEYBOARD_CMD, self.ledstate);
    }

    fn gotkey(&mut self, scancode: uint) {
        // Sanity.
        if scancode > 0x58 { return; }

        let c = match self.shifted {
            true => ScanCodeMappingShifted.char_at(scancode),
            false => ScanCodeMapping.char_at(scancode)
        };

        let off = vga::write_char(c, self.x, self.y, vga::White, vga::Black);

        // Update x/y
        self.y = off / 80;
        self.x = off % 80;

        if self.y >= vga::ROWS {
            self.y = vga::ROWS - 1;
        }
    }
}

impl mach::IrqHandler for PS2Keyboard {
    fn irq(&mut self, _: uint) {
        // Check status, make sure a key is actually pending.
        let status: u8 = machine().inport(KEYBOARD_DATA);
        if status & 0x1 == 0 {
            return;
        }

        // Get scancode.
        let scancode: u8 = machine().inport(KEYBOARD_CMD);

        // Top bit set means 'key up'
        if scancode & 0x80 != 0 {
            let code = scancode & !0x80u8;
            match code {
                0x2A | 0x36 => { self.shifted = false },
                0x3A => self.leds(0b100), // Caps lock
                0x45 => self.leds(0b10), // Number lock
                0x46 => self.leds(0b1), // Scroll lock
                _ => self.gotkey(code as uint)
            }
        } else {
            match scancode {
                0x2A | 0x36 => { self.shifted = true },
                _ => {}
            }
        }
    }
}
