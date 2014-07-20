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

use vga;
use mach::{IrqHandler, IoPort};

use machine;

static BaseFrequency: int = 1193180;

pub struct Pit {
    ticks: int,
    timer_hz: int,
}

impl Pit {
    pub fn new() -> Pit{
        Pit{ticks: 0, timer_hz: 0}
    }

    pub fn init(hz: int) -> Pit {
        let state = Pit{ticks: 0, timer_hz: hz};

        // Program periodic mode, with our desired divisor for the given
        // frequency (in hertz).
        let div = BaseFrequency / state.timer_hz;
        machine().outport(0x43, 0x36u8);
        machine().outport(0x40, (div & 0xFF) as u8);
        machine().outport(0x40, ((div >> 8) & 0xFF) as u8);

        state
    }

    pub fn irq_num() -> uint {
        0
    }
}

impl IrqHandler for Pit {
    fn irq(&mut self, _: uint) {
        self.ticks += 1000 / self.timer_hz;

        if self.ticks % 1000 == 0 {
            if self.ticks == 4000 {
                vga::write_char('\\', vga::COLS - 1, vga::ROWS - 1, vga::White, vga::Black);
                self.ticks = 0;
            } else if self.ticks == 3000 {
                vga::write_char('-', vga::COLS - 1, vga::ROWS - 1, vga::White, vga::Black);
            } else if self.ticks == 2000 {
                vga::write_char('/', vga::COLS - 1, vga::ROWS - 1, vga::White, vga::Black);
            } else if self.ticks == 1000 {
                vga::write_char('|', vga::COLS - 1, vga::ROWS - 1, vga::White, vga::Black);
            }
        }
    }
}
