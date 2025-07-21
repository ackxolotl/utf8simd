/// A UTF-8 error.
#[derive(Debug, Clone, PartialEq)]
pub struct Utf8Error;

impl core::fmt::Display for Utf8Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "invalid utf-8 sequence")
    }
}