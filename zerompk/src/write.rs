use core::hint::cold_path;

use alloc::vec::Vec;

use crate::{Error, Result, consts::*};

const MAX_CONTAINER_PREALLOC: usize = 4 * 1024;

#[inline(always)]
unsafe fn encode_unsigned_at(output: *mut u8, value: u64) -> usize {
    unsafe {
        if value <= POS_FIXINT_END as u64 {
            output.write(value as u8);
            1
        } else if value <= u8::MAX as u64 {
            output.write(UINT8_MARKER);
            output.add(1).write(value as u8);
            2
        } else if value <= u16::MAX as u64 {
            output.write(UINT16_MARKER);
            output
                .add(1)
                .cast::<u16>()
                .write_unaligned((value as u16).to_be());
            3
        } else if value <= u32::MAX as u64 {
            output.write(UINT32_MARKER);
            output
                .add(1)
                .cast::<u32>()
                .write_unaligned((value as u32).to_be());
            5
        } else {
            output.write(UINT64_MARKER);
            output.add(1).cast::<u64>().write_unaligned(value.to_be());
            9
        }
    }
}

#[inline(always)]
unsafe fn encode_signed_at(output: *mut u8, value: i64, width: usize) -> usize {
    unsafe {
        if (0..=127).contains(&value) || (-32..=-1).contains(&value) {
            output.write(value as u8);
            1
        } else if width == 1 || (-128..=127).contains(&value) {
            output.write(INT8_MARKER);
            output.add(1).write(value as u8);
            2
        } else if width == 2 || (-32768..=32767).contains(&value) {
            output.write(INT16_MARKER);
            output
                .add(1)
                .cast::<i16>()
                .write_unaligned((value as i16).to_be());
            3
        } else if width == 4 || (-2147483648..=2147483647).contains(&value) {
            output.write(INT32_MARKER);
            output
                .add(1)
                .cast::<i32>()
                .write_unaligned((value as i32).to_be());
            5
        } else {
            output.write(INT64_MARKER);
            output.add(1).cast::<i64>().write_unaligned(value.to_be());
            9
        }
    }
}

macro_rules! impl_unsigned_slice {
    ($method:ident, $scalar:ident, $ty:ty, $max_len:expr) => {
        #[inline(always)]
        fn $method(&mut self, values: &[$ty]) -> Result<()> {
            let Some(max_len) = values.len().checked_mul($max_len) else {
                return Err(Error::BufferTooSmall);
            };
            if max_len > self.buffer.len() - self.pos {
                for &value in values {
                    self.$scalar(value)?;
                }
                return Ok(());
            }

            let output = unsafe { self.buffer.as_mut_ptr().add(self.pos) };
            let mut written = 0;
            for &value in values {
                // SAFETY: the fast path reserved the maximum encoded length
                // for every remaining value.
                written += unsafe { encode_unsigned_at(output.add(written), value as u64) };
            }
            self.pos += written;
            Ok(())
        }
    };
}

