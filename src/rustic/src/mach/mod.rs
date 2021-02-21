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

use std::cell::RefCell;
use std::rc::Rc;

use crate::util::colour;

#[cfg(feature="plat_pc")]
mod pc;

#[cfg(feature="plat_beagle")]
mod beagle;

#[cfg(feature="plat_rpi")]
mod rpi;

// Pull in the 'state' module - this defines the State type as the correct
// private type for the relevant target machine.
mod state;

pub mod parity {
    pub enum Parity {
        NoParity,
        Odd,
        Even,
        Mark,
        Space
    }
}

pub trait Machine {
    fn mach_initialise(&mut self) -> bool;
}

pub trait IrqController<'a> {
    fn init_irqs(&mut self);

    fn register_irq(&mut self, irq: usize, f: &'a dyn IrqHandler, level_trigger: bool);

    // Mask or unmask the given IRQ using the machine-specific implementation.
    fn enable_irq(&self, irq: usize);
    fn disable_irq(&self, irq: usize);

    // Mark end of interrupt for the IRQ controller
    fn eoi(&self, irq: usize);
}

pub trait IrqHandler {
    fn irq(&self, irqnum: usize);
}

pub trait Keyboard {
    fn kb_init(&mut self);
    fn kb_leds(&mut self, state: u8);
}

pub trait HardwareTimer {
    fn init_timers(&mut self, freq: usize);
}

pub trait TimerHandlers {
    fn register_timer(&mut self, f: extern "Rust" fn(usize));
    fn timer_fired(&mut self, ticks: usize);
}

pub trait Gpio {
    fn gpio_write(&mut self, pin: u32, value: bool);
    fn gpio_read(&mut self, pin: u32) -> bool;
}

pub trait IoPort {
    fn outport<T>(&self, port: u16, val: T);
    fn inport<T: std::default::Default>(&self, port: u16) -> T;
}

pub trait Serial<'a> {
    fn serial_config(&'a self, baud: i32, data_bits: i32, parity: parity::Parity, stop_bits: i32);
    fn serial_write(&self, s: &str);
    fn serial_read_char(&self) -> char;
    fn serial_write_char(&self, c: char);
}

pub trait Screen {
    fn screen_clear(&self);
    fn screen_fill(&self, with: char);

    fn screen_cols(&self) -> u32;
    fn screen_rows(&self) -> u32;

    fn screen_save_cursor(&mut self);
    fn screen_restore_cursor(&mut self);
    fn screen_cursor(&mut self, x: u32, y: u32);

    fn screen_save_attrib(&mut self);
    fn screen_restore_attrib(&mut self);
    fn screen_attrib(&mut self, fg: colour::Colour, bg: colour::Colour);

    fn screen_write_char(&mut self, c: char);
    fn screen_write(&mut self, s: &str);
}

pub trait Mmio {
    fn mmio_write<T>(&self, address: u32, val: T);
    fn mmio_read<T>(&self, address: u32) -> T;
}

pub struct MachineState<'a> {
    initialised: bool,
    state: state::State<'a>,
}

impl<'a> MachineState<'a> {
    fn new() -> MachineState<'a> {
        MachineState{initialised: false, state: state::State::new()}
    }
}

pub fn create<'a>() -> MachineState<'a> {
    MachineState::new()
}
