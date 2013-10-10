# Rustic Operating System

This is a simple and small kernel for x86 systems, with the goal of
writing as much of the system as possible in Rust.

There are naturally components that are written in C and Assembly,
including general runtime support for Rust itself.

It currently simply writes some text to the screen and loops forever.

## Build Configuration

Create a file 'config.mk' in the root directory of the repository.

Set `RUST_ROOT`, `LLVM_ROOT`, and `GCC_PREFIX` in this directory to:
`RUST_ROOT`: directory containing `bin/rustc`
`LLVM_ROOT`: directory containing `bin/clang`
`GCC_PREFIX`: prefix for GCC commands (eg, `/usr/bin/`)

`GCC_PREFIX` is prefixed to `ld`, so if you are using a cross-compiler
use the full name, eg `/usr/bin/i686-linux-elf-`.

If your system uses `genisoimage` instead of `mkisofs`, set the `MKISOFS`
variable to that as well.

## Running the Kernel

The kernel will be output in the `build` directory, and can be run with
`qemu -kernel build/kernel`.

An ISO is also generated, but this does not currently boot correctly.

## License

See the LICENSE file in the root of the repository for the licensing
terms for Rustic.

