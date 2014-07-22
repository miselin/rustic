
/* Loosely based on libstd in mainline Rust, but specifically written for Rustic. */

#![crate_name = "std"]
#![unstable]
#![comment = "libstd for the Rustic framework."]
#![license = "ISC"]
#![crate_type = "rlib"]

#![feature(macro_rules, globs, managed_boxes, linkage)]
#![feature(default_type_params, phase, lang_items, unsafe_destructor)]

#![allow(deprecated)]
#![allow(missing_doc)]

// This is libstd - don't reference libstd.
#![no_std]

// Note: remember to update RUST_LIBS in Makefile when adding more extern
// crates here.

extern crate alloc;
extern crate unicode;
#[phase(plugin, link)] extern crate core;
extern crate core_collections = "collections";
extern crate core_rand = "rand";
// extern crate core_sync = "sync";
extern crate rlibc;

pub use core::any;
pub use core::bool;
pub use core::cell;
pub use core::clone;
pub use core::cmp;
pub use core::default;
pub use core::finally;
pub use core::intrinsics;
pub use core::iter;
pub use core::kinds;
pub use core::mem;
pub use core::ops;
pub use core::ptr;
pub use core::raw;
pub use core::simd;
pub use core::tuple;
pub use core::unit;
pub use core::ty;
pub use core::result;
pub use core::option;

pub use alloc::boxed;
#[deprecated = "use boxed instead"]
pub use owned = boxed;

pub use alloc::rc;

pub use core_collections::slice;
pub use core_collections::str;
pub use core_collections::string;
pub use core_collections::vec;

pub use unicode::char;

// pub use core_sync::comm;

pub use int = core::int;
pub use i8 = core::i8;
pub use i16 = core::i16;
pub use i32 = core::i32;
pub use i64 = core::i64;

pub use uint = core::uint;
pub use u8 = core::u8;
pub use u16 = core::u16;
pub use u32 = core::u32;
pub use u64 = core::u64;

pub use f32 = core::f32;
pub use f64 = core::f64;

pub use num = core::num;

pub use collections = core_collections;

pub use fmt = core::fmt;

pub mod prelude {
    pub use core::prelude::*;

    // Things that core::prelude does not export that we want.
    pub use boxed::Box;
    pub use string::String;
    pub use vec::Vec;
}
