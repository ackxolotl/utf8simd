use super::Simd8x16;

use core::simd::Simd;
use core::arch::aarch64::*;

impl Simd8x16 {
    #[inline]
    pub fn prev<const N: i32>(&self, previous: Simd8x16) -> Simd8x16 where [(); { 16 - N } as usize]: {
        let c = uint8x16_t::from(self.value);
        let p = uint8x16_t::from(previous.value);

        let r = unsafe { vextq_u8::<{ 16 - N }>(p, c) };

        Simd8x16::from(r)
    }

    #[inline]
    pub fn shr<const N: i32>(&self) -> Simd8x16 where [(); { 16 - N } as usize]: {
        let input = uint8x16_t::from(self.value);

        let shifted = unsafe { vshrq_n_u8::<N>(input) };

        let mask = unsafe { vdupq_n_u8(0xff >> N) };
        let result = unsafe { vandq_u8(shifted, mask) };

        Simd8x16::from(result)
    }

    #[inline]
    pub fn lookup_16(&self, table: Simd8x16) -> Simd8x16 {
        let indices = uint8x16_t::from(self.value);
        let tbl = uint8x16_t::from(table.value);

        let result = unsafe { vqtbl1q_u8(tbl, indices) };

        Simd8x16::from(result)
    }

    #[inline]
    pub fn saturating_sub(&self, other: Simd8x16) -> Simd8x16 {
        let a = uint8x16_t::from(self.value);
        let b = uint8x16_t::from(other.value);

        let result = unsafe { vqsubq_u8(a, b) };

        Simd8x16::from(result)
    }
}

impl From<Simd8x16> for uint8x16_t {
    fn from(value: Simd8x16) -> Self {
        value.value.into()
    }
}

impl From<uint8x16_t> for Simd8x16 {
    fn from(value: uint8x16_t) -> Self {
        Self {
            value: Simd::from(value),
        }
    }
}