use core::hint::cold_path;

#[cfg(feature = "std")]
use alloc::vec;

use crate::Error;
use crate::FromMessagePack;
use crate::Result;
use crate::consts::*;

#[cold]
#[inline(never)]
fn buffer_too_small<T>() -> Result<T> {
    Err(Error::BufferTooSmall)
}

#[cold]
#[inline(never)]
fn invalid_marker<T>(marker: u8) -> Result<T> {
    Err(Error::InvalidMarker(marker))
}

/// The maximum allowed depth of nested structures during deserialization.
pub const MAX_DEPTH: usize = 500;

/// A tag read from a MessagePack stream, which can be either an integer or a string.
pub enum Tag<'de> {
    Int(u64),
    String(alloc::borrow::Cow<'de, str>),
}

/// A trait for reading values from a MessagePack-encoded input.
pub trait Read<'de> {
    /// Returns the next marker byte without consuming it.
    fn peek_marker(&mut self) -> Result<u8>;

    /// Increments the current depth of nested structures.
    ///
    /// ### Errors
    ///
    /// Returns an error if the maximum depth is exceeded.
    ///
    /// ### Examples
    ///
    /// ```rust
    /// use zerompk::{Read, FromMessagePack, Result};
    ///
    /// struct Outer {
    ///     inner: Inner,
    /// }
    ///
    /// struct Inner {
    ///     value: i32,
    /// }
    ///
    /// impl<'de> FromMessagePack<'de> for Outer {
    ///     fn read<R: Read<'de>>(reader: &mut R) -> Result<Self> {
    ///         reader.increment_depth()?;
    ///         let inner = Inner::read(reader)?;
    ///         reader.decrement_depth();
    ///         Ok(Self { inner })
    ///     }
    /// }
    ///
    /// impl<'de> FromMessagePack<'de> for Inner {
    ///     fn read<R: Read<'de>>(reader: &mut R) -> Result<Self> {
    ///         reader.increment_depth()?;
    ///         let value = reader.read_i32()?;
    ///         reader.decrement_depth();
    ///         Ok(Self { value })
    ///     }
    /// }
    /// ``
    ///
    fn increment_depth(&mut self) -> Result<()>;

    /// Decrements the current depth of nested structures.
    /// This should be called after finishing reading a nested structure.
    fn decrement_depth(&mut self);

    /// Reads a nil value from the input.
    fn read_nil(&mut self) -> Result<()>;

    /// Reads a boolean value from the input.
    fn read_boolean(&mut self) -> Result<bool>;

    /// Reads an unsigned 8-bit integer from the input.
    fn read_u8(&mut self) -> Result<u8>;

    /// Reads an unsigned 16-bit integer from the input.
    fn read_u16(&mut self) -> Result<u16>;

    /// Reads an unsigned 32-bit integer from the input.
    fn read_u32(&mut self) -> Result<u32>;

    /// Reads an unsigned 64-bit integer from the input.
    fn read_u64(&mut self) -> Result<u64>;

    /// Reads a signed 8-bit integer from the input.
    fn read_i8(&mut self) -> Result<i8>;

    /// Reads a signed 16-bit integer from the input.
    fn read_i16(&mut self) -> Result<i16>;

    /// Reads a signed 32-bit integer from the input.
    fn read_i32(&mut self) -> Result<i32>;

    /// Reads a signed 64-bit integer from the input.
    fn read_i64(&mut self) -> Result<i64>;

    /// Reads a 32-bit floating-point number from the input.
    fn read_f32(&mut self) -> Result<f32>;

    /// Reads a 64-bit floating-point number from the input.
    fn read_f64(&mut self) -> Result<f64>;

    /// Reads a timestamp from the input, returning the seconds and nanoseconds components.
    fn read_timestamp(&mut self) -> Result<(i64, u32)>;

    /// Reads the array header and returns the length of the array.
    fn read_array_len(&mut self) -> Result<usize>;

    /// Reads the map header and returns the number of key-value pairs in the map.
    fn read_map_len(&mut self) -> Result<usize>;

    /// Reads the extension header and returns the extension type and length of the data.
    fn read_ext_len(&mut self) -> Result<(i8, usize)>;

    /// Reads an extension value, including its type ID and payload.
    fn read_ext(&mut self) -> Result<(i8, alloc::borrow::Cow<'de, [u8]>)>;

    /// Reads a UTF-8 string from the input.
    /// Returns a `Cow<str>` which may borrow from the input data if possible.
    fn read_string(&mut self) -> Result<alloc::borrow::Cow<'de, str>>;

    /// Reads the raw bytes of a string from the input, without validating UTF-8.
    /// Returns a `Cow<[u8]>` which may borrow from the input data if possible.
    fn read_string_bytes(&mut self) -> Result<alloc::borrow::Cow<'de, [u8]>>;

    /// Reads the raw bytes of a binary blob from the input.
    /// Returns a `Cow<[u8]>` which may borrow from the input data if possible.
    fn read_binary(&mut self) -> Result<alloc::borrow::Cow<'de, [u8]>>;

    /// Reads an optional value from the input.
    /// Returns `None` if the next value is nil, or `Some(value)` if it is not.
    fn read_option<T: FromMessagePack<'de>>(&mut self) -> Result<Option<T>>;

    /// Reads an array into an existing `Vec`, reusing its allocation.
    ///
    /// On error, all partially decoded elements are dropped and `out` is
    /// left empty.
    #[inline(always)]
    fn read_array<T: FromMessagePack<'de>>(&mut self, out: &mut alloc::vec::Vec<T>) -> Result<()>
    where
        Self: Sized,
    {
        out.clear();
        let len = self.read_array_len()?;
        for _ in 0..len {
            match T::read(self) {
                Ok(value) => out.push(value),
                Err(error) => {
                    out.clear();
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    /// Reads a tag from the input, which can be either an integer or a string.
    fn read_tag(&mut self) -> Result<Tag<'de>>;

    /// Validates that the next value in the input is an array of the expected length, and consumes the array header.
    #[inline(always)]
    fn check_array_len(&mut self, expected: usize) -> Result<()> {
        let actual = self.read_array_len()?;
        if actual == expected {
            Ok(())
        } else {
            cold_path();
            Err(Error::ArrayLengthMismatch { expected, actual })
        }
    }

    /// Validates that the next value in the input is a map of the expected length, and consumes the map header.
    #[inline(always)]
    fn check_map_len(&mut self, expected: usize) -> Result<()> {
        let actual = self.read_map_len()?;
        if actual == expected {
            Ok(())
        } else {
            cold_path();
            Err(Error::MapLengthMismatch { expected, actual })
        }
    }

    /// Consumes exactly one MessagePack value from the input, regardless of
    /// its type. Used by `#[msgpack(allow_unknown)]` map-mode decoding to
    /// skip over unknown keys' values without needing to know their type.
    fn skip_value(&mut self) -> Result<()>;
}

pub struct SliceReader<'de> {
    data: &'de [u8],
    pos: usize,
    depth: usize,
}

impl<'de> SliceReader<'de> {
    pub fn new(data: &'de [u8]) -> Self {
        Self {
            data,
            pos: 0,
            depth: 0,
        }
    }

    #[inline(always)]
    fn peek_byte(&mut self) -> Result<u8> {
        if self.pos < self.data.len() {
            unsafe { Ok(*self.data.get_unchecked(self.pos)) }
        } else {
            cold_path();
            buffer_too_small()
        }
    }

    #[inline(always)]
    fn peek_slice(&mut self, len: usize) -> Result<&'de [u8]> {
        if len <= self.data.len() - self.pos {
            unsafe { Ok(self.data.get_unchecked(self.pos..(self.pos + len))) }
        } else {
            cold_path();
            buffer_too_small()
        }
    }

    #[inline(always)]
    fn take_byte(&mut self) -> Result<u8> {
        if self.pos < self.data.len() {
            let byte = unsafe { *self.data.get_unchecked(self.pos) };
            self.pos += 1;
            Ok(byte)
        } else {
            cold_path();
            buffer_too_small()
        }
    }

    #[inline(always)]
    fn take_slice(&mut self, len: usize) -> Result<&'de [u8]> {
        let slice = self.peek_slice(len)?;
        self.pos += len;
        Ok(slice)
    }

    #[inline(always)]
    fn take_array<const N: usize>(&mut self) -> Result<&'de [u8; N]> {
        if N <= self.data.len() - self.pos {
            let array = unsafe { &*(self.data.as_ptr().add(self.pos) as *const [u8; N]) };
            self.pos += N;
            Ok(array)
        } else {
            cold_path();
            buffer_too_small()
        }
    }

    #[inline(always)]
    fn skip_array_values(&mut self, len: usize) -> Result<()> {
        self.increment_depth()?;
        let result = (0..len).try_for_each(|_| self.skip_value());
        self.decrement_depth();
        result
    }

    #[inline(always)]
    fn skip_map_entries(&mut self, len: usize) -> Result<()> {
        self.increment_depth()?;
        let result = (0..len).try_for_each(|_| {
            self.skip_value()?;
            self.skip_value()
        });
        self.decrement_depth();
        result
    }
}

