// This function calls the kernel entry point.
// Parameters:
// - %rcx = boot info pointer
// - %rdx = processor id
// - %r8 = address of the kernel entry_point
// - %r9 = stack pointer
.global jmp_kernel_entry
jmp_kernel_entry:

    // move arguments into correct registers for SystemV x86_64 ABI
    movq %rcx, %rdi
    movq %rdx, %rsi

    // load new stack pointer
    movq %r9, %rsp

    // call entry point (no return)
    jmpq *%r8