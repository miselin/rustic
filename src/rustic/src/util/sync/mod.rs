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

use std::cell::UnsafeCell;
use std::sync::{TryLockResult, TryLockError, LockResult};
use std::sync::atomic;
use std::sync::atomic::AtomicBool;
use std::ops::{Deref, DerefMut};

pub struct Spinlock<T: ?Sized> {
    atom: AtomicBool,
    data: UnsafeCell<T>
}

#[must_use = "if unused the Spinlock will immediately unlock"]
pub struct SpinlockGuard<'a, T: ?Sized + 'a> {
    lock: &'a Spinlock<T>
}

impl<T: ?Sized> !Send for SpinlockGuard<'_, T> {}
unsafe impl<T: ?Sized + Sync> Sync for SpinlockGuard<'_, T> {}

unsafe impl<T: ?Sized + Send> Send for Spinlock<T> {}
unsafe impl<T: ?Sized + Send> Sync for Spinlock<T> {}

impl<T> Spinlock<T> {
    pub fn new(t: T) -> Spinlock<T> {
        return Spinlock { atom: AtomicBool::new(false), data: UnsafeCell::new(t) };
    }
}

impl<T: ?Sized> Spinlock<T> {
    pub fn lock(&self) -> LockResult<SpinlockGuard<'_, T>> {
        unsafe {
            loop {
                // We do a single load first, because compare_exchange can be
                // implemented in a way that stores the current value of the
                // atomic back into itself (which is bad for cache).
                if self.atom.load(atomic::Ordering::Acquire) == false {
                    // It looks like we might be able to acquire - we'll try
                    match self.atom.compare_exchange(false, true, atomic::Ordering::Acquire, atomic::Ordering::Acquire) {
                        Ok(_) => break,
                        // Failed, might have already acquired by someone else in the meantime.
                        Err(_) => {
                            std::hint::spin_loop()
                        },
                    }
                }
            }

            SpinlockGuard::new(self)
        }
    }

    pub fn try_lock(&self) -> TryLockResult<SpinlockGuard<'_, T>> {
        unsafe {
            match self.atom.compare_exchange(false, true, atomic::Ordering::Acquire, atomic::Ordering::Acquire) {
                Ok(_) => Ok(SpinlockGuard::new(self)?),
                Err(_) => Err(TryLockError::WouldBlock)
            }
        }
    }

    pub fn get_mut(&mut self) -> LockResult<&mut T> {
        let data = self.data.get_mut();
        Ok(data)
    }
}

impl<'spinlock, T: ?Sized> SpinlockGuard<'spinlock, T> {
    unsafe fn new(lock: &'spinlock Spinlock<T>) -> LockResult<SpinlockGuard<'spinlock, T>> {
        Ok(SpinlockGuard { lock })
    }
}

impl<T: ?Sized> Deref for SpinlockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> DerefMut for SpinlockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized> Drop for SpinlockGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        loop {
            // Unlike the acquire case, if we see the lock unlocked but are
            // still in this loop, we're trying to unlock an unlocked spinlock
            // and that is a bug in the implementation.
            if self.lock.atom.load(atomic::Ordering::Acquire) != true {
                panic!("trying to unlock an already unlocked lock");
            }

            match self.lock.atom.compare_exchange(true, false, atomic::Ordering::Acquire, atomic::Ordering::Acquire) {
                // successful compare & exchange, good to return
                Ok(_) => break,
                // unsuccessful, retry
                Err(_) => {}
            }
        }
    }
}
