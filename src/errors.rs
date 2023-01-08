use thiserror::Error;

use std::str::Utf8Error;

#[allow(unused_imports)]
use crate::parser::BytesParser;

/// All the errors that [BytesParser] can potentially produce.
#[derive(Error, Debug, Eq, PartialEq)]
pub enum BytesParserError {
    /// Not enough bytes left (i.e. [BytesParser::parseable]) to parse a scalar type from it.
    #[error("Not enough bytes left to parse for {0}")]
    NotEnoughBytesForTypeError(String),

    /// Not enough bytes left (i.e. [BytesParser::parseable]) to parse a string of given bytes from it.
    #[error("Not enough bytes left to parse a string of {0} bytes")]
    NotEnoughBytesForStringError(usize),

    /// Not enough bytes left (i.e. [BytesParser::parseable]) to cut a slice of given bytes from it.
    #[error("Not enough bytes left to cut a slice of size {0}")]
    NotEnoughBytesForSlice(usize),

    /// Position resulting from moving the cursor to or by a given amount, would place the cursor out-of-bound.
    #[error("Moving cursor to/by {0} would place it out-of-bound: bytes array length is {1} and cursor is at {2}")]
    CursorOutOfBoundError(isize, usize, usize),

    /// Failed to parse a UTF-8 [String] from the given bytes.
    #[error("Failed to parse UTF-8 string: {0}")]
    StringParseError(#[source] Utf8Error),

    /// Failed to parse a [char] from a [u32] worth of bytes (i.e. 4 bytes).
    #[error("Invalid char found in u32")]
    InvalidU32ForCharError,
}
