use super::Simd8x16;

use core::simd::{Simd, cmp::SimdPartialOrd, num::SimdUint};

impl Simd8x16 {
    #[inline]
    pub fn prev<const N: i32>(&self, previous: Simd8x16) -> Simd8x16 where [(); { 16 - N } as usize]: {
        let mut result = [0u8; 16];
        let prev_array = previous.value.to_array();
        let curr_array = self.value.to_array();

        for i in 0..16 {
            let src_idx = (16 - N as usize + i) % 32;
            result[i] = if src_idx < 16 {
                prev_array[src_idx]
            } else {
                curr_array[src_idx - 16]
            };
        }

        Simd8x16::from(Simd::from_array(result))
    }

    #[inline]
    pub fn shr<const N: i32>(&self) -> Simd8x16 where [(); { 16 - N } as usize]: {
        let mut result = [0u8; 16];
        let input = self.value.to_array();
        for i in 0..16 {
            result[i] = input[i] >> N;
        }
        Simd8x16::from(Simd::from_array(result))
    }

    #[inline]
    pub fn lookup_16(&self, table: Simd8x16) -> Simd8x16 {
        let masked_indices = self.value & Simd::splat(0x0f);
        let result = table.value.swizzle_dyn(masked_indices);

        let mask = self.value.simd_lt(Simd::splat(16));
        let final_result = mask.select(result, Simd::splat(0));

        Simd8x16::from(final_result)
    }

    #[inline]
    pub fn saturating_sub(&self, other: Simd8x16) -> Simd8x16 {
        Simd8x16::from(self.value.saturating_sub(other.value))
    }
}