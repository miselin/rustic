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

#[no_mangle]
pub extern fn memcpy(dst: *mut u8, src: *const u8, n: isize) -> *const u8 {
    for i in 0..n {
        unsafe { *dst.offset(i) = *src.offset(i) };
    }

    return dst;
}

#[no_mangle]
pub extern fn memmove(dst: *mut u8, src: *const u8, n: isize) -> *const u8 {
    // TODO: this is buggy because it just memcpy's
    for i in 0..n {
        unsafe { *dst.offset(i) = *src.offset(i) };
    }

    return dst;
}

#[no_mangle]
pub extern fn memset(dst: *mut u8, val: u8, n: isize) -> *const u8 {
    for i in 0..n {
        unsafe { *dst.offset(i) = val };
    }

    return dst;
}