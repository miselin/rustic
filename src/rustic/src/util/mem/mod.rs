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

use std::alloc::{GlobalAlloc, Layout, alloc};
use std::ptr::null_mut;

// TODO: write a proper allocator, prime with multiboot memory map.
static mut HeapBase: usize = 0x200000;

struct RusticAllocator;

unsafe impl GlobalAlloc for RusticAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        direct_alloc(layout.size(), layout.align())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        direct_dealloc(ptr)
    }
}

#[global_allocator]
static A: RusticAllocator = RusticAllocator;

pub unsafe fn direct_alloc(sz: usize, _align: usize) -> *mut u8 {
    // TODO: handle alignment.
    let uint_size = std::mem::size_of::<usize>();

    let ret = (HeapBase + sz) as *mut u8;
    let tag = HeapBase as *mut usize;

    HeapBase += uint_size + sz;

    // Always succeeds, so no need to return null_mut() in any code path
    *tag = sz;
    ret
}

pub unsafe fn direct_dealloc(_ptr: *mut u8) {
    // does nothing as we're just incrementing a heap base
}

/*
#[no_mangle]
pub extern "C" fn malloc(sz: u32) -> *mut u8 {
    unsafe { alloc(sz, 4) }
}

#[no_mangle]
pub unsafe fn realloc() -> *mut u8 {
    null_mut()
}

#[no_mangle]
pub unsafe fn free(_: *const u8) {
    // no-op at the moment.
}

#[no_mangle]
pub extern "C" fn posix_memalign(memptr: *mut *mut u8, alignment: u32, sz: u32) -> i32 {
    unsafe { *memptr = alloc(sz, alignment) };
    0
}
*/
