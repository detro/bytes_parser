use crate::endianness::ParsingEndian;
use crate::errors::BytesParserError;

use std::convert::TryInto;
use std::mem;
use std::str;

/// A zero-copy bytes parser, useful when parsing bespoke binary protocols.
///
/// It wraps a reference to a byte-array, and adds a thin parsing layer: calls to the `parse_*`
/// reads the bytes and updates an internal cursor sequentially.
/// This makes for a very linear sequence of calls, when in need to consume, for example,
/// messages for a bespoke binary protocol.
///
/// By default, the parsing is done using the [`ParsingEndian::BE`] endian system, but that
/// can be configured.
///
/// The internal cursor progresses sequentially from position `0`
/// (i.e. no bytes has been parsed yet) to maximum position of [`BytesParser::length`]
/// (i.e. all bytes have been parsed).
///
/// If necessary, methods are provided to move the cursor around, with error checking in case the
/// cursor is moved outside the boundaries of the underlying array.
#[derive(Debug, Copy, Clone)]
pub struct BytesParser<'a> {
    buffer: &'a [u8],
    length: usize,
    cursor: usize,
    endian: ParsingEndian,
}

impl<'a> From<&'a [u8]> for BytesParser<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        BytesParser {
            buffer: bytes,
            length: bytes.len(),
            cursor: 0,
            endian: ParsingEndian::default(),
        }
    }
}

macro_rules! build_parse_type_fn {
    ($fn_name:ident, $parsed_type:ty) => {
        #[doc = "Parse a"]
        #[doc=stringify!($parsed_type)]
        #[doc = "and update the internal cursor accordingly.\n\n"]
        #[doc = "It produces an error if `BytesParser::parseable()` returns an amount inferior to"]
        #[doc = "the amount of bytes occupied by a "]
        #[doc=stringify!($parsed_type)]
        #[doc = "."]
        pub fn $fn_name(&mut self) -> Result<$parsed_type, BytesParserError> {
            let size = mem::size_of::<$parsed_type>();
            if self.parseable() < size {
                return Err(BytesParserError::NotEnoughBytesForTypeError(
                    stringify!($parsed_type).to_string(),
                ));
            }

            let start = self.cursor;
            let end = self.cursor + size;
            let slice = self.buffer[start..end].try_into().unwrap();

            let value = match self.endian {
                ParsingEndian::BE => <$parsed_type>::from_be_bytes(slice),
                ParsingEndian::LE => <$parsed_type>::from_le_bytes(slice),
            };

            self.cursor += size;

            Ok(value)
        }
    };
}

impl<'a> BytesParser<'a> {
    build_parse_type_fn!(parse_i8, i8);
    build_parse_type_fn!(parse_u8, u8);

    build_parse_type_fn!(parse_i16, i16);
    build_parse_type_fn!(parse_u16, u16);

    build_parse_type_fn!(parse_i32, i32);
    build_parse_type_fn!(parse_u32, u32);

    build_parse_type_fn!(parse_i64, i64);
    build_parse_type_fn!(parse_u64, u64);

    build_parse_type_fn!(parse_i128, i128);
    build_parse_type_fn!(parse_u128, u128);

    build_parse_type_fn!(parse_f32, f32);
    build_parse_type_fn!(parse_f64, f64);

    /// Parse a [`String`] and update the internal cursor accordingly.
    ///
    /// It produces an error if `BytesParser::parseable()` returns an amount
    /// inferior to the given `size`.
    ///
    /// Typically for binary protocols, the string is preceded by an integer representation of
    /// the size of the string in bytes. Unfortunately there is no single standard for how that
    /// integer is encoded (16 bits? 32 bits?), hence the required argument `size`.
    ///
    /// # Arguments
    ///
    /// * `size` - Size of the UTF-8 [`String`] to parse, in bytes. For Arabic characters, this will
    ///   be equivalent to the string length, as UTF-8 uses 1 byte per scalar value.
    ///   But for non-Arabic characters, UTF-8 might requires multiple bytes per scalar value.
    ///   More details can be found in the
    ///   [Rust Programming Language book](https://doc.rust-lang.org/book/ch08-02-strings.html#internal-representation).
    ///   Because of this, determining how many bytes to consume to parse the [`String`] is left
    ///   to the user.
    pub fn parse_str_utf8(&mut self, size: usize) -> Result<String, BytesParserError> {
        if self.parseable() < size {
            return Err(BytesParserError::NotEnoughBytesForStringError(size));
        }

        let start = self.cursor;
        let end = self.cursor + size;
        let slice = self.buffer[start..end].try_into().unwrap();

        match str::from_utf8(slice) {
            Ok(result) => {
                self.cursor += size;
                Ok(result.to_string())
            },
            Err(err) => Err(BytesParserError::StringParseError(err)),
        }
    }

