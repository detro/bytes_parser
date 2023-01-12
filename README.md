# bytes_parser

<div align="center" style="text-align: center;">

[![CI](https://img.shields.io/github/actions/workflow/status/detro/bytes_parser/ci.yml?branch=main&label=CI%20%28main%29&logo=Github&style=flat-square)](https://github.com/detro/bytes_parser/actions/workflows/ci.yml)
[![Crates.io downloads](https://img.shields.io/crates/d/bytes_parser?logo=rust&style=flat-square)](https://crates.io/crates/bytes_parser)
[![](https://img.shields.io/crates/v/bytes_parser?label=latest&logo=rust&style=flat-square)](https://crates.io/crates/bytes_parser/versions)
[![Docs.rs](https://img.shields.io/docsrs/bytes_parser?logo=rust&style=flat-square)](https://docs.rs/bytes_parser/latest/bytes_parser/)
![Apache 2.0](https://img.shields.io/crates/l/bytes_parser?style=flat-square)

</div>

A simple wrapper to parse primitive Rust types from a slice of bytes `[u8]`.

This is the crate for you, if all you need is to parse a **bespoke binary protocol**.
You provide a reference slice of bytes, and assuming you know what those bytes represent,
read the original values out of it.

The core of this crate is `BytesParser`, built with few principles in mind:

* **simplicity:** just a _thin wrapper_ around an array of bytes,
  with a cursor to track progress.
* **zero-copy:** all you get back is either a bit-copied primitive,
  or a reference backed by the original bytes - never cloning.
* **only primitive types**: if you need to serialize/deserialize complex data structures,
  you probably want [serde](https://crates.io/crates/serde).

## Features

* Parse all primitive
  [scalar types](https://doc.rust-lang.org/book/ch03-02-data-types.html#scalar-types),
  signed and unsigned, as well as `&str` and sub-slice of `&[u8]`.
* Internal, auto-updating cursor, to implement a simple scanning logic.
* Options to move the cursor arbitrarily, but safely, along the input slice.
* Support for [Endianness](https://en.wikipedia.org/wiki/Endianness)
  selection (see `ParsingEndian`).
* Descriptive errors (see `BytesParserError`).
* Minimal dependencies.

## Examples

```rust
use bytes_parser::{BytesParser, ParsingEndian};

let input: &[u8] = /* a slice of bytes from somewhere */;

// Crate a parser from a given slice of bytes
let mut parser = BytesParser::from(input);

// Will use Big-Endian parsing
assert_eq!(parser.endian(), ParsingEndian::BE);
assert_eq!(ParsingEndian::BE, ParsingEndian::default());

// Parse a string length and the string itself
let str_len = parser.parse_usize()?;
let str = parser.parse_str_utf8(str_len)?;
assert_eq!(str, "Forza Napoli Sempre!");
```

## Alternatives

* [nom](https://crates.io/crates/nom): steeper learning curve, but more feature complete.
* [serde](https://crates.io/crates/serde): mostly focused on SERialization/DEserialization of well known formats,
  but it can be setup to handle a slice of bytes.

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
