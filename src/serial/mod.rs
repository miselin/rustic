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

use core;
use io;

pub enum Parity {
    NoParity,
    Odd,
    Even,
    Mark,
    Space
}

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

pub fn config(baud: int, dbits: int, parity: Parity, sbits: int) {
    // Disable IRQs.
    io::outport(SERIAL_BASE + Inten as u16, 0 as u8);

    // Enable DLAB to set the baud rate divisor.
    io::outport(SERIAL_BASE + LCtrl as u16, 0x80 as u8);

    // Set the divisor for the given baud rate.
    let divisor = 115200 / baud;
    io::outport(SERIAL_BASE + RxTx as u16, (divisor & 0xF) as u8);
    io::outport(SERIAL_BASE + Inten as u16, ((divisor & 0xF0) >> 8) as u8);

    // Set data/stop bits and parity, which will also clear DLAB.
    let meta =
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
            Odd   => 0b001000,
            Even  => 0b011000,
            Mark  => 0b101000,
            Space => 0b111000,
            _     => 0,
        };
    io::outport(SERIAL_BASE + LCtrl as u16, meta as u8);

    // Enable and clear the FIFO.
    io::outport(SERIAL_BASE + IIFifo as u16, 0xC7 as u8);

    // Set RTS/DSR, and enable IRQs for if/when INTEN == 1.
    io::outport(SERIAL_BASE + MCtrl as u16, 0x0B as u8);
}

fn writechar(c: u8) {
    // Wait until we are permitted to write.
    loop {
        let status: u8 = io::inport(SERIAL_BASE + LStat as u16);
        if (status & 0x20) != 0 {
            break;
        }
    }

    io::outport(SERIAL_BASE + RxTx as u16, c);
}

pub fn write(s: &str) {
    // Pull out the buffer length from the str
    let (_, buflen): (*u8, uint) = unsafe {
        core::intrinsics::transmute(s)
    };

    let mut index = 0;
    while index < buflen {
        writechar(s[index]);

        index += 1;
    }
}

pub fn read() -> char {
    // Wait until bytes are pending in the FIFO.
    loop {
        let status: u8 = io::inport(SERIAL_BASE + LStat as u16);
        if (status & 0x1) != 0 {
            break;
        }
    }

    let result: u8 = io::inport(SERIAL_BASE + RxTx as u16);
    result as char
}
