use super::Simd8x16;

use core::simd::Simd;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

impl Simd8x16 {
    #[inline]
    pub fn prev<const N: i32>(&self, previous: Simd8x16) -> Simd8x16 where [(); { 16 - N } as usize]: {
        let c = __m128i::from(self.value);
        let p = __m128i::from(previous.value);

        let r = unsafe { _mm_alignr_epi8::<{ 16 - N }>(c, p) };

        Simd8x16::from(r)
    }

    #[inline]
    pub fn shr<const N: i32>(&self) -> Simd8x16 where [(); { 16 - N } as usize]: {
        let c = __m128i::from(self.value);

        let r = unsafe { _mm_srli_epi16::<N>(c) };

        Simd8x16::from(r) & Simd8x16::from(0xff >> N)
    }

    #[inline]
    pub fn lookup_16(&self, table: Simd8x16) -> Simd8x16 {
        let c = __m128i::from(self.value);
        let t = __m128i::from(table.value);

        let r = unsafe { _mm_shuffle_epi8(t, c) };

        Simd8x16::from(r)
    }

    #[inline]
    pub fn saturating_sub(&self, other: Simd8x16) -> Simd8x16 {
        let s = __m128i::from(self.value);
        let o = __m128i::from(other.value);

        let r = unsafe { _mm_subs_epu8(s, o) };

        Simd8x16::from(r)
    }
}

impl From<Simd8x16> for __m128i {
    fn from(value: Simd8x16) -> Self {
        value.value.into()
    }
}

impl From<__m128i> for Simd8x16 {
    fn from(value: __m128i) -> Self {
        Self {
            value: Simd::from(value),
        }
    }
}