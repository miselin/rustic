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

#![macro_use]

pub mod mem;
pub mod libc;
pub mod io;

pub mod colour {
    #[derive(Copy, Clone)]
    pub enum Colour {
        Black       = 0,
        Blue        = 1,
        Green       = 2,
        Cyan        = 3,
        Red         = 4,
        Pink        = 5,
        Brown       = 6,
        LightGray   = 7,
        DarkGray    = 8,
        LightBlue   = 9,
        LightGreen  = 10,
        LightCyan   = 11,
        LightRed    = 12,
        LightPink   = 13,
        Yellow      = 14,
        White       = 15,
    }
}

// format! -> format and return string

/*

#[macro_export]
macro_rules! format(
    ($($arg:tt)*) => (
        format_args!($($arg)*)
    )
);

// Define println! and print! macros, which write to Screen.

#[macro_export]
macro_rules! print(
    ($fmt:expr) => (
        printto!(mach::screen, $fmt)
    );
    ($fmt:expr, $($arg:tt)*) => (
        printto!(mach::screen, $fmt $($arg)*)
    )
);

#[macro_export]
macro_rules! println(
    ($fmt:expr) => (
        printlnto!(mach::screen, $fmt)
    );
    ($fmt:expr, $($arg:tt)*) => (
        printlnto!(mach::screen, $fmt $($arg)*)
    )
);

// printlnto! and printto! macros take a method to call, in the form:
// fn f<T: Trait>(m: &mut rustic::mach::MachineState, s: &str)

#[macro_export]
macro_rules! printto(
    ($kernel:ident, $f:ident, $fmt:expr) => ({
        $f(kernel_mut().machine_mut(), $fmt)
    });
    ($f:ident, $fmt:expr, $($arg:tt)*) => ({
        let x = format!($fmt $($arg)*);
        $f(kernel_mut().machine_mut(), x.as_slice())
    })
);

#[macro_export]
macro_rules! printlnto(
    ($f:ident, $fmt:expr) => ({
        $f(kernel_mut().machine_mut(), concat!($fmt, "\n"))
    });
    ($f:ident, $fmt:expr, $($arg:tt)*) => ({
        let x = format!(concat!($fmt, "\n") $($arg)*);
        $f(kernel_mut().machine_mut(), x.as_slice())
    });
);

*/
