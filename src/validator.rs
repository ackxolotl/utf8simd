use core::simd::num::SimdUint;
use core::simd::Simd;

use crate::error::Utf8Error;
use crate::simd::Simd8x16;

/// A stateful UTF-8 validator that processes data in 64-byte chunks.
///
/// The validator maintains state between chunks to handle multibyte UTF-8
/// sequences that may span chunk boundaries. It uses SIMD operations to
/// achieve high performance by processing multiple bytes simultaneously.
///
/// # Examples
///
/// ```rust
/// # #![feature(portable_simd)]
/// # use utf8simd::Utf8Validator;
/// # use core::simd::Simd;
/// let mut validator = Utf8Validator::new();
///
/// // process some UTF-8 data
/// let data = "Hello, world! 🦀".as_bytes();
/// let chunk = Simd::load_or_default(data);
/// validator.next(&chunk).unwrap();
///
/// // finish validation
/// validator.finish().unwrap();
/// ```
#[derive(Debug, Default)]
pub struct Utf8Validator {
    /// Accumulated error state across processed chunks
    error: Simd8x16,
    /// Previous chunk
    previous: Simd8x16,
    /// Incomplete multibyte sequences at the end of the previous chunk
    incomplete: Simd8x16,
}

impl Utf8Validator {
    /// Creates a new UTF-8 validator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use utf8simd::Utf8Validator;
    /// let validator = Utf8Validator::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates a 64-byte chunk of data.
    ///
    /// This method processes exactly 64 bytes of input data using SIMD operations.
    /// It includes an ASCII fast-path optimization that quickly validates pure ASCII.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(portable_simd)]
    /// # use utf8simd::Utf8Validator;
    /// # use core::simd::Simd;
    /// let mut validator = Utf8Validator::new();
    /// let data = "A".repeat(64);
    /// let chunk = Simd::from_slice(data.as_bytes());
    /// validator.next(&chunk).unwrap();
    /// ```
    #[inline]
    pub fn next(&mut self, data: &Simd<u8, 64>) -> crate::Result<()> {
        // fast path for ASCII-only data
        if core::intrinsics::likely(is_ascii(data)) {
            return Ok(());
        }

        self.validate_utf8(data)
    }

    /// Finalizes validation and checks for incomplete sequences.
    ///
    /// This method must be called after processing all input data to ensure
    /// that no incomplete multibyte UTF-8 sequences remain. Any incomplete
    /// sequence at the end of the input is considered an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use utf8simd::Utf8Validator;
    /// let mut validator = Utf8Validator::new();
    /// // ... process some data with validator.next() ...
    /// validator.finish().unwrap();
    /// ```
    #[inline]
    pub fn finish(&mut self) -> crate::Result<()> {
        // any incomplete sequences at the end of input are errors
        self.error |= self.incomplete;
        self.check_error()
    }

    /// Validates a 64-byte chunk containing non-ASCII data.
    #[inline]
    fn validate_utf8(&mut self, data: &Simd<u8, 64>) -> crate::Result<()> {
        let ptr = data.as_array().as_ptr();

        // split 64 byte chunk into four 16-byte SIMD vectors with minimal data movement
        let chunks = unsafe {
            [
                Simd8x16::from(Simd::from_slice(core::slice::from_raw_parts(ptr, 16))),
                Simd8x16::from(Simd::from_slice(core::slice::from_raw_parts(ptr.add(16), 16))),
                Simd8x16::from(Simd::from_slice(core::slice::from_raw_parts(ptr.add(32), 16))),
                Simd8x16::from(Simd::from_slice(core::slice::from_raw_parts(ptr.add(48), 16))),
            ]
        };

        let previous = self.previous;

        // validate the chunks
        self.validate_utf8_chunk(chunks[0], previous);
        self.validate_utf8_chunk(chunks[1], chunks[0]);
        self.validate_utf8_chunk(chunks[2], chunks[1]);
        self.validate_utf8_chunk(chunks[3], chunks[2]);

        // update validator state for the next chunk
        self.incomplete = is_incomplete(chunks[3]);
        self.previous = chunks[3];

        self.check_error()
    }

    /// Validates a single 16-byte chunk using the UTF-8 state machine.
    #[inline]
    fn validate_utf8_chunk(&mut self, data: Simd8x16, previous: Simd8x16) {
        let prev1 = data.prev::<1>(previous);
        let sc = special_cases(data, prev1);
        self.error |= multibyte_lengths(data, previous, sc);
    }

