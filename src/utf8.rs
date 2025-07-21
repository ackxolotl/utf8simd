use core::{mem, slice};
use core::simd::Simd;

use crate::{Utf8Error, Utf8Validator};

/// Converts a slice of bytes to a string slice.
pub fn from_utf8(v: &[u8]) -> Result<&str, Utf8Error> {
    // not worth it to use SIMD
    if v.len() < 128 {
        return core::str::from_utf8(v).map_err(|_| Utf8Error);
    }

    let mut validator = Utf8Validator::new();

    // data and length
    let mut ptr = v.as_ptr();
    let len = v.len();

    // end of the slice
    let end = unsafe { ptr.add(len) };

    // alignment offset for 64-byte boundary
    let offset = ptr.align_offset(64);

    // unaligned prefix if needed
    if offset < len {
        let mut padded = [0u8; 64];
        padded[64 - offset..].copy_from_slice(&v[..offset]);
        let chunk = Simd::from_array(padded);
        validator.next(&chunk)?;
        ptr = unsafe { ptr.add(offset) };
    }

    // process aligned 64-byte chunks
    while unsafe { ptr.add(64) } <= end {
        let chunk = unsafe { &*(ptr as *const _) };
        validator.next(chunk)?;
        ptr = unsafe { ptr.add(64) };
    }

    // handle remainder
    let len = unsafe { end.offset_from_unsigned(ptr) };
    let remaining = unsafe { slice::from_raw_parts(ptr, len) };
    let mut padded = [0u8; 64];
    padded[..len].copy_from_slice(remaining);
    let chunk = Simd::from_array(padded);
    validator.next(&chunk)?;

    // check for incomplete bytes
    validator.finish()?;

    Ok(unsafe { from_utf8_unchecked(v) })
}

/// Converts a slice of bytes to a string slice without checking that the string contains valid UTF-8.
///
/// # Safety
/// The bytes passed in must be valid UTF-8.
pub const unsafe fn from_utf8_unchecked(v: &[u8]) -> &str {
    #[allow(clippy::transmute_bytes_to_str)]
    unsafe { mem::transmute(v) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_utf8() {
        let bytes = b"Hello, world!";
        let str = from_utf8(bytes).unwrap();
        assert_eq!(bytes, str.as_bytes());
    }

    #[test]
    fn valid_utf8_empty() {
        let bytes = b"";
        let str = from_utf8(bytes).unwrap();
        assert_eq!(bytes, str.as_bytes());
    }

    #[test]
    fn invalid_utf8() {
        let bytes = b"\x1F\x8Babcdefg";
        let err = from_utf8(bytes).unwrap_err();
        assert_eq!(err, Utf8Error);
    }
}
