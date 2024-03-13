//! This module implements a rudimentary implementation to
//! measure elapsed time using the tsc register of the x86 cpu.
//!
//! Note that the tsc is normally not recomended to measure time
//! since it is not guaranteed that every increment of the tsc takes
//! the same time. But since we are not changing power states or cpu
//! modes this should not be a serious problem.
//!
//! It is also not recomended to use this module for longer time periods because
//! there is no stride correction.
//!
//! Some ideas and calculations are taken from the linux kernel:
//! https://github.com/torvalds/linux/blob/master/arch/x86/kernel/tsc.c
//!
//! The idea of how to measure time using the tsc is as follows:
//! 1. First compute the frequency of the tsc using the PIT
//! 2. To get the current nanoseconds we scale the tsc count with the precomputed constant
//!
//! More information is in the `init()` and `now_ns()` function.
//!
use core::{arch::asm, sync::atomic::Ordering};

use crate::devices::pit;

use super::pic;

const NS_SCALE: u64 = 16777216;

/// This variable will contain the value: `(1_000_000_000 * NS_SCALE) / cycles_per_second`
static mut TSC_NS_FACTOR: u64 = 0;

fn rdtsc() -> u64 {
    unsafe {
        let lower: u64;
        let higher: u64;
        asm!(
            "lfence",
            "rdtsc",
            out("edx") higher,
            out("eax") lower,
        );

        higher << 32 | lower
    }
}

pub fn init() {
    // We have to figure out how often the rdtsc counter increments per second.
    // this is done using the pit because it has a fixed frequency.
    //
    // The frequnecy of the pit is 1.193182 MHz.
    // We choose a divisor of 2685.
    //
    // pit_freq = 1193182 Hz
    // pit_div  = 2685
    // time_per_tick = 1_000_000_000_000 / (pit_freq / pit_div) = 2250285 ns
    //
    // These constants are found in pit.rs
    //
    // Now we have to figure out how many tsc cyles happen in one pit-tick.

    // Unmask the first IRQ
    pic::unmask_irq(0);

    // enable interrupts
    unsafe {
        asm!("sti");
    }

    let initial_tick = pit::PIT_TICKS.load(Ordering::SeqCst);

    // wait until the start of the next pit-tick
    let mut start_tick;
    let mut start_cycle;
    loop {
        start_tick = pit::PIT_TICKS.load(Ordering::SeqCst);
        start_cycle = rdtsc();
        if initial_tick != start_tick {
            break;
        }
    }

    // wait one pit-tick;
    let mut end_tick;
    let mut end_cycle;
    loop {
        end_tick = pit::PIT_TICKS.load(Ordering::SeqCst);
        end_cycle = rdtsc();
        if end_tick != start_tick {
            break;
        }
    }

    // disable interrupts
    unsafe {
        asm!("cli");
    }

    // mask IRQ 0 again
    pic::mask_irq(0);

    // we should only have waited one pit-tick.
    assert_eq!(start_tick + 1, end_tick);

    let cycles_per_billion_pits: u64 = 1_000_000_000 * (end_cycle - start_cycle);
    let cycles_per_second = cycles_per_billion_pits / pit::NANO_SECONDS_PER_PIT;

    unsafe {
        TSC_NS_FACTOR = (1_000_000_000 * NS_SCALE) / cycles_per_second;
    }
}

/// Time from arbitrary epoch in nano seconds
pub fn now_ns() -> u64 {
    let cycle = rdtsc();

    let factor = unsafe { TSC_NS_FACTOR };

    // the basic idea is here from the linux kernel
    // https://github.com/torvalds/linux/blob/master/arch/x86/kernel/tsc.c

    // the calculation here is:
    //      ns = (cycle * 10⁹) / rate
    //         = cycle * (10⁹ / rate)
    //         = (cycle * ((10⁹ * SCALE) / rate)) / SCALE
    //
    //
    // the term: `(10⁹ * SCALE) / rate` can be precomputed during initialization
    //
    // that gives us following equation:
    //      ns = (cycle * tsc_ns_factor) / SCALE

    (cycle * factor) / NS_SCALE
}

pub fn busy_sleep_ms(millis: u64) {
    let start = now_ns();

    let goal = start + millis * 1_000_000;

    while now_ns() < goal {
        core::hint::spin_loop()
    }
}
