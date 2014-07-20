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

use core::iter::Iterator;
use core::option::{Some, None};
use core::str::{StrSlice};

use machine;

use mach::IoPort;

pub enum Colour {
    Black       = 0,
    Blue        = 1,
    Green       = 2,
    Cyan        = 3,
    Red         = 4,
    Pink        = 5,
    Brown       = 6,
    LightGray   = 7,
    DarkGray    = 8,
    LightBlue   = 9,
    LightGreen  = 10,
    LightCyan   = 11,
    LightRed    = 12,
    LightPink   = 13,
    Yellow      = 14,
    White       = 15,
}

pub static COLS: uint = 80;
pub static ROWS: uint = 25;

static VGABASE: uint = 0xB8000;

pub fn fill(with: char, colour: Colour) {
    let field: u16 = with as u16 | (colour as u16 << 12);
    let max = ROWS * COLS * 2;

    let mut offset = 0;
    while offset < max {
        unsafe {
            *((VGABASE + offset) as *mut u16) = field;
        }
        offset += 2;
    }
}

pub fn clear(colour: Colour) {
    fill(' ', colour);
}

fn cursor(x: uint, y: uint) {
    let position = (y * COLS) + x;

    machine().outport(0x3D4, 0x0Fu8);
    machine().outport(0x3D5, ((position & 0xFF) as u8));
    machine().outport(0x3D4, 0x0Eu8);
    machine().outport(0x3D5, (((position >> 8) & 0xFF) as u8));

    unsafe {
        let curr: u16 = *((VGABASE + (position * 2)) as *const u16);
        let attr: u8 = (curr >> 8) as u8;
        if attr & 0xFu8 == 0 {
            // No foreground colour attribute. Fix.
            *((VGABASE + (position * 2)) as *mut u16) = curr | (LightGray as u16 << 8);
        }
    }
}

fn write_char_internal(c: char, mut offset: uint, attr: u8) -> uint {
    // Handle crazy codepoints that we can't render.
    let glyph = if c as u32 > 0xFF {
        '\xdb'  // ASCII shaded box.
    } else {
        c
    };

    match glyph {
        '\n' => {
            offset += COLS;
            offset -= offset % COLS;
        },
        '\r' => {
            offset -= offset % COLS;
        },
        '\t' => {
            offset += 4;
            offset -= offset % 4;
        },
        '\0' => {},
        _ => {
            unsafe {
                let p: *mut u16 = (VGABASE + (offset * 2)) as *mut u16;
                *p = (glyph as u16) | (attr as u16 << 8);
            }
            offset += 1;
        }
    }

    offset
}

pub fn write_char(c: char, x: uint, y: uint, fg: Colour, bg: Colour) -> uint {
    let attr = (bg as u8 << 4) | (fg as u8);
    let offset = (y * COLS) + x;

    write_char_internal(c, offset, attr)
}

pub fn write(s: &str, x: uint, y: uint, fg: Colour, bg: Colour) -> uint {
    let attr = (bg as u8 << 4) | (fg as u8);
    let mut offset = (y * COLS) + x;

    for c in s.chars() {
        offset = write_char_internal(c, offset, attr);
        if offset > (ROWS * COLS) {
            // TODO: scroll.
            break;
        }
    }

    cursor((offset % 80), offset / 80);

    offset
}

