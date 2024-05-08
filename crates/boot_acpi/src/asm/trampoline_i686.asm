.global ap_trampoline
.global ap_trampoline_end

ap_trampoline_dest = 0x8000

.section .text

.code16

// Note: we can only use relative addresses in ap_trampoline since
// it is relocated to ap_trampoline_dest (this includes addresses in the GDT)
ap_trampoline:
    cli
    cld

    /* ------------------------------------------------------------------------ *
     *  Directly after startup every AP is in 16-bit mode. This means we now    *
     *  need to enable protected mode.                                          *
     *                                                                          *
     *  Enabling protected mode is done by first loading a 16-bit GDT, then we  *
     *  enable protected mode using cr0 and then perform a ljmp to reload %cs.  *
     *  After that we load the 32-bit GDT defined in boot.s                     *
     * ------------------------------------------------------------------------ */

    // clear the data segment register to zero
    xorw %ax, %ax
    movw %ax, %dx

    // load gdt
    lgdtl (temp_gdt_ptr - ap_trampoline) + ap_trampoline_dest

    // enable protected mode
    movl %cr0, %eax
    orl  $1, %eax                  // set protected mode enable bit
    movl %eax, %cr0

    // load code segment into %cs with a long jump
    ljmpl $0x08, $prot_mode

.align 8
temp_gdt:
    .long 0, 0                     // null descriptor
    .long 0x0000FFFF, 0x00CF9A00   // 16 bit code descriptor
    .long 0x0000FFFF, 0x008F9200   // 16 bit data descriptor
temp_gdt_ptr:
    .short . - temp_gdt - 1
    .long (temp_gdt - ap_trampoline) + ap_trampoline_dest

ap_trampoline_end:

.code32

prot_mode:
    // load data segment register
    movw $0x10, %ax
    movw %ax, %ds

    // load gdt
    lgdtl gdt32_ptr

    // reload code segment register with long jump
    ljmpl $0x08, $reload_cs_ap
reload_cs_ap:
    // load data segments
    movw $0x10, %ax
    movw %ax, %ds
    movw %ax, %ss
    movw %ax, %es
    movw %ax, %fs
    movw %ax, %gs


    /* ------------------------------------------------------------------------ *
     *  Now that we are in long mode we need to set up a working stack.         *
     *  To do this we use the previously loaded registers that contain          *
     *  KERNEL_STACK_SIZE, KERNEL_STACKS_VADDR, NUM_CORES and the local apic id *
     *                                                                          *
     *  The register allocation is as follows:                                  *
     *                                                                          *
     *  %ebp = NUM_CORES                                                        *
     *  %edi = KERNEL_STACKS_VADDR                                              *
     *  %esi = KERNEL_STACK_SIZE                                                *
     *  %ebx = apic_id                                                          *
     *                                                                          *
     *                                                                          *
     *  The local apic id is obtained either by reading from IA32_X2APIC_APICID *
     *  when we are in X2Apic mode or by using cpuid when we are in XAPIC mode. *
     * ------------------------------------------------------------------------ */

    // find out if we are in X2APIC or XAPIC mode
    // and load the apic id into %ebx


    // read IA32_APIC_BASE msr
    movl $0x1b, %ecx
    rdmsr

    // bit 10 of the IA32_APIC_BASE msr specifies if the x2apic is enabled
    test $(1 << 10), %eax       
    jz no_x2apic

    // read IA32_X2APIC_APICID msr which loads the apic id into %eax
    movl $0x802, %ecx
    movl %eax, %ebx

    jmp calc_stack

no_x2apic:
    // execute cpuid with %eax=1 to get the apic id into bits [31..24] of ebx
    movl $1, %eax
    cpuid
    shr $24, %ebx
calc_stack:

    // load the global variables into registers
    movl NUM_CORES, %ebp
    movl KERNEL_STACKS_VADDR, %edi
    movl KERNEL_STACK_SIZE, %esi

    // Do a quick sanity check on the apic_id because the apic_id should never be greater than num_cores
    //
    // Note: the Intel X2APIC specification does not guarantee that
    // apic_id's are in the range 0..(number of cores). However, for now
    // this code assumes that this is true in order to calculate the start
    // address of the stack. Therefore we do a sanity check here.
    cmpl %ebp, %ebx
    jge apic_id_invalid

    // now calculate the top of the stack with: base + size * apic_id

    // load the size of each stack
    movl %esi, %eax

    // multiply stack size with processor id
    mull %ebx

    addl %edi, %eax 

    // finnally load the stack pointer
    movl %eax, %esp

    // put the first parameter on the stack
    pushl %ebx

    // call rust ap entry point (doesn't return)
    call rust_entry_ap

    jmp hang  

//     movb $0x64, %al
//     movw $0x3F8, %dx
//     outb %al, %dx

// 1:
//     hlt
//     jmp 1b


apic_id_invalid:
    lea apic_id_invalid_err_msg, %esi
    jmp display_error

// This function writes an error message to the serial console.
// Arguments:
//      %esi: pointer to the message (null-terminated) 
display_error:
    movw $0x3F8, %dx            // we also want to write the message to port 0x3F8

1:
    lodsb                       // load a character from the message into %al and increment %esi

    testb %al, %al              // check for the null-byte
    jz hang                     // and exit the loop

    outb %al, %dx               // write the character to the serial output port

    jmp 1b                      // continue the loop

hang:
    hlt
    jmp hang


.section .rodata

apic_id_invalid_err_msg:
    .asciz "ap startup failed: apic id was greater than number of cores\n"

.align 8
gdt32:
    // null descriptor
    .quad 0

    // 32-bit code descriptor
    .long 0x000FFFF
    .byte  0x00
    .byte  0b10011010       // P=1, DPL=0, S=1, E=1, DC=0, RW=1, A=0
    .byte  0b11001111       // G=1, DB=1
    .byte  0x00

    // 32-bit data descriptor
    .long 0x000FFFF
    .byte  0x00
    .byte  0b10010010       // P=1, DPL=0, S=1, E=0, DC=0, RW=1, A=0
    .byte  0b11001111       // G=1, DB=1
    .byte  0x00
gdt32_ptr:
    .short . - gdt32 - 1
    .long gdt32