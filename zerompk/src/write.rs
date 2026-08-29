use core::hint::cold_path;

use alloc::vec::Vec;

use crate::{Error, Result, consts::*};

const MAX_CONTAINER_PREALLOC: usize = 4 * 1024;

/// A trait for writing MessagePack-encoded data.
///
/// ## Examples
///
/// ```rust
/// use zerompk::{ToMessagePack, Write, Result};
///
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// impl ToMessagePack for Point {
///     fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
///         writer.write_array_len(2)?;
///         writer.write_i32(self.x)?;
///         writer.write_i32(self.y)?;
///         Ok(())
///     }
/// }
/// ```
pub trait Write {
    /// Writes a nil value.
    fn write_nil(&mut self) -> Result<()>;

    /// Writes a boolean value.
    fn write_boolean(&mut self, b: bool) -> Result<()>;

    /// Writes an unsigned 8-bit integer.
    fn write_u8(&mut self, u: u8) -> Result<()>;

    /// Writes an unsigned 16-bit integer.
    fn write_u16(&mut self, u: u16) -> Result<()>;

    /// Writes an unsigned 32-bit integer.
    fn write_u32(&mut self, u: u32) -> Result<()>;

    /// Writes an unsigned 64-bit integer.
    fn write_u64(&mut self, u: u64) -> Result<()>;

    /// Writes a signed 8-bit integer.
    fn write_i8(&mut self, i: i8) -> Result<()>;

    /// Writes a signed 16-bit integer.
    fn write_i16(&mut self, i: i16) -> Result<()>;

    /// Writes a signed 32-bit integer.
    fn write_i32(&mut self, i: i32) -> Result<()>;

    /// Writes a signed 64-bit integer.
    fn write_i64(&mut self, i: i64) -> Result<()>;

    /// Writes a 32-bit floating-point number.
    fn write_f32(&mut self, f: f32) -> Result<()>;

    /// Writes a 64-bit floating-point number.
    fn write_f64(&mut self, f: f64) -> Result<()>;

    /// Writes a UTF-8 string.
    fn write_string(&mut self, s: &str) -> Result<()>;

    /// Writes a binary blob.
    fn write_binary(&mut self, data: &[u8]) -> Result<()>;

    /// Writes a timestamp.
    fn write_timestamp(&mut self, seconds: i64, nanoseconds: u32) -> Result<()>;

    /// Writes the array header with the length.
    fn write_array_len(&mut self, len: usize) -> Result<()>;

    /// Writes the map header with the length.
    fn write_map_len(&mut self, len: usize) -> Result<()>;

    /// Writes an extension type with the given type ID and data.
    fn write_ext(&mut self, type_id: i8, data: &[u8]) -> Result<()>;
}

pub struct SliceWriter<'a> {
    buffer: &'a mut [u8],
    pos: usize,
}

impl<'a> SliceWriter<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        SliceWriter { buffer, pos: 0 }
    }

    #[inline(always)]
    fn take_array<const N: usize>(&mut self) -> Result<&mut [u8; N]> {
        if N > self.buffer.len() - self.pos {
            cold_path();
            return Err(Error::BufferTooSmall);
        }
        let array = unsafe { &mut *(self.buffer.as_mut_ptr().add(self.pos) as *mut [u8; N]) };
        self.pos += N;
        Ok(array)
    }

    #[inline(always)]
    fn take_slice(&mut self, len: usize) -> Result<&mut [u8]> {
        if len > self.buffer.len() - self.pos {
            cold_path();
            return Err(Error::BufferTooSmall);
        }

        let slice = unsafe { self.buffer.get_unchecked_mut(self.pos..self.pos + len) };
        self.pos += len;
        Ok(slice)
    }

    #[inline(always)]
    pub fn position(&self) -> usize {
        self.pos
    }
}

impl<'a> Write for SliceWriter<'a> {
    #[inline(always)]
    fn write_u8(&mut self, value: u8) -> Result<()> {
        if value <= POS_FIXINT_END {
            *self.take_array::<1>()? = [value];
        } else {
            *self.take_array::<2>()? = [UINT8_MARKER, value];
        }
        Ok(())
    }

