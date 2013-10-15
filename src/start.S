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
.align 4
stack_bottom:
.skip 131072 # 128 KiB
stack_top:

.align 4
tlsemul:
.skip 1024

// Initial GDT, before we load the real one later.
.align 4
initial_gdt:
    .long 0x0
    .long 0x0

    # Code.
    .word 0xFFFF
    .word 0x0
    .byte 0x00
    .byte 0x98
    .byte 0xCF
    .byte 0x00

    # Data.
    .word 0xFFFF
    .word 0x0
    .byte 0x00
    .byte 0x92
    .byte 0xCF
    .byte 0x00

    # Temporary TLS emulation segment
    .word 0xFFFF
    .word 0x0
    .byte 0x00
    .byte 0x92
    .byte 0xCF
    .byte 0x00
initial_gdt_end:

.align 4
gdtr:
    .word 0x21
    .long 0x0

.section .init
.global _start
.type _start, @function
_start:
    cli

    mov $gdtr, %esi
    mov $initial_gdt, %edi

    // Fix up GDTR and the TLS emulation segment.
    movl %edi, 2(%esi)

    mov $tlsemul, %edx
    movw %dx, 26(%edi)
    shr $16, %edx
    movb %dl, 28(%edi)

    // Load GDT
    lgdt (%esi)

    // Load new CS
    jmp $0x08, $.newgdt
    .newgdt:

    // Load new auxilliary registers.
    mov $0x10, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %fs
    mov %ax, %ss

    // TODO: don't use %gs:30 - fix the target?
    // This is required as Rust's main() at least checks to see
    // if the stack needs to be expanded in its prologue. If it
    // decides it does, it calls __morestack.
    mov $0x18, %ax
    mov %ax, %gs

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
