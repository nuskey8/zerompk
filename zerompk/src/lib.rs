#![no_std]

mod consts;
mod error;
mod r#impl;
mod read;
mod write;

use alloc::vec::Vec;

pub use error::{Error, Result};
pub use read::{Read, Tag};
pub use write::Write;

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "derive")]
pub use zerompk_derive::{FromMessagePack, ToMessagePack};

/// A data structure that can be deserialized from MessagePack format.
pub trait FromMessagePack<'a>
where
    Self: Sized,
{
    /// Reads the MessagePack representation of this value from the provided reader.
    fn read<R: Read<'a>>(reader: &mut R) -> Result<Self>;
}

/// A trait for types that can be deserialized from MessagePack format without borrowing.
pub trait FromMessagePackOwned: for<'a> FromMessagePack<'a> {}

impl<T> FromMessagePackOwned for T where T: for<'a> FromMessagePack<'a> {}

/// A data structure that can be serialized into MessagePack format.
pub trait ToMessagePack {
    /// Writes the MessagePack representation of this value into the provided writer.
    fn write<W: Write>(&self, writer: &mut W) -> Result<()>;
}

/// Deserializes a value of type `T` from a MessagePack-encoded byte slice.
///
/// ## Errors
///
/// Deserialization can fail if `T`'s implementation of `FromMessagePack` returns an error.
///
/// ## Examples
///
/// ```rust
/// #[derive(zerompk::FromMessagePack)]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// fn main() {
///     let msgpack = vec![0x92, 0x01, 0x02];
///     let point: Point = zerompk::from_msgpack(&msgpack).unwrap();
///     assert_eq!(point.x, 1);
///     assert_eq!(point.y, 2);
/// }
/// ```
pub fn from_msgpack<'a, T: FromMessagePack<'a>>(data: &'a [u8]) -> Result<T> {
    let mut reader = read::SliceReader::new(data);
    let result = T::read(&mut reader);
    result
}

/// Serializes a value of type `T` into a `Vec<u8>` containing its MessagePack representation.
///
/// ## Errors
///
/// Serialization can fail if `T`'s implementation of `ToMessagePack` returns an error.
///
/// ## Examples
///
/// ```rust
/// #[derive(zerompk::ToMessagePack)]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// fn main() {
///     let point = Point { x: 1, y: 2 };
///     let msgpack: Vec<u8> = zerompk::to_msgpack_vec(&point).unwrap();
///     assert_eq!(msgpack, vec![0x92, 0x01, 0x02]);
/// }
/// ```
pub fn to_msgpack_vec<T: ToMessagePack>(value: &T) -> Result<Vec<u8>> {
    let mut writer = write::VecWriter::new();
    value.write(&mut writer)?;
    Ok(writer.into_vec())
}

/// Serializes a value of type `T` into the provided buffer, returning the number of bytes written.
///
/// ## Errors
///
/// Serialization can fail if `T`'s implementation of `ToMessagePack` returns an error,
/// or if the provided buffer is too small.
///
/// ## Examples
///
/// ```rust
/// #[derive(zerompk::ToMessagePack)]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// fn main() {
///     let point = Point { x: 1, y: 2 };
///     let mut buf = [0u8; 10];
///     let bytes_written = zerompk::to_msgpack(&point, &mut buf).unwrap();
///
///     assert_eq!(bytes_written, 3);
///     assert_eq!(&buf[..bytes_written], &[0x92, 0x01, 0x02]);
/// }
/// ```
pub fn to_msgpack<T: ToMessagePack>(value: &T, buf: &mut [u8]) -> Result<usize> {
    let mut writer = write::SliceWriter::new(buf);
    value.write(&mut writer)?;
    Ok(writer.position())
}

/// Serializes a value of type `T` into the I/O stream.
///
/// ## Errors
///
/// Serialization can fail if `T`'s implementation of `ToMessagePack` returns an error, or if the underlying I/O operation fails.
///
/// ## Examples
///
/// ```rust
/// #[derive(zerompk::ToMessagePack)]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// fn main() {
///     let point = Point { x: 1, y: 2 };
///     let mut buf = Vec::new();
///     zerompk::write_msgpack(&mut buf, &point).unwrap();
///     assert_eq!(buf, vec![0x92, 0x01, 0x02]);
/// }
/// ```
#[cfg(feature = "std")]
pub fn write_msgpack<T: ToMessagePack, W: std::io::Write>(writer: &mut W, value: &T) -> Result<()> {
    let mut io_writer = write::IOWriter::new(writer);
    value.write(&mut io_writer)
}

/// Deserializes a value of type `T` from the I/O stream.
///
/// ## Errors
///
/// Deserialization can fail if `T`'s implementation of `FromMessagePack` returns an error, or if the underlying I/O operation fails.
///
/// ## Examples
///
/// ```rust
/// #[derive(zerompk::FromMessagePack)]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// fn main() {
///     let data: &[u8] = &[0x92, 0x01, 0x02];
///     let point: Point = zerompk::read_msgpack(data).unwrap();
///     assert_eq!(point.x, 1);
///     assert_eq!(point.y, 2);
/// }
/// ```
#[cfg(feature = "std")]
pub fn read_msgpack<R: std::io::Read, T: FromMessagePackOwned>(reader: R) -> Result<T> {
    let mut io_reader = read::IOReader::new(reader);
    T::read(&mut io_reader)
}