    impl_write_methods! {
        write = |writer, data| {
            match data {
                [a] => *writer.take_array::<1>()? = [*a],
                [a, b] => *writer.take_array::<2>()? = [*a, *b],
                [a, b, c] => *writer.take_array::<3>()? = [*a, *b, *c],
                [a, b, c, d, e] => *writer.take_array::<5>()? = [*a, *b, *c, *d, *e],
                [a, b, c, d, e, f] => {
                    *writer.take_array::<6>()? = [*a, *b, *c, *d, *e, *f]
                }
                [a, b, c, d, e, f, g, h, i] => {
                    *writer.take_array::<9>()? = [*a, *b, *c, *d, *e, *f, *g, *h, *i]
                }
                [a, b, c, d, e, f, g, h, i, j] => {
                    *writer.take_array::<10>()? = [*a, *b, *c, *d, *e, *f, *g, *h, *i, *j]
                }
                _ => writer.take_slice(data.len())?.copy_from_slice(data),
            }
            Ok(())
        },
        write_parts = |writer, header, payload| {
            let output = writer.take_slice(header.len() + payload.len())?;
            unsafe {
                let ptr = output.as_mut_ptr();
                match header {
                    [a] => *ptr = *a,
                    [a, b] => {
                        *ptr = *a;
                        *ptr.add(1) = *b;
                    }
                    [a, b, c] => {
                        *ptr = *a;
                        *ptr.add(1) = *b;
                        *ptr.add(2) = *c;
                    }
                    [a, b, c, d] => {
                        *ptr = *a;
                        *ptr.add(1) = *b;
                        *ptr.add(2) = *c;
                        *ptr.add(3) = *d;
                    }
                    [a, b, c, d, e] => {
                        *ptr = *a;
                        *ptr.add(1) = *b;
                        *ptr.add(2) = *c;
                        *ptr.add(3) = *d;
                        *ptr.add(4) = *e;
                    }
                    [a, b, c, d, e, f] => {
                        *ptr = *a;
                        *ptr.add(1) = *b;
                        *ptr.add(2) = *c;
                        *ptr.add(3) = *d;
                        *ptr.add(4) = *e;
                        *ptr.add(5) = *f;
                    }
                    _ => unreachable!(),
                }
                ptr.add(header.len())
                    .copy_from_nonoverlapping(payload.as_ptr(), payload.len());
            }
            Ok(())
        },
        write_container = |writer, header, _reserve| {
            match header {
                [a] => *writer.take_array::<1>()? = [*a],
                [a, b, c] => *writer.take_array::<3>()? = [*a, *b, *c],
                [a, b, c, d, e] => *writer.take_array::<5>()? = [*a, *b, *c, *d, *e],
                _ => unreachable!(),
            }
            Ok(())
        },
    }
}

pub struct VecWriter {
    buffer: Vec<u8>,
}

impl VecWriter {
    pub fn new() -> Self {
        VecWriter { buffer: Vec::new() }
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.buffer
    }
}

impl Write for VecWriter {
    #[inline(always)]
    fn write_u8(&mut self, value: u8) -> Result<()> {
        if value <= POS_FIXINT_END {
            self.buffer.push(value);
        } else {
            self.buffer.extend_from_slice(&[UINT8_MARKER, value]);
        }
        Ok(())
    }

    impl_write_methods! {
        write = |writer, data| {
            if let [byte] = data {
                writer.buffer.push(*byte);
            } else {
                writer.buffer.reserve(data.len());
                unsafe {
                    let len = writer.buffer.len();
                    let output = writer.buffer.as_mut_ptr().add(len);
                    output.copy_from_nonoverlapping(data.as_ptr(), data.len());
                    writer.buffer.set_len(len + data.len());
                }
            }
            Ok(())
        },
        write_parts = |writer, header, payload| {
            let additional = header.len() + payload.len();
            writer.buffer.reserve(additional);
            unsafe {
                let len = writer.buffer.len();
                let output = writer.buffer.as_mut_ptr().add(len);
                output.copy_from_nonoverlapping(header.as_ptr(), header.len());
                output
                    .add(header.len())
                    .copy_from_nonoverlapping(payload.as_ptr(), payload.len());
                writer.buffer.set_len(len + additional);
            }
            Ok(())
        },
        write_container = |writer, header, reserve| {
            writer.buffer.reserve(header.len() + reserve);
            unsafe {
                let len = writer.buffer.len();
                let output = writer.buffer.as_mut_ptr().add(len);
                output.copy_from_nonoverlapping(header.as_ptr(), header.len());
                writer.buffer.set_len(len + header.len());
            }
            Ok(())
        },
    }
}

#[cfg(feature = "std")]
pub struct IOWriter<W: std::io::Write> {
    writer: W,
}

#[cfg(feature = "std")]
impl<W: std::io::Write> IOWriter<W> {
    pub fn new(writer: W) -> Self {
        IOWriter { writer }
    }

    #[inline(always)]
    fn write_all(&mut self, data: &[u8]) -> Result<()> {
        self.writer.write_all(data).map_err(Error::IoError)
    }
}

#[cfg(feature = "std")]
impl<W: std::io::Write> Write for IOWriter<W> {
    #[inline(always)]
    fn write_u8(&mut self, value: u8) -> Result<()> {
        if value <= POS_FIXINT_END {
            self.write_all(&[value])
        } else {
            self.write_all(&[UINT8_MARKER, value])
        }
    }

    impl_write_methods! {
        write = |writer, data| writer.write_all(data),
        write_parts = |writer, header, payload| {
            writer.write_all(header)?;
            writer.write_all(payload)
        },
        write_container = |writer, header, _reserve| writer.write_all(header),
    }
}
