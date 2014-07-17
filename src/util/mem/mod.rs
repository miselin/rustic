
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
