PD_ADDR = 0x1000
PD_END_ADDR = 0x2000
PD_LAST_ENTRY = 0x1400

// various flag bits for the paging entries
// Note: ENTRY_USAGE_*_BITS are not used by the hardware, but by EntryUsage enum in rust
PRESENT_FLAG           = 0b01
WRITABLE_FLAG          = 0b10
PAGE_SIZE_FLAG         = 0b01 << 7
ENTRY_USAGE_PAGE_BITS  = 0b01 << 9
ENTRY_USAGE_TABLE_BITS = 0b10 << 9

TABLE_ENTRY_BITS = PRESENT_FLAG | WRITABLE_FLAG | ENTRY_USAGE_TABLE_BITS
HUGE_PAGE_ENTRY_BITS = PRESENT_FLAG | WRITABLE_FLAG | PAGE_SIZE_FLAG | ENTRY_USAGE_PAGE_BITS

// Values for the frame buffer tag
// Note: a value of zero means no preference
FRAME_BUFFER_WIDTH = 0
FRAME_BUFFER_HEIGHT = 0
FRAME_BUFFER_DEPTH = 32

MULTIBOOT2_HEADER_MAGIC = 0xe85250d6
MULTIBOOT2_LOADER_MAGIC = 0x36d76289;
MULTIBOOT2_ARCHITECTURE_I386 = 0

VGA_COLOR = 0x0f

.section .boot, "a"
mboot_header:
    // magic number
    .long MULTIBOOT2_HEADER_MAGIC

    // architecture
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

    // frame buffer tag
    .short 5
    .short 0
    .long 20
    .long FRAME_BUFFER_WIDTH
    .long FRAME_BUFFER_HEIGHT
    .long FRAME_BUFFER_DEPTH
    .long 0

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

    // check for PSE page size extension bit
    mov $1, %eax                        // we will call cpuid function 1
    cpuid                               // execute cpuid
    and $0x08, %edx                     // PSE availability is indicated with bit 3 of edx
    jz pse_error                        // if edx = zero then PSE is not supported

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

    // prepare to identity map the first 1GiB using 4MiB pages
    // paging is only enabled once enable_paging() is called from within rust

    // first clear out the Page Directory and Zero Frame
    xorl %eax, %eax                                 // zero out eax
    xorl %edi, %edi                                 // start at address 0

    movl $PD_END_ADDR, %ecx                         // get the number of bytes to clear
    shr $2, %ecx                                    // divide by 4 because we do 4 bytes at a time

    rep stosl

    // fill in the Page Directory entries
    movl $PD_ADDR, %edi                             // write to the PD
    movl $HUGE_PAGE_ENTRY_BITS, %eax                // start at address 0x00 with PS, P, R/W bits set and usage=0b01
    movl $PD_LAST_ENTRY, %ecx                       // only write 1/4 of the entries for 1GiB
1:
    movl %eax, (%edi)                               // write entry
    addl $0x400000, %eax                            // increment address by 4 MiB
    addl $4, %edi                                   // move to next entry

    cmpl %ecx, %edi                                 // check if we are finished
    jb 1b

    // the last entry of the PD points to itself, this enables recursive mapping
    movl $(PD_ADDR | TABLE_ENTRY_BITS), (PD_ADDR + 4 * 1023)

    // first parameter multiboot pointer is already on the stack
    call rust_entry

    ud2

.global enable_paging
enable_paging:
    pushl %eax

    // tell the cpu where to find out PD by setting cr3
    movl $PD_ADDR, %eax
    movl %eax, %cr3

    // enable PSE
    movl %cr4, %eax
    orl $(1 << 4), %eax                // set bit 4 which is the PSE-bit
    movl %eax, %cr4

    // enable paging
    movl %cr0, %eax
    or $(1 << 31), %eax                 // set bit 31 which is the PG-bit
    movl %eax, %cr0

    popl %eax
    ret


multiboot2_error:
    movl $multiboot2_error_msg, %esi
    jmp display_error
    
cpuid_error:
    movl $cpuid_error_msg, %esi
    jmp display_error

pse_error:
    movl $pse_error_msg, %esi
    jmp display_error

// This function writes an error message into the vga memory as well as
// to the serial console.
// Arguments:
//      %esi: pointer to the message (null-terminated) 
display_error:
    movl $0xb8000, %edi         // we want to write to the vga memory at 0xb8000
    movb $VGA_COLOR, %ah        // set the color to white text with black background
    movw $0x3F8, %dx            // we also want to write the message to port 0x3F8

1:
    lodsb                       // load a character from the message into %al and increment %esi

    testb %al, %al              // check for the null-byte
    jz hang                     // and exit the loop

    stosw                       // write the character plus the color into the vga memory and increment %edi
    outb %al, %dx               // also write the character to the serial output port

    jmp 1b                      // continue the loop

hang:
    hlt
    jmp hang


// This function calls the kernel entry point.
// Parameters:
// - 1 = boot info pointer
// - 2 = processor id
// - 3 = address of the kernel entry_point
// - 4 = stack pointer
.global jmp_kernel_entry
jmp_kernel_entry:
    pushl %ebp
    movl %esp, %ebp

    movl +8(%ebp), %eax     // boot info pointer
    movl +12(%ebp), %ebx    // processor id
    movl +16(%ebp), %ecx    // entry_point
    movl +20(%ebp), %edx    // stack_pointer


    movl %edx, %esp         // load new stack pointer
    pushl %ebx              // second argument to kernel_main() is the processor id
    pushl %eax              // first argument to kernel_main() is the boot info pointer

    call *%ecx              // jump to kernel (doesn't return)
    ud2

.section .rodata

multiboot2_error_msg:
    .asciz "boot failed: not loaded from multiboot2 compliant loader (signature missmatch)!"

cpuid_error_msg:
    .asciz "boot failed: cpuid instruction not available!"

pse_error_msg:
    .asciz "boot failed: PSE (page size extension) not available!"

.align 8
gdt32:
    // null descriptor
    .quad 0

    // 32-bit code descriptor
    .long 0x000FFFF
    .byte  0x00
    .byte  0b10011010       // P=1, DPL=0, S=1, E=1, DC=0, RW=1, A=0
    .byte  0b11001111       // G=1, DB=1
    .byte  0x00

    // 32-bit data descriptor
    .long 0x000FFFF
    .byte  0x00
    .byte  0b10010010       // P=1, DPL=0, S=1, E=0, DC=0, RW=1, A=0
    .byte  0b11001111       // G=1, DB=1
    .byte  0x00
gdt32_ptr:
    .short . - gdt32 - 1
    .long gdt32