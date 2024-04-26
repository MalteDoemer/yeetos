.section .text

.code16
.global ap_trampoline
.global ap_trampoline_end
.align 4096
ap_trampoline:
    cli
    cld
1:  hlt
    jmp 1b

.align 4096
ap_trampoline_end:


.code64

// This function waits until bit 12 of the 0x300 register
// of the Local APIC is cleared.
// Note: %rdi, %rsi and %rdx are preserved
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
// Note: %rdi, %rsi and %rdx are preserved
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
// Note: %rdi, %rsi and %rdx are preserved
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
    pushq %rdx
    movq $10, %rdi
    call sleep_ms
    popq %rdx
    popq %rsi
    popq %rdi

    ret


// Sends a STARTUP IPI and sleeps for 200 microseconds.
// Note: %rdi, %rsi and %rdx are preserved
// Parameters:
// - %rdi = local apic address of bsp
// - %rsi = id of the ap shifted left by 24
// - %rdx = vector for startup ipi
send_startup_ipi:
    // clear APIC error register
    movl $0, 0x280(%rdi)

    call select_ap

    // send STARTUP IPI for vector XX00:0000
    // where XX is the value in %rdx
    movl 0x300(%rdi), %eax
    andl $0xfff0f800, %eax
    orl $0x000600, %eax     // set delivery mode
    orl %edx, %eax          // set vector
    movl %eax, 0x300(%rdi)

    // sleep for 200 microseconds
    pushq %rdi
    pushq %rsi
    pushq %rdx
    movq $200, %rdi
    call sleep_us
    popq %rdx
    popq %rsi
    popq %rdi

    call wait_for_delivery
    ret


// Starts an application processor.
// Parameters:
// - %rdi = local apic address of bsp
// - %rsi = id of the ap to start
// - %rdx = vector for startup ipi
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
