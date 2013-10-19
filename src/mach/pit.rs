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

use io;
use vga;
use mach;

static mut TimerHertz: int = 10;
static mut Ticks: int = 0;

static BaseFrequency: int = 1193180;

pub fn init(hz: int) {
    unsafe { TimerHertz = hz; }

    // Program periodic mode, with our desired divisor for the given
    // frequency (in hertz).
    let div = unsafe { BaseFrequency / TimerHertz };
    io::outport(0x43, 0x36u8);
    io::outport(0x40, (div & 0xFF) as u8);
    io::outport(0x40, ((div >> 8) & 0xFF) as u8);

    // Register our IRQ.
    mach::registerirq(0, irq);
}

fn irq() {
    unsafe {
        Ticks += 1000 / TimerHertz;

        if Ticks % 1000 == 0 {
            if Ticks == 4000 {
                vga::write("\\", vga::COLS - 1, vga::ROWS - 1, vga::White, vga::Black);
                Ticks = 0;
            } else if(Ticks == 3000) {
                vga::write("-", vga::COLS - 1, vga::ROWS - 1, vga::White, vga::Black);
            } else if(Ticks == 2000) {
                vga::write("/", vga::COLS - 1, vga::ROWS - 1, vga::White, vga::Black);
            } else if(Ticks == 1000) {
                vga::write("|", vga::COLS - 1, vga::ROWS - 1, vga::White, vga::Black);
            }
        }
    }
}