    /// Parse a single [`char`] from a [`u32`] (i.e. 4 bytes).
    ///
    /// As per [`char` representation](https://doc.rust-lang.org/1.66.0/std/primitive.char.html#representation),
    /// Rust uses the fixed amount of 4 bytes to encode a single character.
    pub fn parse_char_u32(&mut self) -> Result<char, BytesParserError> {
        let u32value = self.parse_u32()?;
        let result = char::from_u32(u32value).ok_or(BytesParserError::InvalidU32ForCharError)?;
        Ok(result)
    }

    /// Length of the internal bytes array.
    pub const fn length(&self) -> usize {
        self.length
    }

    /// Returns [true] if the internal bytes array is empty.
    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Returns the 0-based position of the cursor.
    ///
    /// The index returned corresponds to the next bytes that would be parsed.
    pub const fn position(&self) -> usize {
        self.cursor
    }

    /// Returns [`true`] if the internal cursor points at the very start of the bytes array.
    ///
    /// When first created, this will return [`true`].
    pub const fn is_at_start(&self) -> bool {
        self.cursor == 0
    }

    /// Returns [`true`] if the internal cursor points at the very end of the bytes array.
    ///
    /// When all bytes have been parsed, this will return [`true`].
    pub const fn is_at_end(&self) -> bool {
        self.cursor == self.length
    }

    /// Returns the amount of bytes that can still be parsed.
    ///
    /// If [`BytesParser::is_at_start`] returns [`true`],
    /// then this will return the same as [`BytesParser::length`].
    /// If [`BytesParser::is_at_end`] returns [`true`],
    /// then this will return `0`.
    pub const fn parseable(&self) -> usize {
        self.length - self.cursor
    }

    /// Reset cursor to the very start of the bytes array.
    ///
    /// This can be used to re-parse bytes.
    ///
    /// After this is called, [`BytesParser::is_at_start`] will return [`true`]
    pub fn reset(&mut self) {
        self.cursor = 0
    }

    /// Move internal cursor forward by `amount`.
    ///
    /// The new cursor position corresponds to the next byte that would be parsed.
    /// It produces an error if the new cursor position would fall out-of-bound of the
    /// internal bytes array.
    ///
    /// # Arguments
    ///
    /// * `amount` - Amount of bytes to move forward the cursor.
    pub fn move_forward(&mut self, amount: &usize) -> Result<(), BytesParserError> {
        let mut new_cursor = self.cursor;
        new_cursor += amount;

        if new_cursor >= self.length {
            Err(BytesParserError::CursorOutOfBoundError(
                new_cursor as isize,
                self.length,
                self.cursor,
            ))
        } else {
            self.cursor = new_cursor;
            Ok(())
        }
    }

    /// Move internal cursor backward by `amount`.
    ///
    /// The new cursor position corresponds to the next byte that would be parsed.
    /// It produces an error if the new cursor position would fall out-of-bound of the
    /// internal bytes array.
    ///
    /// # Arguments
    ///
    /// * `amount` - Amount of bytes to move backward the cursor.
    pub fn move_backward(&mut self, amount: &usize) -> Result<(), BytesParserError> {
        let mut new_cursor = self.cursor as isize;
        new_cursor -= *amount as isize;

        if new_cursor < 0 {
            Err(BytesParserError::CursorOutOfBoundError(
                new_cursor,
                self.length,
                self.cursor,
            ))
        } else {
            self.cursor = new_cursor as usize;
            Ok(())
        }
    }

    /// Move internal cursor at `position`.
    ///
    /// The new cursor position corresponds to the next byte that would be parsed.
    /// It produces an error if the new cursor position would fall out-of-bound of the
    /// internal bytes array.
    ///
    /// # Arguments
    ///
    /// * `position` - Where to move the cursor at.
    pub fn move_at(&mut self, position: &usize) -> Result<(), BytesParserError> {
        if *position >= self.length {
            Err(BytesParserError::CursorOutOfBoundError(
                *position as isize,
                self.length,
                self.cursor,
            ))
        } else {
            self.cursor = *position;
            Ok(())
        }
    }

