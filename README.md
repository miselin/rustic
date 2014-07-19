# Rustic Operating System

This is a simple and small kernel for x86 systems, with the goal of
writing as much of the system as possible in Rust.

There are naturally components that are written Assembly, but the goal is to
write as much of the systema s possible in Rust.

It currently simply writes some text to the screen, displays a slow
spinning status indicator in the bottom right corner, and echoes
characters entered on the keyboard. Keyboard LEDs work too.

## Build Configuration

Create a file `config.mk` in the root directory of the repository.

Set `RUST_ROOT`, `LLVM_ROOT`, and `GCC_PREFIX` in this file to:
* `RUST_ROOT`: directory containing `bin/rustc`
* `LLVM_ROOT`: directory containing `bin/clang`
* `GCC_PREFIX`: prefix for GCC commands (eg, `/usr/bin/`)

`GCC_PREFIX` is prefixed to `ld`, so if you are using a cross-compiler
use the full prefix, eg `/usr/bin/i686-linux-elf-`.

If `config.mk` is not found, `/bin/rustc`, `/bin/clang`, `/usr/bin/ld`,
and `/usr/bin/gcc` will be used automatically.

If your system uses `genisoimage` instead of `mkisofs`, set the `MKISOFS`
variable to that as well.

To see what variables can be set in `config.mk` to adjust the Rustic build,
read the beginning of `Makefile`.

## Building on OSX

To build on OSX, you will need a build of Rust that has the 'i686-apple-darwin'
target enabled.

An example configuration for building on OSX is as follows:

```
TARGET=i686-apple-darwin
LLVM_ROOT=/usr

RUST_ROOT=$(HOME)/local
GCC_PREFIX=$(HOME)/local/xcompiler/bin/i686-elf-

USE_GCC_AS=true

ASDEFS=-Dmain=_main
```

You will need a GCC cross-compiler that targets `i686-elf`. You do not need a
libc or any system-specific support - only a working GCC and Binutils.

The ASDEFS variable is required as a hacky way to work around symbols with a
prefixed underscore emitted by the OSX toolchains.

USE_GCC_AS=true is required to use the GCC cross-compiler's assembler, rather
than `clang`. This is necessary as the default system `clang` on OSX is both
modified and assumes Mach-O object formats.

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

