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

type GdtTable = [GdtEntry; 16];

#[repr(C, packed)]
struct GdtRegister {
    limit: u16,
    addr: *const GdtTable,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access: u8,
    gran: u8,
    base_high: u8,
}

pub struct Gdt {
    table: GdtTable,
    reg: GdtRegister,
}


impl Gdt {
    pub fn new() -> Gdt {
        Gdt{table: [GdtEntry::new(); 16], reg: GdtRegister::new(0 as *const GdtTable)}
    }

    pub fn entry(&mut self, index: usize, base: u32, limit: u32, access: u8, gran: u8) {
        self.table[index] = GdtEntry::create(base, limit, access, gran);
    }

    pub fn load(&mut self, codeseg: u16, dataseg: u16, tlsemulseg: u16) {
        self.reg.addr = &self.table as *const GdtTable;
        load_gdt(&self.reg as *const GdtRegister, codeseg, dataseg, tlsemulseg);
    }
}

impl GdtRegister {
    fn new(gdt: *const GdtTable) -> GdtRegister {
        GdtRegister {
            addr: gdt,
            limit: (std::mem::size_of::<GdtTable>() + 1) as u16,
        }
    }
}

impl GdtEntry {
    fn new() -> GdtEntry {
        GdtEntry{limit_low: 0, base_low: 0, base_mid: 0, access: 0, gran: 0, base_high: 0}
    }

    fn create(base: u32, limit: u32, access: u8, gran: u8) -> GdtEntry {
        GdtEntry {
            limit_low: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_mid: ((base >> 16) & 0xFF) as u8,
            access: access,
            gran: gran,
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }
}

#[inline(never)]
fn load_gdt(reg: *const GdtRegister, codeseg: u16, dataseg: u16, tlsemulseg: u16) {
    // TODO: use RFC 2873 asm! syntax
    unsafe { llvm_asm!(" \
        lgdt ($0); \
        ljmp $$0x08, $$g; \
        g: \
        movw $2, %ax; \
        movw %ax, %ds; \
        movw %ax, %es; \
        movw %ax, %fs; \
        movw %ax, %ss; \
        movw $3, %ax; \
        movw %ax, %gs;" :: "r" (reg), "Ir" (codeseg), "Ir" (dataseg), "Ir" (tlsemulseg) : "ax"); }
}