    /// Sets the [ParsingEndian] to be used when parsing scalar types from the internal bytes array.
    ///
    /// # Arguments
    ///
    /// * `endian` - The [ParsingEndian] to use when calling `BytesParser::parse_<scalar_type>`.
    pub fn set_endian(&mut self, endian: ParsingEndian) {
        self.endian = endian;
    }

    /// Return the [ParsingEndian] currently used.
    pub const fn endian(&self) -> ParsingEndian {
        self.endian
    }
}

#[cfg(test)]
mod tests {
    use super::BytesParser;
    use crate::ParsingEndian;

    #[test]
    fn parse_unsigned_scalars_using_big_endian() {
        let input: &[u8] = &[
            0x12, //< u8
            0x12, 0x34, //< u16
            0x12, 0x34, 0x56, 0x78, //< u32
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, //< u64
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE,
            0xF0, //< u128
        ];

        let mut p = BytesParser::from(input);

        assert_eq!(p.endian(), ParsingEndian::BE);
        assert_eq!(p.length(), 31);
        assert_eq!(p.parseable(), 31);
        assert_eq!(p.is_empty(), false);
        assert_eq!(p.is_at_start(), true);

        assert_eq!(p.parse_u8().unwrap(), 0x12);
        assert_eq!(p.parseable(), 30);
        assert_eq!(p.parse_u16().unwrap(), 0x1234);
        assert_eq!(p.parseable(), 28);
        assert_eq!(p.parse_u32().unwrap(), 0x12345678);
        assert_eq!(p.parseable(), 24);
        assert_eq!(p.parse_u64().unwrap(), 0x123456789ABCDEF0);
        assert_eq!(p.parseable(), 16);
        assert_eq!(p.parse_u128().unwrap(), 0x123456789ABCDEF0123456789ABCDEF0);
        assert_eq!(p.parseable(), 0);

        assert_eq!(p.is_at_end(), true);
    }

    #[test]
    fn parse_unsigned_scalars_using_little_endian() {
        let input: &[u8] = &[
            0x12, //< u8
            0x34, 0x12, //< u16
            0x78, 0x56, 0x34, 0x12, //< u32
            0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12, //< u64
            0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12, 0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34,
            0x12, //< u128
        ];

        let mut p = BytesParser::from(input);
        p.set_endian(ParsingEndian::LE);

        assert_eq!(p.endian(), ParsingEndian::LE);
        assert_eq!(p.length(), 31);
        assert_eq!(p.parseable(), 31);
        assert_eq!(p.is_empty(), false);
        assert_eq!(p.is_at_start(), true);

        assert_eq!(p.parse_u8().unwrap(), 0x12);
        assert_eq!(p.parseable(), 30);
        assert_eq!(p.parse_u16().unwrap(), 0x1234);
        assert_eq!(p.parseable(), 28);
        assert_eq!(p.parse_u32().unwrap(), 0x12345678);
        assert_eq!(p.parseable(), 24);
        assert_eq!(p.parse_u64().unwrap(), 0x123456789ABCDEF0);
        assert_eq!(p.parseable(), 16);
        assert_eq!(p.parse_u128().unwrap(), 0x123456789ABCDEF0123456789ABCDEF0);
        assert_eq!(p.parseable(), 0);

        assert_eq!(p.is_at_end(), true);
    }

    #[test]
    fn parse_signed_scalars_using_big_endian() {
        let input: &[u8] = &[
            0x12, //< i8
            0x12, 0x34, //< i16
            0x12, 0x34, 0x56, 0x78, //< i32
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, //< i64
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE,
            0xF0, //< i128
            0xFF, 0x7F, 0xFF, 0xFF, //< f32
            0x7F, 0xEF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, //< f64
        ];

        let mut p = BytesParser::from(input);

        assert_eq!(p.endian(), ParsingEndian::BE);
        assert_eq!(p.length(), 43);
        assert_eq!(p.is_empty(), false);
        assert_eq!(p.is_at_start(), true);

        assert_eq!(p.parse_i8().unwrap(), 0x12);
        assert_eq!(p.parse_i16().unwrap(), 0x1234);
        assert_eq!(p.parse_i32().unwrap(), 0x12345678);
        assert_eq!(p.parse_i64().unwrap(), 0x123456789ABCDEF0);
        assert_eq!(p.parse_i128().unwrap(), 0x123456789ABCDEF0123456789ABCDEF0);
        assert_eq!(p.parse_f32().unwrap(), f32::MIN);
        assert_eq!(p.parse_f64().unwrap(), f64::MAX);

        assert_eq!(p.is_at_end(), true);
    }

