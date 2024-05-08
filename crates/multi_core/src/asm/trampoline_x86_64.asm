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
     *  Due to a weird quirk when targeting UEFI platforms, reading a globally  *
     *  defined variable with movq VARIABLE, %rax computes a wrong address and  *
     *  thus crashes. Now in 32-bit mode loading global variables works fine so *
     *  we load KERNEL_STACK_SIZE, KERNEL_STACKS_VADDR, NUM_CORES directly into *
     *  registers right here.                                                   *
     * ------------------------------------------------------------------------ */

    /* ------------------------------------------------------------------------ *
     *  Now that we are in protected mode we need to enter long mode. This      *
     *  requires us to enable paging setting the LM bit in the EFER MSR and     *
     *  loading a 64-bit GDT. Most of the work here is simmilar to boot.s.      *
     * ------------------------------------------------------------------------ */

    // tell the cpu where to find our PML4T by setting the cr3 register
    // Note: we get the address of the PML4T from a global variable set in ap_startup.rs

    movl PAGE_TABLE_ADDRESS, %eax
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

    // load a 64-bit GDT
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
     *  To do this we use the previously loaded registers that contain          *
     *  KERNEL_STACK_SIZE, KERNEL_STACKS_VADDR, NUM_CORES and the local apic id *
     *                                                                          *
     *  The register allocation is as follows:                                  *
     *                                                                          *
     *  %ebp = NUM_CORES                                                        *
     *  %rdi = KERNEL_STACKS_VADDR                                              *
     *  %rsi = KERNEL_STACK_SIZE                                                *
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
    rdmsr
    movl %eax, %ebx

    jmp calc_stack

no_x2apic:
    // execute cpuid with %eax=1 to get the apic id into bits [31..24] of ebx
    movl $1, %eax
    cpuid
    shr $24, %ebx
calc_stack:

    // Load the global variables into registers
    // Note: we need to use movabs because somehow on uefi targets a wrong address gets calculated
    // (probably a wrong setting in the linker/assembler)
    movabs $NUM_CORES, %rax
    movl (%rax), %ebp

    movabs $KERNEL_STACKS_VADDR, %rax
    movq (%rax), %rdi

    movabs $KERNEL_STACK_SIZE, %rax
    movq (%rax), %rsi

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
    movq %rsi, %rax

    // multiply stack size with processor id
    mulq %rbx

    // add the base address of the kernel stack area
    addq %rdi, %rax

    // finnally load the stack pointer
    movq %rax, %rsp

    // HACK: we call rust_entry_ap with the C calling convention
    // Now on multiboot2 this means we would use the SystemV x86_64 ABI
    // which expects the first parameter in %rdi
    // But on UEFI we need to use the Microsoft x64 ABI which expects
    // the first parameter in %rcx
    // In order to not have to maintain two .asm files we just load the
    // parameter into both registers %rdi and %rcx
    movq %rbx, %rdi
    movq %rbx, %rcx

    // call rust ap entry point (doesn't return)
    call rust_entry_ap
    
    jmp hang

apic_id_invalid:
    movabs $apic_id_invalid_err_msg, %rsi
    jmp display_error
    

// This function writes an error message to the serial console.
// Arguments:
//      %rsi: pointer to the message (null-terminated) 
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
gdt64:
    // null descriptor
    .quad 0

    // 64-bit code descriptor

    .short 0x0000
    .short 0x0000
    .byte 0x00
    .byte 0b10011010
    .byte 0b10100000    
    .byte 0x00

    // 64-bit data descriptor
    .short 0x0000
    .short 0x0000
    .byte 0x00
    .byte 0b10010010
    .byte 0b11000000
    .byte 0x00
gdt64_ptr:
    .short . - gdt64 - 1
    .quad gdt64