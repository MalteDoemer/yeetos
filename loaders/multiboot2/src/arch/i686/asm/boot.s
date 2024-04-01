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

    // save the pointer to the multiboot2 struct on the stack
    push %ebx

    // check for cpuid availability with the eflags method
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

    // load gdt
    lgdtl gdt32_ptr

    // reload code segment register with long jump
    ljmpl $0x08, $reload_cs
reload_cs:
    // load data segments
    movw $0x10, %ax
    movw %ax, %ds
    movw %ax, %ss
    movw %ax, %es
    movw %ax, %fs
    movw %ax, %gs

    // first parameter multiboot pointer is already on the stack
    call rust_entry

    ud2

multiboot2_error:
    movl $multiboot2_error_msg, %esi
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


.global jmp_kernel_entry
jmp_kernel_entry:
    // TODO
    hlt
    jmp jmp_kernel_entry


.section .rodata

multiboot2_error_msg:
    .asciz "boot failed: not loaded from multiboot2 compliant loader (signature missmatch)!"

cpuid_error_msg:
    .asciz "boot failed: cpuid instruction not available!"

.align 8
gdt32:
    .long 0, 0                     // null descriptor
    .long 0x0000FFFF, 0x00CF9A00   // code descriptor
    .long 0x0000FFFF, 0x008F9200   // data descriptor
gdt32_ptr:
    .short . - gdt32 - 1
    .long gdt32