    #[test]
    fn parse_signed_scalars_using_little_endian() {
        let input: &[u8] = &[
            0x12, //< i8
            0x34, 0x12, //< i16
            0x78, 0x56, 0x34, 0x12, //< i32
            0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12, //< i64
            0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12, 0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34,
            0x12, //< i128
            0xFF, 0xFF, 0x7F, 0xFF, //< f32
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xEF, 0x7F, //< f64
        ];

        let mut p = BytesParser::from(input);
        p.set_endian(ParsingEndian::LE);
        assert_eq!(p.length(), 43);
        assert_eq!(p.is_empty(), false);
        assert_eq!(p.is_at_start(), true);

        assert_eq!(p.endian(), ParsingEndian::LE);

        assert_eq!(p.parse_i8().unwrap(), 0x12);
        assert_eq!(p.parse_i16().unwrap(), 0x1234);
        assert_eq!(p.parse_i32().unwrap(), 0x12345678);
        assert_eq!(p.parse_i64().unwrap(), 0x123456789ABCDEF0);
        assert_eq!(p.parse_i128().unwrap(), 0x123456789ABCDEF0123456789ABCDEF0);
        assert_eq!(p.parse_f32().unwrap(), f32::MIN);
        assert_eq!(p.parse_f64().unwrap(), f64::MAX);

        assert_eq!(p.is_at_end(), true);
    }

    #[test]
    fn parse_moving_the_cursor_around() {
        let input: &[u8] = &[
            0x12, //< u8
            0x12, 0x34, //< u16
            0x12, 0x34, 0x56, 0x78, //< u32
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, //< u64
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE,
            0xF0, //< u128
        ];

        let mut p = BytesParser::from(input);

        // Parse the first
        assert_eq!(p.position(), 0);
        assert_eq!(p.parse_u8().unwrap(), 0x12);

        // Move forward to the last (u128)
        assert_eq!(p.move_forward(&14), Ok(()));
        assert_eq!(p.parse_u128().unwrap(), 0x123456789ABCDEF0123456789ABCDEF0);
        assert_eq!(p.position(), 31);

        // Move backward to the third scalar value (u32)
        assert_eq!(p.move_backward(&28), Ok(()));
        assert_eq!(p.parse_u32().unwrap(), 0x12345678);

        // Move to where the last scalar begin (u128)
        assert_eq!(p.move_at(&15), Ok(()));
        assert_eq!(p.parse_u128().unwrap(), 0x123456789ABCDEF0123456789ABCDEF0);

        // Move to where the second scalar begins (u16)
        assert_eq!(p.move_at(&1), Ok(()));
        assert_eq!(p.parse_u16().unwrap(), 0x1234);

        // Move back at the beginning
        p.reset();
        assert_eq!(p.position(), 0);
        assert_eq!(p.parse_u8().unwrap(), 0x12);
    }

    #[test]
    fn parse_string() {
        let input: &[u8] = &[
            0x00, 0x13, //< u16
            0x46, 0x6F, 0x72, 0x7A, 0x61, 0x20, 0x4E, 0x61, 0x70, 0x6F, 0x6C, 0x69, 0x20, 0x53, 0x65, 0x6D, 0x70, 0x72,
            0x65,
        ];

        let mut p = BytesParser::from(input);

        let str_len = p.parse_u16().unwrap();
        assert_eq!(str_len, 19);

        let str = p.parse_str_utf8(str_len as usize).unwrap();
        assert_eq!(str, "Forza Napoli Sempre");
    }

    #[test]
    fn parse_char() {
        let input: &[u8] = &[
            0x00, 0x01, 0xF9, 0x80, //< crab, encoded using the big-endian system
            0x80, 0xF9, 0x01, 0x00, //< crab, encoded using the little-endian system
        ];

        let mut p = BytesParser::from(input);

        assert_eq!(p.parse_char_u32().unwrap(), '🦀');

        p.set_endian(ParsingEndian::LE);

        assert_eq!(p.parse_char_u32().unwrap(), '🦀');
    }
}
