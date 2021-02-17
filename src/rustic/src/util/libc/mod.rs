
/*
use std::sync::atomic;

use crate::arch::Threads;

use crate::kernel_mut;

#[no_mangle]
pub extern "C" fn strlen(_: *const libc::c_char) -> libc::size_t {
    0 as libc::size_t
}

#[no_mangle]
pub extern "C" fn write(_: libc::c_int, _: *const libc::c_void, _: libc::size_t) -> libc::c_int {
    0 as libc::c_int
}

#[no_mangle]
pub extern "C" fn sched_yield() {
    kernel_mut().architecture_mut().reschedule();
}

#[allow(non_camel_case_types)]
pub struct pthread_mutex_t {
    ready: bool,
    value: atomic::AtomicUsize,
}

#[allow(non_camel_case_types)]
pub struct pthread_cond_t {
    value: atomic::AtomicUsize,
}

fn check_mutex(l: *mut pthread_mutex_t) {
    let lock = unsafe { &mut *l };
    if !lock.ready {
        lock.value = atomic::AtomicUsize::new(0);
        lock.ready = true;
    }
}

#[no_mangle]
pub extern "C" fn pthread_mutex_destroy(lock: *mut pthread_mutex_t) -> libc::c_int {
    pthread_mutex_unlock(lock);
    0
}

#[no_mangle]
pub extern "C" fn pthread_cond_destroy(_: *mut pthread_cond_t) -> libc::c_int {
    0
}

#[no_mangle]
pub extern "C" fn pthread_mutex_lock(lock: *mut pthread_mutex_t) -> libc::c_int {
    // TODO(miselin): we really want to actually sleep the thread and wake it
    // later, rather than this...
    while pthread_mutex_trylock(lock) == -1 {
        sched_yield();
    }
    0
}

#[no_mangle]
pub extern "C" fn pthread_mutex_trylock(l: *mut pthread_mutex_t) -> libc::c_int {
    check_mutex(l);

    let lock = unsafe { &mut *l };
    if lock.value.compare_exchange(0, 1, atomic::Ordering::SeqCst, atomic::Ordering::SeqCst).is_ok() {
        -1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn pthread_mutex_unlock(l: *mut pthread_mutex_t) -> libc::c_int {
    check_mutex(l);

    let lock = unsafe { &mut *l };
    while !lock.value.compare_exchange(1, 0, atomic::Ordering::SeqCst, atomic::Ordering::SeqCst).is_ok() {}
    0
}

#[no_mangle]
pub extern "C" fn pthread_cond_wait(_: *mut pthread_cond_t, _: *mut pthread_mutex_t) -> libc::c_int {
    0
}

#[no_mangle]
pub extern "C" fn pthread_cond_signal(_: *mut pthread_cond_t) -> libc::c_int {
    0
}

*/