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

use crate::mach;
use crate::mach::{Keyboard, IoPort};
use crate::Kernel;

static KEYBOARD_IRQ: usize = 1;
static KEYBOARD_CMD: u16 = 0x60;
static KEYBOARD_DATA: u16 = 0x64;

// Scan code set #1
static ScanCodeMapping: &'static str = "\
\x00\x1B1234567890-=\x08\tqwertyuiop[]\n?asdfghjkl;'`?\\zxcvbnm,./?*? ?????????????789-456+1230.?????";
static ScanCodeMappingShifted: &'static str = "\
\x00\x1B!@#$%^&*()_+\x08\tQWERTYUIOP{}\n?ASDFGHJKL:\"~?|ZXCVBNM<>??*? ?????????????789-456+1230.?????";

pub struct PS2Keyboard {
    shifted: bool,
    ledstate: u8,
}

impl PS2Keyboard {
    pub fn new() -> PS2Keyboard {
        PS2Keyboard{shifted: false, ledstate: 0u8}
    }

    pub fn irq_num() -> usize {
        KEYBOARD_IRQ
    }

    fn gotkey(&mut self, scancode: usize) {
        // Sanity.
        if scancode > 0x58 { return; }

        let _ = match self.shifted {
            true => ScanCodeMappingShifted,
            false => ScanCodeMapping
        }.chars().nth(scancode).unwrap();

        // TODO: write the key into a queue that can be read out of!
    }

    fn kbcmdwait<'a, 'b>(&self, kernel: &'b Kernel<'a>) {
        loop {
            let status: u8 = kernel.inport(KEYBOARD_DATA);
            if status & 0x2 == 0 { break; }
        }
    }

    fn kbdatawait<'a, 'b>(&self, kernel: &'b Kernel<'a>) {
        loop {
            let status: u8 = kernel.inport(KEYBOARD_DATA);
            if status & 0x1 != 0 { break; }
        }
    }
}

impl<'a> Keyboard for Kernel<'a> {
    fn kb_init(&mut self) {
        // Put the keyboard into scan code set 1, ready for our mapping.
        self.mach.state.keyboard.kbcmdwait(self);
        self.outport(0x60, 0xF0u8);
        self.mach.state.keyboard.kbcmdwait(self);
        self.outport(0x60, 1u8);
    }

    fn kb_leds(&mut self, state: u8) {
        self.mach.state.keyboard.ledstate ^= state;
        self.mach.state.keyboard.kbcmdwait(self);
        self.outport(KEYBOARD_CMD, 0xEDu8);
        self.mach.state.keyboard.kbcmdwait(self);
        self.outport(KEYBOARD_CMD, self.mach.state.keyboard.ledstate);
    }
}

impl mach::IrqHandler for PS2Keyboard {
    fn irq(&self, _: usize) {
        // TODO: figure out the mutable/immutable thing

        /*
        // Check status, make sure a key is actually pending.
        let status: u8 = self.inport(KEYBOARD_DATA);
        if status & 0x1 == 0 {
            return;
        }

        // Get scancode.
        let scancode: u8 = self.inport(KEYBOARD_CMD);

        // Top bit set means 'key up'
        if scancode & 0x80 != 0 {
            let code = scancode & !0x80u8;
            match code {
                0x2A | 0x36 => { self.shifted = false },
                0x3A => self.leds(0b100), // Caps lock
                0x45 => self.leds(0b10), // Number lock
                0x46 => self.leds(0b1), // Scroll lock
                _ => self.gotkey(code as usize)
            }
        } else {
            match scancode {
                0x2A | 0x36 => { self.shifted = true },
                _ => {}
            }
        }
        */
    }
}
