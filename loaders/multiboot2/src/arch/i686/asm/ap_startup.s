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


.global copy_ap_trampoline
copy_ap_trampoline:
    // set up stack frame
    pushl %ebp
    movl %esp, %ebp

    // eax, ecx, edx scratch registers
    // all others must be preserved
    subl $8, %esp
    movl %esi,   (%esp)
    movl %edi, +4(%esp)

    // copy ap_trampoline to ap_trampoline_dest
    movl $ap_trampoline, %esi                           // source address
    movl $ap_trampoline_dest, %edi                      // destination address
    movl $(ap_trampoline_end - ap_trampoline), %ecx     // number of bytes to copy
    rep movsb

    // restore esi and edi from stack
    movl   (%esp), %esi
    movl +4(%esp), %edi

    // restore old stack frame
    movl %ebp, %esp
    popl %ebp
    ret



// This function selects the correct ap by writing into register 0x310
// Parameters: 
// - %edi = local apic address
// - %esi = apic id shifted left by 24
select_ap:
    // select ap by writing 0x310
    movl 0x310(%edi), %eax
    andl $0x00ffffff, %eax
    orl %esi, %eax
    movl %eax, 0x310(%edi)

    ret

// This function waits until bit 12 of register 0x300 is cleared.
// Parameters: 
// - %edi = local apic address
// - %esi = apic id shifted left by 24
wait_for_delivery:
    pause
    movl 0x300(%edi), %eax
    andl $(1 << 12), %eax
    testl %eax, %eax
    jnz wait_for_delivery

    ret

// This function sends an INIT IPI followed by an INIT IPI deassert.
// Parameters: 
// - %edi = local apic address
// - %esi = apic id shifted left by 24
send_init_ipi:
    /* send INIT IPI */

    // clear APIC error register
    movl $0, 0x280(%edi) 

    call select_ap

    // trigger INIT IPI by writing to register 0x300
    movl 0x300(%edi), %eax
    andl $0xfff00000, %eax
    orl $0x00C500, %eax
    movl %eax, 0x300(%edi)

    call wait_for_delivery

    /* deassert INIT IPI */

    call select_ap

    // send INIT IPI deassert signal
    movl 0x300(%edi), %eax
    andl $0xfff00000, %eax
    orl $0x008500, %eax
    movl %eax, 0x300(%edi)

    call wait_for_delivery

    ret

// This function sends a STARTUP IPI with the start address of 0800:0000.
// Note: in contrast to send_init_ipi it does not call wait_for_delivery
// Parameters: 
// - %edi = local apic address
// - %esi = apic id shifted left by 24
send_startup_ipi:
    // clear APIC error register
    movl $0, 0x280(%edi) 

    call select_ap

    // send STARTUP IPI for vector 0800:0000
    movl 0x300(%edi), %eax
    andl $0xfff0f800, %eax
    orl $0x000608, %eax
    movl %eax, 0x300(%edi)

    ret

// Starts an application processor.
// Parameters:
// - 1: local apic address of bsp
// - 2: id of the ap to start
.global startup_ap
startup_ap:
    // set up stack frame
    pushl %ebp
    movl %esp, %ebp

    // our stack will look like this:
    //
    //  +12(%ebp) = parameter 2: apic id
    //  +8 (%ebp) = parameter 1: lapic address
    //* +4 (%ebp) = return address
    //* +0 (%ebp) = saved value of %ebp
    //  +12(%esp) = saved value of %esi
    //  +8 (%esp) = saved value of %edi
    //  +4 (%esp) = upper 32 bits of parameter for sleep_*
    //  +0 (%esp) = lower 32 bits of parameter for sleep_*
    //
    
    subl $16, %esp          // allocate 16 bytes on the stack for local variables and parameters

    movl %esi, +12(%esp)    // save %esi on the stack
    movl %edi, +8 (%esp)    // save %edi on the stack

    movl +12(%ebp), %esi    // load parameter 1: apic id
    movl +8 (%ebp), %edi    // load parameter 2: lapic address

    // the id field of the 0x310 register is in bits 24-27
    // so we do the left shift right here and always pass the shifted value around
    shll $24, %esi

    call send_init_ipi

    // sleep for 10 milliseconds
    xorl %eax, %eax
    movl %eax, +4(%esp)
    movl $10,  +0(%esp)
    call sleep_ms

    /* send STARTUP IPI #1 */
    call send_startup_ipi

    // wait for 200 microseconds
    xorl %eax, %eax
    movl %eax, +4(%esp)
    movl $200, +0(%esp)
    call sleep_us

    call wait_for_delivery

    /* send STARTUP IPI #2 */
    call send_startup_ipi

    // wait for 200 microseconds
    xorl %eax, %eax
    movl %eax, +4(%esp)
    movl $200, +0(%esp)
    call sleep_us

    call wait_for_delivery

    // restore old stack frame and registers

    movl +12(%esp), %esi    // restore %esi
    movl +8 (%esp), %edi    // restore %edi

    addl $16, %esp

    movl %ebp, %esp
    popl %ebp
    ret

    