    /// Checks if any validation errors have been accumulated.
    #[inline]
    fn check_error(&self) -> crate::Result<()> {
        if core::intrinsics::unlikely(self.error.value().reduce_or() != 0) {
            Err(Utf8Error)
        } else {
            Ok(())
        }
    }
}

/// Fast ASCII detection for 64-byte chunks.
#[inline]
fn is_ascii(data: &Simd<u8, 64>) -> bool {
    (data.reduce_or() & 0x80) == 0
}

/// Detects incomplete multibyte sequences at the end of a chunk.
#[inline]
fn is_incomplete(data: Simd8x16) -> Simd8x16 {
    // Check the last 4 bytes for UTF-8 lead bytes that would require continuation
    // bytes: 0xC0-0xDF (2-byte), 0xE0-0xEF (3-byte), 0xF0-0xF7 (4-byte)
    let max_array = Simd8x16::new(
        255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 0xf0-1, 0xe0-1, 0xc0-1
    );

    data.gt_bits(max_array)
}

/// Identifies special UTF-8 validation cases using lookup tables.
#[inline]
fn special_cases(data: Simd8x16, previous: Simd8x16) -> Simd8x16 {
    // Bit 0 = Too Short (lead byte/ASCII followed by lead byte/ASCII)
    // Bit 1 = Too Long (ASCII followed by continuation)
    // Bit 2 = Overlong 3-byte
    // Bit 4 = Surrogate
    // Bit 5 = Overlong 2-byte
    // Bit 7 = Two Continuations
    const TOO_SHORT: u8   = 1 << 0;    // 11______ 0_______
                                       // 11______ 11______
    const TOO_LONG: u8    = 1 << 1;    // 0_______ 10______
    const OVERLONG_3: u8  = 1 << 2;    // 11100000 100_____
    const SURROGATE: u8   = 1 << 4;    // 11101101 101_____
    const OVERLONG_2: u8  = 1 << 5;    // 1100000_ 10______
    const TWO_CONTS: u8   = 1 << 7;    // 10______ 10______
    const TOO_LARGE: u8   = 1 << 3;    // 11110100 1001____
                                       // 11110100 101_____
                                       // 11110101 1001____
                                       // 11110101 101_____
                                       // 1111011_ 1001____
                                       // 1111011_ 101_____
                                       // 11111___ 1001____
                                       // 11111___ 101_____
    const TOO_LARGE_1000: u8 = 1 << 6; // 11110101 1000____
                                       // 1111011_ 1000____
                                       // 11111___ 1000____
    const OVERLONG_4: u8 = 1 << 6;     // 11110000 1000____

    let byte_1_high = previous.shr::<4>().lookup_16(
        Simd8x16::new(
            // 0_______ ________ <ASCII in byte 1>
            TOO_LONG, TOO_LONG, TOO_LONG, TOO_LONG,
            TOO_LONG, TOO_LONG, TOO_LONG, TOO_LONG,
            // 10______ ________ <continuation in byte 1>
            TWO_CONTS, TWO_CONTS, TWO_CONTS, TWO_CONTS,
            // 1100____ ________ <two byte lead in byte 1>
            TOO_SHORT | OVERLONG_2,
            // 1101____ ________ <two byte lead in byte 1>
            TOO_SHORT,
            // 1110____ ________ <three byte lead in byte 1>
            TOO_SHORT | OVERLONG_3 | SURROGATE,
            // 1111____ ________ <four+ byte lead in byte 1>
            TOO_SHORT | TOO_LARGE | TOO_LARGE_1000 | OVERLONG_4
        )
    );

    const CARRY: u8 = TOO_SHORT | TOO_LONG | TWO_CONTS; // These all have ____ in byte 1

    let byte_1_low = (previous & Simd8x16::from(0x0f)).lookup_16(
        Simd8x16::new(
            // ____0000 ________
            CARRY | OVERLONG_3 | OVERLONG_2 | OVERLONG_4,
            // ____0001 ________
            CARRY | OVERLONG_2,
            // ____001_ ________
            CARRY,
            CARRY,

            // ____0100 ________
            CARRY | TOO_LARGE,
            // ____0101 ________
            CARRY | TOO_LARGE | TOO_LARGE_1000,
            // ____011_ ________
            CARRY | TOO_LARGE | TOO_LARGE_1000,
            CARRY | TOO_LARGE | TOO_LARGE_1000,

            // ____1___ ________
            CARRY | TOO_LARGE | TOO_LARGE_1000,
            CARRY | TOO_LARGE | TOO_LARGE_1000,
            CARRY | TOO_LARGE | TOO_LARGE_1000,
            CARRY | TOO_LARGE | TOO_LARGE_1000,
            CARRY | TOO_LARGE | TOO_LARGE_1000,
            // ____1101 ________
            CARRY | TOO_LARGE | TOO_LARGE_1000 | SURROGATE,
            CARRY | TOO_LARGE | TOO_LARGE_1000,
            CARRY | TOO_LARGE | TOO_LARGE_1000
        )
    );

    let byte_2_high = data.shr::<4>().lookup_16(
        Simd8x16::new(
            // ________ 0_______ <ASCII in byte 2>
            TOO_SHORT, TOO_SHORT, TOO_SHORT, TOO_SHORT,
            TOO_SHORT, TOO_SHORT, TOO_SHORT, TOO_SHORT,

            // ________ 1000____
            TOO_LONG | OVERLONG_2 | TWO_CONTS | OVERLONG_3 | TOO_LARGE_1000 | OVERLONG_4,
            // ________ 1001____
            TOO_LONG | OVERLONG_2 | TWO_CONTS | OVERLONG_3 | TOO_LARGE,
            // ________ 101_____
            TOO_LONG | OVERLONG_2 | TWO_CONTS | SURROGATE  | TOO_LARGE,
            TOO_LONG | OVERLONG_2 | TWO_CONTS | SURROGATE  | TOO_LARGE,

            // ________ 11______
            TOO_SHORT, TOO_SHORT, TOO_SHORT, TOO_SHORT
        )
    );

    byte_1_high & byte_1_low & byte_2_high
}

