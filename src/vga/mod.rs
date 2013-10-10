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

#[fixed_stack_segment]
pub fn clear(colour: Colour) {
    let mut offset = 0;
    let max = 80 * 25 * 2;
    loop {
        if offset >= max { break; }
        unsafe {
            *((0xb8000 + offset) as *mut u8) = ' ' as u8;
            *((0xb8001 + offset) as *mut u8) = colour as u8 << 4;
        }
        offset += 2;
    }
}

#[fixed_stack_segment]
pub fn write(s: &str, l: uint, x: int, y: int, fg: Colour, bg: Colour) {
    let mut offset = (y * 80) + x;
    let mut index = 0;

    loop {
        if index >= l { break; }
        let c = s[index] as char;
        if(c == '\n') {
            offset += 80;
        } else if(c == '\r') {
            offset -= offset % 80;
        } else if(c == '\t') {
            offset += 4;
        } else if(c == '\0') {
            break;
        } else {
            unsafe {
                *((0xb8000 + (offset * 2)) as *mut u8) = s[index];
                *((0xb8001 + (offset * 2)) as *mut u8) = (bg as u8 << 4) | (fg as u8);
            }
            offset = offset + 1;
            if(offset > (80 * 25)) { break; }
        }
        index = index + 1;
    }
}

