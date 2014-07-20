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
#![no_std]
#![feature(asm)]
#![feature(lang_items)]
#![feature(globs)]
#![allow(dead_code)]

// Note: remember to update RUST_LIBS in Makefile when adding more extern
// crates here.

// Pull in the 'core' crate.
extern crate core;

// Pull in the 'rlibc' crate.
extern crate rlibc;

// Pull in 'alloc' crate for Arc, Rc, Box, etc...
extern crate alloc;

use mach::{Machine, Keyboard};
use arch::Architecture;

// Pull in VGA utils - clear screen, write text, etc...
mod vga;

// Grab serial port I/O stuff.
mod serial;

// Pull in the architectural layer (CPU etc).
pub mod arch;

// Pull in the machine layer.
pub mod mach;

// Pull in utils library.
mod util;

#[no_mangle]
pub extern "C" fn abort() -> ! {
    //architecture().setirqs(false);
    vga::clear(vga::Red);
    serial::write("ABORT");
    loop {}
}

static mut global_architecture: *mut arch::ArchitectureState = 0 as *mut arch::ArchitectureState;
static mut global_machine: *mut mach::MachineState = 0 as *mut mach::MachineState;

#[no_mangle]
pub extern "C" fn main(argc: int, _: *const *const u8) -> int {
    // Clear to black.
    vga::clear(vga::Black);

    if argc != 1 {
        abort();
    }

    // Start up the serial port...
    serial::config(115200, 8, serial::NoParity, 1);

    // Dump some startup junk to the serial port.
    serial::write("Rustic starting up...\n");

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

    // Set LEDs for fun.
    ::machine().kb_leds(1);

    // Welcome message.
    vga::write("Welcome to Rustic!", 0, 0, vga::LightGray, vga::Black);

    // All done with initial startup.
    serial::write("Rustic startup complete.\n");

    // Loop forever, IRQ handling will do the rest!
    architecture.set_interrupts(true);
    loop {
        architecture.wait_for_event();
    }
}

pub fn architecture() -> &mut arch::ArchitectureState {
    unsafe { &mut *global_architecture }
}

pub fn machine() -> &mut mach::MachineState {
    unsafe { &mut *global_machine }
}

#[lang="begin_unwind"]
pub fn begin_unwind() {
    abort();
}

#[lang="stack_exhausted"]
pub fn stack_exhausted() {
    abort();
}

#[lang="eh_personality"]
pub fn eh_personality() {
    abort();
}
