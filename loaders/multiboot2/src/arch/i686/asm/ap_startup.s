

// Copies the ap_trampoline code to address 0x8000.
// This is necessary since during ap_startup we set the starting
// address to 0x8000.
.global copy_ap_trampoline
copy_ap_trampoline:
    ret
    // hlt
    // jmp copy_ap_trampoline


// Starts an application processor.
// Parameters:
// - ?? local apic address of bsp
// - ?? id of the ap to start
.global startup_ap
startup_ap:
    ret
    // hlt
    // jmp startup_ap