impl<'de> Read<'de> for SliceReader<'de> {
    #[inline(always)]
    fn peek_marker(&mut self) -> Result<u8> {
        self.peek_byte()
    }

    #[inline(always)]
    fn increment_depth(&mut self) -> Result<()> {
        if self.depth >= MAX_DEPTH {
            cold_path();
            Err(Error::DepthLimitExceeded { max: MAX_DEPTH })
        } else {
            self.depth += 1;
            Ok(())
        }
    }

    #[inline(always)]
    fn decrement_depth(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        } else {
            cold_path();
        }
    }

    impl_read_methods! {
        read_byte = |reader| reader.take_byte()?,
        read_2 = |reader| *reader.take_array::<2>()?,
        read_4 = |reader| *reader.take_array::<4>()?,
        read_5 = |reader| *reader.take_array::<5>()?,
        read_8 = |reader| *reader.take_array::<8>()?,
        read_9 = |reader| *reader.take_array::<9>()?,
        read_13 = |reader| *reader.take_array::<13>()?,
        read_bytes = |reader, len| alloc::borrow::Cow::Borrowed(reader.take_slice(len)?),
        invalid = |reader, marker| {
            cold_path();
            reader.pos -= 1;
            invalid_marker(marker)
        },
    }

    #[inline(always)]
    fn check_array_len(&mut self, expected: usize) -> Result<()> {
        if expected <= 15 && self.pos < self.data.len() {
            let marker = unsafe { *self.data.get_unchecked(self.pos) };
            if marker == FIXARRAY_START | expected as u8 {
                self.pos += 1;
                return Ok(());
            }
        }
        let actual = self.read_array_len()?;
        if actual == expected {
            Ok(())
        } else {
            cold_path();
            Err(Error::ArrayLengthMismatch { expected, actual })
        }
    }

    #[inline(always)]
    fn read_array<T: FromMessagePack<'de>>(&mut self, out: &mut alloc::vec::Vec<T>) -> Result<()> {
        out.clear();
        let len = self.read_array_len()?;

        // Every MessagePack value consumes at least one byte. Reject an
        // impossible length before using attacker-controlled data to grow
        // the output allocation.
        if self.data.len() - self.pos < len {
            cold_path();
            return Err(Error::BufferTooSmall);
        }

        if out.capacity() < len {
            out.reserve(len);
        }
        let ptr = out.as_mut_ptr();
        for initialized in 0..len {
            let value = match T::read(self) {
                Ok(value) => value,
                Err(error) => {
                    out.clear();
                    return Err(error);
                }
            };
            // SAFETY: capacity is at least `len`, and each slot is written
            // once. Updating length also makes unwinding drop every value
            // that has already been initialized.
            unsafe {
                ptr.add(initialized).write(value);
                out.set_len(initialized + 1);
            }
        }
        Ok(())
    }

    fn skip_value(&mut self) -> Result<()> {
        let byte = self.peek_byte()?;
        match byte {
            POS_FIXINT_START..=POS_FIXINT_END | NEG_FIXINT_START..=NEG_FIXINT_END => {
                self.pos += 1;
            }
            NIL_MARKER | TRUE_MARKER | FALSE_MARKER => {
                self.pos += 1;
            }
            UINT8_MARKER | INT8_MARKER => {
                self.take_slice(2)?;
            }
            UINT16_MARKER | INT16_MARKER => {
                self.take_slice(3)?;
            }
            UINT32_MARKER | INT32_MARKER | FLOAT32_MARKER => {
                self.take_slice(5)?;
            }
            UINT64_MARKER | INT64_MARKER | FLOAT64_MARKER => {
                self.take_slice(9)?;
            }
            FIXSTR_START..=FIXSTR_END => {
                let len = (byte - FIXSTR_START) as usize;
                self.take_slice(len + 1)?;
            }
            STR8_MARKER | BIN8_MARKER => {
                self.pos += 1;
                let len = self.take_byte()? as usize;
                self.take_slice(len)?;
            }
            STR16_MARKER | BIN16_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<2>()?;
                let len = u16::from_be_bytes(*bytes) as usize;
                self.take_slice(len)?;
            }
            STR32_MARKER | BIN32_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<4>()?;
                let len = u32::from_be_bytes(*bytes) as usize;
                self.take_slice(len)?;
            }
            FIXARRAY_START..=FIXARRAY_END => {
                let len = (byte - FIXARRAY_START) as usize;
                self.pos += 1;
                self.skip_array_values(len)?;
            }
            ARRAY16_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<2>()?;
                let len = u16::from_be_bytes(*bytes) as usize;
                self.skip_array_values(len)?;
            }
            ARRAY32_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<4>()?;
                let len = u32::from_be_bytes(*bytes) as usize;
                self.skip_array_values(len)?;
            }
            FIXMAP_START..=FIXMAP_END => {
                let len = (byte - FIXMAP_START) as usize;
                self.pos += 1;
                self.skip_map_entries(len)?;
            }
            MAP16_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<2>()?;
                let len = u16::from_be_bytes(*bytes) as usize;
                self.skip_map_entries(len)?;
            }
            MAP32_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<4>()?;
                let len = u32::from_be_bytes(*bytes) as usize;
                self.skip_map_entries(len)?;
            }
            FIXEXT1_MARKER => {
                self.take_slice(3)?;
            }
            FIXEXT2_MARKER => {
                self.take_slice(4)?;
            }
            FIXEXT4_MARKER => {
                self.take_slice(6)?;
            }
            FIXEXT8_MARKER => {
                self.take_slice(10)?;
            }
            FIXEXT16_MARKER => {
                self.take_slice(18)?;
            }
            EXT8_MARKER => {
                self.pos += 1;
                let len = self.take_byte()? as usize;
                self.take_slice(len.checked_add(1).ok_or(Error::BufferTooSmall)?)?;
            }
            EXT16_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<2>()?;
                let len = u16::from_be_bytes(*bytes) as usize;
                self.take_slice(len.checked_add(1).ok_or(Error::BufferTooSmall)?)?;
            }
            EXT32_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<4>()?;
                let len = u32::from_be_bytes(*bytes) as usize;
                self.take_slice(len.checked_add(1).ok_or(Error::BufferTooSmall)?)?;
            }
            _ => return Err(Error::InvalidMarker(byte)),
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
pub struct IOReader<R: std::io::Read> {
    reader: R,
    depth: usize,
    peeked: Option<u8>,
}

