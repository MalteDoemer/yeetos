PML4T_ADDR = 0x1000

ap_trampoline_dest = 0x8000

.section .text

.code16
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

// Copies the ap_trampoline code to address 0x8000.
// This is necessary since during ap_startup we set the starting
// address to 0x8000.
.global copy_ap_trampoline
copy_ap_trampoline:
    movq $ap_trampoline, %rsi
    movq $ap_trampoline_dest, %rdi

    movq $(ap_trampoline_end - ap_trampoline), %rcx
    rep movsb

    ret


// This function waits until bit 12 of the 0x300 register
// of the Local APIC is cleared.
// Note: %rdi and %rsi are preserved
// Parameters: 
// - %rdi = local apic address of bsp
wait_for_delivery:
    pause
    movl 0x300(%rdi), %eax
    andl $(1 << 12), %eax
    testl %eax, %eax
    jnz wait_for_delivery
    ret


// Selects the destination ap for an interrupt by writing into register 0x310.
// Note: %rdi and %rsi are preserved
// Parameters:
// - %rdi = local apic address of bsp
// - %rsi = id of the ap shifted left by 24
select_ap:
    movl 0x310(%rdi), %eax
    andl $0x00ffffff, %eax
    orl %esi, %eax
    movl %eax, 0x310(%rdi)
    ret


// Sends an INIT IPI followed by and INIT IPI deassert.
// Note: %rdi and %rsi are preserved
// Parameters:
// - %rdi = local apic address of bsp
// - %rsi = id of the ap shifted left by 24
send_init_ipi:
    /* send INIT IPI */

    // clear APIC error register
    movl $0, 0x280(%rdi) 

    call select_ap

    // trigger INIT IPI by writing to register 0x300
    movl 0x300(%rdi), %eax
    andl $0xfff00000, %eax
    orl $0x00C500, %eax
    movl %eax, 0x300(%rdi)

    call wait_for_delivery

    /* deassert INIT IPI */

    // select ap
    call select_ap

    // send INIT IPI deassert signal
    movl 0x300(%rdi), %eax
    andl $0xfff00000, %eax
    orl $0x008500, %eax
    movl %eax, 0x300(%rdi)

    call wait_for_delivery

    // 10 millisecond delay
    pushq %rdi
    pushq %rsi
    movq $10, %rdi
    call sleep_ms
    popq %rsi
    popq %rdi

    ret


// Sends a STARTUP IPI and sleeps for 200 microseconds.
// Note: %rdi and %rsi are preserved
// Parameters:
// - %rdi = local apic address of bsp
// - %rsi = id of the ap shifted left by 24
send_startup_ipi:
    // clear APIC error register
    movl $0, 0x280(%rdi)

    call select_ap

    // send STARTUP IPI for vector 0800:0000
    movl 0x300(%rdi), %eax
    andl $0xfff0f800, %eax
    orl $0x000608, %eax
    movl %eax, 0x300(%rdi)

    // sleep for 200 microseconds
    pushq %rdi
    pushq %rsi
    movq $200, %rdi
    call sleep_us
    popq %rsi
    popq %rdi

    call wait_for_delivery
    ret


// Starts an application processor.
// Parameters:
// - %rdi = local apic address of bsp
// - %rsi = id of the ap to start
.global startup_ap
startup_ap:
    // this function follows the description in
    // https://wiki.osdev.org/Symmetric_Multiprocessing


    // the id field is in bits 24-27 of the 0x310 register
    // so we do the left shift right at the start
    shlq $24, %rsi 

    call send_init_ipi

    call send_startup_ipi

    call send_startup_ipi

    ret
