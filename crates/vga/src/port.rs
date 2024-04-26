use core::marker::PhantomData;

use x86::io::{inb, inl, inw, outb, outl, outw};

pub trait PortSize {
    unsafe fn read(addr: u16) -> Self;

    unsafe fn write(addr: u16, value: Self);
}

impl PortSize for u8 {
    unsafe fn read(addr: u16) -> Self {
        unsafe { inb(addr) }
    }

    unsafe fn write(addr: u16, value: Self) {
        unsafe { outb(addr, value) }
    }
}

impl PortSize for u16 {
    unsafe fn read(addr: u16) -> Self {
        unsafe { inw(addr) }
    }

    unsafe fn write(addr: u16, value: Self) {
        unsafe { outw(addr, value) }
    }
}

impl PortSize for u32 {
    unsafe fn read(addr: u16) -> Self {
        unsafe { inl(addr) }
    }

    unsafe fn write(addr: u16, value: Self) {
        unsafe { outl(addr, value) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Port<T: PortSize> {
    addr: u16,
    _phantom: PhantomData<T>,
}

impl<T: PortSize> Port<T> {
    pub const fn new(addr: u16) -> Self {
        Self {
            addr,
            _phantom: PhantomData,
        }
    }

    pub unsafe fn read(&self) -> T {
        unsafe { T::read(self.addr) }
    }

    pub unsafe fn write(&mut self, value: T) {
        unsafe { T::write(self.addr, value) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortReadOnly<T: PortSize> {
    addr: u16,
    _phantom: PhantomData<T>,
}

impl<T: PortSize> PortReadOnly<T> {
    pub const fn new(addr: u16) -> Self {
        Self {
            addr,
            _phantom: PhantomData,
        }
    }

    pub unsafe fn read(&self) -> T {
        unsafe { T::read(self.addr) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortWriteOnly<T: PortSize> {
    addr: u16,
    _phantom: PhantomData<T>,
}

impl<T: PortSize> PortWriteOnly<T> {
    pub const fn new(addr: u16) -> Self {
        Self {
            addr,
            _phantom: PhantomData,
        }
    }

    pub unsafe fn write(&mut self, value: T) {
        unsafe { T::write(self.addr, value) }
    }
}
