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

.set ALIGN,    1<<0
.set MEMINFO,  1<<1
.set FLAGS,    ALIGN | MEMINFO
.set MAGIC,    0x1BADB002
.set CHECKSUM, -(MAGIC + FLAGS)

.section .multiboot
.align 4
.long MAGIC
.long FLAGS
.long CHECKSUM

.section .data
stack_bottom:
.skip 131072 # 128 KiB
stack_top:

.section .init
.global _start
.type _start, @function
_start:
    cli

    // TODO: don't use %gs:30 - fix the target?
    // This is required as Rust's main() at least checks to see
    // if the stack needs to be expanded in its prologue. If it
    // decides it does, it calls __morestack.
    // Still need to identify a much better solution than this.
    // Set up stack.
    movl $stack_top, %esp
    movl $stack_bottom, %gs:0x30

    // Call into the C code proper.
    call _cstart

    // Halt the system completely if we ever return from the C
    // glue code.
    cli
    hlt
    jmp .

