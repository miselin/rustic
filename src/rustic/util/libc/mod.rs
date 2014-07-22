
use libc;

#[no_mangle]
pub fn strlen(_: *const libc::c_char) -> libc::size_t {
    0 as libc::size_t
}

#[no_mangle]
pub fn write(_: libc::c_int, _: *const libc::c_void, _: libc::size_t) -> libc::c_int {
    0 as libc::c_int
}
