
AP_TRAMPOLINE_DEST = 0x8000

.section .text

.code16
ap_trampoline:
    cli
    cld
    hlt
ap_trampoline_end:

.code64
copy_ap_trampoline:
    pushq %rsi
    pushq %rdi
    pushq %rcx

    movq $ap_trampoline, %rsi
    movq $AP_TRAMPOLINE_DEST, %rdi

    movq $(ap_trampoline_end - ap_trampoline), %rcx
    rep movsb

    popq %rcx
    popq %rdi
    popq %rsi

    ret


