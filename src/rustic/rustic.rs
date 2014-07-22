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
#![feature(lang_items)]
#![feature(globs)]
#![feature(macro_rules)]
#![feature(phase, macro_rules)]
#![allow(dead_code)]

#![crate_name = "rustic"]
#![desc = "Rustic Embedded Framework"]
#![license = "ISC"]
#![comment = "Provides an framework upon which to build embedded software in Rust."]

// Pull in the libc crate for us to implement libc functions needed by the Rust
// runtime.
extern crate libc;

// Publish the main things users care about.
pub use mach::{Machine, TimerHandlers, Mmio, Gpio, IoPort, IrqHandler};
pub use arch::Architecture;

// Magic for macros.
pub use screen = mach::screen;
pub use serial = mach::serial;

// Pull in the architectural layer (CPU etc).
pub mod arch;

// Pull in the machine layer.
pub mod mach;

// Pull in utils library.
pub mod util;

// Required to be defined by the application.
extern { fn run(); }

#[no_mangle]
pub extern "C" fn abort() -> ! {
    // TODO: should this be provided by the application?
    architecture().set_interrupts(false);
    printlnto!(serial, "Abort!");
    loop {}
}

static mut global_architecture: *mut arch::ArchitectureState = 0 as *mut arch::ArchitectureState;
static mut global_machine: *mut mach::MachineState = 0 as *mut mach::MachineState;

#[no_mangle]
pub extern "C" fn main(argc: int, _: *const *const u8) -> int {
    if argc != 1 {
        abort();
    }

    // Create boxed abstractions.
    let mut arch_object = arch::create();
    let mut machine = mach::create();

    // Pass a borrow of the contents of the box to the main trampoline, which
    // will set up the global singleton.
    main_trampoline(&mut *arch_object, &mut *machine);

    0
}

fn main_trampoline(architecture: &mut arch::ArchitectureState, machine: &mut mach::MachineState) {
    // Load global state for singleton pattern.
    unsafe {
        global_architecture = architecture as *mut arch::ArchitectureState;
        global_machine = machine as *mut mach::MachineState;
    }

    // Now we can initialise the system.
    ::architecture().initialise();
    ::machine().initialise();

    // All done with initial startup.
    printlnto!(serial, "Built on the Rustic Framework.");

    // Enable IRQs and start up the application.
    architecture.set_interrupts(true);
    unsafe { run() };
}

pub fn architecture() -> &mut arch::ArchitectureState {
    unsafe { &mut *global_architecture }
}

pub fn machine() -> &mut mach::MachineState {
    unsafe { &mut *global_machine }
}
