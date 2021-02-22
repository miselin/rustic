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
#![feature(llvm_asm)]
#![feature(globs)]

#![no_main]
#![no_std]

extern crate alloc;

use rustic::{Kernel, Idle};

use rustic::arch;
use rustic::mach;

use rustic::arch::{Architecture, Threads, ThreadSpawn};
use rustic::mach::{Keyboard, Screen, TimerHandlers, Serial, Machine};
use rustic::util;

use alloc::sync::Arc;
use alloc::boxed::Box;
use util::sync::Spinlock;

static mut global_ticks: usize = 0;

/*
fn demo_screen() {
    println!("Hello from the Rustic demo!");
    println!("The Rustic framework is currently providing keyboard handling, so try hitting some keys.");
    println!("The screen only supports ASCII - no snowmen: ☃☃☃!");
}
*/

fn demo_serial(kernel: &Kernel) {
    kernel.serial_write("Hello from the Rustic demo!\n");
    kernel.serial_write("The serial port supports full UTF-8 - ☃.\n");
}

/*
fn ticks(ms: usize) {
    let tick_count = unsafe {
        global_ticks += ms;
        global_ticks
    };

    machine().screen_save_cursor();
    machine().screen_save_attrib();
    machine().screen_cursor(machine().screen_cols() - 1, machine().screen_rows() - 1);
    machine().screen_attrib(util::colour::Colour::White, util::colour::Colour::Black);

    if tick_count % 1000 == 0 {
        if tick_count == 4000 {
            machine().screen_write_char('\\');
            unsafe { global_ticks = 0 };
        } else if tick_count == 3000 {
            machine().screen_write_char('-');
        } else if tick_count == 2000 {
            machine().screen_write_char('/');
        } else if tick_count == 1000 {
            machine().screen_write_char('|');
        }
    }

    machine().screen_restore_attrib();
    machine().screen_restore_cursor();
}
*/

#[no_mangle]
pub extern "C" fn main(_argc: i32, _: *const *const u8) -> ! {
    let mut kernel_state = Kernel::new();
    let mut locked_kernel = kernel_state.start();

    // Do some initial demo work by taking the lock for an extended period
    let mut kernel = locked_kernel.lock().unwrap();

    // Wipe screen, prepare for writing text.
    kernel.screen_attrib(util::colour::Colour::Black, util::colour::Colour::LightGray);
    kernel.screen_clear();
    kernel.screen_cursor(0, 0);
    kernel.screen_write("Hello from Rustic");

    // Demo messages.
    //demo_screen();
    demo_serial(&kernel);

    // Set up our timer handler.
    // kernel.machine().register_timer(ticks);

    // Welcome messages.
    //print!("This is an example where you just want to say... ");
    //println!("Hello, world!");

    // Set LEDs for fun.
    kernel.kb_leds(1);

    // Test serial port.
    kernel.serial_write("This is on the serial port, awesome!\n");

    // Test concurrency
    let mut cloned_kernel = Arc::clone(&locked_kernel);
    kernel.spawn_thread(move || {
        let guard = cloned_kernel.lock().unwrap();
        guard.serial_write("This is ANOTHER thread using the serial port, awesome!\n");
        drop(guard);

        loop { Kernel::reschedule(Arc::clone(&cloned_kernel)) };
    });

    let mut cloned_kernel = Arc::clone(&locked_kernel);
    kernel.spawn_thread(move || {
        let guard = cloned_kernel.lock().unwrap();
        guard.serial_write("This is a thread using the serial port, awesome!\n");
        drop(guard);

        loop { Kernel::reschedule(Arc::clone(&cloned_kernel)) };
    });

    drop(kernel);

    loop {
        Kernel::reschedule(Arc::clone(&locked_kernel));
    }
}