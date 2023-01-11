//! A simple wrapper to parse primitive Rust types from a slice of bytes `[u8]`.
//!
//! This is the crate for you, if all you need is to parse a **bespoke binary protocol**.
//! You provide a reference slice of bytes, and assuming you know what those bytes represent,
//! read the original values out of it.
//!
//! The core of this crate is [`BytesParser`], built with few principles in mind:
//!
//! * **simplicity:** just a _thin wrapper_ around an array of bytes,
//!   with a cursor to track progress.
//! * **zero-copy:** all you get back is either a bit-copied primitive,
//!   or a reference backed by the original bytes - never cloning.
//! * **only primitive types**: if you need to serialize/deserialize complex data structures,
//!   you probably want [serde].
//!
//! ## Features
//!
//! * Parse all primitive [scalar types], signed and unsigned,
//!   as well as [`&str`] and sub-slice of `&[u8]`.
//! * Internal, auto-updating cursor, to implement a simple scanning logic.
//! * Options to move the cursor arbitrarily, but safely, along the input slice.
//! * Support for [Endianness] selection (see [`ParsingEndian`]).
//! * Descriptive errors (see [`BytesParserError`]).
//! * Minimal dependencies.
//!
//! ## Examples
//!
//! ```compile_fail
//! use bytes_parser::{BytesParser, ParsingEndian};
//!
//! let input: &[u8] = /* a slice of bytes from somewhere */;
//!
//! // Crate a parser from a given slice of bytes
//! let mut parser = BytesParser::from(input);
//!
//! // Will use Big-Endian parsing
//! assert_eq!(parser.endian(), ParsingEndian::BE);
//! assert_eq!(ParsingEndian::BE, ParsingEndian::default());
//!
//! // Parse a string length and the string itself
//! let str_len = parser.parse_usize()?;
//! let str = parser.parse_str_utf8(str_len)?;
//! assert_eq!(str, "Forza Napoli Sempre!");
//! ```
//!
//! ## Alternatives
//!
//! * [nom]: steeper learning curve, but more feature complete.
//! * [serde]: mostly focused on SERialization/DEserialization of well known formats,
//!   but it can be setup to handle a slice of bytes.
//!
//! [nom]: https://crates.io/crates/nom
//! [serde]: https://crates.io/crates/serde
//! [Endianness]: https://en.wikipedia.org/wiki/Endianness
//! [scalar types]: https://doc.rust-lang.org/book/ch03-02-data-types.html#scalar-types

mod endianness;
mod errors;
mod parser;

pub use self::endianness::ParsingEndian;
pub use self::errors::BytesParserError;
pub use self::parser::BytesParser;
