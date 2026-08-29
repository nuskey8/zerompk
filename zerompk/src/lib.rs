#![no_std]

#[cfg(test)]
extern crate self as zerompk;

#[macro_use]
mod read_macro;
#[macro_use]
mod write_macro;

#[cfg(feature = "std")]
mod bufread;
mod consts;
mod error;
mod r#impl;
mod read;
mod value;
mod write;

use alloc::vec::Vec;

pub use error::{Error, Result};
pub use read::{Read, SliceReader, Tag};
pub use value::Value;
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

/// A trusted upper bound for the next serialization of a value.
pub struct TrustedSizeHint(usize);

impl TrustedSizeHint {
    /// Creates a trusted encoded-size upper bound.
    ///
    /// # Safety
    /// The next serialization of the value this hint describes must write at most
    /// `upper_bound` bytes. Serialization-relevant state must not change in between.
    #[doc(hidden)]
    #[inline(always)]
    pub const unsafe fn new_unchecked(upper_bound: usize) -> Self {
        Self(upper_bound)
    }

    #[inline(always)]
    pub const fn upper_bound(&self) -> usize {
        self.0
    }
}

/// A data structure that can be serialized into MessagePack format.
pub trait ToMessagePack {
    /// Writes the MessagePack representation of this value into the provided writer.
    fn write<W: Write>(&self, writer: &mut W) -> Result<()>;

    /// Writes the MessagePack representation of a slice of values into the provided writer.
    #[inline(always)]
    fn write_slice<W: Write>(values: &[Self], writer: &mut W) -> Result<()>
    where
        Self: Sized,
    {
        for value in values {
            value.write(writer)?;
        }
        Ok(())
    }

    /// Returns a trusted upper bound for the number of bytes written by the next
    /// call to [`Self::write`], or `None` when it cannot be determined cheaply.
    ///
    /// Implementations must run in O(1) with respect to the serialized value's
    /// runtime-sized contents. Return `None` if determining the size requires
    /// traversing a slice, collection, string contents, or another variable-size value.
    ///
    /// ## Safety
    ///
    /// The returned upper bound must never be smaller than the serialized representation.
    #[inline]
    fn size_hint(&self) -> Option<TrustedSizeHint> {
        None
    }

    /// Returns a trusted upper bound valid for every value of this type.
    ///
    /// This is used to compute O(1) hints for runtime-sized homogeneous containers.
    /// The returned upper bound must never be smaller than any serialized value
    /// of this type.
    ///
    /// ## Safety
    ///
    /// The returned upper bound must never be smaller than any serialized value of this type.
    #[inline]
    fn max_size() -> Option<TrustedSizeHint>
    where
        Self: Sized,
    {
        None
    }
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
    T::read(&mut reader)
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
    let mut writer = match value.size_hint() {
        Some(hint) => write::VecWriter::with_capacity(hint.upper_bound()),
        None => write::VecWriter::new(),
    };
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
    if let Some(hint) = value.size_hint()
        && hint.upper_bound() <= buf.len()
    {
        // SAFETY: the trusted hint guarantees that the write fits in `buf`.
        let mut writer = unsafe { write::SliceWriter::new_unchecked(buf) };
        value.write(&mut writer)?;
        debug_assert!(writer.position() <= hint.upper_bound());
        return Ok(writer.position());
    }
    to_msgpack_checked(value, buf)
}

#[cold]
#[inline(never)]
fn to_msgpack_checked<T: ToMessagePack>(value: &T, buf: &mut [u8]) -> Result<usize> {
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
///    x: i32,
///    y: i32,
/// }
///
/// fn main() {
///     let point = Point { x: 1, y: 2 };
///
///     let mut buf = Vec::new();
///     let mut cursor = std::io::Cursor::new(&mut buf);
///
///     zerompk::write_msgpack(&mut cursor, &point).unwrap();
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
///     let vec = vec![0x92, 0x01, 0x02];
///     let mut cursor = std::io::Cursor::new(vec);
///
///     let point: Point = zerompk::read_msgpack(&mut cursor).unwrap();
///     assert_eq!(point.x, 1);
///     assert_eq!(point.y, 2);
/// }
/// ```
#[cfg(feature = "std")]
pub fn read_msgpack<'a, R: std::io::Read, T: FromMessagePack<'a>>(reader: R) -> Result<T> {
    let mut io_reader = read::IOReader::new(reader);
    T::read(&mut io_reader)
}

/// Deserializes a value of type `T` from a buffered I/O stream.
///
/// Unlike [`read_msgpack`], this function reads directly from the slice exposed
/// by [`std::io::BufRead`]. Any bytes read ahead by the buffered reader remain
/// available through that same reader, so consecutive MessagePack values can
/// be decoded without losing data.
///
/// ## Errors
///
/// Deserialization can fail if `T`'s implementation of [`FromMessagePack`]
/// returns an error, or if the underlying I/O operation fails.
///
/// ## Examples
///
/// ```
/// let data = [0x01, 0x02];
/// let mut reader = std::io::BufReader::new(data.as_slice());
///
/// let first: u8 = zerompk::read_msgpack_bufread(&mut reader).unwrap();
/// let second: u8 = zerompk::read_msgpack_bufread(&mut reader).unwrap();
///
/// assert_eq!((first, second), (1, 2));
/// ```
#[cfg(feature = "std")]
pub fn read_msgpack_bufread<'a, R: std::io::BufRead, T: FromMessagePack<'a>>(
    reader: &mut R,
) -> Result<T> {
    let mut bufread_reader = bufread::BufReadReader::new(reader);
    T::read(&mut bufread_reader)
}
