ap_trampoline_dest = 0x8000


.section .text

.code16

ap_trampoline:
    cli
    cld

    movw $0x3F8, %dx
    movb $0x62, %al
    outb %al, %dx

1:
    hlt
    jmp 1b
ap_trampoline_end:

.code32

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


    /* send INIT IPI */

    // clear APIC error register
    movl $0, 0x280(%edi) 

    // select ap
    movl 0x310(%edi), %eax
    andl $0x00ffffff, %eax
    orl %esi, %eax
    movl %eax, 0x310(%edi)

    // trigger INIT IPI by writing to register 0x300
    movl 0x300(%edi), %eax
    andl $0xfff00000, %eax
    orl $0x00C500, %eax
    movl %eax, 0x300(%edi)

    // wait until bit 12 of 0x300 is cleard
2:
    pause
    movl 0x300(%edi), %eax
    andl $(1 << 12), %eax
    testl %eax, %eax
    jnz 2b


    /* deassert INIT IPI */

    // select ap
    movl 0x310(%edi), %eax
    andl $0x00ffffff, %eax
    orl %esi, %eax
    movl %eax, 0x310(%edi)

    // send INIT IPI deassert signal
    movl 0x300(%edi), %eax
    andl $0xfff00000, %eax
    orl $0x008500, %eax
    movl %eax, 0x300(%edi)

    // wait for delivery
3:
    pause
    movl 0x300(%edi), %eax
    andl $(1 << 12), %eax
    testl %eax, %eax
    jnz 3b


    // sleep for 10 milliseconds
    xorl %eax, %eax
    movl %eax, +4(%esp)
    movl $10,  +0(%esp)
    call sleep_ms

    /* send STARTUP IPI #1 */

    // clear APIC error register
    movl $0, 0x280(%edi) 

    // select ap
    movl 0x310(%edi), %eax
    andl $0x00ffffff, %eax
    orl %esi, %eax
    movl %eax, 0x310(%edi)


    // send STARTUP IPI for vector 0800:0000
    movl 0x300(%edi), %eax
    andl $0xfff0f800, %eax
    orl $0x000608, %eax
    movl %eax, 0x300(%edi)


    // wait for 200 microseconds
    xorl %eax, %eax
    movl %eax, +4(%esp)
    movl $200, +0(%esp)
    call sleep_us

    // wait for delivery
4:
    pause
    movl 0x300(%edi), %eax
    andl $(1 << 12), %eax
    testl %eax, %eax
    jnz 4b


    /* send STARTUP IPI #2 */

    // clear APIC error register
    movl $0, 0x280(%edi) 

    // select ap
    movl 0x310(%edi), %eax
    andl $0x00ffffff, %eax
    orl %esi, %eax
    movl %eax, 0x310(%edi)


    // send STARTUP IPI for vector 0800:0000
    movl 0x300(%edi), %eax
    andl $0xfff0f800, %eax
    orl $0x000608, %eax
    movl %eax, 0x300(%edi)


    // wait for 200 microseconds
    xorl %eax, %eax
    movl %eax, +4(%esp)
    movl $200, +0(%esp)
    call sleep_us

    // wait for delivery
5:
    pause
    movl 0x300(%edi), %eax
    andl $(1 << 12), %eax
    testl %eax, %eax
    jnz 5b


    // restore old stack frame and registers

    movl +12(%esp), %esi    // restore %esi
    movl +8 (%esp), %edi    // restore %edi

    addl $16, %esp

    movl %ebp, %esp
    popl %ebp
    ret









    // sleep for 10 milliseconds
    // subl $8, %esp           // allocate 8 bytes on the stack
    // xorl %eax, %eax         // zero out eax
    // movl $10, +0(%esp)      // move the argument onto the stack
    // movl %eax, +4(%esp)     // argument is 64-bit so we need zero padding
    // call sleep_ms           // call the function
    // addl $8, %esp           // deallocated the stack parameters

    