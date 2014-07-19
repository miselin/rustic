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
    }

    .rodata : {
        . = ALIGN(4096);
        *(.rodata*);
    }

    .bss : {
        . = ALIGN(4096);
        *(.bss*);
    }

    /DISCARD/ : { *(.comment) *(.eh_frame) *(.debug_abbrev) *(.debug_info)
        *(.debug_line) *(.debug_loc) *(.debug_pubnames) *(.debug_aranges)
        *(.debug_str) *(.debug_macinfo) *(.debug_ranges)}

    . = ALIGN(4096);
    end = .; _end = .; __end = .;
}
