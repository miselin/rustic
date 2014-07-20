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

#[cfg(plat_pc)]
mod pc;

#[cfg(plat_beagle)]
mod beagle;

#[cfg(plat_rpi)]
mod rpi;

// Pull in the 'state' module - this defines the State type as the correct
// private type for the relevant target machine.
mod state;

pub trait Machine {
    fn initialise(&mut self) -> bool;

    fn register_irq(&mut self, irq: uint, f: Rc<RefCell<Box<IrqHandler>>>, level_trigger: bool);
}

pub trait IrqHandler {
    fn irq(&mut self, irqnum: uint);
}

#[cfg(mach_kb)]
pub trait Keyboard {
    fn kb_leds(&mut self, state: u8);
}

#[cfg(mach_gpio)]
pub trait Gpio {
    fn gpio_write(&mut self, pin: uint, value: bool);
    fn gpio_read(&mut self, pin: uint) -> bool;
}

#[cfg(mach_ports)]
pub trait IoPort {
    fn outport<T: core::num::Int>(&self, port: u16, val: T);
    fn inport<T: core::num::Int + core::default::Default>(&self, port: u16) -> T;
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
