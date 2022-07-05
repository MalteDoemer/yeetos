ENTRY(kernel_main)

KERNEL_BASE = 0xfffff00000000000;

SECTIONS 
{
    . = 0x100000;
    . += KERNEL_BASE;
    
    /********* EXECUTABLE CODE *********/

    __text_start = ALIGN(4K);
    .text ALIGN(4K) : {
        *(.text*)
    }
    __text_end = ALIGN(4K);


    /********** READONLY DATA **********/

    __rodata_start = ALIGN(4K);
    .rodata ALIGN(4K) : {
        *(.got)
        *(.rodata*)
        . = ALIGN(4K);
    }
    __rodata_end = ALIGN(4K);
    

    /********** MUTABLE DATA ***********/

    __data_start = ALIGN(4K);
    .data ALIGN (4K) : {
        *(.data*)
    }
    __data_end = ALIGN(4K);


    /************ ZERO DATA ************/

    __bss_start = ALIGN(4K);
    .bss ALIGN (4K) (NOLOAD) : {
        *(COMMON)
        *(.bss*)
    }
    __bss_end = ALIGN(4K);
   
    /************** Trash **************/

    /DISCARD/ : {
       *(.eh_frame_hdr)
       *(.eh_frame)
    }
}