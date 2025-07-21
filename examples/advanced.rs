#![feature(portable_simd)]

use core::simd::Simd;

use utf8simd::Utf8Validator;

fn main() -> utf8simd::Result<()> {
    let data = Simd::load_or_default(b"hello world!");

    let mut validator = Utf8Validator::default();
    validator.next(&data)?;

    // remember to check the end for incomplete bytes!
    validator.finish()
}
