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
 #![feature(phase, macro_rules)]

#![crate_name = "rustic-example"]
#![desc = "Rustic Embedded Framework Example"]
#![license = "ISC"]
#![comment = "A demonstration of an embedded application that uses the Rustic framework."]

#[phase(plugin, link)] extern crate rustic;

pub use rustic::*;

use rustic::mach::{Keyboard, Screen, TimerHandlers};

static mut global_ticks: uint = 0;

fn demo_screen() {
    println!("Hello from the Rustic demo!");
    println!("The Rustic framework is currently providing keyboard handling, so try hitting some keys.");
    println!("The screen only supports ASCII - no snowmen: ☃☃☃!");
}

fn demo_serial() {
    printlnto!(serial, "Hello from the Rustic demo!");
    printlnto!(serial, "The serial port supports full UTF-8 - ☃.");
}

fn ticks(ms: uint) {
    let tick_count = unsafe {
        global_ticks += ms;
        global_ticks
    };

    machine().screen_save_cursor();
    machine().screen_save_attrib();
    machine().screen_cursor(machine().screen_cols() - 1, machine().screen_rows() - 1);
    machine().screen_attrib(util::colour::White, util::colour::Black);

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

// Demo - shows off some of the features Rustic can provide.
#[no_mangle]
pub fn run() {
    // Wipe screen, prepare for writing text.
    machine().screen_attrib(util::colour::LightGray, util::colour::Black);
    machine().screen_clear();
    machine().screen_cursor(0, 0);

    // Demo messages.
    demo_screen();
    demo_serial();

    // Set up our timer handler.
    machine().register_timer(ticks);

    // Welcome messages.
    print!("This is an example where you just want to say... ");
    println!("Hello, world!");

    // Set LEDs for fun.
    machine().kb_leds(1);

    // Test serial port.
    printlnto!(serial, "This is on the serial port, awesome!");

    loop {
      architecture().wait_for_event();
    }
} 
