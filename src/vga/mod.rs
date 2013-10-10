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

use zero;

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

static COLS: uint = 80;
static ROWS: uint = 25;

static VGABASE: uint = 0xB8000;

#[fixed_stack_segment]
pub fn clear(colour: Colour) {
    let mut offset = 0;
    let field: u16 = ' ' as u16 | (colour as u16 << 12);
    let max = ROWS * COLS * 2;
    loop {
        if offset >= max { break; }
        unsafe {
            *((VGABASE + offset) as *mut u16) = field;
        }
        offset += 2;
    }
}

#[fixed_stack_segment]
pub fn write(s: &str, x: uint, y: uint, fg: Colour, bg: Colour) {
    let mut offset = (y * 80) + x;
    let mut index = 0;

    let buflen =
    unsafe {
        let (_, slen): (*u8, uint) = zero::transmute(s);
        slen
    };

    let attr = (bg as u8 << 4) | (fg as u8);

    loop {
        if index >= buflen { break; }

        let c = s[index] as char;
        if(c == '\n') {
            // \n implies \r
            offset += COLS;
            offset -= offset % COLS;
        } else if(c == '\r') {
            offset -= offset % COLS;
        } else if(c == '\t') {
            offset += 4;
            offset -= offset % 4;
        } else if(c == '\0') {
            break;
        } else {
            unsafe {
                *((VGABASE + (offset * 2)) as *mut u16) = (s[index] as u16) | (attr as u16 << 8);
            }
            offset = offset + 1;

            if(offset > (ROWS * COLS)) {
                // TODO: scroll!
                break;
            }
        }
        index = index + 1;
    }
}

