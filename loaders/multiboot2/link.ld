ENTRY(start)

SECTIONS 
{
    . = 0x100000;

    __load_start = .;

    .text ALIGN(4K) : {
        KEEP(*(.boot))
        KEEP(*(.text*))
    }

    .data ALIGN (4K) : {
        *(.got)
        *(.rodata*)
        *(.data*)
    }

    __bss_start = ALIGN(4K);
    .bss ALIGN (4K) (NOLOAD) : {
        *(COMMON)
        *(.bss*)
    }

    .stack ALIGN (4K) (NOLOAD) : {
        __stack_start = .;
        . += 4 * 4096;
        __stack_end = .;
    }

    .heap ALIGN (4K) (NOLOAD) : {
        __heap_start = .;
        . += 4 * 4096;
        __heap_end = .;
    }

    __load_end = ALIGN(4K);

    /DISCARD/ : {
       *(.eh_frame_hdr)
       *(.eh_frame)
    }
}