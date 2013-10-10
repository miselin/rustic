/*
 * Rustic Operating System Kernel Linker Script.
 * Kernel starts at 0x100000.
 */

OUTPUT_FORMAT(elf32-i386)

ENTRY(_start)
SECTIONS {
    . = 0x100000;

    .init : {
        . = ALIGN(4096);
        init = .; _init = .; __init = .;
        *(.multiboot*);
        *(.init*);
        init_end = .; _init_end = .; __init_end = .;
    }

    .text : {
        . = ALIGN(4096);
        code = .; _code = .; __code = .;
        *(.text*);
    }

    .data : {
        . = ALIGN(4096);
        *(.data*);
        *(.rodata*);
    }

    .bss : {
        . = ALIGN(4096);
        *(.bss*);
    }

    . = ALIGN(4096);
    end = .; _end = .; __end = .;
}
