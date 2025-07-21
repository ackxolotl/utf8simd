//! # utf8simd
//!
//! A high-performance UTF-8 validation library that uses SIMD operations for
//! fast validation of byte sequences, based on simdjson's UTF-8 validation.

#![no_std]

#![feature(portable_simd)]
#![feature(core_intrinsics)]
#![feature(generic_const_exprs)]

mod error;
mod simd;
mod utf8;
mod validator;

pub use error::Utf8Error;
pub use utf8::{from_utf8, from_utf8_unchecked};
pub use validator::Utf8Validator;

/// A UTF-8 validation result.
pub type Result<T> = core::result::Result<T, Utf8Error>;