/// Validates multibyte UTF-8 sequence lengths.
#[inline]
fn multibyte_lengths(data: Simd8x16, previous: Simd8x16, special_cases: Simd8x16) -> Simd8x16 {
    let prev2 = data.prev::<2>(previous);
    let prev3 = data.prev::<3>(previous);
    let must23 = must_be_2_3_continuation(prev2, prev3);
    let must23_80 = must23 & Simd8x16::from(0x80);
    must23_80 ^ special_cases
}

/// Determines which positions must be continuation bytes for 3 and 4-byte sequences.
#[inline]
fn must_be_2_3_continuation(previous2: Simd8x16, previous3: Simd8x16) -> Simd8x16 {
    let is_third_byte  = previous2.saturating_sub(Simd8x16::from(0xe0-0x80)); // Only 111_____ will be >= 0x80
    let is_fourth_byte = previous3.saturating_sub(Simd8x16::from(0xf0-0x80)); // Only 1111____ will be >= 0x80
    is_third_byte | is_fourth_byte
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ascii() {
        let simd: Simd<u8, 64> = Simd::from_slice("832,qqq\n123,aaa\n456,bbb\n666,ccc\n321,qqq\n394,ddd\n123,ask\n291,aew\n".as_bytes());
        assert!(is_ascii(&simd));

        let simd: Simd<u8, 64> = Simd::from_slice("832,qqq\n😀234\n456,bbb\n666,ccc\n321,qqq\n394,ddd\n123,ask\n291,aew\n".as_bytes());
        assert!(!is_ascii(&simd));
    }

    #[test]
    fn test_valid_utf8() {
        let mut v = Utf8Validator::new();

        let sequences = [
            "832,qqq\n123,aaa\n456,bbb\n666,ccc\n321,qqq\n394,ddd\n123,ask\n291,aew\n".as_bytes(),
            "832,qqq\n😀234\n456,bbb\n666,ccc\n321,qqq\n394,ddd\n123,ask\n291,aew\n".as_bytes(),
        ];

        for sequence in sequences {
            // Valid sequence for Rust standard library?
            core::str::from_utf8(sequence).unwrap();

            // Valid sequence for us?
            let simd: Simd<u8, 64> = Simd::from_slice(sequence);
            v.next(&simd).unwrap();
        }
    }

    #[test]
    fn test_invalid_utf8() {
        let sequences = [
            b"832,qqq\n\xC1\x3F12234\n456,bbb\n666,ccc\n321,qqq\n394,ddd\n123,ask\n291,aew\n",
            b"\x1F\x8Babc,def\nabc,def\nabc,def\n,abc,def\nabc,def\nabc,def\nabc,def\nab,c\n",
        ];

        for sequence in sequences {
            let mut v = Utf8Validator::new();

            // Invalid sequence for Rust standard library?
            assert!(core::str::from_utf8(sequence).is_err());

            // Invalid sequence for us?
            let simd: Simd<u8, 64> = Simd::from_slice(sequence);
            assert!(v.next(&simd).is_err());
        }
    }
}