macro_rules! impl_signed_slice {
    ($method:ident, $scalar:ident, $ty:ty, $max_len:expr, $width:expr) => {
        #[inline(always)]
        fn $method(&mut self, values: &[$ty]) -> Result<()> {
            let Some(max_len) = values.len().checked_mul($max_len) else {
                return Err(Error::BufferTooSmall);
            };
            if max_len > self.buffer.len() - self.pos {
                for &value in values {
                    self.$scalar(value)?;
                }
                return Ok(());
            }

            let output = unsafe { self.buffer.as_mut_ptr().add(self.pos) };
            let mut written = 0;
            for &value in values {
                // SAFETY: the fast path reserved the maximum encoded length
                // for every remaining value.
                written += unsafe { encode_signed_at(output.add(written), value as i64, $width) };
            }
            self.pos += written;
            Ok(())
        }
    };
}

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

    /// Writes a slice of boolean values without an array header.
    #[inline(always)]
    fn write_boolean_slice(&mut self, values: &[bool]) -> Result<()>
    where
        Self: Sized,
    {
        for &value in values {
            self.write_boolean(value)?;
        }
        Ok(())
    }

    /// Writes an unsigned 8-bit integer.
    fn write_u8(&mut self, u: u8) -> Result<()>;

    /// Writes a slice of unsigned 8-bit integers without an array header.
    #[inline(always)]
    fn write_u8_slice(&mut self, values: &[u8]) -> Result<()>
    where
        Self: Sized,
    {
        for &value in values {
            self.write_u8(value)?;
        }
        Ok(())
    }

    /// Writes an unsigned 16-bit integer.
    fn write_u16(&mut self, u: u16) -> Result<()>;

    /// Writes a slice of unsigned 16-bit integers without an array header.
    #[inline(always)]
    fn write_u16_slice(&mut self, values: &[u16]) -> Result<()>
    where
        Self: Sized,
    {
        for &value in values {
            self.write_u16(value)?;
        }
        Ok(())
    }

    /// Writes an unsigned 32-bit integer.
    fn write_u32(&mut self, u: u32) -> Result<()>;

    /// Writes a slice of unsigned 32-bit integers without an array header.
    #[inline(always)]
    fn write_u32_slice(&mut self, values: &[u32]) -> Result<()>
    where
        Self: Sized,
    {
        for &value in values {
            self.write_u32(value)?;
        }
        Ok(())
    }

    /// Writes an unsigned 64-bit integer.
    fn write_u64(&mut self, u: u64) -> Result<()>;

    /// Writes a slice of unsigned 64-bit integers without an array header.
    #[inline(always)]
    fn write_u64_slice(&mut self, values: &[u64]) -> Result<()>
    where
        Self: Sized,
    {
        for &value in values {
            self.write_u64(value)?;
        }
        Ok(())
    }

    /// Writes a signed 8-bit integer.
    fn write_i8(&mut self, i: i8) -> Result<()>;

    /// Writes a slice of signed 8-bit integers without an array header.
    #[inline(always)]
    fn write_i8_slice(&mut self, values: &[i8]) -> Result<()>
    where
        Self: Sized,
    {
        for &value in values {
            self.write_i8(value)?;
        }
        Ok(())
    }

    /// Writes a signed 16-bit integer.
    fn write_i16(&mut self, i: i16) -> Result<()>;

    /// Writes a slice of signed 16-bit integers without an array header.
    #[inline(always)]
    fn write_i16_slice(&mut self, values: &[i16]) -> Result<()>
    where
        Self: Sized,
    {
        for &value in values {
            self.write_i16(value)?;
        }
        Ok(())
    }

    /// Writes a signed 32-bit integer.
    fn write_i32(&mut self, i: i32) -> Result<()>;

    /// Writes a slice of signed 32-bit integers without an array header.
    #[inline(always)]
    fn write_i32_slice(&mut self, values: &[i32]) -> Result<()>
    where
        Self: Sized,
    {
        for &value in values {
            self.write_i32(value)?;
        }
        Ok(())
    }

    /// Writes a signed 64-bit integer.
    fn write_i64(&mut self, i: i64) -> Result<()>;

    /// Writes a slice of signed 64-bit integers without an array header.
    #[inline(always)]
    fn write_i64_slice(&mut self, values: &[i64]) -> Result<()>
    where
        Self: Sized,
    {
        for &value in values {
            self.write_i64(value)?;
        }
        Ok(())
    }

    /// Writes a 32-bit floating-point number.
    fn write_f32(&mut self, f: f32) -> Result<()>;

    /// Writes fixed-size 32-bit floating-point values without an array header.
    #[inline(always)]
    fn write_f32_slice(&mut self, values: &[f32]) -> Result<()>
    where
        Self: Sized,
    {
        for &value in values {
            self.write_f32(value)?;
        }
        Ok(())
    }

    /// Writes a 64-bit floating-point number.
    fn write_f64(&mut self, f: f64) -> Result<()>;

    /// Writes a slice of 64-bit floating-point values without an array header.
    #[inline(always)]
    fn write_f64_slice(&mut self, values: &[f64]) -> Result<()>
    where
        Self: Sized,
    {
        for &value in values {
            self.write_f64(value)?;
        }
        Ok(())
    }

    /// Writes a UTF-8 string.
    fn write_string(&mut self, s: &str) -> Result<()>;

    /// Writes a static UTF-8 string literal.
    #[inline(always)]
    fn write_static_string(&mut self, value: &'static str, _encoded: &'static [u8]) -> Result<()> {
        self.write_string(value)
    }

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
    fn write_static_string(&mut self, _value: &'static str, encoded: &'static [u8]) -> Result<()> {
        self.take_slice(encoded.len())?.copy_from_slice(encoded);
        Ok(())
    }

    #[inline(always)]
    fn write_boolean_slice(&mut self, values: &[bool]) -> Result<()> {
        let output = self.take_slice(values.len())?.as_mut_ptr();
        for (index, &value) in values.iter().enumerate() {
            // SAFETY: `take_slice` reserved one byte for every input value.
            unsafe {
                output
                    .add(index)
                    .write(if value { TRUE_MARKER } else { FALSE_MARKER });
            }
        }
        Ok(())
    }

    impl_unsigned_slice!(write_u8_slice, write_u8, u8, 2);
    impl_unsigned_slice!(write_u16_slice, write_u16, u16, 3);
    impl_unsigned_slice!(write_u32_slice, write_u32, u32, 5);
    impl_unsigned_slice!(write_u64_slice, write_u64, u64, 9);
    impl_signed_slice!(write_i8_slice, write_i8, i8, 2, 1);
    impl_signed_slice!(write_i16_slice, write_i16, i16, 3, 2);
    impl_signed_slice!(write_i32_slice, write_i32, i32, 5, 4);
    impl_signed_slice!(write_i64_slice, write_i64, i64, 9, 8);

    #[inline(always)]
    fn write_u8(&mut self, value: u8) -> Result<()> {
        if value <= POS_FIXINT_END {
            *self.take_array::<1>()? = [value];
        } else {
            *self.take_array::<2>()? = [UINT8_MARKER, value];
        }
        Ok(())
    }

    #[inline(always)]
    fn write_f32_slice(&mut self, values: &[f32]) -> Result<()> {
        let len = match values.len().checked_mul(5) {
            Some(len) => len,
            None => return Err(Error::BufferTooSmall),
        };
        let output = self.take_slice(len)?.as_mut_ptr();
        for (index, &value) in values.iter().enumerate() {
            // SAFETY: `take_slice` reserved exactly five bytes per value.
            // Each iteration writes its own marker byte and unaligned u32 payload.
            unsafe {
                let ptr = output.add(index * 5);
                ptr.write(FLOAT32_MARKER);
                ptr.add(1)
                    .cast::<u32>()
                    .write_unaligned(value.to_bits().to_be());
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn write_f64_slice(&mut self, values: &[f64]) -> Result<()> {
        let len = match values.len().checked_mul(9) {
            Some(len) => len,
            None => return Err(Error::BufferTooSmall),
        };
        let output = self.take_slice(len)?.as_mut_ptr();
        for (index, &value) in values.iter().enumerate() {
            // SAFETY: `take_slice` reserved exactly nine bytes per value.
            unsafe {
                let ptr = output.add(index * 9);
                ptr.write(FLOAT64_MARKER);
                ptr.add(1)
                    .cast::<u64>()
                    .write_unaligned(value.to_bits().to_be());
            }
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
    fn write_static_string(&mut self, _value: &'static str, encoded: &'static [u8]) -> Result<()> {
        self.buffer.extend_from_slice(encoded);
        Ok(())
    }

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
    fn write_static_string(&mut self, _value: &'static str, encoded: &'static [u8]) -> Result<()> {
        self.write_all(encoded)
    }

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

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::{SliceWriter, Write};

    #[test]
    fn primitive_slice_writers_match_scalar_encoding() {
        macro_rules! check_slice {
            ($slice:ident, $scalar:ident, $values:expr) => {{
                let values = $values;
                let mut scalar_output = vec![0; values.len() * 10 + 1];
                let mut scalar_writer = SliceWriter::new(&mut scalar_output);
                for &value in &values {
                    scalar_writer.$scalar(value).unwrap();
                }
                let scalar_len = scalar_writer.position();
                let expected = scalar_output[..scalar_len].to_vec();

                // Variable-width integers take their scalar fallback here.
                let mut exact = vec![0; expected.len()];
                let mut exact_writer = SliceWriter::new(&mut exact);
                exact_writer.$slice(&values).unwrap();
                assert_eq!(exact_writer.position(), expected.len());
                assert_eq!(exact, expected);

                // A roomy buffer exercises the single-check fast path.
                let mut roomy = vec![0; values.len() * 10 + 1];
                let mut roomy_writer = SliceWriter::new(&mut roomy);
                roomy_writer.$slice(&values).unwrap();
                assert_eq!(roomy_writer.position(), expected.len());
                assert_eq!(&roomy[..expected.len()], expected);
            }};
        }

        check_slice!(write_boolean_slice, write_boolean, [false, true]);
        check_slice!(write_u8_slice, write_u8, [0, 127, 128, u8::MAX]);
        check_slice!(write_u16_slice, write_u16, [0, 128, 256, u16::MAX]);
        check_slice!(write_u32_slice, write_u32, [0, 256, 65536, u32::MAX]);
        check_slice!(write_u64_slice, write_u64, [0, 65536, 1 << 32, u64::MAX]);
        check_slice!(write_i8_slice, write_i8, [i8::MIN, -32, 0, i8::MAX]);
        check_slice!(write_i16_slice, write_i16, [i16::MIN, -128, 128, i16::MAX]);
        check_slice!(
            write_i32_slice,
            write_i32,
            [i32::MIN, -32769, 32768, i32::MAX]
        );
        check_slice!(
            write_i64_slice,
            write_i64,
            [i64::MIN, -2147483649, 2147483648, i64::MAX]
        );
        check_slice!(write_f32_slice, write_f32, [f32::MIN, -0.0, f32::MAX]);
        check_slice!(write_f64_slice, write_f64, [f64::MIN, -0.0, f64::MAX]);
    }
}
