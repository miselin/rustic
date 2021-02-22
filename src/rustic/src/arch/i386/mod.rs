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

use alloc::collections::VecDeque;
use core::ffi::c_void;
use alloc::sync::Arc;
use core::mem::ManuallyDrop;
use alloc::boxed::Box;

use crate::util::sync::Spinlock;

use crate::arch::{Architecture, Threads, ThreadSpawn};
use crate::mach::Machine;

use simplealloc;

use crate::{Kernel,  Idle};

mod gdt;
mod idt;

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct ThreadState {
    edi: u32,
    esi: u32,
    ebx: u32,
    ebp: u32,
    esp: u32,
    eip: u32,
}

struct Thread {
    exec_state: ThreadState,
    is_alive: bool
}

pub struct State {
    idt: idt::Idt,
    ready_threads: VecDeque<Thread>,
    running_thread: Option<Thread>,
    alive: bool,
}

// External variable in assembly code (not actually a function)
extern { fn thread_trampoline(); }

extern "C" {
    fn save_state(state: *mut ThreadState) -> u32;
    fn restore_state(state: *const ThreadState) -> u32;
}

impl State {
    pub fn new() -> State {
        State{
            idt: idt::Idt::new(),
            ready_threads: VecDeque::new(),
            running_thread: None,
            alive: false,
        }
    }
}

impl Thread {
    pub fn new() -> Thread {
        Thread{
            exec_state: ThreadState::new(),
            is_alive: false
        }
    }
}

impl ThreadState {
    fn new() -> ThreadState {
        ThreadState{
            edi: 0,
            esi: 0,
            ebx: 0,
            ebp: 0,
            esp: 0,
            eip: 0,
        }
    }
}

impl<'a> Architecture for Kernel {
    fn arch_initialise(&mut self) -> bool {
        gdt::setup_gdt();

        self.arch.state.idt.init();

        self.arch.initialised = true;
        self.arch.initialised
    }

    fn register_trap(&mut self, which: usize, handler: extern "Rust" fn(usize)) {
        self.arch.state.idt.register(which, handler)
    }

    fn get_interrupts(&self) -> bool {
        // TODO: write
        false
    }

    fn set_interrupts(&mut self, state: bool) {
        if state == true {
            unsafe { llvm_asm!("sti") }
        } else {
            unsafe { llvm_asm!("cli") }
        }
    }

    fn wait_for_event(&self) {
        unsafe { llvm_asm!("sti; hlt") }
    }
}

impl Idle for Kernel {
    fn idle() {
        unsafe { llvm_asm!("sti; hlt") }
    }
}

impl<'a, F: FnMut() + Send + 'static> ThreadSpawn<F> for Kernel {
    fn spawn_thread(&mut self, f: F) {
        let mut new_thread = Thread::new();
        new_thread.exec_state.eip = thread_trampoline as u32;
        new_thread.exec_state.ebx = get_thread_trampoline(&f) as u32;
        new_thread.exec_state.esi = Box::into_raw(Box::new(f)) as *mut () as u32;

        // TODO(miselin): do this way better than this.
        let stack = unsafe { simplealloc::direct_alloc(4096, 16) } as *mut u32;
        let stack_top = stack as u32 + 4096;
        new_thread.exec_state.esp = stack_top;

        new_thread.is_alive = true;

        self.arch.state.ready_threads.push_front(new_thread)
    }
}

impl Threads for Kernel {
    fn thread_terminate(&mut self) -> ! {
        // self.arch.state.running_thread.is_alive = false;
        // todo...
        loop {}
    }

    fn reschedule(lock: Arc<Spinlock<Kernel>>) {
        // We wrap the guard in a ManuallyDrop to avoid it being dropped
        // in the code paths here that run without a lock (in particular,
        // save_state will return twice).
        let guard = lock.lock().unwrap();
        let mut obj = ManuallyDrop::new(guard);
        let state = &mut obj.arch.state;

        if state.ready_threads.is_empty() {
            unsafe { ManuallyDrop::drop(&mut obj) };
            return;
        }

        // Only save old state if there is an old state to save.
        if state.alive {
            if let Some(mut old_thread) = state.running_thread.take() {
                if unsafe { save_state(&mut old_thread.exec_state) } == 1 {
                    // Just got context-switched to.
                    return;
                }

                // Now that state is saved, move the thread back to the ready queue.
                state.ready_threads.push_back(old_thread);
            }
        }

        // Load new state.
        let new_thread = state.ready_threads.pop_front().unwrap();
        let new_state = new_thread.exec_state.clone();
        state.running_thread = Some(new_thread);  // move into the Option
        state.alive = true;
        unsafe { ManuallyDrop::drop(&mut obj) };  // unlock right before we load the new context
        unsafe { restore_state(&new_state) };

        // unreachable
        loop {}
    }
}

pub type RustThreadTrampoline = unsafe extern "C" fn(*mut c_void) -> !;

pub unsafe extern "C" fn rust_spawned_trampoline<F>(data: *mut c_void) -> !
where
    F: FnMut(),
    F: Send,
    F: 'static
{
    let entry = &mut *(data as *mut F);
    entry();

    // On return from the thread entry point, drop the closure
    Box::from_raw(data as *mut F);

    // TODO: figure out how to do termination.
    loop {}
}

fn get_thread_trampoline<F>(_closure: &F) -> RustThreadTrampoline
where
    F: FnMut(),
    F: Send,
    F: 'static
{
    rust_spawned_trampoline::<F>
}
