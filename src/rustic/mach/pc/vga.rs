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

use crate::kernel;

use crate::mach::{MachineState, IoPort, Screen, Mmio};

use crate::util::colour::Colour;

pub static COLS: u32 = 80;
pub static ROWS: u32 = 25;

static VGABASE: u32 = 0xB8000;

pub struct Vga {
    x: u32,
    y: u32,
    saved_x: u32,
    saved_y: u32,

    fg: Colour,
    bg: Colour,
    saved_fg: Colour,
    saved_bg: Colour,
}

impl Vga {
    pub fn new() -> Vga {
        Vga{
            x: 0,
            y: 0,
            saved_x: 0,
            saved_y: 0,
            fg: Colour::LightGray,
            bg: Colour::Black,
            saved_fg: Colour::LightGray,
            saved_bg: Colour::Black,
        }
    }

    pub fn init(&mut self) {
        // no-op
    }
}

impl<'a> Screen for MachineState<'a> {
    fn screen_clear(&self) {
        self.screen_fill(' ');
    }

    fn screen_fill(&self, with: char) {
        let real_char = safe_char(with);

        let field: u16 = (real_char as u16) | ((self.state.screen.bg as u16) << 12);
        let max = self.screen_rows() * self.screen_cols() * 2;

        // TODO: we can do this better - a memset?
        for offset in (0..max).step_by(2) {
            kernel().machine().mmio_write(VGABASE + offset, field);
        }
    }

    fn screen_cols(&self) -> u32 {
        return COLS;
    }

    fn screen_rows(&self) -> u32 {
        return ROWS;
    }

    fn screen_save_cursor(&mut self) {
        self.state.screen.saved_x = self.state.screen.x;
        self.state.screen.saved_y = self.state.screen.y;
    }

    fn screen_restore_cursor(&mut self) {
        let new_x = self.state.screen.saved_x;
        let new_y = self.state.screen.saved_y;
        self.screen_cursor(new_x, new_y);
    }

    fn screen_cursor(&mut self, x: u32, y: u32) {
        self.state.screen.x = x;
        self.state.screen.y = y;

        let position = (y * self.screen_cols()) + x;

        kernel().machine().outport(0x3D4, 0x0Fu8);
        kernel().machine().outport(0x3D5, (position & 0xFF) as u8);
        kernel().machine().outport(0x3D4, 0x0Eu8);
        kernel().machine().outport(0x3D5, ((position >> 8) & 0xFF) as u8);

        let curr: u16 = kernel().machine().mmio_read(VGABASE + (position * 2));
        let attr: u8 = (curr >> 8) as u8;
        if attr & 0xFu8 == 0 {
            // No foreground colour attribute for cursor location. Fix.
            kernel().machine().mmio_write(VGABASE + (position * 2), curr | ((Colour::LightGray as u16) << 8));
        }
    }

    fn screen_save_attrib(&mut self) {
        self.state.screen.saved_fg = self.state.screen.fg;
        self.state.screen.saved_bg = self.state.screen.bg;
    }

    fn screen_restore_attrib(&mut self) {
        self.state.screen.fg = self.state.screen.saved_fg;
        self.state.screen.bg = self.state.screen.saved_bg;
    }

    fn screen_attrib(&mut self, fg: Colour, bg: Colour) {
        self.state.screen.fg = fg;
        self.state.screen.bg = bg;
    }

    fn screen_write_char(&mut self, c: char) {
        let attr = ((self.state.screen.bg as u8) << 4) | (self.state.screen.fg as u8);

        match safe_char(c) {
            // newline
            0x0A => {
                self.state.screen.x = 0;
                self.state.screen.y += 1;
            },
            // carriage return
            0x0D => {
                self.state.screen.x = 0;
            },
            // tab
            0x09 => {
                self.state.screen.x += 4;
                self.state.screen.x -= self.state.screen.x % 4;
            },
            0x00 => {},
            glyph => {
                let offset = (self.state.screen.y * self.screen_cols()) + self.state.screen.x;
                let val = (glyph as u16) | ((attr as u16) << 8);
                kernel().machine().mmio_write(VGABASE + (offset * 2), val);

                self.state.screen.x += 1;
            }
        };

        if self.state.screen.x >= self.screen_cols() {
            self.state.screen.x = 0;
            self.state.screen.y += 1;
        }

        // TODO: scroll.
        if self.state.screen.y >= self.screen_rows() {
            self.state.screen.y = self.screen_rows() - 1;
        }
    }

    fn screen_write(&mut self, s: &str) {
        for c in s.chars() {
            if c == '\0' {
                continue;
            }

            self.screen_write_char(c);
        }
    }
}

fn safe_char(c: char) -> u8 {
    if c as u32 > 0xFF {
        0xDB
    } else {
        c as u8
    }
}
