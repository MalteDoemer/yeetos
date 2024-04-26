use core::arch::asm;

use uefi::table::boot::BootServices;

const NS_SCALE: u64 = 16777216;

/// This variable will contain the value: `(1_000_000_000 * NS_SCALE) / cycles_per_second`
static mut TSC_NS_FACTOR: u64 = 0;

#[inline(always)]
fn rdtsc() -> u64 {
    unsafe {
        let lower: u32;
        let higher: u32;
        asm!(
            "lfence",
            "rdtsc",
            out("edx") higher,
            out("eax") lower,
        );

        (higher as u64) << 32 | (lower as u64)
    }
}

pub fn init(boot_services: &BootServices) {
    let start_cycles = rdtsc();
    boot_services.stall(1000); // 1000 us = 1000_000 ns
    let end_cycles = rdtsc();

    let diff = end_cycles - start_cycles;

    let cycles_per_second = diff * 1_000;

    unsafe {
        TSC_NS_FACTOR = (1_000_000_000 * NS_SCALE) / cycles_per_second;
    }
}

/// Time from arbitrary epoch in nano seconds
#[inline(always)]
pub fn now_ns() -> u64 {
    let cycle = rdtsc();

    let factor = unsafe { TSC_NS_FACTOR };

    // the calculation here is:
    //      ns = (cycle * 10⁹) / rate
    //         = cycle * (10⁹ / rate)
    //         = (cycle * ((10⁹ * SCALE) / rate)) / SCALE
    //
    //
    // the term: `(10⁹ * SCALE) / rate` can be precomputed during initialization
    //
    // that gives us following equation:
    //      ns = (cycle * TSC_NS_FACTOR) / SCALE

    (cycle * factor) / NS_SCALE
}

#[inline(always)]
pub fn busy_sleep_ms(millis: u64) {
    let start = now_ns();

    let goal = start + millis * 1_000_000;

    while now_ns() < goal {
        core::hint::spin_loop()
    }
}

#[inline(always)]
pub fn busy_sleep_us(micros: u64) {
    let start = now_ns();

    let goal = start + micros * 1_000;

    while now_ns() < goal {
        core::hint::spin_loop()
    }
}
