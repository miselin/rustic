
use libc;
use sync::atomics;

use arch::Threads;

use architecture;

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
    architecture().reschedule();
}

#[allow(non_camel_case_types)]
struct pthread_mutex_t {
    ready: bool,
    value: atomics::AtomicUint,
}

#[allow(non_camel_case_types)]
struct pthread_cond_t {
    value: atomics::AtomicUint,
}

fn check_mutex(l: *mut pthread_mutex_t) {
    let lock = unsafe { &mut *l };
    if !lock.ready {
        lock.value = atomics::AtomicUint::new(0);
        lock.ready = true;
    }
}

#[allow(visible_private_types)]
#[no_mangle]
pub extern "C" fn pthread_mutex_destroy(lock: *mut pthread_mutex_t) -> libc::c_int {
    pthread_mutex_unlock(lock);
    0
}

#[allow(visible_private_types)]
#[no_mangle]
pub extern "C" fn pthread_cond_destroy(_: *mut pthread_cond_t) -> libc::c_int {
    0
}

#[allow(visible_private_types)]
#[no_mangle]
pub extern "C" fn pthread_mutex_lock(lock: *mut pthread_mutex_t) -> libc::c_int {
    // TODO(miselin): we really want to actually sleep the thread and wake it
    // later, rather than this...
    while pthread_mutex_trylock(lock) == -1 {
        sched_yield();
    }
    0
}

#[allow(visible_private_types)]
#[no_mangle]
pub extern "C" fn pthread_mutex_trylock(l: *mut pthread_mutex_t) -> libc::c_int {
    check_mutex(l);

    let lock = unsafe { &mut *l };
    if lock.value.compare_and_swap(0, 1, atomics::SeqCst) == 0 {
        -1
    } else {
        0
    }
}

#[allow(visible_private_types)]
#[no_mangle]
pub extern "C" fn pthread_mutex_unlock(l: *mut pthread_mutex_t) -> libc::c_int {
    check_mutex(l);

    let lock = unsafe { &mut *l };
    lock.value.compare_and_swap(1, 0, atomics::SeqCst);
    0
}

#[allow(visible_private_types)]
#[no_mangle]
pub extern "C" fn pthread_cond_wait(_: *mut pthread_cond_t, _: *mut pthread_mutex_t) -> libc::c_int {
    0
}

#[allow(visible_private_types)]
#[no_mangle]
pub extern "C" fn pthread_cond_signal(_: *mut pthread_cond_t) -> libc::c_int {
    0
}

