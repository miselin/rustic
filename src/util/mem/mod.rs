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
use core;

// Private definition of c_void type.
enum c_void {
    __variant1,
    __variant2,
}

extern "C" {
    fn malloc(sz: uint) -> *mut c_void;
    fn calloc(num: uint, sz: uint) -> *mut c_void;
    fn free(ptr: *const c_void);
}

pub unsafe fn allocate<T>() -> *mut T {
    calloc(1, core::mem::size_of::<T>()) as *mut T
}

pub unsafe fn deallocate<T>(ptr: *const T) {
    free(ptr as *const c_void)
}
