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

// Some parts of this file are based on the equivalents (e.g. Mutex) in the
// Rust standard library.

use core::cell::UnsafeCell;
use core::sync::atomic;
use core::sync::atomic::AtomicBool;
use core::ops::{Deref, DerefMut};
use core::fmt;

use crate::Kernel;
use crate::arch::Architecture;

pub struct Spinlock<T: ?Sized> {
    atom: AtomicBool,
    interrupts: AtomicBool,
    data: UnsafeCell<T>
}

pub struct PoisonError<T> {
    guard: T
}

pub enum TryLockError<T> {
    Poisoned(PoisonError<T>),
    WouldBlock,
}

pub type TryLockResult<Guard> = Result<Guard, TryLockError<Guard>>;

pub type LockResult<Guard> = Result<Guard, PoisonError<Guard>>;

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
        return Spinlock { atom: AtomicBool::new(false), interrupts: AtomicBool::new(false), data: UnsafeCell::new(t) };
    }
}

impl<T: ?Sized> Spinlock<T> {
    pub fn lock(&self) -> LockResult<SpinlockGuard<'_, T>> {
        unsafe {
            // Disable interrupts while we are in the critical section
            let was = Kernel::get_interrupts_static();
            Kernel::set_interrupts_static(false);

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
                            core::hint::spin_loop()
                        },
                    }
                } else {
                    // TODO: this is not a deadlock if we have multiple CPU cores active
                    // (but Rustic doesn't support that yet)

                    // Nothing else can unlock the lock if we're trying to
                    // acquire here - deadlock.
                    panic!("Spinlock deadlock: lock already acquired: {:p}", &self.atom);
                }
            }

            self.interrupts.store(was, atomic::Ordering::Release);
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

            match self.lock.atom.compare_exchange(true, false, atomic::Ordering::Release, atomic::Ordering::Relaxed) {
                // successful compare & exchange, good to return
                Ok(_) => break,
                // unsuccessful, retry
                Err(_) => {}
            }
        }

        // Restore interrupts now that we are out of the critical section
        let interrupts = self.lock.interrupts.load(atomic::Ordering::Acquire);
        Kernel::set_interrupts_static(interrupts);
    }
}

impl<T> From<PoisonError<T>> for TryLockError<T> {
    fn from(err: PoisonError<T>) -> TryLockError<T> {
        TryLockError::Poisoned(err)
    }
}

impl<T> fmt::Debug for PoisonError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "PoisonError { inner: .. }".fmt(f)
    }
}

impl<T> fmt::Debug for TryLockError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TryLockError::Poisoned(..) => "Poisoned(..)".fmt(f),
            TryLockError::WouldBlock => "WouldBlock".fmt(f),
        }
    }
}
