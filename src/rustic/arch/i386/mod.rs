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

use std::collections::VecDeque;

use crate::arch::{Architecture, ArchitectureState, Threads};

use crate::kernel_mut;
use crate::util;

mod gdt;
mod idt;
pub mod mem;

type ThreadEntryPoint = fn();

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
    is_alive: bool,
    entry: Option<ThreadEntryPoint>,
}

pub struct State {
    gdt: gdt::Gdt,
    idt: idt::Idt,
    ready_threads: VecDeque<Thread>,
    running_thread: Thread,
    alive: bool,
}

// External variable in assembly code (not actually a function)
extern {fn tls_emul_segment(); }

extern "C" {
    fn save_state(state: *mut ThreadState) -> std::os::raw::c_uint;
    fn restore_state(state: *const ThreadState) -> std::os::raw::c_uint;
}

impl State {
    pub fn new() -> State {
        State{
            gdt: gdt::Gdt::new(),
            idt: idt::Idt::new(),
            ready_threads: VecDeque::with_capacity(1),
            running_thread: Thread::new(),
            alive: false,
        }
    }
}

impl Thread {
    pub fn new() -> Thread {
        Thread{
            exec_state: ThreadState::new(),
            is_alive: false,
            entry: None,
        }
    }

    pub fn copy(&self) -> Thread {
        Thread{
            exec_state: self.exec_state,
            is_alive: self.is_alive,
            entry: None,
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

impl Architecture for ArchitectureState {
    fn initialise(&mut self) -> bool {
        self.state.gdt.entry(0, 0, 0, 0, 0); // 0x00 - NULL
        self.state.gdt.entry(1, 0, 0xFFFFFFFF, 0x98, 0xCF); // 0x08 - Kernel Code
        self.state.gdt.entry(2, 0, 0xFFFFFFFF, 0x92, 0xCF); // 0x10 - Kernel Data
        self.state.gdt.entry(3, 0, 0xFFFFFFFF, 0xF8, 0xCF); // 0x18 - User Code
        self.state.gdt.entry(4, 0, 0xFFFFFFFF, 0xF2, 0xCF); // 0x20 - User Data
        self.state.gdt.entry(5, tls_emul_segment as u32, 0xFFFFFFFF, 0x92, 0xCF); // 0x28 - TLS emulation (for stack switching support)
        self.state.gdt.load(0x08, 0x10, 0x28);

        self.state.idt.init();

        true
    }

    fn register_trap(&mut self, which: usize, handler: extern "Rust" fn(usize)) {
        self.state.idt.register(which, handler)
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

impl Threads for ArchitectureState {
    fn spawn_thread(&mut self, f: ThreadEntryPoint) {
        let mut new_thread = Thread::new();
        new_thread.exec_state.eip = rust_spawned_trampoline as u32;

        // TODO(miselin): do this way better than this.
        let stack = unsafe { util::mem::alloc(4096, 16) } as *mut u32;
        let stack_top = stack as u32 + 4096;
        new_thread.exec_state.esp = stack_top;

        new_thread.entry = Some(f);
        new_thread.is_alive = true;

        self.state.ready_threads.push_front(new_thread);
    }

    fn thread_terminate(&mut self) -> ! {
        self.state.running_thread.is_alive = false;
        self.reschedule();
        loop {}
    }

    fn reschedule(&mut self) {
        let state = &mut self.state;

        if state.ready_threads.len() == 0 {
            return;
        }

        // Only save old state if there is an old state to save.
        if state.alive {
            let mut old_thread = state.running_thread.copy();

            if old_thread.is_alive {
                if unsafe { save_state(&mut old_thread.exec_state) } == 1 {
                    // Just got context-switched to.
                    return;
                }

                // Now that state is saved, push the old thread to the running queue.
                state.ready_threads.push_back(old_thread);
            }
        }

        // Load new state.
        state.running_thread = state.ready_threads.pop_front().unwrap();
        state.alive = true;
        unsafe { restore_state(&state.running_thread.exec_state) };
    }
}

extern "C" fn rust_spawned_trampoline() -> ! {
    let f = kernel_mut().architecture_mut().state.running_thread.entry.unwrap();
    f();

    kernel_mut().architecture_mut().thread_terminate();
}
