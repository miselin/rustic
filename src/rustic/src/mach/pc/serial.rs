/*
 * Copyright (c) 2013 Matthew Iselin
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

use crate::Kernel;

use crate::mach::{MachineState, IoPort, Serial};
use crate::mach::parity::Parity;

enum Registers {
    RxTx = 0,
    Inten = 1,
    IIFifo = 2,
    LCtrl = 3,
    MCtrl = 4,
    LStat = 5,
    MStat = 6,
    Scratch = 7,
}

static SERIAL_BASE: u16 = 0x3F8;

impl Serial for Kernel {
    fn serial_config(&self, baud: i32, dbits: i32, parity: Parity, sbits: i32) {
        // Disable IRQs.
        self.outport(SERIAL_BASE + Registers::Inten as u16, 0 as u8);

        // Enable DLAB to set the baud rate divisor.
        self.outport(SERIAL_BASE + Registers::LCtrl as u16, 0x80 as u8);

        // Set the divisor for the given baud rate.
        let divisor = 115200 / baud;
        self.outport(SERIAL_BASE + Registers::RxTx as u16, (divisor & 0xF) as u8);
        self.outport(SERIAL_BASE + Registers::Inten as u16, ((divisor & 0xF0) >> 8) as u8);

        // Set data/stop bits and parity, which will also clear DLAB.
        let meta: u8 =
            match dbits {
                5 => 0,
                6 => 0b01,
                7 => 0b10,
                _ => 0b11,
            } |
            match sbits {
                1 => 0,
                _ => 0b100,
            } |
            match parity {
                Parity::Odd   => 0b001000,
                Parity::Even  => 0b011000,
                Parity::Mark  => 0b101000,
                Parity::Space => 0b111000,
                _             => 0,
            };
        self.outport(SERIAL_BASE + Registers::LCtrl as u16, meta);

        // Enable and clear the FIFO.
        self.outport(SERIAL_BASE + Registers::IIFifo as u16, 0xC7 as u8);

        // Set RTS/DSR, and enable IRQs for if/when INTEN == 1.
        self.outport(SERIAL_BASE + Registers::MCtrl as u16, 0x0B as u8);
    }

    fn serial_write(&self, s: &str) {
        // Pass str as bytes to the serial line (UTF-8 can be read by the other
        // side, we don't have to do any transformations).
        for c in s.chars() {
            if c == '\0' {
                continue;
            }
            self.serial_write_char(c);
        }
    }

    fn serial_read_char(&self) -> char {
        // Wait until bytes are pending in the FIFO.
        loop {
            let status: u8 = self.inport(SERIAL_BASE + Registers::LStat as u16);
            if (status & 0x1) != 0 {
                break;
            }
        }

        let result: u8 = self.inport(SERIAL_BASE + Registers::RxTx as u16);
        result as char
    }

    fn serial_write_char(&self, c: char) {
        // Wait until we are permitted to write.
        loop {
            let status: u8 = self.inport(SERIAL_BASE + Registers::LStat as u16);
            if (status & 0x20) != 0 {
                break;
            }
        }

        // char -> UTF-8 conversion; we must use the length return value rather
        // than iteration as the number of bytes to write is not static.
        let mut bytes = [0u8; 6];
        let encoded = c.encode_utf8(&mut bytes);
        for index in 0..encoded.len() {
            self.outport(SERIAL_BASE + Registers::RxTx as u16, bytes[index]);
        }
    }
}
