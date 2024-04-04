use x86::bits64::rflags::{self, RFlags};

/// This function checks if interrupts are enabled.
#[inline]
#[must_use]
pub fn are_enabled() -> bool {
    rflags::read().contains(RFlags::FLAGS_IF)
}

/// This function enables interrupts.
///
/// # Safety
/// Enabling interrupts can cause serious memory unsafety if handled incorrectly.
/// - Interrupt handling needs to be correctly set up beforehand.
/// - Deadlocks and/or data-races must be considered carefully.
pub unsafe fn enable() {
    unsafe { x86::irq::enable() }
}

/// This function disables interrupts.
///
/// # Safety
/// Disabling interrupts can have an overall effect on the system and must be done with care.
pub unsafe fn disable() {
    unsafe { x86::irq::disable() }
}
