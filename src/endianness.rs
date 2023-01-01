/// Control which endian system to use when parsing raw bytes.
///
/// This is crucial when parsing scalar values from byte-representation, as it determines the order
/// in which bytes are read and interpreted to reconstruct the original scalar value.
///
/// More details about Endianness can be found [here](https://en.wikipedia.org/wiki/Endianness).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ParsingEndian {
    /// Read using the big-endian (BE) byte order system (default).
    ///
    /// A big-endian system stores the most significant byte of a word at the smallest memory
    /// address and the least significant byte at the largest.
    ///
    /// **NOTE:** This is the **default** endian for this crate.
    BE,

    /// Read using the little-endian (LE) byte order system.
    ///
    /// A little-endian system stores the least significant byte of a word at the smallest memory
    /// address, and the most significant byte at the largest.
    LE,
}

impl Default for ParsingEndian {
    /// Default value for [ParsingEndian] is [ParsingEndian::BE].
    fn default() -> Self {
        ParsingEndian::BE
    }
}
