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

use core::default::Default;
use core::cell::RefCell;

use alloc::boxed::Box;
use alloc::rc::Rc;

use mach::{IrqHandler, Machine, MachineState, Keyboard, IoPort, Serial, parity};

mod kb;
mod pic;
mod pit;
mod serial;
mod vga;

pub struct State {
    irq_ctlr: pic::Pic,
    timer: pit::Pit,
    keyboard: kb::PS2Keyboard,
    screen: vga::Vga,
}

impl State {
    pub fn new() -> State {
        State{irq_ctlr: pic::Pic::new(),
              timer: pit::Pit::new(),
              keyboard: kb::PS2Keyboard::new(),
              screen: vga::Vga::new()}
    }
}

impl Machine for MachineState {
    fn initialise(&mut self) -> bool {
        // Configure serial port.
        self.serial_config(115200, 8, parity::NoParity, 1);
        self.serial_write("Rustic starting...\n");

        // Bring up the PIC.
        self.state.irq_ctlr = pic::Pic::init();

        // Bring up the PIT at 100hz.
        self.state.timer = pit::Pit::init(100);

        // Bring up the keyboard.
        self.state.keyboard = kb::PS2Keyboard::init();

        // Register the PIT and keyboard IRQs.
        let timer_irq = Rc::new(RefCell::new(box self.state.timer as Box<IrqHandler>));
        let keyboard_irq = Rc::new(RefCell::new(box self.state.keyboard as Box<IrqHandler>));
        self.register_irq(pit::Pit::irq_num(), timer_irq, true);
        self.register_irq(kb::PS2Keyboard::irq_num(), keyboard_irq, true);

        // Set up the VGA screen.
        self.state.screen.init();

        self.initialised = true;

        self.initialised
    }

    fn register_irq(&mut self, irq: uint, f: Rc<RefCell<Box<IrqHandler>>>, level_trigger: bool) {
        self.state.irq_ctlr.register(irq, f, level_trigger);
        self.state.irq_ctlr.enable(irq);
    }
}

impl Keyboard for MachineState {
    fn kb_leds(&mut self, state: u8) {
        self.state.keyboard.leds(state)
    }
}

impl IoPort for MachineState {
    fn outport<T: core::num::Int>(&self, port: u16, val: T) {
        unsafe {
            asm!("out $0, $1" :: "{ax}" (val), "N{dx}" (port));
        }
    }

    fn inport<T: core::num::Int + core::default::Default>(&self, port: u16) -> T {
        unsafe {
            let mut val: T;
            asm!("in $1, $0" : "={ax}" (val) : "N{dx}" (port));
            val
        }
    }
}
