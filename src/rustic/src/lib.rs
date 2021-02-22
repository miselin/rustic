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
#![feature(negative_impls)]
#![feature(alloc_error_handler)]
#![allow(dead_code)]

#![no_main]
#![no_std]

extern crate alloc;

// Publish the main things users care about.
pub use mach::{Machine, TimerHandlers, Mmio, Gpio, IoPort, IrqHandler, Serial};
pub use arch::{Architecture, Threads, ThreadSpawn};

// Pull in the architectural layer (CPU etc).
pub mod arch;

// Pull in the machine layer.
pub mod mach;

// Pull in utils library.
pub mod util;

use alloc::format;
use alloc::sync::Arc;
use core::panic::PanicInfo;
use core::fmt::{Write, Error};
use util::sync::Spinlock;

struct Debug {
}

impl Write for Debug {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        Kernel::debug(s);
        Ok(())
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut v = Debug{};
    core::fmt::write(&mut v, format_args!("panic: {}", info));
    loop {}
}

pub struct Kernel {
    mach: mach::MachineState,
    arch: arch::ArchitectureState,
}

// Required to be defined by the application.
extern "Rust" { fn run(k: &mut Kernel); }

pub trait Idle {
    fn idle();
}

impl Kernel {
    pub fn new() -> Kernel {
        Kernel {
            mach: mach::create(),
            arch: arch::create()
        }
    }

    // Sets up the kernel, and then returns a wrapped version of the Kernel
    // that is correctly prepared for concurrency.
    pub fn start(mut self) -> Arc<Spinlock<Kernel>> {
        // Now we can initialise the system.
        self.arch_initialise();
        self.mach_initialise();

        // All done with initial startup.
        self.serial_write("Built on the Rustic Framework.\n");

        // Enable IRQs and start up the application.
        self.set_interrupts(true);

        Arc::new(Spinlock::new(self))
    }
}
