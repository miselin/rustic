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
#![feature(globs)]

use rustic::{Kernel, println, printlnto, kernel_mut};

use rustic::arch;
use rustic::mach;

use rustic::arch::{Architecture, Threads};
use rustic::mach::{Keyboard, Screen, TimerHandlers, serial};
use rustic::util;

static mut global_ticks: usize = 0;

/*
fn demo_screen() {
    println!("Hello from the Rustic demo!");
    println!("The Rustic framework is currently providing keyboard handling, so try hitting some keys.");
    println!("The screen only supports ASCII - no snowmen: ☃☃☃!");
}
*/

fn demo_serial(kernel: &mut Kernel) {
    printlnto!(serial, "Hello from the Rustic demo!");
    printlnto!(serial, "The serial port supports full UTF-8 - ☃.");
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

// Demo - shows off some of the features Rustic can provide.
#[no_mangle]
pub fn run(kernel: &mut Kernel) {
    // Wipe screen, prepare for writing text.
    kernel.machine_mut().screen_attrib(util::colour::Colour::LightGray, util::colour::Colour::Black);
    kernel.machine_mut().screen_clear();
    kernel.machine_mut().screen_cursor(0, 0);

    // Demo messages.
    //demo_screen();
    demo_serial(kernel);

    // Set up our timer handler.
    // kernel.machine().register_timer(ticks);

    // Welcome messages.
    //print!("This is an example where you just want to say... ");
    //println!("Hello, world!");

    // Set LEDs for fun.
    kernel.machine_mut().kb_leds(1);

    // Test serial port.
    printlnto!(serial, "This is on the serial port, awesome!");

    // Demo a thread printing a message.
    /*
    spawn(proc() {
        println!("Hello, from a thread!");
    });
    */

    loop {
      kernel.architecture().wait_for_event();
      kernel.architecture_mut().reschedule();
    }
}
