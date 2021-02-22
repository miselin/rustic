
#![no_std]

use core;

static mut HeapBase: usize = 0x200000;

pub unsafe fn direct_alloc(sz: usize, align: usize) -> *mut u8 {
    let uint_size = core::mem::size_of::<usize>();

    let mut alloc_base = HeapBase;
    let mut actual = alloc_base;

    // Round up as needed for alignment
    if (actual & (align - 1)) != 0 {
        actual += align - (actual % align);
    }

    let end = actual + sz;
    let final_size = end - alloc_base;

    HeapBase += final_size;

    // Always succeeds, so no need to return null_mut() in any code path
    actual as *mut u8
}

pub unsafe fn direct_dealloc(_ptr: *mut u8) {
    // does nothing as we're just incrementing a heap base
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aligned_allocs() {
        assert_eq!(0x200000 as *mut u8, unsafe { direct_alloc(0x100, 0x10) });
        assert_eq!(0x200100 as *mut u8, unsafe { direct_alloc(0x123, 0x100) });
        assert_eq!(0x200230 as *mut u8, unsafe { direct_alloc(0x100, 0x10) });
        assert_eq!(0x200330 as *mut u8, unsafe { direct_alloc(0x3, 0x10) });
        assert_eq!(0x200340 as *mut u8, unsafe { direct_alloc(0x3, 0x10) });
    }
}
