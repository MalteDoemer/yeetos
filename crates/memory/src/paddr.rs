use core::{fmt, ops};

use crate::{PAGE_SHIFT, PAGE_SIZE};

use super::vaddr::VirtAddr;

pub struct TryFromVirtAddrError;

impl fmt::Debug for TryFromVirtAddrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("VirtAddr too big to fit into PhysAddr")
    }
}

#[cfg(target_arch = "x86_64")]
type Inner = u64;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysAddr(Inner);

impl PhysAddr {
    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn new(addr: Inner) -> Self {
        Self(addr)
    }

    pub const fn to_inner(self) -> Inner {
        self.0
    }

    /// Aligns the address down to `PAGE_SIZE`.
    pub fn page_align_down(self) -> Self {
        let addr = self.0 >> PAGE_SHIFT;
        Self(addr << PAGE_SHIFT)
    }

    /// Aligns the address up to `PAGE_SIZE`.
    ///
    /// ### Panics
    /// based on the `overflow-checks` setting
    pub fn page_align_up(self) -> Self {
        let addr = self.0.next_multiple_of(PAGE_SIZE as Inner);
        Self(addr)
    }

    /// Aligns the address up to `PAGE_SIZE`.
    ///
    /// Returns `None` if the operation would overflow.
    pub fn page_align_up_checked(self) -> Option<Self> {
        let addr = self.0.checked_next_multiple_of(PAGE_SIZE as Inner)?;
        Some(Self(addr))
    }

    /// Checks if the address is aligned to `PAGE_SIZE`.
    pub fn is_page_aligned(self) -> bool {
        self == self.page_align_down()
    }

    /// Casts this physical address to a virtual address.
    /// This does a bit by bit conversion, not a translation.
    ///
    /// ## Panics
    /// Panics if the physical address is to big to fit in a virtual address.   
    pub fn to_virt(self) -> VirtAddr {
        self.try_into().unwrap()
    }
}

impl TryFrom<VirtAddr> for PhysAddr {
    type Error = TryFromVirtAddrError;

    fn try_from(value: VirtAddr) -> Result<Self, Self::Error> {
        let inner: Inner = value
            .to_inner()
            .try_into()
            .map_err(|_| TryFromVirtAddrError)?;
        Ok(PhysAddr(inner))
    }
}

impl From<Inner> for PhysAddr {
    fn from(num: Inner) -> Self {
        Self(num)
    }
}

impl Into<Inner> for PhysAddr {
    fn into(self) -> Inner {
        self.0
    }
}

impl ops::Add for PhysAddr {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl ops::Add<Inner> for PhysAddr {
    type Output = Self;

    fn add(self, rhs: Inner) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl ops::AddAssign for PhysAddr {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl ops::AddAssign<Inner> for PhysAddr {
    fn add_assign(&mut self, rhs: Inner) {
        self.0 += rhs;
    }
}

impl ops::Sub for PhysAddr {
    type Output = Inner;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0 - rhs.0
    }
}

impl ops::Sub<Inner> for PhysAddr {
    type Output = Self;

    fn sub(self, rhs: Inner) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl ops::Mul for PhysAddr {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl ops::Mul<Inner> for PhysAddr {
    type Output = Self;

    fn mul(self, rhs: Inner) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl ops::Div for PhysAddr {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl ops::Div<Inner> for PhysAddr {
    type Output = Self;

    fn div(self, rhs: Inner) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl ops::Rem for PhysAddr {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl ops::Rem<Inner> for PhysAddr {
    type Output = Self;

    fn rem(self, rhs: Inner) -> Self::Output {
        Self(self.0 % rhs)
    }
}

impl ops::BitAnd for PhysAddr {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl ops::BitAnd<Inner> for PhysAddr {
    type Output = Self;

    fn bitand(self, rhs: Inner) -> Self::Output {
        Self(self.0 & rhs)
    }
}

impl ops::BitOr for PhysAddr {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl ops::BitOr<Inner> for PhysAddr {
    type Output = Self;

    fn bitor(self, rhs: Inner) -> Self::Output {
        Self(self.0 | rhs)
    }
}

impl ops::BitXor for PhysAddr {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl ops::BitXor<Inner> for PhysAddr {
    type Output = Self;

    fn bitxor(self, rhs: Inner) -> Self::Output {
        Self(self.0 ^ rhs)
    }
}

impl ops::Shl<Inner> for PhysAddr {
    type Output = Self;

    fn shl(self, rhs: Inner) -> Self::Output {
        Self(self.0 << rhs)
    }
}

impl ops::Shr<Inner> for PhysAddr {
    type Output = Self;

    fn shr(self, rhs: Inner) -> Self::Output {
        Self(self.0 >> rhs)
    }
}

impl fmt::Display for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PhysAddr({:#x})", self.0)
    }
}

impl fmt::Binary for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Octal for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::LowerHex for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::UpperHex for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Pointer for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#018x}", self.0)
    }
}
