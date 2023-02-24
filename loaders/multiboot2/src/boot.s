
// keep in sync with mmap.rs

ZER0_FRAME = 0x0000
PML4T_ADDR = 0x1000
PDPT_ADDR = 0x2000
PDT_START_ADDR = 0x3000
PDT_END_ADDR = 0x7000


READ_WRITE_BITS = 0b11
PAGE_SIZE_BITS = (1 << 7)


MULTIBOOT2_HEADER_MAGIC = 0xe85250d6
MULTIBOOT2_LOADER_MAGIC = 0x36d76289;
MULTIBOOT2_ARCHITECTURE_I386 = 0

VGA_COLOR = 0x0f

.section .boot, "a"
mboot_header:
    // magic number
    .long MULTIBOOT2_HEADER_MAGIC

    // architecure
    .long MULTIBOOT2_ARCHITECTURE_I386 // we use i386 since there is no number for x86_64

    // header size
    .long mboot_header_end - mboot_header

    // checksum
    .long -(MULTIBOOT2_HEADER_MAGIC + MULTIBOOT2_ARCHITECTURE_I386 + mboot_header_end - mboot_header)

    // load address tag
    .short 2
    .short 0
    .long 24
    .long mboot_header
    .long __load_start
    .long __bss_start
    .long __load_end

    // entry point address tag
    .short 3
    .short 0
    .long 12
    .long start
    .long 0 // padding

    // module alignment tag
    .short 6
    .short 0
    .long 8

    // end tag
    .long 0
    .long 8
mboot_header_end:

.code32
.section .text
start:
    cli
    cld

    movl $__stack_end, %esp
    movl %esp, %ebp

    // check if we were booted from a multiboot2 compliant loader.
    cmpl $MULTIBOOT2_LOADER_MAGIC, %eax
    jne multiboot2_error

    // push a 64-bit pointer to the multiboot2 struct on the stack
    push $0x00
    push %ebx


    /* ------------------------------------------------------------------------ *
     *  We could be on any x86 cpu right now, so we need to check if it         *
     *  supports x86_64 long mode. To do this we call a specific "function" with*
     *  the cpuid instruction (0x80000000). But first we have to check whether  *
     *  cpuid and function 0x80000000 are available.                            *
     * ------------------------------------------------------------------------ */ 

    // check for cpuid with the eflags method
    pushfd                              // save EFLAGS so we can restore them laters
    pushfd                              // store EFLAGS on the stack for manipulation
    xorl $0x00200000, (%esp)            // invert the ID bit
    popfd                               // load EFLAGS with ID bit inverted
    pushfd                              // store EFLAGS on the stack again
    pop %eax                            // move the EFLAGS into eax
    xorl (%esp), %eax                   // test is any bits have changed
    popfd                               // restore the original EFLAGS
    and $0x00200000, %eax               // eax = zero if ID bit can't be changed, else non-zero

    jz cpuid_error                      // if eax = zero then cpuid is not supported

    // check for extended cpuid info
    movl $0x80000000, %eax              // set bit 31 in eax
    cpuid
    cmpl $0x80000001, %eax              // if eax is below 0x80000001, no info is available
    jb ext_cpuid_error                  // proceed to print the error


    // finally check for long mode
    movl $0x80000001, % eax             // eax = 0x80000001 to request extended cpu info
    cpuid
    test $0x20000000, %edx              // check bit 29 (LM-bit)
    jz long_mode_error                  // if it is not set we don't have long mode

    /* ------------------------------------------------------------------------ *
     *  Identity map the memory range and enable paging.                        *
     *                                                                          *
     *  The first 4GiB of the ram will be identity mapped using 2MiB pages.     *
     * ------------------------------------------------------------------------ */ 

    // clear the memory from 0 to PDT_END_ADDR

    xorl %eax, %eax                                 // zero out eax
    xorl %edi, %edi                                 // start at address 0

    movl $PDT_END_ADDR, %ecx                        // get the number of bytes to clear
    shr $2, %ecx                                    // divide by 4 because we do 4 bytes at a time

    rep stosl
 
    // the first PML4 entry points to the PDPT
    movl $(PDPT_ADDR | READ_WRITE_BITS), PML4T_ADDR
    

    // write the PDPT entries to point to the PDT
    movl $PDPT_ADDR, %edi                           // write to the PDPT
    movl $(PDT_START_ADDR | READ_WRITE_BITS), %eax  // start with the first PDT
    movl $(PDT_END_ADDR | READ_WRITE_BITS), %ecx    // stop on the last PDT
1:
    movl %eax, (%edi)
    addl $0x1000, %eax
    addl $8, %edi

    cmpl %ecx, %eax
    jb 1b
    
    // fill out the PDT's

    movl $PDT_START_ADDR, %edi                      // write to the PDT's
    movl $(PAGE_SIZE_BITS | READ_WRITE_BITS), %eax  // start add address 0 with PS, R/W and P bit set
    movl $PDT_END_ADDR, %ecx                        // stop at the last PDT
1:
    movl %eax, (%edi)
    addl $0x200000, %eax
    addl $8, %edi

    cmpl %ecx, %edi
    jb 1b

    // tell the cpu where to find our PML4T by setting the cr3 register
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

    // load a 64-bit GDT
    lgdt gdt64_ptr

    // enter 64-bit long mode
    ljmpl $0x08, $long_mode

multiboot2_error:
    movl $multiboot2_error_msg, %esi
    jmp display_error
cpuid_error:
    movl $cpuid_error_msg, %esi
    jmp display_error

ext_cpuid_error:
    movl $ext_cpuid_error_msg, %esi
    jmp display_error

long_mode_error:
    movl $long_mode_error_msg, %esi
    jmp display_error

// Arguments:
//      %esi: pointer to the message (null-terminated) 
display_error:
    movl $0xb8000, %edi         // we want to write to the vga memory at 0xb8000
    movb $VGA_COLOR, %ah        // set the color to white text with black background

1:
    lodsb                       // load a character from the message into %al and increment %esi

    testb %al, %al              // check for the null-byte
    jz hang                     // and exit the loop

    stosw                       // write the character plus the color into the vga memory and increment %edi    
    jmp 1b                      // continue the loop

hang:
    hlt
    jmp hang


.code64
long_mode:
    movw $0x10, %ax
    movw %ax, %ss
    movw %ax, %ds
    movw %ax, %es
    movw %ax, %fs
    movw %ax, %gs

    // pop the saved pointer to the multiboot2 struct
    popq %rdi

    // create a empty stackframe
    xorq %rax, %rax
    pushq %rax
    pushq %rax

    // call rust entry point (doesn't return)
    call rust_entry

.section .rodata

multiboot2_error_msg:
    .asciz "boot failed: not loaded from multiboot2 compliant loader (signature missmatch)!"

cpuid_error_msg:
    .asciz "boot failed: cpuid instruction not available!"

ext_cpuid_error_msg:
    .asciz "boot failed: extended cpuid information not available!"

long_mode_error_msg:
    .asciz "boot failed: 64-bit mode not available!"

.align 8
gdt64:
    // null descriptor
    .quad 0

    // 64-bit code descriptor
    .short 0xFF
    .short 0x00
    .byte 0x00
    .byte 0b10011010
    .byte 0b10101111    
    .byte 0x00

    // 64-bit data descriptor
    .short 0x00
    .short 0x00
    .byte 0x00
    .byte 0b10010010
    .byte 0b00000000
    .byte 0x00
gdt64_ptr:
    .short . - gdt64 - 1
    .quad gdt64
