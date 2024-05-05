.global ap_trampoline
.global ap_trampoline_end
.global ap_trampoline_dest

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
     *  Now that we are in protected mode we need to set up a working stack.    *
     *  To do this we use the previously initialized variables                  *
     *  KERNEL_STACK_SIZE, KERNEL_STACKS_VADDR and the local apic id.           *
     * ------------------------------------------------------------------------ */

    // load local apic id into %ebx
    movl     $1, %eax
    cpuid
    shrl    $24, %ebx

    
    movl KERNEL_STACK_SIZE, %eax            // load the size of each stack
    mull %ebx                               // multiply stack size with processor id
    movl KERNEL_STACKS_VADDR, %ecx          // load the base address of the kernel stack area
    addl %ecx, %eax                         // add the base adddress
    movl %eax, %esp                         // load the stack pointer


    // now we call rust_entry_ap() with one parameter
    subl $4, %esp                           // allocate 4 bytes for the parameter
    movl %ebx, (%esp)                       // the first parameter is the processor id

    call rust_entry_ap                      // call rust ap entry point (doesn't return)
    ud2

1:
    hlt
    jmp 1b