#[cfg(feature = "std")]
impl<R: std::io::Read> IOReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            depth: 0,
            peeked: None,
        }
    }

    #[inline(always)]
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        self.reader.read_exact(buf).map_err(Error::IoError)
    }

    #[inline(always)]
    fn read_byte(&mut self) -> Result<u8> {
        if let Some(byte) = self.peeked.take() {
            Ok(byte)
        } else {
            let mut buf = [0u8; 1];
            self.read_exact(&mut buf)?;
            Ok(buf[0])
        }
    }

    #[inline(always)]
    fn unread_byte(&mut self, byte: u8) {
        debug_assert!(self.peeked.is_none());
        self.peeked = Some(byte);
    }

    #[inline(always)]
    fn read_exact_vec(&mut self, len: usize) -> Result<alloc::vec::Vec<u8>> {
        const CHUNK_SIZE: usize = 8192;

        if len == 0 {
            return Ok(alloc::vec::Vec::new());
        } else if len < CHUNK_SIZE {
            let mut buf = vec![0u8; len];
            self.reader.read_exact(&mut buf).map_err(Error::IoError)?;
            return Ok(buf);
        }

        let mut out = alloc::vec::Vec::new();
        let mut remaining = len;
        let mut chunk = [0u8; CHUNK_SIZE];

        while remaining > 0 {
            let to_read = core::cmp::min(remaining, chunk.len());
            let n = self
                .reader
                .read(&mut chunk[..to_read])
                .map_err(Error::IoError)?;
            if n == 0 {
                return Err(Error::BufferTooSmall);
            }
            out.extend_from_slice(&chunk[..n]);
            remaining -= n;
        }

        Ok(out)
    }
}

