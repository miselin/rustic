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

use core;

use core::cell::RefCell;

use alloc::boxed::Box;
use alloc::rc::Rc;

use util::colour;

#[cfg(plat_pc)]
mod pc;

#[cfg(plat_beagle)]
mod beagle;

#[cfg(plat_rpi)]
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
    fn initialise(&mut self) -> bool;

    fn register_irq(&mut self, irq: uint, f: Rc<RefCell<Box<IrqHandler>>>, level_trigger: bool);
}

pub trait IrqHandler {
    fn irq(&mut self, irqnum: uint);
}

pub trait Keyboard {
    fn kb_leds(&mut self, state: u8);
}

pub trait TimerHandlers {
    fn register_timer(&mut self, extern "Rust" fn(uint));
    fn timer_fired(&mut self, uint);
}

pub trait Gpio {
    fn gpio_write(&mut self, pin: uint, value: bool);
    fn gpio_read(&mut self, pin: uint) -> bool;
}

pub trait IoPort {
    fn outport<T: core::num::Int>(&self, port: u16, val: T);
    fn inport<T: core::num::Int + core::default::Default>(&self, port: u16) -> T;
}

pub trait Serial {
    fn serial_config(&self, baud: int, data_bits: int, parity: parity::Parity, stop_bits: int);
    fn serial_write(&self, s: &str);
    fn serial_read_char(&self) -> char;
    fn serial_write_char(&self, c: char);
}

pub trait Screen {
    fn screen_clear(&self);
    fn screen_fill(&self, with: char);

    fn screen_cols(&self) -> uint;
    fn screen_rows(&self) -> uint;

    fn screen_save_cursor(&mut self);
    fn screen_restore_cursor(&mut self);
    fn screen_cursor(&mut self, x: uint, y: uint);

    fn screen_save_attrib(&mut self);
    fn screen_restore_attrib(&mut self);
    fn screen_attrib(&mut self, fg: colour::Colour, bg: colour::Colour);

    fn screen_write_char(&mut self, c: char);
    fn screen_write(&mut self, s: &str);
}

pub trait Mmio {
    fn mmio_write<T>(&self, address: uint, val: T);
    fn mmio_read<T>(&self, address: uint) -> T;
}

pub struct MachineState {
    initialised: bool,
    state: state::State,
}

impl MachineState {
    fn new() -> MachineState {
        MachineState{initialised: false, state: state::State::new()}
    }
}

pub fn create() -> Box<MachineState> {
    box MachineState::new()
}

// Helpers.

pub fn screen<T: Screen>(m: &mut T, s: &str) {
    m.screen_write(s);
}

pub fn serial<T: Serial>(m: &mut T, s: &str) {
    m.serial_write(s);
}
