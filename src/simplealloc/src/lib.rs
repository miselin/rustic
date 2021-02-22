
#![no_std]

static mut HEAP_BASE: usize = 0x200000;

pub unsafe fn direct_alloc(sz: usize, align: usize) -> *mut u8 {
    if (HEAP_BASE & (align -1)) != 0 {
        HEAP_BASE += align - (HEAP_BASE % align);
    }

    let result = HEAP_BASE;
    HEAP_BASE += sz;

    result as *mut u8
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
        assert_eq!(0x200100 as *mut u8, unsafe { direct_alloc(0x123, 0x8) });
        assert_eq!(0x200230 as *mut u8, unsafe { direct_alloc(0x100, 0x10) });
        assert_eq!(0x200330 as *mut u8, unsafe { direct_alloc(0x3, 0x10) });
        assert_eq!(0x200340 as *mut u8, unsafe { direct_alloc(0x3, 0x10) });
    }
}
