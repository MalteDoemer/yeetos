PML4T_ADDR = 0x1000

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
     *  Directly after startup every AP is in 16-bit mode. In order to get to   *
     *  long mode we need to first enable protected mode.                       *
     *                                                                          *
     *  In order to enable protected mode we first need to load a temporary     *
     *  32-bit GDT. Then we enable protected mode using cr0 and perform a ljmp  *
     *  to load the code segment descriptor.                                    *
     * ------------------------------------------------------------------------ */ 

    // clear the data segment register to zero
    xorw %ax, %ax
    movw %ax, %dx

    // load gdt
    lgdtl (gdt32_ptr - ap_trampoline) + ap_trampoline_dest

    // enable protected mode
    movl %cr0, %eax
    orl  $1, %eax                  // set protected mode enable bit
    movl %eax, %cr0

    // load code segment into %cs with a jump
    ljmpl $0x08, $prot_mode

.align 8
gdt32:
    // null descriptor
    .quad 0

    // 32-bit code descriptor
    .short 0xFFFF
    .short 0x0000
    .byte  0x00
    .byte  0b10011010       // P=1, DPL=0, S=1, E=1, DC=0, RW=1, A=0
    .byte  0b11001111       // G=1, DB=1
    .byte  0x00

    // 32-bit data descriptor
    .short 0xFFFF
    .short 0x0000
    .byte  0x00
    .byte  0b10010010       // P=1, DPL=0, S=1, E=0, DC=0, RW=1, A=0
    .byte  0b11001111       // G=1, DB=1
    .byte  0x00
gdt32_ptr:
    .short . - gdt32 - 1
    .long (gdt32 - ap_trampoline) + ap_trampoline_dest

ap_trampoline_end:


.code32
prot_mode:
    // load data segments
    movw $16, %ax
    movw %ax, %ds
    movw %ax, %ss
    movw %ax, %es
    movw %ax, %fs
    movw %ax, %gs


    /* ------------------------------------------------------------------------ *
     *  Now that we are in protected mode we need to enter long mode. This      *
     *  requires us to enable paging setting the LM bit in the EFER MSR and     *
     *  loading a 64-bit GDT. Most of the work here is simmilar to boot.s.      *
     * ------------------------------------------------------------------------ */

    // tell the cpu where to find our PML4T by setting the cr3 register
    // the PML4T and other paging structs are set up in boot.s
    movl $PML4T_ADDR, %eax
    movl %eax, %cr3

    // enable PAE which is required for long mode
    movl %cr4, %eax
    or $(1 << 5), %eax                  // set bit 5 which is the PAE-bit
    movl %eax, %cr4

    // set the LM-bit in the EFER MSR
    movl $0xC0000080, %ecx              // the ID of EFER MSR
    rdmsr                               // read the contents of the MSR into eax
    or $(1 << 8), %eax                  // set bit 8 which is the LM-bit
    wrmsr                               // write the contents of eax into the MSR

    // enable paging
    movl %cr0, %eax
    or $(1 << 31), %eax                 // set bit 31 which is the PG-bit
    movl %eax, %cr0

    // load a 64-bit GDT (the gdt is defined in boot.s)
    lgdt gdt64_ptr

    // enter 64-bit long mode
    ljmpl $0x08, $long_mode_ap


.code64
long_mode_ap:
    movw $0x10, %ax
    movw %ax, %ss
    movw %ax, %ds
    movw %ax, %es
    movw %ax, %fs
    movw %ax, %gs


    /* ------------------------------------------------------------------------ *
     *  Now that we are in long mode we need to set up a working stack.         *
     *  To do this we use the previously initialized variables                  *
     *  KERNEL_STACK_SIZE, KERNEL_STACKS_VADDR and the local apic id.           *
     * ------------------------------------------------------------------------ */

    // load local apic id into %ebx
    movl     $1, %eax
    cpuid
    shrl    $24, %ebx

    // load the size of each stack
    movq KERNEL_STACK_SIZE, %rax

    // multiply stack size with processor id
    mulq %rbx

    // load the base address of the kernel stack area
    movq KERNEL_STACKS_VADDR, %rcx

    // add the base adddress
    addq %rcx, %rax

    // load the stack pointer
    movq %rax, %rsp

    // create a empty stackframe
    xorq %rax, %rax
    pushq %rax
    pushq %rax

    // the first parameter of rust_entry_ap is the local apic id
    movq %rbx, %rdi

    // call rust ap entry point (doesn't return)
    call rust_entry_ap

    ud2