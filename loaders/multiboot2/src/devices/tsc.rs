use core::{
    arch::asm,
    sync::atomic::{AtomicU32, Ordering},
};

use x86::dtables::{lidt, DescriptorTablePointer};

use crate::devices::pit;

use super::pic;

const NS_SCALE: u64 = 16777216;

/// This variable will contain the value: `(1_000_000_000 * NS_SCALE) / cycles_per_second`
static mut TSC_NS_FACTOR: u64 = 0;

static PIT_TICKS: AtomicU32 = AtomicU32::new(0);

#[repr(C)]
struct IntStackFrame {
    frame: [u64; 5],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset1: u16,
    selector: u16,
    ist: u8,
    type_attrs: u8,
    offset2: u16,
    offset3: u32,
    reserved: u32,
}

impl IdtEntry {
    pub const fn empty() -> Self {
        IdtEntry {
            offset1: 0,
            selector: 0,
            ist: 0,
            type_attrs: 0,
            offset2: 0,
            offset3: 0,
            reserved: 0,
        }
    }

    pub fn irq_handler(handler_address: usize) -> Self {
        let raw = handler_address;

        let offset1 = (raw & 0xFFFF) as u16;
        let offset2 = ((raw >> 16) & 0xFFFF) as u16;
        let offset3 = ((raw >> 32) & 0xFFFFFFFF) as u32;

        let selector = 1 << 3; // Kernel code segment
        let ist = 0;
        let type_attrs = 0x8E; // p=0b1, dpl=0b00, type=0b1110

        IdtEntry {
            offset1,
            selector,
            ist,
            type_attrs,
            offset2,
            offset3,
            reserved: 0,
        }
    }
}

// #[repr(C, packed)]
// struct IDTR {
//     size: u16,
//     offset: u64,
// }

extern "x86-interrupt" fn pit_interrupt(_frame: IntStackFrame) {
    PIT_TICKS.fetch_add(1, Ordering::SeqCst);
    pic::send_eoi(0);
}

pub fn rdtsc() -> u64 {
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
    // To do this we install a temporary interrupt handler for irq0.
    //

    // Create a temporary IDT
    let mut temp_idt: [IdtEntry; 256] = [IdtEntry::empty(); 256];
    temp_idt[0x20] = IdtEntry::irq_handler(pit_interrupt as usize);

    // Load IDT
    let desc = DescriptorTablePointer::new_from_slice(&temp_idt);
    unsafe {
        lidt(&desc);
    };

    // Unmask the first IRQ
    pic::unmask_irq(0);

    // enable interrupts
    unsafe {
        asm!("sti");
    }

    let initial_tick = PIT_TICKS.load(Ordering::SeqCst);

    // wait until the start of the next pit-tick
    let mut start_tick;
    let mut start_cycle;
    loop {
        start_tick = PIT_TICKS.load(Ordering::SeqCst);
        start_cycle = rdtsc();
        if initial_tick != start_tick {
            break;
        }
    }

    // wait one pit-tick;
    let mut end_tick;
    let mut end_cycle;
    loop {
        end_tick = PIT_TICKS.load(Ordering::SeqCst);
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

    // remove temporary IDT
    let desc: DescriptorTablePointer<()> = DescriptorTablePointer::default();
    unsafe {
        lidt(&desc);
    }

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
