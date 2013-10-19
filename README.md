# Rustic Operating System

This is a simple and small kernel for x86 systems, with the goal of
writing as much of the system as possible in Rust.

There are naturally components that are written in C and Assembly,
including general runtime support for Rust itself.

It currently simply writes some text to the screen, displays a slow
spinning status indicator in the bottom right corner, and echoes
characters entered on the keyboard. Keyboard LEDs work too.

## Build Configuration

Make sure you have run `git submodule --init` before you build. You
may also want to run `git submodule update` to ensure `rust-core`
is fully up-to-date.

Create a file `config.mk` in the root directory of the repository.

Set `RUST_ROOT`, `LLVM_ROOT`, and `GCC_PREFIX` in this file to:
* `RUST_ROOT`: directory containing `bin/rustc`
* `LLVM_ROOT`: directory containing `bin/clang`
* `GCC_PREFIX`: prefix for GCC commands (eg, `/usr/bin/`)

`GCC_PREFIX` is prefixed to `ld` and `gcc`, so if you are using a cross-compiler
use the full name, eg `/usr/bin/i686-linux-elf-`.

If `config.mk` is not found, `/bin/rustc`, `/bin/clang`, `/usr/bin/ld`,
and `/usr/bin/gcc` will be used automatically.

If your system uses `genisoimage` instead of `mkisofs`, set the `MKISOFS`
variable to that as well.

## Running the Kernel

The kernel will be output in the `build` directory, and can be run with
`qemu-system-i386 -kernel build/kernel -serial stdio`.

An ISO is also generated, but this does not currently boot correctly.

## License

See the LICENSE file in the root of the repository for the licensing
terms for Rustic.

## Other Kernels

There are a few other Rust kernels out there that are worth looking at:
* https://github.com/pczarn/rustboot
* https://github.com/LeoTestard/Quasar
* https://github.com/cmr/cmoss

The Rust OSDev community hangs out in #rust-osdev on irc.mozilla.org.

