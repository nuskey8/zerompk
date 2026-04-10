use alloc::vec::Vec;

use crate::{Error, Result, consts::*};

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
        if self.pos + N > self.buffer.len() {
            return Err(Error::BufferTooSmall);
        }
        let array: &mut [u8; N] =
            unsafe { &mut *(self.buffer.as_mut_ptr().add(self.pos) as *mut [u8; N]) };
        self.pos += N;
        Ok(array)
    }

    #[inline(always)]
    fn take_slice(&mut self, len: usize) -> Result<&mut [u8]> {
        if self.pos + len > self.buffer.len() {
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
    fn write_nil(&mut self) -> Result<()> {
        let buf = self.take_array::<1>()?;
        buf[0] = NIL_MARKER;
        Ok(())
    }

    #[inline(always)]
    fn write_boolean(&mut self, b: bool) -> Result<()> {
        let buf = self.take_array::<1>()?;
        buf[0] = if b { TRUE_MARKER } else { FALSE_MARKER };
        Ok(())
    }

    #[inline(always)]
    fn write_u8(&mut self, u: u8) -> Result<()> {
        if u <= POS_FIXINT_END {
            let buf = self.take_array::<1>()?;
            buf[0] = u;
            Ok(())
        } else {
            let buf = self.take_array::<2>()?;
            *buf = [UINT8_MARKER, u];
            Ok(())
        }
    }

    #[inline(always)]
    fn write_u16(&mut self, u: u16) -> Result<()> {
        match u {
            0..=127 => {
                let buf = self.take_array::<1>()?;
                buf[0] = u as u8;
                Ok(())
            }
            128..=255 => {
                let buf = self.take_array::<2>()?;
                *buf = [UINT8_MARKER, u as u8];
                Ok(())
            }
            _ => {
                let buf = self.take_array::<3>()?;
                let [head, tail @ ..] = buf;
                *head = UINT16_MARKER;
                *tail = u.to_be_bytes();
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_u32(&mut self, u: u32) -> Result<()> {
        match u {
            0..=127 => {
                let buf = self.take_array::<1>()?;
                buf[0] = u as u8;
                Ok(())
            }
            128..=255 => {
                let buf = self.take_array::<2>()?;
                *buf = [UINT8_MARKER, u as u8];
                Ok(())
            }
            256..=65535 => {
                let buf = self.take_array::<3>()?;
                let [head, tail @ ..] = buf;
                *head = UINT16_MARKER;
                *tail = (u as u16).to_be_bytes();
                Ok(())
            }
            _ => {
                let buf = self.take_array::<5>()?;
                let [head, tail @ ..] = buf;
                *head = UINT32_MARKER;
                *tail = u.to_be_bytes();
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_u64(&mut self, u: u64) -> Result<()> {
        match u {
            0..=127 => {
                let buf = self.take_array::<1>()?;
                buf[0] = u as u8;
                Ok(())
            }
            128..=255 => {
                let buf = self.take_array::<2>()?;
                *buf = [UINT8_MARKER, u as u8];
                Ok(())
            }
            256..=65535 => {
                let buf = self.take_array::<3>()?;
                let [head, tail @ ..] = buf;
                *head = UINT16_MARKER;
                *tail = (u as u16).to_be_bytes();
                Ok(())
            }
            65536..=4294967295 => {
                let buf = self.take_array::<5>()?;
                let [head, tail @ ..] = buf;
                *head = UINT32_MARKER;
                *tail = (u as u32).to_be_bytes();
                Ok(())
            }
            _ => {
                let buf = self.take_array::<9>()?;
                let [head, tail @ ..] = buf;
                *head = UINT64_MARKER;
                *tail = u.to_be_bytes();
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_i8(&mut self, i: i8) -> Result<()> {
        match i {
            0..=127 => {
                let buf = self.take_array::<1>()?;
                buf[0] = i as u8;
                Ok(())
            }
            -32..=-1 => {
                let buf = self.take_array::<1>()?;
                buf[0] = 0xe0 | ((i + 32) as u8);
                Ok(())
            }
            _ => {
                let buf = self.take_array::<2>()?;
                *buf = [INT8_MARKER, i as u8];
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_i16(&mut self, i: i16) -> Result<()> {
        match i {
            0..=127 => {
                let buf = self.take_array::<1>()?;
                buf[0] = i as u8;
                Ok(())
            }
            -32..=-1 => {
                let buf = self.take_array::<1>()?;
                buf[0] = 0xe0 | ((i + 32) as u8);
                Ok(())
            }
            -128..=127 => {
                let buf = self.take_array::<2>()?;
                *buf = [INT8_MARKER, i as u8];
                Ok(())
            }
            _ => {
                let buf = self.take_array::<3>()?;
                let [head, tail @ ..] = buf;
                *head = INT16_MARKER;
                *tail = i.to_be_bytes();
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_i32(&mut self, i: i32) -> Result<()> {
        match i {
            0..=127 => {
                let buf = self.take_array::<1>()?;
                buf[0] = i as u8;
                Ok(())
            }
            -32..=-1 => {
                let buf = self.take_array::<1>()?;
                buf[0] = 0xe0 | ((i + 32) as u8);
                Ok(())
            }
            -128..=127 => {
                let buf = self.take_array::<2>()?;
                *buf = [INT8_MARKER, i as u8];
                Ok(())
            }
            -32768..=32767 => {
                let buf = self.take_array::<3>()?;
                let [head, tail @ ..] = buf;
                *head = INT16_MARKER;
                *tail = (i as i16).to_be_bytes();
                Ok(())
            }
            _ => {
                let buf = self.take_array::<5>()?;
                let [head, tail @ ..] = buf;
                *head = INT32_MARKER;
                *tail = i.to_be_bytes();
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_i64(&mut self, i: i64) -> Result<()> {
        match i {
            0..=127 => {
                let buf = self.take_array::<1>()?;
                buf[0] = i as u8;
                Ok(())
            }
            -32..=-1 => {
                let buf = self.take_array::<1>()?;
                buf[0] = 0xe0 | ((i + 32) as u8);
                Ok(())
            }
            -128..=127 => {
                let buf = self.take_array::<2>()?;
                *buf = [INT8_MARKER, i as u8];
                Ok(())
            }
            -32768..=32767 => {
                let buf = self.take_array::<3>()?;
                let [head, tail @ ..] = buf;
                *head = INT16_MARKER;
                *tail = (i as i16).to_be_bytes();
                Ok(())
            }
            -2147483648..=2147483647 => {
                let buf = self.take_array::<5>()?;
                let [head, tail @ ..] = buf;
                *head = INT32_MARKER;
                *tail = (i as i32).to_be_bytes();
                Ok(())
            }
            _ => {
                let buf = self.take_array::<9>()?;
                let [head, tail @ ..] = buf;
                *head = INT64_MARKER;
                *tail = i.to_be_bytes();
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_f32(&mut self, f: f32) -> Result<()> {
        let buf = self.take_array::<5>()?;
        let [head, tail @ ..] = buf;
        *head = FLOAT32_MARKER;
        *tail = f.to_be_bytes();
        Ok(())
    }

    #[inline(always)]
    fn write_f64(&mut self, f: f64) -> Result<()> {
        let buf = self.take_array::<9>()?;
        let [head, tail @ ..] = buf;
        *head = FLOAT64_MARKER;
        *tail = f.to_be_bytes();
        Ok(())
    }

    #[inline(always)]
    fn write_string(&mut self, s: &str) -> Result<()> {
        let len = s.len();
        match len {
            // FixStr
            0..=31 => {
                let buf = self.take_slice(1 + len)?;
                unsafe {
                    let ptr = buf.as_mut_ptr();
                    *ptr = 0xa0 | (len as u8);
                    core::ptr::copy_nonoverlapping(s.as_bytes().as_ptr(), ptr.add(1), len);
                }
                Ok(())
            }
            // Str8
            32..=255 => {
                let buf = self.take_slice(2 + len)?;
                unsafe {
                    let ptr = buf.as_mut_ptr();
                    *ptr = STR8_MARKER;
                    *ptr.add(1) = len as u8;
                    core::ptr::copy_nonoverlapping(s.as_bytes().as_ptr(), ptr.add(2), len);
                }
                Ok(())
            }
            // Str16
            256..=65535 => {
                let buf = self.take_slice(3 + len)?;
                unsafe {
                    let ptr = buf.as_mut_ptr();
                    *ptr = STR16_MARKER;
                    core::ptr::copy_nonoverlapping(
                        (len as u16).to_be_bytes().as_ptr(),
                        ptr.add(1),
                        2,
                    );
                    core::ptr::copy_nonoverlapping(s.as_bytes().as_ptr(), ptr.add(3), len);
                }
                Ok(())
            }
            // Str32
            _ => {
                let buf = self.take_slice(5 + len)?;
                unsafe {
                    let ptr = buf.as_mut_ptr();
                    *ptr = STR32_MARKER;
                    core::ptr::copy_nonoverlapping(
                        (len as u32).to_be_bytes().as_ptr(),
                        ptr.add(1),
                        4,
                    );
                    core::ptr::copy_nonoverlapping(s.as_bytes().as_ptr(), ptr.add(5), len);
                }
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_binary(&mut self, data: &[u8]) -> Result<()> {
        let len = data.len();
        match len {
            // Bin8
            0..=255 => {
                let buf = self.take_slice(2 + len)?;
                unsafe {
                    let ptr = buf.as_mut_ptr();
                    *ptr = BIN8_MARKER;
                    *ptr.add(1) = len as u8;
                    core::ptr::copy_nonoverlapping(data.as_ptr(), ptr.add(2), len);
                }
                Ok(())
            }
            // Bin16
            256..=65535 => {
                let buf = self.take_slice(3 + len)?;
                unsafe {
                    let ptr = buf.as_mut_ptr();
                    *ptr = BIN16_MARKER;
                    core::ptr::copy_nonoverlapping(
                        (len as u16).to_be_bytes().as_ptr(),
                        ptr.add(1),
                        2,
                    );
                    core::ptr::copy_nonoverlapping(data.as_ptr(), ptr.add(3), len);
                }
                Ok(())
            }
            // Bin32
            _ => {
                let buf = self.take_slice(5 + len)?;
                unsafe {
                    let ptr = buf.as_mut_ptr();
                    *ptr = BIN32_MARKER;
                    core::ptr::copy_nonoverlapping(
                        (len as u32).to_be_bytes().as_ptr(),
                        ptr.add(1),
                        4,
                    );
                    core::ptr::copy_nonoverlapping(data.as_ptr(), ptr.add(5), len);
                }
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_timestamp(&mut self, seconds: i64, nanoseconds: u32) -> Result<()> {
        if nanoseconds >= 1_000_000_000 {
            return Err(Error::InvalidTimestamp);
        }

        // timestamp 32: sec in [0, 2^32-1], nsec == 0
        if nanoseconds == 0 && (0..=u32::MAX as i64).contains(&seconds) {
            let buf = self.take_array::<6>()?;
            let [head, type_marker, tail @ ..] = buf;
            *head = TIMESTAMP32_MARKER;
            *type_marker = 0xff;
            *tail = (seconds as u32).to_be_bytes();
            return Ok(());
        }

        // timestamp 64: sec in [0, 2^34-1]
        if (0..=(1i64 << 34) - 1).contains(&seconds) {
            let data = ((nanoseconds as u64) << 34) | (seconds as u64);
            let buf = self.take_array::<10>()?;
            let [head, type_marker, tail @ ..] = buf;
            *head = TIMESTAMP64_MARKER;
            *type_marker = 0xff;
            *tail = data.to_be_bytes();
            return Ok(());
        }

        // timestamp 96
        let buf = self.take_array::<15>()?;
        let [head, len_marker, type_marker, tail @ ..] = buf;
        *head = TIMESTAMP96_MARKER;
        *len_marker = 12;
        *type_marker = 0xff;
        unsafe {
            let tail_ptr = tail.as_mut_ptr();
            core::ptr::copy_nonoverlapping(nanoseconds.to_be_bytes().as_ptr(), tail_ptr, 4);
            core::ptr::copy_nonoverlapping(seconds.to_be_bytes().as_ptr(), tail_ptr.add(4), 8);
        }
        Ok(())
    }

    #[inline(always)]
    fn write_array_len(&mut self, len: usize) -> Result<()> {
        match len {
            0..=15 => {
                let buf = self.take_array::<1>()?;
                buf[0] = FIXARRAY_START | (len as u8);
                Ok(())
            }
            16..=65535 => {
                let buf = self.take_array::<3>()?;
                let [head, tail @ ..] = buf;
                *head = ARRAY16_MARKER;
                *tail = (len as u16).to_be_bytes();
                Ok(())
            }
            _ => {
                let buf = self.take_array::<5>()?;
                let [head, tail @ ..] = buf;
                *head = ARRAY32_MARKER;
                *tail = (len as u32).to_be_bytes();
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_map_len(&mut self, len: usize) -> Result<()> {
        match len {
            0..=15 => {
                let buf = self.take_array::<1>()?;
                buf[0] = FIXMAP_START | (len as u8);
                Ok(())
            }
            16..=65535 => {
                let buf = self.take_array::<3>()?;
                let [head, tail @ ..] = buf;
                *head = MAP16_MARKER;
                *tail = (len as u16).to_be_bytes();
                Ok(())
            }
            _ => {
                let buf = self.take_array::<5>()?;
                let [head, tail @ ..] = buf;
                *head = MAP32_MARKER;
                *tail = (len as u32).to_be_bytes();
                Ok(())
            }
        }
    }

    fn write_ext(&mut self, type_id: i8, data: &[u8]) -> Result<()> {
        let len = data.len();
        match len {
            1 => {
                let buf = self.take_array::<3>()?;
                let [head, type_marker, tail] = buf;
                *head = FIXEXT1_MARKER;
                *type_marker = type_id as u8;
                *tail = data[0];
                Ok(())
            }
            2 => {
                let buf = self.take_array::<4>()?;
                let [head, type_marker, tail @ ..] = buf;
                *head = FIXEXT2_MARKER;
                *type_marker = type_id as u8;
                *tail = data.try_into().unwrap();
                Ok(())
            }
            4 => {
                let buf = self.take_array::<6>()?;
                let [head, type_marker, tail @ ..] = buf;
                *head = FIXEXT4_MARKER;
                *type_marker = type_id as u8;
                *tail = data.try_into().unwrap();
                Ok(())
            }
            8 => {
                let buf = self.take_array::<10>()?;
                let [head, type_marker, tail @ ..] = buf;
                *head = FIXEXT8_MARKER;
                *type_marker = type_id as u8;
                *tail = data.try_into().unwrap();
                Ok(())
            }
            16 => {
                let buf = self.take_array::<18>()?;
                let [head, type_marker, tail @ ..] = buf;
                *head = FIXEXT16_MARKER;
                *type_marker = type_id as u8;
                *tail = data.try_into().unwrap();
                Ok(())
            }
            0..=255 => {
                let buf = self.take_slice(3 + len)?;
                unsafe {
                    let (header, body) = buf.split_at_mut(3);
                    header.copy_from_slice(&[EXT8_MARKER, len as u8, type_id as u8]);
                    core::ptr::copy_nonoverlapping(data.as_ptr(), body.as_mut_ptr(), len);
                }
                Ok(())
            }
            256..=65535 => {
                let buf = self.take_slice(4 + len)?;
                unsafe {
                    let (header, body) = buf.split_at_mut(4);
                    let len_bytes = (len as u16).to_be_bytes();
                    header.copy_from_slice(&[
                        EXT16_MARKER,
                        len_bytes[0],
                        len_bytes[1],
                        type_id as u8,
                    ]);
                    core::ptr::copy_nonoverlapping(data.as_ptr(), body.as_mut_ptr(), len);
                }
                Ok(())
            }
            _ => {
                let buf = self.take_slice(6 + len)?;
                unsafe {
                    let (header, body) = buf.split_at_mut(6);
                    let len_bytes = (len as u32).to_be_bytes();
                    header.copy_from_slice(&[
                        EXT32_MARKER,
                        len_bytes[0],
                        len_bytes[1],
                        len_bytes[2],
                        len_bytes[3],
                        type_id as u8,
                    ]);
                    core::ptr::copy_nonoverlapping(data.as_ptr(), body.as_mut_ptr(), len);
                }
                Ok(())
            }
        }
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
    fn write_nil(&mut self) -> Result<()> {
        self.buffer.push(NIL_MARKER);
        Ok(())
    }

    #[inline(always)]
    fn write_boolean(&mut self, b: bool) -> Result<()> {
        self.buffer.push(if b { TRUE_MARKER } else { FALSE_MARKER });
        Ok(())
    }

    #[inline(always)]
    fn write_u8(&mut self, u: u8) -> Result<()> {
        if u <= POS_FIXINT_END {
            self.buffer.push(u);
        } else {
            self.buffer.reserve(2);
            unsafe {
                let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                *ptr = UINT8_MARKER;
                *ptr.add(1) = u;
                self.buffer.set_len(self.buffer.len() + 2);
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn write_u16(&mut self, u: u16) -> Result<()> {
        match u {
            0..=127 => {
                self.buffer.push(u as u8);
                Ok(())
            }
            128..=255 => {
                self.buffer.reserve(2);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = UINT8_MARKER;
                    *ptr.add(1) = u as u8;
                    self.buffer.set_len(self.buffer.len() + 2);
                }
                Ok(())
            }
            _ => {
                self.buffer.reserve(3);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = UINT16_MARKER;
                    ptr.add(1)
                        .copy_from_nonoverlapping(u.to_be_bytes().as_ptr(), 2);
                    self.buffer.set_len(self.buffer.len() + 3);
                }
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_u32(&mut self, u: u32) -> Result<()> {
        match u {
            0..=127 => {
                self.buffer.push(u as u8);
                Ok(())
            }
            128..=255 => {
                self.buffer.reserve(2);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = UINT8_MARKER;
                    *ptr.add(1) = u as u8;
                    self.buffer.set_len(self.buffer.len() + 2);
                }
                Ok(())
            }
            256..=65535 => {
                self.buffer.reserve(3);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = UINT16_MARKER;
                    ptr.add(1)
                        .copy_from_nonoverlapping((u as u16).to_be_bytes().as_ptr(), 2);
                    self.buffer.set_len(self.buffer.len() + 3);
                }
                Ok(())
            }
            _ => {
                self.buffer.reserve(5);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = UINT32_MARKER;
                    ptr.add(1)
                        .copy_from_nonoverlapping(u.to_be_bytes().as_ptr(), 4);
                    self.buffer.set_len(self.buffer.len() + 5);
                }
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_u64(&mut self, u: u64) -> Result<()> {
        match u {
            0..=127 => {
                self.buffer.push(u as u8);
                Ok(())
            }
            128..=255 => {
                self.buffer.reserve(2);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = UINT8_MARKER;
                    *ptr.add(1) = u as u8;
                    self.buffer.set_len(self.buffer.len() + 2);
                }
                Ok(())
            }
            256..=65535 => {
                self.buffer.reserve(3);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = UINT16_MARKER;
                    ptr.add(1)
                        .copy_from_nonoverlapping((u as u16).to_be_bytes().as_ptr(), 2);
                    self.buffer.set_len(self.buffer.len() + 3);
                }
                Ok(())
            }
            65536..=4294967295 => {
                self.buffer.reserve(5);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = UINT32_MARKER;
                    ptr.add(1)
                        .copy_from_nonoverlapping((u as u32).to_be_bytes().as_ptr(), 4);
                    self.buffer.set_len(self.buffer.len() + 5);
                }
                Ok(())
            }
            _ => {
                self.buffer.reserve(9);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = UINT64_MARKER;
                    ptr.add(1)
                        .copy_from_nonoverlapping(u.to_be_bytes().as_ptr(), 8);
                    self.buffer.set_len(self.buffer.len() + 9);
                }
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_i8(&mut self, i: i8) -> Result<()> {
        match i {
            0..=127 => {
                self.buffer.push(i as u8);
                Ok(())
            }
            -32..=-1 => {
                self.buffer.push(0xe0 | ((i + 32) as u8));
                Ok(())
            }
            _ => {
                self.buffer.reserve(2);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = INT8_MARKER;
                    *ptr.add(1) = i as u8;
                    self.buffer.set_len(self.buffer.len() + 2);
                }
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_i16(&mut self, i: i16) -> Result<()> {
        match i {
            0..=127 => {
                self.buffer.push(i as u8);
                Ok(())
            }
            -32..=-1 => {
                self.buffer.push(0xe0 | ((i + 32) as u8));
                Ok(())
            }
            -128..=127 => {
                self.buffer.reserve(2);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = INT8_MARKER;
                    *ptr.add(1) = i as u8;
                    self.buffer.set_len(self.buffer.len() + 2);
                }
                Ok(())
            }
            _ => {
                self.buffer.reserve(3);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = INT16_MARKER;
                    ptr.add(1)
                        .copy_from_nonoverlapping(i.to_be_bytes().as_ptr(), 2);
                    self.buffer.set_len(self.buffer.len() + 3);
                }
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_i32(&mut self, i: i32) -> Result<()> {
        match i {
            0..=127 => {
                self.buffer.push(i as u8);
                Ok(())
            }
            -32..=-1 => {
                self.buffer.push(0xe0 | ((i + 32) as u8));
                Ok(())
            }
            -128..=127 => {
                self.buffer.reserve(2);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = INT8_MARKER;
                    *ptr.add(1) = i as u8;
                    self.buffer.set_len(self.buffer.len() + 2);
                }
                Ok(())
            }
            -32768..=32767 => {
                self.buffer.reserve(3);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = INT16_MARKER;
                    ptr.add(1)
                        .copy_from_nonoverlapping((i as i16).to_be_bytes().as_ptr(), 2);
                    self.buffer.set_len(self.buffer.len() + 3);
                }
                Ok(())
            }
            _ => {
                self.buffer.reserve(5);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = INT32_MARKER;
                    ptr.add(1)
                        .copy_from_nonoverlapping(i.to_be_bytes().as_ptr(), 4);
                    self.buffer.set_len(self.buffer.len() + 5);
                }
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_i64(&mut self, i: i64) -> Result<()> {
        match i {
            0..=127 => {
                self.buffer.push(i as u8);
                Ok(())
            }
            -32..=-1 => {
                self.buffer.push(0xe0 | ((i + 32) as u8));
                Ok(())
            }
            -128..=127 => {
                self.buffer.reserve(2);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = INT8_MARKER;
                    *ptr.add(1) = i as u8;
                    self.buffer.set_len(self.buffer.len() + 2);
                }
                Ok(())
            }
            -32768..=32767 => {
                self.buffer.reserve(3);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = INT16_MARKER;
                    ptr.add(1)
                        .copy_from_nonoverlapping((i as i16).to_be_bytes().as_ptr(), 2);
                    self.buffer.set_len(self.buffer.len() + 3);
                }
                Ok(())
            }
            -2147483648..=2147483647 => {
                self.buffer.reserve(5);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = INT32_MARKER;
                    ptr.add(1)
                        .copy_from_nonoverlapping((i as i32).to_be_bytes().as_ptr(), 4);
                    self.buffer.set_len(self.buffer.len() + 5);
                }
                Ok(())
            }
            _ => {
                self.buffer.reserve(9);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = INT64_MARKER;
                    ptr.add(1)
                        .copy_from_nonoverlapping(i.to_be_bytes().as_ptr(), 8);
                    self.buffer.set_len(self.buffer.len() + 9);
                }
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_f32(&mut self, f: f32) -> Result<()> {
        self.buffer.reserve(5);
        unsafe {
            let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
            *ptr = FLOAT32_MARKER;
            ptr.add(1)
                .copy_from_nonoverlapping(f.to_be_bytes().as_ptr(), 4);
            self.buffer.set_len(self.buffer.len() + 5);
        }
        Ok(())
    }

    #[inline(always)]
    fn write_f64(&mut self, f: f64) -> Result<()> {
        self.buffer.reserve(9);
        unsafe {
            let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
            *ptr = FLOAT64_MARKER;
            ptr.add(1)
                .copy_from_nonoverlapping(f.to_be_bytes().as_ptr(), 8);
            self.buffer.set_len(self.buffer.len() + 9);
        }
        Ok(())
    }

    #[inline(always)]
    fn write_string(&mut self, s: &str) -> Result<()> {
        let len = s.len();
        match len {
            0..=31 => {
                self.buffer.reserve(1 + len);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = 0xa0 | (len as u8);
                    ptr.add(1)
                        .copy_from_nonoverlapping(s.as_bytes().as_ptr(), len);
                    self.buffer.set_len(self.buffer.len() + 1 + len);
                }
                Ok(())
            }
            32..=255 => {
                self.buffer.reserve(2 + len);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = STR8_MARKER;
                    *ptr.add(1) = len as u8;
                    ptr.add(2)
                        .copy_from_nonoverlapping(s.as_bytes().as_ptr(), len);
                    self.buffer.set_len(self.buffer.len() + 2 + len);
                }
                Ok(())
            }
            256..=65535 => {
                self.buffer.reserve(3 + len);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = STR16_MARKER;
                    let len_bytes = (len as u16).to_be_bytes();
                    ptr.add(1).copy_from_nonoverlapping(len_bytes.as_ptr(), 2);
                    ptr.add(3)
                        .copy_from_nonoverlapping(s.as_bytes().as_ptr(), len);
                    self.buffer.set_len(self.buffer.len() + 3 + len);
                }
                Ok(())
            }
            _ => {
                self.buffer.reserve(5 + len);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = STR32_MARKER;
                    let len_bytes = (len as u32).to_be_bytes();
                    ptr.add(1).copy_from_nonoverlapping(len_bytes.as_ptr(), 4);
                    ptr.add(5)
                        .copy_from_nonoverlapping(s.as_bytes().as_ptr(), len);
                    self.buffer.set_len(self.buffer.len() + 5 + len);
                }
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_binary(&mut self, data: &[u8]) -> Result<()> {
        let len = data.len();
        match len {
            0..=255 => {
                self.buffer.reserve(2 + len);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = BIN8_MARKER;
                    *ptr.add(1) = len as u8;
                    ptr.add(2).copy_from_nonoverlapping(data.as_ptr(), len);
                    self.buffer.set_len(self.buffer.len() + 2 + len);
                }
                Ok(())
            }
            256..=65535 => {
                self.buffer.reserve(3 + len);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = BIN16_MARKER;
                    let len_bytes = (len as u16).to_be_bytes();
                    ptr.add(1).copy_from_nonoverlapping(len_bytes.as_ptr(), 2);
                    ptr.add(3).copy_from_nonoverlapping(data.as_ptr(), len);
                    self.buffer.set_len(self.buffer.len() + 3 + len);
                }
                Ok(())
            }
            _ => {
                self.buffer.reserve(5 + len);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = BIN32_MARKER;
                    let len_bytes = (len as u32).to_be_bytes();
                    ptr.add(1).copy_from_nonoverlapping(len_bytes.as_ptr(), 4);
                    ptr.add(5).copy_from_nonoverlapping(data.as_ptr(), len);
                    self.buffer.set_len(self.buffer.len() + 5 + len);
                }
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_timestamp(&mut self, seconds: i64, nanoseconds: u32) -> Result<()> {
        if nanoseconds >= 1_000_000_000 {
            return Err(Error::InvalidTimestamp);
        }

        // timestamp 32: sec in [0, 2^32-1], nsec == 0
        if nanoseconds == 0 && (0..=u32::MAX as i64).contains(&seconds) {
            self.buffer.reserve(6);
            unsafe {
                let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                *ptr = TIMESTAMP32_MARKER;
                *ptr.add(1) = 0xff;
                ptr.add(2)
                    .copy_from_nonoverlapping((seconds as u32).to_be_bytes().as_ptr(), 4);
                self.buffer.set_len(self.buffer.len() + 6);
            }
            return Ok(());
        }

        // timestamp 64: sec in [0, 2^34-1]
        if (0..=(1i64 << 34) - 1).contains(&seconds) {
            let data = ((nanoseconds as u64) << 34) | (seconds as u64);
            self.buffer.reserve(10);
            unsafe {
                let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                *ptr = TIMESTAMP64_MARKER;
                *ptr.add(1) = 0xff;
                ptr.add(2)
                    .copy_from_nonoverlapping(data.to_be_bytes().as_ptr(), 8);
                self.buffer.set_len(self.buffer.len() + 10);
            }
            return Ok(());
        }

        // timestamp 96
        self.buffer.reserve(15);
        unsafe {
            let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
            *ptr = TIMESTAMP96_MARKER;
            *ptr.add(1) = 12;
            *ptr.add(2) = 0xff;
            ptr.add(3)
                .copy_from_nonoverlapping(nanoseconds.to_be_bytes().as_ptr(), 4);
            ptr.add(7)
                .copy_from_nonoverlapping(seconds.to_be_bytes().as_ptr(), 8);
            self.buffer.set_len(self.buffer.len() + 15);
        }
        Ok(())
    }

    #[inline(always)]
    fn write_array_len(&mut self, len: usize) -> Result<()> {
        match len {
            0..=15 => {
                self.buffer.push(FIXARRAY_START | (len as u8));
                Ok(())
            }
            16..=65535 => {
                self.buffer.reserve(3);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = ARRAY16_MARKER;
                    ptr.add(1)
                        .copy_from_nonoverlapping((len as u16).to_be_bytes().as_ptr(), 2);
                    self.buffer.set_len(self.buffer.len() + 3);
                }
                Ok(())
            }
            _ => {
                self.buffer.reserve(5);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = ARRAY32_MARKER;
                    ptr.add(1)
                        .copy_from_nonoverlapping((len as u32).to_be_bytes().as_ptr(), 4);
                    self.buffer.set_len(self.buffer.len() + 5);
                }
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_map_len(&mut self, len: usize) -> Result<()> {
        match len {
            0..=15 => {
                self.buffer.push(FIXMAP_START | (len as u8));
                Ok(())
            }
            16..=65535 => {
                self.buffer.reserve(3);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = MAP16_MARKER;
                    ptr.add(1)
                        .copy_from_nonoverlapping((len as u16).to_be_bytes().as_ptr(), 2);
                    self.buffer.set_len(self.buffer.len() + 3);
                }
                Ok(())
            }
            _ => {
                self.buffer.reserve(5);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = MAP32_MARKER;
                    ptr.add(1)
                        .copy_from_nonoverlapping((len as u32).to_be_bytes().as_ptr(), 4);
                    self.buffer.set_len(self.buffer.len() + 5);
                }
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_ext(&mut self, type_id: i8, data: &[u8]) -> Result<()> {
        let len = data.len();
        match len {
            1 => {
                self.buffer.reserve(3);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = FIXEXT1_MARKER;
                    *ptr.add(1) = type_id as u8;
                    *ptr.add(2) = data[0];
                    self.buffer.set_len(self.buffer.len() + 3);
                }
                Ok(())
            }
            2 => {
                self.buffer.reserve(4);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = FIXEXT2_MARKER;
                    *ptr.add(1) = type_id as u8;
                    ptr.add(2).copy_from_nonoverlapping(data.as_ptr(), 2);
                    self.buffer.set_len(self.buffer.len() + 4);
                }
                Ok(())
            }
            4 => {
                self.buffer.reserve(6);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = FIXEXT4_MARKER;
                    *ptr.add(1) = type_id as u8;
                    ptr.add(2).copy_from_nonoverlapping(data.as_ptr(), 4);
                    self.buffer.set_len(self.buffer.len() + 6);
                }
                Ok(())
            }
            8 => {
                self.buffer.reserve(10);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = FIXEXT8_MARKER;
                    *ptr.add(1) = type_id as u8;
                    ptr.add(2).copy_from_nonoverlapping(data.as_ptr(), 8);
                    self.buffer.set_len(self.buffer.len() + 10);
                }
                Ok(())
            }
            16 => {
                self.buffer.reserve(18);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = FIXEXT16_MARKER;
                    *ptr.add(1) = type_id as u8;
                    ptr.add(2).copy_from_nonoverlapping(data.as_ptr(), 16);
                    self.buffer.set_len(self.buffer.len() + 18);
                }
                Ok(())
            }
            0..=255 => {
                self.buffer.reserve(3 + len);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = EXT8_MARKER;
                    *ptr.add(1) = len as u8;
                    *ptr.add(2) = type_id as u8;
                    ptr.add(3).copy_from_nonoverlapping(data.as_ptr(), len);
                    self.buffer.set_len(self.buffer.len() + 3 + len);
                }
                Ok(())
            }
            256..=65535 => {
                self.buffer.reserve(4 + len);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = EXT16_MARKER;
                    let len_bytes = (len as u16).to_be_bytes();
                    ptr.add(1).copy_from_nonoverlapping(len_bytes.as_ptr(), 2);
                    *ptr.add(3) = type_id as u8;
                    ptr.add(4).copy_from_nonoverlapping(data.as_ptr(), len);
                    self.buffer.set_len(self.buffer.len() + 4 + len);
                }
                Ok(())
            }
            _ => {
                self.buffer.reserve(6 + len);
                unsafe {
                    let ptr = self.buffer.as_mut_ptr().add(self.buffer.len());
                    *ptr = EXT32_MARKER;
                    let len_bytes = (len as u32).to_be_bytes();
                    ptr.add(1).copy_from_nonoverlapping(len_bytes.as_ptr(), 4);
                    *ptr.add(5) = type_id as u8;
                    ptr.add(6).copy_from_nonoverlapping(data.as_ptr(), len);
                    self.buffer.set_len(self.buffer.len() + 6 + len);
                }
                Ok(())
            }
        }
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
    fn write_nil(&mut self) -> Result<()> {
        self.write_all(&[NIL_MARKER])?;
        Ok(())
    }

    #[inline(always)]
    fn write_boolean(&mut self, b: bool) -> Result<()> {
        self.write_all(&[if b { TRUE_MARKER } else { FALSE_MARKER }])?;
        Ok(())
    }

    #[inline(always)]
    fn write_u8(&mut self, u: u8) -> Result<()> {
        if u <= 127 {
            self.write_all(&[u])?;
        } else {
            self.write_all(&[UINT8_MARKER, u])?;
        }
        Ok(())
    }

    #[inline(always)]
    fn write_u16(&mut self, u: u16) -> Result<()> {
        match u {
            0..=127 => {
                self.write_all(&[u as u8])?;
                Ok(())
            }
            128..=255 => {
                self.write_all(&[UINT8_MARKER, u as u8])?;
                Ok(())
            }
            _ => {
                let len_bytes = u.to_be_bytes();
                self.write_all(&[UINT16_MARKER, len_bytes[0], len_bytes[1]])?;
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_u32(&mut self, u: u32) -> Result<()> {
        match u {
            0..=127 => {
                self.write_all(&[u as u8])?;
                Ok(())
            }
            128..=255 => {
                self.write_all(&[UINT8_MARKER, u as u8])?;
                Ok(())
            }
            256..=65535 => {
                let len_bytes = (u as u16).to_be_bytes();
                self.write_all(&[UINT16_MARKER, len_bytes[0], len_bytes[1]])?;
                Ok(())
            }
            _ => {
                let len_bytes = u.to_be_bytes();
                self.write_all(&[
                    UINT32_MARKER,
                    len_bytes[0],
                    len_bytes[1],
                    len_bytes[2],
                    len_bytes[3],
                ])?;
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_u64(&mut self, u: u64) -> Result<()> {
        match u {
            0..=127 => {
                self.write_all(&[u as u8])?;
                Ok(())
            }
            128..=255 => {
                self.write_all(&[UINT8_MARKER, u as u8])?;
                Ok(())
            }
            256..=65535 => {
                let len_bytes = (u as u16).to_be_bytes();
                self.write_all(&[UINT16_MARKER, len_bytes[0], len_bytes[1]])?;
                Ok(())
            }
            65536..=4294967295 => {
                let len_bytes = (u as u32).to_be_bytes();
                self.write_all(&[
                    UINT32_MARKER,
                    len_bytes[0],
                    len_bytes[1],
                    len_bytes[2],
                    len_bytes[3],
                ])?;
                Ok(())
            }
            _ => {
                let len_bytes = u.to_be_bytes();
                self.write_all(&[
                    UINT64_MARKER,
                    len_bytes[0],
                    len_bytes[1],
                    len_bytes[2],
                    len_bytes[3],
                    len_bytes[4],
                    len_bytes[5],
                    len_bytes[6],
                    len_bytes[7],
                ])?;
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_i8(&mut self, i: i8) -> Result<()> {
        match i {
            0..=127 => {
                self.write_all(&[i as u8])?;
                Ok(())
            }
            -32..=-1 => {
                self.write_all(&[0xe0 | ((i + 32) as u8)])?;
                Ok(())
            }
            _ => {
                self.write_all(&[INT8_MARKER, i as u8])?;
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_i16(&mut self, i: i16) -> Result<()> {
        match i {
            0..=127 => {
                self.write_all(&[i as u8])?;
                Ok(())
            }
            -32..=-1 => {
                self.write_all(&[0xe0 | ((i + 32) as u8)])?;
                Ok(())
            }
            -128..=127 => {
                self.write_all(&[INT8_MARKER, i as u8])?;
                Ok(())
            }
            _ => {
                let len_bytes = i.to_be_bytes();
                self.write_all(&[INT16_MARKER, len_bytes[0], len_bytes[1]])?;
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_i32(&mut self, i: i32) -> Result<()> {
        match i {
            0..=127 => {
                self.write_all(&[i as u8])?;
                Ok(())
            }
            -32..=-1 => {
                self.write_all(&[0xe0 | ((i + 32) as u8)])?;
                Ok(())
            }
            -128..=127 => {
                self.write_all(&[INT8_MARKER, i as u8])?;
                Ok(())
            }
            -32768..=32767 => {
                let len_bytes = (i as i16).to_be_bytes();
                self.write_all(&[INT16_MARKER, len_bytes[0], len_bytes[1]])?;
                Ok(())
            }
            _ => {
                let len_bytes = i.to_be_bytes();
                self.write_all(&[
                    INT32_MARKER,
                    len_bytes[0],
                    len_bytes[1],
                    len_bytes[2],
                    len_bytes[3],
                ])?;
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_i64(&mut self, i: i64) -> Result<()> {
        match i {
            0..=127 => {
                self.write_all(&[i as u8])?;
                Ok(())
            }
            -32..=-1 => {
                self.write_all(&[0xe0 | ((i + 32) as u8)])?;
                Ok(())
            }
            -128..=127 => {
                self.write_all(&[INT8_MARKER, i as u8])?;
                Ok(())
            }
            -32768..=32767 => {
                let len_bytes = (i as i16).to_be_bytes();
                self.write_all(&[INT16_MARKER, len_bytes[0], len_bytes[1]])?;
                Ok(())
            }
            -2147483648..=2147483647 => {
                let len_bytes = (i as i32).to_be_bytes();
                self.write_all(&[
                    INT32_MARKER,
                    len_bytes[0],
                    len_bytes[1],
                    len_bytes[2],
                    len_bytes[3],
                ])?;
                Ok(())
            }
            _ => {
                let len_bytes = i.to_be_bytes();
                self.write_all(&[
                    INT64_MARKER,
                    len_bytes[0],
                    len_bytes[1],
                    len_bytes[2],
                    len_bytes[3],
                    len_bytes[4],
                    len_bytes[5],
                    len_bytes[6],
                    len_bytes[7],
                ])?;
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_f32(&mut self, f: f32) -> Result<()> {
        let len_bytes = f.to_be_bytes();
        self.write_all(&[
            FLOAT32_MARKER,
            len_bytes[0],
            len_bytes[1],
            len_bytes[2],
            len_bytes[3],
        ])?;
        Ok(())
    }

    #[inline(always)]
    fn write_f64(&mut self, f: f64) -> Result<()> {
        let len_bytes = f.to_be_bytes();
        self.write_all(&[
            FLOAT64_MARKER,
            len_bytes[0],
            len_bytes[1],
            len_bytes[2],
            len_bytes[3],
            len_bytes[4],
            len_bytes[5],
            len_bytes[6],
            len_bytes[7],
        ])?;
        Ok(())
    }

    fn write_string(&mut self, s: &str) -> Result<()> {
        let len = s.len();
        match len {
            0..=31 => {
                self.write_all(&[0xa0 | (len as u8)])?;
                self.write_all(s.as_bytes())?;
                Ok(())
            }
            32..=255 => {
                self.write_all(&[STR8_MARKER, len as u8])?;
                self.write_all(s.as_bytes())?;
                Ok(())
            }
            256..=65535 => {
                let len_bytes = (len as u16).to_be_bytes();
                self.write_all(&[STR16_MARKER, len_bytes[0], len_bytes[1]])?;
                self.write_all(s.as_bytes())?;
                Ok(())
            }
            _ => {
                let len_bytes = (len as u32).to_be_bytes();
                self.write_all(&[
                    STR32_MARKER,
                    len_bytes[0],
                    len_bytes[1],
                    len_bytes[2],
                    len_bytes[3],
                ])?;
                self.write_all(s.as_bytes())?;
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_binary(&mut self, data: &[u8]) -> Result<()> {
        let len = data.len();
        match len {
            0..=255 => {
                self.write_all(&[BIN8_MARKER, len as u8])?;
                self.write_all(data)?;
                Ok(())
            }
            256..=65535 => {
                let len_bytes = (len as u16).to_be_bytes();
                self.write_all(&[BIN16_MARKER, len_bytes[0], len_bytes[1]])?;
                self.write_all(data)?;
                Ok(())
            }
            _ => {
                let len_bytes = (len as u32).to_be_bytes();
                self.write_all(&[
                    BIN32_MARKER,
                    len_bytes[0],
                    len_bytes[1],
                    len_bytes[2],
                    len_bytes[3],
                ])?;
                self.write_all(data)?;
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_timestamp(&mut self, seconds: i64, nanoseconds: u32) -> Result<()> {
        if nanoseconds >= 1_000_000_000 {
            return Err(Error::InvalidTimestamp);
        }

        // timestamp 32: sec in [0, 2^32-1], nsec == 0
        if nanoseconds == 0 && (0..=u32::MAX as i64).contains(&seconds) {
            let sec_bytes = (seconds as u32).to_be_bytes();
            self.write_all(&[
                TIMESTAMP32_MARKER,
                0xff,
                sec_bytes[0],
                sec_bytes[1],
                sec_bytes[2],
                sec_bytes[3],
            ])?;
            return Ok(());
        }

        // timestamp 64: sec in [0, 2^34-1]
        if (0..=(1i64 << 34) - 1).contains(&seconds) {
            let data = ((nanoseconds as u64) << 34) | (seconds as u64);
            let data_bytes = data.to_be_bytes();
            self.write_all(&[
                TIMESTAMP64_MARKER,
                0xff,
                data_bytes[0],
                data_bytes[1],
                data_bytes[2],
                data_bytes[3],
                data_bytes[4],
                data_bytes[5],
                data_bytes[6],
                data_bytes[7],
            ])?;
            return Ok(());
        }

        // timestamp 96
        let sec_bytes = seconds.to_be_bytes();
        let nsec_bytes = nanoseconds.to_be_bytes();
        self.write_all(&[
            TIMESTAMP96_MARKER,
            12,
            0xff,
            nsec_bytes[0],
            nsec_bytes[1],
            nsec_bytes[2],
            nsec_bytes[3],
            sec_bytes[0],
            sec_bytes[1],
            sec_bytes[2],
            sec_bytes[3],
            sec_bytes[4],
            sec_bytes[5],
            sec_bytes[6],
            sec_bytes[7],
        ])?;
        Ok(())
    }

    #[inline(always)]
    fn write_array_len(&mut self, len: usize) -> Result<()> {
        match len {
            0..=15 => {
                self.write_all(&[FIXARRAY_START | (len as u8)])?;
                Ok(())
            }
            16..=65535 => {
                let len_bytes = (len as u16).to_be_bytes();
                self.write_all(&[ARRAY16_MARKER, len_bytes[0], len_bytes[1]])?;
                Ok(())
            }
            _ => {
                let len_bytes = (len as u32).to_be_bytes();
                self.write_all(&[
                    ARRAY32_MARKER,
                    len_bytes[0],
                    len_bytes[1],
                    len_bytes[2],
                    len_bytes[3],
                ])?;
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_map_len(&mut self, len: usize) -> Result<()> {
        match len {
            0..=15 => {
                self.write_all(&[FIXMAP_START | (len as u8)])?;
                Ok(())
            }
            16..=65535 => {
                let len_bytes = (len as u16).to_be_bytes();
                self.write_all(&[MAP16_MARKER, len_bytes[0], len_bytes[1]])?;
                Ok(())
            }
            _ => {
                let len_bytes = (len as u32).to_be_bytes();
                self.write_all(&[
                    MAP32_MARKER,
                    len_bytes[0],
                    len_bytes[1],
                    len_bytes[2],
                    len_bytes[3],
                ])?;
                Ok(())
            }
        }
    }

    #[inline(always)]
    fn write_ext(&mut self, type_id: i8, data: &[u8]) -> Result<()> {
        let len = data.len();
        match len {
            1 => {
                self.write_all(&[FIXEXT1_MARKER, type_id as u8, data[0]])?;
                Ok(())
            }
            2 => {
                self.write_all(&[FIXEXT2_MARKER, type_id as u8, data[0], data[1]])?;
                Ok(())
            }
            4 => {
                self.write_all(&[
                    FIXEXT4_MARKER,
                    type_id as u8,
                    data[0],
                    data[1],
                    data[2],
                    data[3],
                ])?;
                Ok(())
            }
            8 => {
                self.write_all(&[
                    FIXEXT8_MARKER,
                    type_id as u8,
                    data[0],
                    data[1],
                    data[2],
                    data[3],
                    data[4],
                    data[5],
                    data[6],
                    data[7],
                ])?;
                Ok(())
            }
            16 => {
                self.write_all(&[
                    FIXEXT16_MARKER,
                    type_id as u8,
                    data[0],
                    data[1],
                    data[2],
                    data[3],
                    data[4],
                    data[5],
                    data[6],
                    data[7],
                    data[8],
                    data[9],
                    data[10],
                    data[11],
                    data[12],
                    data[13],
                    data[14],
                    data[15],
                ])?;
                Ok(())
            }
            0..=255 => {
                self.write_all(&[EXT8_MARKER, len as u8, type_id as u8])?;
                self.write_all(data)?;
                Ok(())
            }
            256..=65535 => {
                let len_bytes = (len as u16).to_be_bytes();
                self.write_all(&[EXT16_MARKER, len_bytes[0], len_bytes[1], type_id as u8])?;
                self.write_all(data)?;
                Ok(())
            }
            _ => {
                let len_bytes = (len as u32).to_be_bytes();
                self.write_all(&[
                    EXT32_MARKER,
                    len_bytes[0],
                    len_bytes[1],
                    len_bytes[2],
                    len_bytes[3],
                    type_id as u8,
                ])?;
                self.write_all(data)?;
                Ok(())
            }
        }
    }
}
