


// This function reloads the cs segment register and takes relative addressing into account.
// Parameters:
// - 1: the segment selector to use (32-bit value)
.global load_cs
load_cs:
    // set up the stack frame
    pushl %ebp
    movl %esp, %ebp

    // load parameter 1 - the segment selector into %edx
    movl 8(%ebp), %edx  

    // obtain the value of %eip into %eax
    call load_cs_1
load_cs_1:
    popl %eax

    // add an offset to %eax so that we land at load_cs_2 after executing retf
    addl $(load_cs_2 - load_cs_1), %eax    

    // push the segment selector and return address onto the stack
    pushl %edx
    pushl %eax
    lret

load_cs_2:

    // restore the old stack frame and return
    movl %ebp, %esp
    popl %ebp
    ret
