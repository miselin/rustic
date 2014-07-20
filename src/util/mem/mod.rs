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

pub fn allocate<T>() -> *mut T {
    unsafe {
        let ptr = alloc(core::mem::size_of::<T>(), 4) as *mut T;
        core::ptr::zero_memory(ptr, 1);
        ptr
    }
}

pub fn deallocate<T>(p: *const T) {
    unsafe { free(p as *const u8); }
}

pub unsafe fn alloc(sz: uint, _: uint) -> *mut u8 {
    // TODO: handle alignment.
    let uint_size = core::mem::size_of::<uint>();
    let object_size = sz;

    let ret = (HeapBase + uint_size) as *mut u8;
    let tag = HeapBase as *mut uint;

    HeapBase += uint_size + object_size;

    *tag = object_size;
    ret
}

#[no_mangle]
pub extern "C" fn malloc(sz: uint) -> *mut u8 {
    unsafe { alloc(sz, 4) }
}

pub unsafe fn realloc() -> *mut u8 {
    0 as *mut u8
}

#[no_mangle]
pub unsafe fn free(_: *const u8) {
    // no-op at the moment.
}

#[no_mangle]
pub extern "C" fn posix_memalign(memptr: *mut *mut u8, alignment: uint, sz: uint) -> int {
    unsafe { *memptr = alloc(sz, alignment) };
    0
}
