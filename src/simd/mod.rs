use core::simd::Simd;
use core::ops::{BitAnd, BitOr, BitOrAssign, BitXor};

/// 16-element u8 SIMD vector for UTF-8 validation
#[derive(Copy, Clone, Debug, Default)]
pub struct Simd8x16 {
    value: Simd<u8, 16>,
}

impl Simd8x16 {
    /// Create a new SIMD vector from 16 individual bytes
    #[allow(clippy::too_many_arguments)]
    pub fn new(v0: u8, v1: u8, v2: u8, v3: u8, v4: u8, v5: u8, v6: u8, v7: u8, v8: u8, v9: u8, v10: u8, v11: u8, v12: u8, v13: u8, v14: u8, v15: u8) -> Self {
        Self {
            value: Simd::from_array([v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10, v11, v12, v13, v14, v15]),
        }
    }

    /// Greater than bits (used for comparison)
    #[inline]
    pub fn gt_bits(&self, other: Self) -> Self {
        self.saturating_sub(other)
    }

    /// Access the underlying SIMD value
    #[inline]
    pub fn value(&self) -> Simd<u8, 16> {
        self.value
    }
}

// common trait implementations
impl BitAnd for Simd8x16 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self { value: self.value & rhs.value }
    }
}

impl BitOr for Simd8x16 {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self { value: self.value | rhs.value }
    }
}

impl BitOrAssign for Simd8x16 {
    fn bitor_assign(&mut self, rhs: Self) {
        self.value |= rhs.value;
    }
}

impl BitXor for Simd8x16 {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self { value: self.value ^ rhs.value }
    }
}

impl From<u8> for Simd8x16 {
    fn from(value: u8) -> Self {
        Self {
            value: Simd::splat(value),
        }
    }
}

impl From<Simd<u8, 16>> for Simd8x16 {
    fn from(value: Simd<u8, 16>) -> Self {
        Self { value }
    }
}

impl From<Simd8x16> for Simd<u8, 16> {
    fn from(value: Simd8x16) -> Self {
        value.value
    }
}

// architecture-specific implementations
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;

#[cfg(target_arch = "aarch64")]
mod aarch64;

// fallback portable implementation for other architectures
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
mod portable;