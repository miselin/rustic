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

/// Rust entry point.
extern int main(int, char **);

/// Prototype for abort(), for the attribute.
void abort() __attribute__((noreturn));

/// Poke a byte into memory at the given address.
void poke(unsigned int addr, unsigned char val) {
    *((unsigned char *) addr) = val;
}

/// Read a byte from memory at the given address.
unsigned char peek(unsigned int addr) {
    return *((unsigned char *) addr);
}

/// Required by Rust. \todo Needs to be implemented!
void __morestack() {
}

/// Required by zero.rs, and needs to be more noisy.
void abort() {
    while(1) asm volatile("cli;hlt");
}

/**
 * Ultra stupid malloc for quick testing (replace me with dlmalloc or write me
 * in rust!)
 */
unsigned int base = 0x200000;
void *malloc(unsigned int len) {
    unsigned int ret = base;
    base += len;

    // Align next allocation to 4-byte boundary.
    if(base % 4)
        base += 4 - (base % 4);

    return (void *) ret;
}

/// Even more naive free()
void free(void *p) {
}

/// Entry point from the assembly code startup code.
void _cstart() {
    main(0, 0);
}

