use core::{fmt, ops};

use crate::{phys::PhysAddr, PAGE_SHIFT, PAGE_SIZE};

pub struct TryFromPhysAddrError;

impl fmt::Debug for TryFromPhysAddrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PhysAddr too big to fit into VirtAddr")
    }
}

type Inner = usize;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtAddr(Inner);

impl VirtAddr {
    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn new(addr: Inner) -> Self {
        Self(addr)
    }

    pub const fn to_inner(self) -> Inner {
        self.0
    }

    pub fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }

    pub fn as_ptr_mut<T>(self) -> *mut T {
        self.0 as *mut T
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
        let addr = self.0.next_multiple_of(PAGE_SIZE);
        Self(addr)
    }

    /// Aligns the address up to `PAGE_SIZE`.
    ///
    /// Returns `None` if the operation would overflow.
    pub fn page_align_up_checked(self) -> Option<Self> {
        let addr = self.0.checked_next_multiple_of(PAGE_SIZE)?;
        Some(Self(addr))
    }

    /// Casts this virtual address to a physical address.
    /// This does a bit by bit conversion, not a translation.
    ///
    /// ## Panics
    /// Panics if the virtual address is to big to fit in a physical address.
    pub fn to_phys(self) -> PhysAddr {
        self.try_into().unwrap()
    }
}

impl TryFrom<PhysAddr> for VirtAddr {
    type Error = TryFromPhysAddrError;

    fn try_from(value: PhysAddr) -> Result<Self, Self::Error> {
        let inner: Inner = value
            .to_inner()
            .try_into()
            .map_err(|_| TryFromPhysAddrError)?;
        Ok(VirtAddr(inner))
    }
}

impl From<Inner> for VirtAddr {
    fn from(num: Inner) -> Self {
        Self(num)
    }
}

impl Into<Inner> for VirtAddr {
    fn into(self) -> Inner {
        self.0
    }
}

impl ops::Add for VirtAddr {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl ops::Add<Inner> for VirtAddr {
    type Output = Self;

    fn add(self, rhs: Inner) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl ops::AddAssign for VirtAddr {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl ops::AddAssign<Inner> for VirtAddr {
    fn add_assign(&mut self, rhs: Inner) {
        self.0 += rhs;
    }
}

impl ops::Sub for VirtAddr {
    type Output = Inner;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0 - rhs.0
    }
}

impl ops::Sub<Inner> for VirtAddr {
    type Output = Self;

    fn sub(self, rhs: Inner) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl ops::Mul for VirtAddr {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl ops::Mul<Inner> for VirtAddr {
    type Output = Self;

    fn mul(self, rhs: Inner) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl ops::Div for VirtAddr {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl ops::Div<Inner> for VirtAddr {
    type Output = Self;

    fn div(self, rhs: Inner) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl ops::Rem for VirtAddr {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl ops::Rem<Inner> for VirtAddr {
    type Output = Self;

    fn rem(self, rhs: Inner) -> Self::Output {
        Self(self.0 % rhs)
    }
}

impl ops::BitAnd for VirtAddr {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl ops::BitAnd<Inner> for VirtAddr {
    type Output = Self;

    fn bitand(self, rhs: Inner) -> Self::Output {
        Self(self.0 & rhs)
    }
}

impl ops::BitOr for VirtAddr {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl ops::BitOr<Inner> for VirtAddr {
    type Output = Self;

    fn bitor(self, rhs: Inner) -> Self::Output {
        Self(self.0 | rhs)
    }
}

impl ops::BitXor for VirtAddr {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl ops::BitXor<Inner> for VirtAddr {
    type Output = Self;

    fn bitxor(self, rhs: Inner) -> Self::Output {
        Self(self.0 ^ rhs)
    }
}

impl ops::Shl<Inner> for VirtAddr {
    type Output = Self;

    fn shl(self, rhs: Inner) -> Self::Output {
        Self(self.0 << rhs)
    }
}

impl ops::Shr<Inner> for VirtAddr {
    type Output = Self;

    fn shr(self, rhs: Inner) -> Self::Output {
        Self(self.0 >> rhs)
    }
}

impl fmt::Display for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VirtAddr({:#x})", self.0)
    }
}

impl fmt::Binary for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Octal for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::LowerHex for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::UpperHex for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(target_pointer_width = "64")]
impl fmt::Pointer for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#018x}", self.0)
    }
}

#[cfg(target_pointer_width = "32")]
impl fmt::Pointer for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#010x}", self.0)
    }
}
