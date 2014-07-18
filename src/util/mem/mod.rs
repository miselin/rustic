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

// TODO: write a proper allocator, prime with multiboot memory map.
static mut HeapBase: uint = 0x200000;

pub unsafe fn allocate<T>() -> *mut T {
    let uint_size = core::mem::size_of::<uint>();
    let object_size = core::mem::size_of::<T>();

    let ret = (HeapBase + uint_size) as *mut T;
    let tag = HeapBase as *mut uint;

    HeapBase += uint_size + object_size;

    *tag = object_size;
    core::ptr::zero_memory(ret, 1);

    ret
}

pub unsafe fn deallocate<T>(ptr: *const T) {
    // no-op at the moment.
}
