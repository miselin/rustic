# Rustic Embedded Framework

Rustic is a framework that provides a platform upon which to build embedded
applications. Rustic offers abstractions that make performing common tasks such
as MMIO, working with GPIO pins, and handling timers easy, amongst many other
helpful features.

There are naturally components that are written Assembly, but the goal is to
write as much of the framework as possible in Rust.

The repository contains an example application in `src/example`,
which writes text to the screen and serial port, and shows a spinner in the
lower right corner of the screen.

The initial goal is to support:
* i386 PC
* ARMv6 Rasberry Pi
* ARMv7 BeagleBoard

Rustic currently provides abstractions for:
* A VGA console (via `rustic::mach::Screen` trait)
* A serial line (via `rustic::mach::Serial` trait)
* A keyboard (via `rustic::mach::Keyboard` trait)
* Timers (via `rustic::mach::TimerHandlers` trait)
 * Currently, timers merely call a function every N milliseconds, where N is decided by the machine-specific implementation.
* GPIO on supported platforms (via `rustic::mach::Gpio` trait)
* MMIO (via `rustic::mach::Mmio` trait)
 * This can be used to write to arbitrary addresses and should be used with
 care.
* Custom IRQ handling (via `rustic::mach::IrqHandler` trait)

NOTE: IRQ handling is currently broken, but it'll be back.

## Building Rustic

Rustic builds using Cargo:

```
$ cargo +nightly -Z unstable-options build --out-dir=build
```

## Running the Kernel

After the command above, the example kernel will be output in the `build`
directory, and can be run with:

```
$ qemu-system-i386 -kernel build/rustic-example -serial stdio
```

The `Makefile` generates an ISO image from this binary that can be run on bare
metal or your favorite virtualisation platform.

```
$ make
$ ls build/rustic.iso
```

## Support

Please open issues at https://github.com/miselin/rustic for any issues you
may come across.

Pull requests are also welcome.

## License

See the LICENSE file in the root of the repository for the licensing
terms for Rustic.

## Other Kernels

There are a few other Rust kernels out there that are worth looking at:
* https://github.com/pczarn/rustboot
* https://github.com/LeoTestard/Quasar
* https://github.com/cmr/cmoss

The Rust OSDev community hangs out in #rust-osdev on irc.mozilla.org.
