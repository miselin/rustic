/*
 * Copyright (c) 2013 Matthew Iselin
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
#![feature(asm)]
#![feature(llvm_asm)]
#![feature(lang_items)]
#![feature(rustc_private)]
#![allow(dead_code)]

// Publish the main things users care about.
pub use mach::{Machine, TimerHandlers, Mmio, Gpio, IoPort, IrqHandler, serial};
pub use arch::{Architecture, Threads};

// Pull in the architectural layer (CPU etc).
pub mod arch;

// Pull in the machine layer.
pub mod mach;

// Pull in utils library.
pub mod util;

pub struct Kernel<'a> {
    mach: mach::MachineState<'a>,
    arch: arch::ArchitectureState,
}

// Required to be defined by the application.
extern "Rust" { fn run(k: &mut Kernel); }

pub static mut kernel_static: Option<Kernel> = None;

#[no_mangle]
pub extern "C" fn abort() -> ! {
    // TODO: should this be provided by the application?
    kernel_mut().architecture_mut().set_interrupts(false);
    printlnto!(serial, "Abort!");
    loop {}
}

#[no_mangle]
pub extern "C" fn main(argc: i32, _: *const *const u8) -> i32 {
    if argc != 1 {
        abort();
    }

    // Create abstractions now that we're in the kernel
    let mut kernel_state = Kernel::new();
    kernel_state.main_trampoline();

    0
}

impl<'a> Kernel<'a> {
    fn new() -> Kernel<'a> {
        Kernel {
            mach: mach::create(),
            arch: arch::create()
        }
    }

    fn main_trampoline(&'a mut self) {
        // Now we can initialise the system.
        self.arch.initialise();
        self.mach.initialise();

        // All done with initial startup.
        printlnto!(serial, "Built on the Rustic Framework.");

        // Enable IRQs and start up the application.
        self.arch.set_interrupts(true);
        /*
        self.spawn(|| {
            unsafe { run(self) };
        })
        */

        // Run the application.
        unsafe { run(self) };
    }

    pub fn architecture<'b>(&'b self) -> &'b arch::ArchitectureState {
        &self.arch
    }

    pub fn architecture_mut<'b>(&'b mut self) -> &'b mut arch::ArchitectureState {
        &mut self.arch
    }

    pub fn machine<'b>(&'b self) -> &'b mach::MachineState<'a> {
        &self.mach
    }

    pub fn machine_mut<'b>(&'b mut self) -> &'b mut mach::MachineState<'a> {
        &mut self.mach
    }

    pub fn spawn(&mut self, f: fn()) {
        self.arch.spawn_thread(f);
        self.arch.reschedule();
    }
}

pub fn kernel<'a>() -> &'a Kernel<'static> {
    unsafe {
        match kernel_static {
            Some(ref v) => v,
            None => panic!("kernel is not initialized yet")
        }
    }
}

pub fn kernel_mut<'a>() -> &'a mut Kernel<'static> {
    unsafe {
        match kernel_static {
            Some(ref mut v) => v,
            None => panic!("kernel is not initialized yet")
        }
    }
}
