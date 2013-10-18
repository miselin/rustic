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


/// Required by Rust. \todo Needs to be implemented!
void __morestack() {}

/**
 * Ultra stupid malloc for quick testing (replace me with dlmalloc or write me
 * in rust!)
 */
unsigned int base = 0x200000;
void *malloc(unsigned int len) {
    unsigned int ret = base;
    base += len + sizeof(unsigned int);

    // Align next allocation to 4-byte boundary.
    if(base % 4)
        base += 4 - (base % 4);

    *(unsigned int*)base = len;

    return (void *)(ret + sizeof(unsigned int));
}

/// Even more naive free().
void free(void *p) {}

// TODO(eddyb) reimplement these in Rust ASAP.
void *memcpy(void *dst, const void *src, unsigned int count) {
    unsigned char *d = dst;
    const unsigned char *s = src;
    while(count--)
        *d++ = *s++;
    return dst;
}

void *realloc(void *old, unsigned int len) {
    void *new_mem = malloc(len);
    memcpy(new_mem, old, *((unsigned int*)old - 1));
    return new_mem;
}

/// Rust entry point.
extern int main(int, char **);

/// Entry point from the assembly code startup code.
void _cstart() {
    main(0, 0);
}