#[cfg(feature = "std")]
impl<'de, R: std::io::Read> Read<'de> for IOReader<R> {
    #[inline(always)]
    fn peek_marker(&mut self) -> Result<u8> {
        let byte = self.read_byte()?;
        self.unread_byte(byte);
        Ok(byte)
    }

    #[inline(always)]
    fn increment_depth(&mut self) -> Result<()> {
        if self.depth >= MAX_DEPTH {
            cold_path();
            Err(Error::DepthLimitExceeded { max: MAX_DEPTH })
        } else {
            self.depth += 1;
            Ok(())
        }
    }

    #[inline(always)]
    fn decrement_depth(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        } else {
            cold_path();
        }
    }

    impl_read_methods! {
        read_byte = |reader| reader.read_byte()?,
        read_2 = |reader| {
            let mut bytes = [0; 2];
            reader.read_exact(&mut bytes)?;
            bytes
        },
        read_4 = |reader| {
            let mut bytes = [0; 4];
            reader.read_exact(&mut bytes)?;
            bytes
        },
        read_5 = |reader| {
            let mut bytes = [0; 5];
            reader.read_exact(&mut bytes)?;
            bytes
        },
        read_8 = |reader| {
            let mut bytes = [0; 8];
            reader.read_exact(&mut bytes)?;
            bytes
        },
        read_9 = |reader| {
            let mut bytes = [0; 9];
            reader.read_exact(&mut bytes)?;
            bytes
        },
        read_13 = |reader| {
            let mut bytes = [0; 13];
            reader.read_exact(&mut bytes)?;
            bytes
        },
        read_bytes = |reader, len| {
            alloc::borrow::Cow::Owned(reader.read_exact_vec(len)?)
        },
        invalid = |_reader, marker| Err(Error::InvalidMarker(marker)),
    }

    fn skip_value(&mut self) -> Result<()> {
        self.increment_depth()?;
        let byte = self.read_byte()?;
        match byte {
            POS_FIXINT_START..=POS_FIXINT_END | NEG_FIXINT_START..=NEG_FIXINT_END => {}
            NIL_MARKER | TRUE_MARKER | FALSE_MARKER => {}
            UINT8_MARKER | INT8_MARKER => {
                let mut buf = [0u8; 1];
                self.read_exact(&mut buf)?;
            }
            UINT16_MARKER | INT16_MARKER => {
                let mut buf = [0u8; 2];
                self.read_exact(&mut buf)?;
            }
            UINT32_MARKER | INT32_MARKER | FLOAT32_MARKER => {
                let mut buf = [0u8; 4];
                self.read_exact(&mut buf)?;
            }
            UINT64_MARKER | INT64_MARKER | FLOAT64_MARKER => {
                let mut buf = [0u8; 8];
                self.read_exact(&mut buf)?;
            }
            FIXSTR_START..=FIXSTR_END => {
                let len = (byte - FIXSTR_START) as usize;
                let _ = self.read_exact_vec(len)?;
            }
            STR8_MARKER | BIN8_MARKER => {
                let mut buf = [0u8; 1];
                self.read_exact(&mut buf)?;
                let len = buf[0] as usize;
                let _ = self.read_exact_vec(len)?;
            }
            STR16_MARKER | BIN16_MARKER => {
                let mut buf = [0u8; 2];
                self.read_exact(&mut buf)?;
                let len = u16::from_be_bytes(buf) as usize;
                let _ = self.read_exact_vec(len)?;
            }
            STR32_MARKER | BIN32_MARKER => {
                let mut buf = [0u8; 4];
                self.read_exact(&mut buf)?;
                let len = u32::from_be_bytes(buf) as usize;
                let _ = self.read_exact_vec(len)?;
            }
            FIXARRAY_START..=FIXARRAY_END => {
                let len = (byte - FIXARRAY_START) as usize;
                for _ in 0..len {
                    self.skip_value()?;
                }
            }
            ARRAY16_MARKER => {
                let mut buf = [0u8; 2];
                self.read_exact(&mut buf)?;
                let len = u16::from_be_bytes(buf) as usize;
                for _ in 0..len {
                    self.skip_value()?;
                }
            }
            ARRAY32_MARKER => {
                let mut buf = [0u8; 4];
                self.read_exact(&mut buf)?;
                let len = u32::from_be_bytes(buf) as usize;
                for _ in 0..len {
                    self.skip_value()?;
                }
            }
            FIXMAP_START..=FIXMAP_END => {
                let len = (byte - FIXMAP_START) as usize;
                for _ in 0..len.checked_mul(2).ok_or(Error::BufferTooSmall)? {
                    self.skip_value()?;
                }
            }
            MAP16_MARKER => {
                let mut buf = [0u8; 2];
                self.read_exact(&mut buf)?;
                let len = u16::from_be_bytes(buf) as usize;
                for _ in 0..len.checked_mul(2).ok_or(Error::BufferTooSmall)? {
                    self.skip_value()?;
                }
            }
            MAP32_MARKER => {
                let mut buf = [0u8; 4];
                self.read_exact(&mut buf)?;
                let len = u32::from_be_bytes(buf) as usize;
                for _ in 0..len.checked_mul(2).ok_or(Error::BufferTooSmall)? {
                    self.skip_value()?;
                }
            }
            FIXEXT1_MARKER => {
                let _ = self.read_exact_vec(2)?;
            }
            FIXEXT2_MARKER => {
                let _ = self.read_exact_vec(3)?;
            }
            FIXEXT4_MARKER => {
                let _ = self.read_exact_vec(5)?;
            }
            FIXEXT8_MARKER => {
                let _ = self.read_exact_vec(9)?;
            }
            FIXEXT16_MARKER => {
                let _ = self.read_exact_vec(17)?;
            }
            EXT8_MARKER => {
                let mut buf = [0u8; 1];
                self.read_exact(&mut buf)?;
                let len = buf[0] as usize;
                let _ = self.read_exact_vec(len.checked_add(1).ok_or(Error::BufferTooSmall)?)?;
            }
            EXT16_MARKER => {
                let mut buf = [0u8; 2];
                self.read_exact(&mut buf)?;
                let len = u16::from_be_bytes(buf) as usize;
                let _ = self.read_exact_vec(len.checked_add(1).ok_or(Error::BufferTooSmall)?)?;
            }
            EXT32_MARKER => {
                let mut buf = [0u8; 4];
                self.read_exact(&mut buf)?;
                let len = u32::from_be_bytes(buf) as usize;
                let _ = self.read_exact_vec(len.checked_add(1).ok_or(Error::BufferTooSmall)?)?;
            }
            _ => return Err(Error::InvalidMarker(byte)),
        }
        self.decrement_depth();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Error, FromMessagePack, Read, read::IOReader, read::SliceReader};

    #[allow(dead_code)]
    #[derive(Debug, zerompk_derive::FromMessagePack)]
    #[msgpack(map)]
    struct DuplicateKeyMap {
        x: u8,
        y: u8,
    }

    #[test]
    fn derived_map_struct_duplicate_key_restores_reader_depth() {
        let data = [
            0x82, // fixmap with 2 entries
            0xa1, b'x', // fixstr of length 1: "x"
            0x01, // positive fixint: 1
            0xa1, b'x', // fixstr of length 1: "x" (duplicate key)
            0x02, // positive fixint: 2
        ];
        let mut reader = SliceReader::new(&data);

        let err = DuplicateKeyMap::read(&mut reader).unwrap_err();

        assert!(matches!(err, Error::KeyDuplicated(ref key) if key == "x"));
    }

    #[test]
    fn slice_reader_decrement_depth_at_zero_does_not_underflow() {
        let data = [0xc0];
        let mut reader = SliceReader::new(&data);
        assert_eq!(reader.depth, 0);
        reader.decrement_depth();
        assert_eq!(reader.depth, 0);
    }

    #[test]
    fn slice_reader_decrement_depth_decrements() {
        let data = [0xc0];
        let mut reader = SliceReader::new(&data);
        reader.increment_depth().unwrap();
        assert_eq!(reader.depth, 1);
        reader.decrement_depth();
        assert_eq!(reader.depth, 0);
    }

    #[test]
    fn slice_reader_skip_value_restores_depth_after_nested_error() {
        let data = [
            0x91, // fixarray with one value
            0x91, // nested fixarray with one value
            0xc1, // reserved marker
        ];
        let mut reader = SliceReader::new(&data);

        assert!(matches!(
            reader.skip_value(),
            Err(Error::InvalidMarker(0xc1))
        ));
        assert_eq!(reader.depth, 0);
    }

    #[cfg(feature = "std")]
    #[test]
    fn io_reader_decrement_depth_at_zero_does_not_underflow() {
        let data: &[u8] = &[0xc0];
        let mut reader = IOReader::new(data);
        assert_eq!(reader.depth, 0);
        reader.decrement_depth();
        assert_eq!(reader.depth, 0);
    }

    #[cfg(feature = "std")]
    #[test]
    fn io_reader_decrement_depth_decrements() {
        let data: &[u8] = &[0xc0];
        let mut reader = IOReader::new(data);
        reader.increment_depth().unwrap();
        assert_eq!(reader.depth, 1);
        reader.decrement_depth();
        assert_eq!(reader.depth, 0);
    }
}
