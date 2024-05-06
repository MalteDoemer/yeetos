ap_trampoline_dest = 0x8000

.global ap_trampoline
.global ap_trampoline_end
.global ap_trampoline_dest

.section .text

.code16
ap_trampoline:
    cli
    cld

    movw $0x3F8, %dx
    movb $0x64, %al
    outb %al, %dx

1:  hlt
    jmp 1b
ap_trampoline_end:


.code64