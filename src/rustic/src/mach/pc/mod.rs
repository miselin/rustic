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

use std;
use std::cell::RefCell;
use std::default::Default;
use std::rc::Rc;

use crate::mach::{IrqHandler, IrqController, HardwareTimer, Machine, MachineState, TimerHandlers, Keyboard, IoPort, Serial, Mmio};
use crate::mach::parity::Parity;

use crate::Kernel;

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
    timer_handlers: Vec<extern "Rust" fn(usize)>,
}

impl State {
    pub fn new() -> State {
        State{irq_ctlr: pic::Pic::new(),
              timer: pit::Pit::new(),
              keyboard: kb::PS2Keyboard::new(),
              screen: vga::Vga::new(),
              timer_handlers: Vec::with_capacity(16)}
    }
}

impl Machine for Kernel {
    fn mach_initialise(&mut self) -> bool {
        // Configure serial port.
        self.serial_config(115200, 8, Parity::NoParity, 1);

        // Bring up the PIC.
        self.mach.state.irq_ctlr = pic::Pic::new();
        self.init_irqs();

        // Bring up the PIT at 100hz.
        self.mach.state.timer = pit::Pit::new();
        self.init_timers(100);

        // Bring up the keyboard.
        self.mach.state.keyboard = kb::PS2Keyboard::new();
        self.kb_init();

        // Register the PIT and keyboard IRQs.
        // TODO: fix these borrows
        //self.register_irq(pit::Pit::irq_num(), &self.mach.state.timer, true);
        //self.register_irq(kb::PS2Keyboard::irq_num(), &self.mach.state.keyboard, true);

        // Set up the VGA screen.
        self.mach.state.screen.init();

        self.mach.initialised = true;

        self.mach.initialised
    }
}

impl<'a> TimerHandlers for Kernel {
    fn register_timer(&mut self, f: extern "Rust" fn(usize)) {
        self.mach.state.timer_handlers.push(f);
    }

    fn timer_fired(&mut self, ms: usize) {
        for h in self.mach.state.timer_handlers.iter() {
            let handler = *h;
            handler(ms);
        }
    }
}

impl<'a> IoPort for Kernel {
    fn outport<T>(&self, port: u16, val: T) {
        unsafe {
            llvm_asm!("out $0, $1" :: "{ax}" (val), "N{dx}" (port));
        }
    }

    fn inport<T: Default>(&self, port: u16) -> T {
        unsafe {
            let mut val: T;
            llvm_asm!("in $1, $0" : "={ax}" (val) : "N{dx}" (port));
            val
        }
    }
}

impl<'a> Mmio for Kernel {
    fn mmio_write<T>(&self, address: u32, val: T) {
        let ptr = address as *mut T;
        unsafe { std::ptr::write(ptr, val) };
    }

    fn mmio_read<T>(&self, address: u32) -> T {
        let ptr = address as *const T;
        unsafe { std::ptr::read(ptr) }
    }
}
