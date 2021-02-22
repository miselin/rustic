/*
 * Copyright (c) 2014 Matthew Iselin
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

#![macro_use]

pub mod mem;
pub mod sync;

pub mod colour {
    #[derive(Copy, Clone)]
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
}
