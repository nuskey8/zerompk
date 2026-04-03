#[cfg(feature = "std")]
use alloc::vec;

use crate::Error;
use crate::FromMessagePack;
use crate::Result;
use crate::consts::*;

/// The maximum allowed depth of nested structures during deserialization.
pub const MAX_DEPTH: usize = 500;

/// A tag read from a MessagePack stream, which can be either an integer or a string.
pub enum Tag<'de> {
    Int(u64),
    String(alloc::borrow::Cow<'de, str>),
}

/// A trait for reading values from a MessagePack-encoded input.
pub trait Read<'de> {
    /// Increments the current depth of nested structures.
    ///
    /// ### Errors
    ///
    /// Returns an error if the maximum depth is exceeded.
    ///
    /// ### Examples
    ///
    /// ```rust
    /// struct Outer {
    ///     inner: Inner,
    /// }
    ///
    /// struct Inner {
    ///     value: i32,
    /// }
    ///
    /// impl<'de> zerompk::FromMessagePack<'de> for Outer {
    ///     fn read<R: zerompk::Read<'de>>(reader: &mut R) -> zerompk::Result<Self> {
    ///         reader.increment_depth()?;
    ///         let inner = Inner::read(reader)?;
    ///         reader.decrement_depth();
    ///         Ok(Self { inner })
    ///     }
    /// }
    ///
    /// impl<'de> zerompk::FromMessagePack<'de> for Inner {
    ///     fn read<R: zerompk::Read<'de>>(reader: &mut R) -> zerompk::Result<Self> {
    ///         reader.increment_depth()?;
    ///         let value = reader.read_i32()?;
    ///         reader.decrement_depth();
    ///         Ok(Self { value })
    ///     }
    /// }
    /// ```
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

    /// Reads an array of values from the input, returning a `Vec<T>`.
    fn read_array<T: FromMessagePack<'de>>(&mut self) -> Result<alloc::vec::Vec<T>>;

    /// Reads a tag from the input, which can be either an integer or a string.
    fn read_tag(&mut self) -> Result<Tag<'de>>;

    /// Validates that the next value in the input is an array of the expected length, and consumes the array header.
    #[inline(always)]
    fn check_array_len(&mut self, expected: usize) -> Result<()> {
        let actual = self.read_array_len()?;
        if actual == expected {
            Ok(())
        } else {
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
            Err(Error::MapLengthMismatch { expected, actual })
        }
    }
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
            Ok(self.data[self.pos])
        } else {
            Err(Error::BufferTooSmall)
        }
    }

    #[inline(always)]
    fn peek_slice(&mut self, len: usize) -> Result<&'de [u8]> {
        if self.pos + len <= self.data.len() {
            unsafe { Ok(self.data.get_unchecked(self.pos..(self.pos + len))) }
        } else {
            Err(Error::BufferTooSmall)
        }
    }

    #[inline(always)]
    fn take_byte(&mut self) -> Result<u8> {
        let byte = self.peek_byte()?;
        self.pos += 1;
        Ok(byte)
    }

    #[inline(always)]
    fn take_slice(&mut self, len: usize) -> Result<&'de [u8]> {
        let slice = self.peek_slice(len)?;
        self.pos += len;
        Ok(slice)
    }

    #[inline(always)]
    fn take_array<const N: usize>(&mut self) -> Result<&'de [u8; N]> {
        let slice = self.peek_slice(N)?;
        self.pos += N;
        Ok(unsafe { &*(slice.as_ptr() as *const [u8; N]) })
    }
}

impl<'de> Read<'de> for SliceReader<'de> {
    #[inline(always)]
    fn increment_depth(&mut self) -> Result<()> {
        if self.depth >= MAX_DEPTH {
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
        }
    }

    #[inline(always)]
    fn read_nil(&mut self) -> Result<()> {
        let byte = self.peek_byte()?;
        if byte == NIL_MARKER {
            self.pos += 1;
            Ok(())
        } else {
            Err(Error::InvalidMarker(byte))
        }
    }

    #[inline(always)]
    fn read_boolean(&mut self) -> Result<bool> {
        let byte = self.peek_byte()?;
        match byte {
            FALSE_MARKER => {
                self.pos += 1;
                Ok(false)
            }
            TRUE_MARKER => {
                self.pos += 1;
                Ok(true)
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_u8(&mut self) -> Result<u8> {
        let byte = self.peek_byte()?;
        match byte {
            // Positive FixInt
            POS_FIXINT_START..=POS_FIXINT_END => {
                self.pos += 1;
                Ok(byte)
            }
            // uint 8
            UINT8_MARKER => {
                self.pos += 1;
                let byte = self.take_byte()?;
                Ok(byte)
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_u16(&mut self) -> Result<u16> {
        let byte = self.peek_byte()?;
        match byte {
            // Positive FixInt
            POS_FIXINT_START..=POS_FIXINT_END => {
                self.pos += 1;
                Ok(byte as u16)
            }
            // uint 8
            UINT8_MARKER => {
                self.pos += 1;
                let byte = self.take_byte()?;
                Ok(byte as u16)
            }
            // uint 16
            UINT16_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<2>()?;
                Ok(u16::from_be_bytes(*bytes))
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_u32(&mut self) -> Result<u32> {
        let byte = self.peek_byte()?;
        match byte {
            // Positive FixInt
            POS_FIXINT_START..=POS_FIXINT_END => {
                self.pos += 1;
                Ok(byte as u32)
            }
            // uint 8
            UINT8_MARKER => {
                self.pos += 1;
                let byte = self.take_byte()?;
                Ok(byte as u32)
            }
            // uint 16
            UINT16_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<2>()?;
                Ok(u16::from_be_bytes(*bytes) as u32)
            }
            // uint 32
            UINT32_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<4>()?;
                Ok(u32::from_be_bytes(*bytes))
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_u64(&mut self) -> Result<u64> {
        let byte = self.peek_byte()?;
        match byte {
            // Positive FixInt
            POS_FIXINT_START..=POS_FIXINT_END => {
                self.pos += 1;
                Ok(byte as u64)
            }
            // uint 8
            UINT8_MARKER => {
                self.pos += 1;
                let byte = self.take_byte()?;
                Ok(byte as u64)
            }
            // uint 16
            UINT16_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<2>()?;
                Ok(u16::from_be_bytes(*bytes) as u64)
            }
            // uint 32
            UINT32_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<4>()?;
                Ok(u32::from_be_bytes(*bytes) as u64)
            }
            // uint 64
            UINT64_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<8>()?;
                Ok(u64::from_be_bytes(*bytes))
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_i8(&mut self) -> Result<i8> {
        let byte = self.peek_byte()?;
        match byte {
            // Positive FixInt
            POS_FIXINT_START..=POS_FIXINT_END => {
                self.pos += 1;
                Ok(byte as i8)
            }
            // Negative FixInt
            NEG_FIXINT_START..=NEG_FIXINT_END => {
                self.pos += 1;
                Ok(byte as i8)
            }
            // int 8
            INT8_MARKER => {
                self.pos += 1;
                let byte = self.take_byte()?;
                Ok(byte as i8)
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_i16(&mut self) -> Result<i16> {
        let byte = self.peek_byte()?;
        match byte {
            // Positive FixInt
            POS_FIXINT_START..=POS_FIXINT_END => {
                self.pos += 1;
                Ok(byte as i16)
            }
            // Negative FixInt
            NEG_FIXINT_START..=NEG_FIXINT_END => {
                self.pos += 1;
                Ok((byte as i8) as i16)
            }
            // int 8
            INT8_MARKER => {
                self.pos += 1;
                let byte = self.take_byte()?;
                Ok(byte as i8 as i16)
            }
            // int 16
            INT16_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<2>()?;
                Ok(i16::from_be_bytes(*bytes))
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_i32(&mut self) -> Result<i32> {
        let byte = self.peek_byte()?;
        match byte {
            // Positive FixInt
            POS_FIXINT_START..=POS_FIXINT_END => {
                self.pos += 1;
                Ok(byte as i32)
            }
            // Negative FixInt
            NEG_FIXINT_START..=NEG_FIXINT_END => {
                self.pos += 1;
                Ok((byte as i8) as i32)
            }
            // int 8
            INT8_MARKER => {
                self.pos += 1;
                let byte = self.take_byte()?;
                Ok(byte as i8 as i32)
            }
            // int 16
            INT16_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<2>()?;
                Ok(i16::from_be_bytes(*bytes) as i32)
            }
            // int 32
            INT32_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<4>()?;
                Ok(i32::from_be_bytes(*bytes))
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_i64(&mut self) -> Result<i64> {
        let byte = self.peek_byte()?;
        match byte {
            // Positive FixInt
            POS_FIXINT_START..=POS_FIXINT_END => {
                self.pos += 1;
                Ok(byte as i64)
            }
            // Negative FixInt
            NEG_FIXINT_START..=NEG_FIXINT_END => {
                self.pos += 1;
                Ok((byte as i8) as i64)
            }
            // int 8
            INT8_MARKER => {
                self.pos += 1;
                let byte = self.take_byte()?;
                Ok(byte as i8 as i64)
            }
            // int 16
            INT16_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<2>()?;
                Ok(i16::from_be_bytes(*bytes) as i64)
            }
            // int 32
            INT32_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<4>()?;
                Ok(i32::from_be_bytes(*bytes) as i64)
            }
            // int 64
            INT64_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<8>()?;
                Ok(i64::from_be_bytes(*bytes))
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_f32(&mut self) -> Result<f32> {
        let byte = self.peek_byte()?;
        match byte {
            // float 32
            0xca => {
                self.pos += 1;
                let bytes = self.take_array::<4>()?;
                Ok(f32::from_bits(u32::from_be_bytes(*bytes)))
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_f64(&mut self) -> Result<f64> {
        let byte = self.peek_byte()?;
        match byte {
            // float 64
            0xcb => {
                self.pos += 1;
                let bytes = self.take_array::<8>()?;
                Ok(f64::from_bits(u64::from_be_bytes(*bytes)))
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_string(&mut self) -> Result<alloc::borrow::Cow<'de, str>> {
        let byte = self.peek_byte()?;
        let len = match byte {
            // fixstr
            FIXSTR_START..=FIXSTR_END => {
                self.pos += 1;
                (byte - FIXSTR_START) as usize
            }
            // str 8
            STR8_MARKER => {
                self.pos += 1;
                let byte = self.take_byte()?;
                byte as usize
            }
            // str 16
            STR16_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<2>()?;
                u16::from_be_bytes(*bytes) as usize
            }
            // str 32
            STR32_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<4>()?;
                u32::from_be_bytes(*bytes) as usize
            }
            _ => return Err(Error::InvalidMarker(byte)),
        };
        let bytes = self.take_slice(len)?;
        match core::str::from_utf8(bytes) {
            Ok(s) => Ok(alloc::borrow::Cow::Borrowed(s)),
            Err(err) => Err(Error::InvalidUtf8(err)),
        }
    }

    #[inline(always)]
    fn read_string_bytes(&mut self) -> Result<alloc::borrow::Cow<'de, [u8]>> {
        let byte = self.peek_byte()?;
        let len = match byte {
            // fixstr
            FIXSTR_START..=FIXSTR_END => {
                self.pos += 1;
                (byte - FIXSTR_START) as usize
            }
            // str 8
            STR8_MARKER => {
                self.pos += 1;
                let byte = self.take_byte()?;
                byte as usize
            }
            // str 16
            STR16_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<2>()?;
                u16::from_be_bytes(*bytes) as usize
            }
            // str 32
            STR32_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<4>()?;
                u32::from_be_bytes(*bytes) as usize
            }
            _ => return Err(Error::InvalidMarker(byte)),
        };
        let bytes = self.take_slice(len)?;
        Ok(alloc::borrow::Cow::Borrowed(bytes))
    }

    #[inline(always)]
    fn read_binary(&mut self) -> Result<alloc::borrow::Cow<'de, [u8]>> {
        let byte = self.peek_byte()?;
        let len = match byte {
            // bin 8
            BIN8_MARKER => {
                self.pos += 1;
                let byte = self.take_byte()?;
                byte as usize
            }
            // bin 16
            BIN16_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<2>()?;
                u16::from_be_bytes(*bytes) as usize
            }
            // bin 32
            BIN32_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<4>()?;
                u32::from_be_bytes(*bytes) as usize
            }
            _ => return Err(Error::InvalidMarker(byte)),
        };
        let bytes = self.take_slice(len)?;
        Ok(alloc::borrow::Cow::Borrowed(bytes))
    }

    #[inline(always)]
    fn read_timestamp(&mut self) -> Result<(i64, u32)> {
        let byte = self.peek_byte()?;
        match byte {
            // fixext 4 with type -1
            TIMESTAMP32_MARKER => {
                self.pos += 1;
                let ext_info = self.take_array::<5>()?;
                let [ext, tail @ ..] = *ext_info;
                if ext as i8 != TIMESTAMP_EXT_TYPE {
                    return Err(Error::InvalidMarker(ext));
                }

                let seconds = u32::from_be_bytes(tail) as i64;
                Ok((seconds, 0))
            }
            // fixext 8 with type -1
            TIMESTAMP64_MARKER => {
                self.pos += 1;
                let ext_info = self.take_array::<9>()?;
                let [ext, tail @ ..] = *ext_info;
                if ext as i8 != TIMESTAMP_EXT_TYPE {
                    return Err(Error::InvalidMarker(ext));
                }

                let data64 = u64::from_be_bytes(tail);
                let nanoseconds = (data64 >> 34) as u32;
                let seconds = (data64 & 0x0000_0003_ffff_ffff) as i64;
                if nanoseconds >= 1_000_000_000 {
                    return Err(Error::InvalidTimestamp);
                }
                Ok((seconds, nanoseconds))
            }
            // ext8(12) with type -1
            TIMESTAMP96_MARKER => {
                self.pos += 1;
                let len = self.take_byte()? as usize;
                if len != 12 {
                    return Err(Error::InvalidMarker(len as u8));
                }

                let ext_info = self.take_array::<13>()?;
                let [ext, tail @ ..] = *ext_info;
                if ext as i8 != TIMESTAMP_EXT_TYPE {
                    return Err(Error::InvalidMarker(ext));
                }

                // Instead of using pointers, use `try_into().unwrap()`.
                // This is faster because it is properly optimized by the compiler.
                let nanoseconds = u32::from_be_bytes(tail[0..4].try_into().unwrap());
                let seconds = i64::from_be_bytes(tail[4..12].try_into().unwrap());
                if nanoseconds >= 1_000_000_000 {
                    return Err(Error::InvalidTimestamp);
                }
                Ok((seconds, nanoseconds))
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_array_len(&mut self) -> Result<usize> {
        let byte = self.peek_byte()?;
        match byte {
            // fixarray
            FIXARRAY_START..=FIXARRAY_END => {
                self.pos += 1;
                Ok((byte - FIXARRAY_START) as usize)
            }
            // array 16
            ARRAY16_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<2>()?;
                Ok(u16::from_be_bytes(*bytes) as usize)
            }
            // array 32
            ARRAY32_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<4>()?;
                Ok(u32::from_be_bytes(*bytes) as usize)
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_map_len(&mut self) -> Result<usize> {
        let byte = self.peek_byte()?;
        match byte {
            // fixmap
            FIXMAP_START..=FIXMAP_END => {
                self.pos += 1;
                Ok((byte - FIXMAP_START) as usize)
            }
            // map 16
            MAP16_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<2>()?;
                Ok(u16::from_be_bytes(*bytes) as usize)
            }
            // map 32
            MAP32_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<4>()?;
                Ok(u32::from_be_bytes(*bytes) as usize)
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_ext_len(&mut self) -> Result<(i8, usize)> {
        let byte = self.peek_byte()?;
        let len = match byte {
            // fixext 1
            FIXEXT1_MARKER => {
                self.pos += 1;
                1
            }
            // fixext 2
            FIXEXT2_MARKER => {
                self.pos += 1;
                2
            }
            // fixext 4
            FIXEXT4_MARKER => {
                self.pos += 1;
                4
            }
            // fixext 8
            FIXEXT8_MARKER => {
                self.pos += 1;
                8
            }
            // fixext 16
            FIXEXT16_MARKER => {
                self.pos += 1;
                16
            }
            // ext 8
            EXT8_MARKER => {
                self.pos += 1;
                let byte = self.take_byte()?;
                byte as usize
            }
            // ext 16
            EXT16_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<2>()?;
                u16::from_be_bytes(*bytes) as usize
            }
            // ext 32
            EXT32_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<4>()?;
                u32::from_be_bytes(*bytes) as usize
            }
            _ => return Err(Error::InvalidMarker(byte)),
        };
        let ext_type = self.take_byte()? as i8;
        Ok((ext_type, len))
    }

    #[inline(always)]
    fn read_array<T: FromMessagePack<'de>>(&mut self) -> Result<alloc::vec::Vec<T>>
    where
        Self: Sized,
    {
        let len = self.read_array_len()?;

        // Protect against OOM
        // Strict checks are performed using T::read.
        // This is intended to prevent pre-allocation of memory,
        // which can be used in attacks that exploit abnormal sizes.
        if self.data.len() - self.pos < len {
            return Err(Error::BufferTooSmall);
        }

        let mut vec = alloc::vec::Vec::with_capacity(len);
        unsafe {
            let mut ptr: *mut T = vec.as_mut_ptr();
            for _ in 0..len {
                let value = T::read(self)?;
                ptr.write(value);
                ptr = ptr.add(1);
            }
            vec.set_len(len);
        }
        Ok(vec)
    }

    #[inline(always)]
    fn read_option<T: FromMessagePack<'de>>(&mut self) -> Result<Option<T>>
    where
        Self: Sized,
    {
        let byte = self.peek_byte()?;
        if byte == NIL_MARKER {
            self.pos += 1;
            Ok(None)
        } else {
            Ok(Some(T::read(self)?))
        }
    }

    #[inline(always)]
    fn read_tag(&mut self) -> Result<Tag<'de>> {
        let byte = self.peek_byte()?;
        match byte {
            POS_FIXINT_START..=POS_FIXINT_END => {
                self.pos += 1;
                Ok(Tag::Int(byte as u64))
            }
            UINT8_MARKER => {
                self.pos += 1;
                let byte = self.take_byte()?;
                Ok(Tag::Int(byte as u64))
            }
            UINT16_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<2>()?;
                Ok(Tag::Int(u16::from_be_bytes(*bytes) as u64))
            }
            UINT32_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<4>()?;
                Ok(Tag::Int(u32::from_be_bytes(*bytes) as u64))
            }
            UINT64_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<8>()?;
                Ok(Tag::Int(u64::from_be_bytes(*bytes)))
            }
            FIXSTR_START..=FIXSTR_END => {
                self.pos += 1;
                let len = (byte - FIXSTR_START) as usize;
                let bytes = self.take_slice(len)?;
                match core::str::from_utf8(bytes) {
                    Ok(s) => Ok(Tag::String(alloc::borrow::Cow::Borrowed(s))),
                    Err(err) => Err(Error::InvalidUtf8(err)),
                }
            }
            STR8_MARKER => {
                self.pos += 1;
                let byte = self.take_byte()?;
                let len = byte as usize;
                let bytes = self.take_slice(len)?;
                match core::str::from_utf8(bytes) {
                    Ok(s) => Ok(Tag::String(alloc::borrow::Cow::Borrowed(s))),
                    Err(err) => Err(Error::InvalidUtf8(err)),
                }
            }
            STR16_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<2>()?;
                let len = u16::from_be_bytes(*bytes) as usize;

                let bytes = self.take_slice(len)?;
                match core::str::from_utf8(bytes) {
                    Ok(s) => Ok(Tag::String(alloc::borrow::Cow::Borrowed(s))),
                    Err(err) => Err(Error::InvalidUtf8(err)),
                }
            }
            STR32_MARKER => {
                self.pos += 1;
                let bytes = self.take_array::<4>()?;
                let len = u32::from_be_bytes(*bytes) as usize;

                let bytes = self.take_slice(len)?;
                match core::str::from_utf8(bytes) {
                    Ok(s) => Ok(Tag::String(alloc::borrow::Cow::Borrowed(s))),
                    Err(err) => Err(Error::InvalidUtf8(err)),
                }
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
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
        self.reader
            .read_exact(buf)
            .map_err(|err| Error::IoError(err))
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
    fn increment_depth(&mut self) -> Result<()> {
        if self.depth >= MAX_DEPTH {
            Err(Error::DepthLimitExceeded { max: MAX_DEPTH })
        } else {
            self.depth += 1;
            Ok(())
        }
    }

    #[inline(always)]
    fn decrement_depth(&mut self) {
        self.depth -= 1;
    }

    #[inline(always)]
    fn read_nil(&mut self) -> Result<()> {
        let byte = self.read_byte()?;
        if byte == NIL_MARKER {
            Ok(())
        } else {
            Err(Error::InvalidMarker(byte))
        }
    }

    #[inline(always)]
    fn read_boolean(&mut self) -> Result<bool> {
        let byte = self.read_byte()?;
        match byte {
            FALSE_MARKER => Ok(false),
            TRUE_MARKER => Ok(true),
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_u8(&mut self) -> Result<u8> {
        let byte = self.read_byte()?;
        match byte {
            POS_FIXINT_START..=POS_FIXINT_END => Ok(byte),
            UINT8_MARKER => {
                let value = self.read_byte()?;
                Ok(value)
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_u16(&mut self) -> Result<u16> {
        let byte = self.read_byte()?;
        match byte {
            POS_FIXINT_START..=POS_FIXINT_END => Ok(byte as u16),
            UINT8_MARKER => {
                let value = self.read_byte()?;
                Ok(value as u16)
            }
            UINT16_MARKER => {
                let mut buf = [0u8; 2];
                self.read_exact(&mut buf)?;
                Ok(u16::from_be_bytes(buf))
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_u32(&mut self) -> Result<u32> {
        let byte = self.read_byte()?;
        match byte {
            POS_FIXINT_START..=POS_FIXINT_END => Ok(byte as u32),
            UINT8_MARKER => {
                let value = self.read_byte()?;
                Ok(value as u32)
            }
            UINT16_MARKER => {
                let mut buf = [0u8; 2];
                self.read_exact(&mut buf)?;
                Ok(u16::from_be_bytes(buf) as u32)
            }
            UINT32_MARKER => {
                let mut buf = [0u8; 4];
                self.read_exact(&mut buf)?;
                Ok(u32::from_be_bytes(buf))
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_u64(&mut self) -> Result<u64> {
        let byte = self.read_byte()?;
        match byte {
            POS_FIXINT_START..=POS_FIXINT_END => Ok(byte as u64),
            UINT8_MARKER => {
                let value = self.read_byte()?;
                Ok(value as u64)
            }
            UINT16_MARKER => {
                let mut buf = [0u8; 2];
                self.read_exact(&mut buf)?;
                Ok(u16::from_be_bytes(buf) as u64)
            }
            UINT32_MARKER => {
                let mut buf = [0u8; 4];
                self.read_exact(&mut buf)?;
                Ok(u32::from_be_bytes(buf) as u64)
            }
            UINT64_MARKER => {
                let mut buf = [0u8; 8];
                self.read_exact(&mut buf)?;
                Ok(u64::from_be_bytes(buf))
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_i8(&mut self) -> Result<i8> {
        let byte = self.read_byte()?;
        match byte {
            POS_FIXINT_START..=POS_FIXINT_END => Ok(byte as i8),
            NEG_FIXINT_START..=NEG_FIXINT_END => Ok(byte as i8),
            INT8_MARKER => {
                let value = self.read_byte()?;
                Ok(value as i8)
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_i16(&mut self) -> Result<i16> {
        let byte = self.read_byte()?;
        match byte {
            POS_FIXINT_START..=POS_FIXINT_END => Ok(byte as i16),
            NEG_FIXINT_START..=NEG_FIXINT_END => Ok((byte as i8) as i16),
            INT8_MARKER => {
                let value = self.read_byte()?;
                Ok((value as i8) as i16)
            }
            INT16_MARKER => {
                let mut buf = [0u8; 2];
                self.read_exact(&mut buf)?;
                Ok(i16::from_be_bytes(buf))
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_i32(&mut self) -> Result<i32> {
        let byte = self.read_byte()?;
        match byte {
            POS_FIXINT_START..=POS_FIXINT_END => Ok(byte as i32),
            NEG_FIXINT_START..=NEG_FIXINT_END => Ok((byte as i8) as i32),
            INT8_MARKER => {
                let value = self.read_byte()?;
                Ok((value as i8) as i32)
            }
            INT16_MARKER => {
                let mut buf = [0u8; 2];
                self.read_exact(&mut buf)?;
                Ok(i16::from_be_bytes(buf) as i32)
            }
            INT32_MARKER => {
                let mut buf = [0u8; 4];
                self.read_exact(&mut buf)?;
                Ok(i32::from_be_bytes(buf))
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_i64(&mut self) -> Result<i64> {
        let byte = self.read_byte()?;
        match byte {
            POS_FIXINT_START..=POS_FIXINT_END => Ok(byte as i64),
            NEG_FIXINT_START..=NEG_FIXINT_END => Ok((byte as i8) as i64),
            INT8_MARKER => {
                let value = self.read_byte()?;
                Ok((value as i8) as i64)
            }
            INT16_MARKER => {
                let mut buf = [0u8; 2];
                self.read_exact(&mut buf)?;
                Ok(i16::from_be_bytes(buf) as i64)
            }
            INT32_MARKER => {
                let mut buf = [0u8; 4];
                self.read_exact(&mut buf)?;
                Ok(i32::from_be_bytes(buf) as i64)
            }
            INT64_MARKER => {
                let mut buf = [0u8; 8];
                self.read_exact(&mut buf)?;
                Ok(i64::from_be_bytes(buf))
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_f32(&mut self) -> Result<f32> {
        let byte = self.read_byte()?;
        match byte {
            FLOAT32_MARKER => {
                let mut buf = [0u8; 4];
                self.read_exact(&mut buf)?;
                Ok(f32::from_bits(u32::from_be_bytes(buf)))
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_f64(&mut self) -> Result<f64> {
        let byte = self.read_byte()?;
        match byte {
            FLOAT64_MARKER => {
                let mut buf = [0u8; 8];
                self.read_exact(&mut buf)?;
                Ok(f64::from_bits(u64::from_be_bytes(buf)))
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_string(&mut self) -> Result<alloc::borrow::Cow<'de, str>> {
        let mut buf = [0u8; 1];
        let byte = self.read_byte()?;
        let len = match byte {
            FIXSTR_START..=FIXSTR_END => (byte - FIXSTR_START) as usize,
            STR8_MARKER => {
                self.read_exact(&mut buf)?;
                buf[0] as usize
            }
            STR16_MARKER => {
                let mut buf = [0u8; 2];
                self.read_exact(&mut buf)?;
                u16::from_be_bytes(buf) as usize
            }
            STR32_MARKER => {
                let mut buf = [0u8; 4];
                self.read_exact(&mut buf)?;
                u32::from_be_bytes(buf) as usize
            }
            _ => return Err(Error::InvalidMarker(byte)),
        };

        let str_buf = self.read_exact_vec(len)?;

        match alloc::string::String::from_utf8(str_buf) {
            Ok(s) => Ok(alloc::borrow::Cow::Owned(s)),
            Err(err) => Err(Error::InvalidUtf8(err.utf8_error())),
        }
    }

    #[inline(always)]
    fn read_string_bytes(&mut self) -> Result<alloc::borrow::Cow<'de, [u8]>> {
        let mut buf = [0u8; 1];
        let byte = self.read_byte()?;
        let len = match byte {
            FIXSTR_START..=FIXSTR_END => (byte - FIXSTR_START) as usize,
            STR8_MARKER => {
                self.read_exact(&mut buf)?;
                buf[0] as usize
            }
            STR16_MARKER => {
                let mut buf = [0u8; 2];
                self.read_exact(&mut buf)?;
                u16::from_be_bytes(buf) as usize
            }
            STR32_MARKER => {
                let mut buf = [0u8; 4];
                self.read_exact(&mut buf)?;
                u32::from_be_bytes(buf) as usize
            }
            _ => return Err(Error::InvalidMarker(byte)),
        };

        let str_buf = self.read_exact_vec(len)?;
        Ok(alloc::borrow::Cow::Owned(str_buf))
    }

    #[inline(always)]
    fn read_binary(&mut self) -> Result<alloc::borrow::Cow<'de, [u8]>> {
        let mut buf = [0u8; 1];
        let byte = self.read_byte()?;
        let len = match byte {
            BIN8_MARKER => {
                self.read_exact(&mut buf)?;
                buf[0] as usize
            }
            BIN16_MARKER => {
                let mut buf = [0u8; 2];
                self.read_exact(&mut buf)?;
                u16::from_be_bytes(buf) as usize
            }
            BIN32_MARKER => {
                let mut buf = [0u8; 4];
                self.read_exact(&mut buf)?;
                u32::from_be_bytes(buf) as usize
            }
            _ => return Err(Error::InvalidMarker(byte)),
        };

        let data_buf = self.read_exact_vec(len)?;
        Ok(alloc::borrow::Cow::Owned(data_buf))
    }

    #[inline(always)]
    fn read_timestamp(&mut self) -> Result<(i64, u32)> {
        let byte = self.read_byte()?;
        match byte {
            TIMESTAMP32_MARKER => {
                let mut ext_info = [0u8; 5];
                self.read_exact(&mut ext_info)?;

                let [ext, tail @ ..] = ext_info;
                if ext != TIMESTAMP_EXT_TYPE as u8 {
                    return Err(Error::InvalidMarker(ext));
                }
                let seconds = u32::from_be_bytes(tail) as i64;
                Ok((seconds, 0))
            }
            TIMESTAMP64_MARKER => {
                let mut ext_info = [0u8; 9];
                self.read_exact(&mut ext_info)?;

                let [ext, tail @ ..] = ext_info;
                if ext != -1i8 as u8 {
                    return Err(Error::InvalidMarker(ext));
                }

                let data64 = u64::from_be_bytes(tail);
                let nanoseconds = (data64 >> 34) as u32;
                let seconds = (data64 & 0x0000_0003_ffff_ffff) as i64;
                if nanoseconds >= 1_000_000_000 {
                    return Err(Error::InvalidTimestamp);
                }
                Ok((seconds, nanoseconds))
            }
            TIMESTAMP96_MARKER => {
                let len = self.read_byte()? as usize;
                if len != 12 {
                    return Err(Error::InvalidMarker(len as u8));
                }

                let mut ext_info = [0u8; 13];
                self.read_exact(&mut ext_info)?;
                let [ext, tail @ ..] = ext_info;
                if ext != TIMESTAMP_EXT_TYPE as u8 {
                    return Err(Error::InvalidMarker(ext));
                }

                // Instead of using pointers, use `try_into().unwrap()`.
                // This is faster because it is properly optimized by the compiler.
                let nanoseconds = u32::from_be_bytes(tail[0..4].try_into().unwrap());
                let seconds = i64::from_be_bytes(tail[4..12].try_into().unwrap());
                if nanoseconds >= 1_000_000_000 {
                    return Err(Error::InvalidTimestamp);
                }

                Ok((seconds, nanoseconds))
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_array_len(&mut self) -> Result<usize> {
        let byte = self.read_byte()?;
        match byte {
            FIXARRAY_START..=FIXARRAY_END => Ok((byte - FIXARRAY_START) as usize),
            ARRAY16_MARKER => {
                let mut buf = [0u8; 2];
                self.read_exact(&mut buf)?;
                Ok(u16::from_be_bytes(buf) as usize)
            }
            ARRAY32_MARKER => {
                let mut buf = [0u8; 4];
                self.read_exact(&mut buf)?;
                Ok(u32::from_be_bytes(buf) as usize)
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_map_len(&mut self) -> Result<usize> {
        let byte = self.read_byte()?;
        match byte {
            FIXMAP_START..=FIXMAP_END => Ok((byte - FIXMAP_START) as usize),
            MAP16_MARKER => {
                let mut buf = [0u8; 2];
                self.read_exact(&mut buf)?;
                Ok(u16::from_be_bytes(buf) as usize)
            }
            MAP32_MARKER => {
                let mut buf = [0u8; 4];
                self.read_exact(&mut buf)?;
                Ok(u32::from_be_bytes(buf) as usize)
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }

    #[inline(always)]
    fn read_ext_len(&mut self) -> Result<(i8, usize)> {
        let mut buf = [0u8; 1];
        let byte = self.read_byte()?;
        let len = match byte {
            FIXEXT1_MARKER => 1,
            FIXEXT2_MARKER => 2,
            FIXEXT4_MARKER => 4,
            FIXEXT8_MARKER => 8,
            FIXEXT16_MARKER => 16,
            EXT8_MARKER => {
                self.read_exact(&mut buf)?;
                buf[0] as usize
            }
            EXT16_MARKER => {
                let mut buf = [0u8; 2];
                self.read_exact(&mut buf)?;
                u16::from_be_bytes(buf) as usize
            }
            EXT32_MARKER => {
                let mut buf = [0u8; 4];
                self.read_exact(&mut buf)?;
                u32::from_be_bytes(buf) as usize
            }
            _ => return Err(Error::InvalidMarker(byte)),
        };
        self.read_exact(&mut buf)?;
        let ext_type = buf[0] as i8;
        Ok((ext_type, len))
    }

    #[inline(always)]
    fn read_option<T: FromMessagePack<'de>>(&mut self) -> Result<Option<T>> {
        let byte = self.read_byte()?;
        if byte == NIL_MARKER {
            Ok(None)
        } else {
            self.unread_byte(byte);
            Ok(Some(T::read(self)?))
        }
    }

    #[inline(always)]
    fn read_array<T: FromMessagePack<'de>>(&mut self) -> Result<alloc::vec::Vec<T>> {
        let len = self.read_array_len()?;
        let mut vec = alloc::vec::Vec::new();
        for _ in 0..len {
            vec.push(T::read(self)?);
        }
        Ok(vec)
    }

    fn read_tag(&mut self) -> Result<Tag<'de>> {
        let mut buf = [0u8; 1];
        let byte = self.read_byte()?;
        match byte {
            POS_FIXINT_START..=POS_FIXINT_END => Ok(Tag::Int(byte as u64)),
            UINT8_MARKER => {
                let value = self.read_byte()?;
                Ok(Tag::Int(value as u64))
            }
            UINT16_MARKER => {
                let mut buf = [0u8; 2];
                self.read_exact(&mut buf)?;
                Ok(Tag::Int(u16::from_be_bytes(buf) as u64))
            }
            UINT32_MARKER => {
                let mut buf = [0u8; 4];
                self.read_exact(&mut buf)?;
                Ok(Tag::Int(u32::from_be_bytes(buf) as u64))
            }
            UINT64_MARKER => {
                let mut buf = [0u8; 8];
                self.read_exact(&mut buf)?;
                Ok(Tag::Int(u64::from_be_bytes(buf)))
            }
            FIXSTR_START..=FIXSTR_END => {
                let len = (byte - FIXSTR_START) as usize;
                let str_buf = self.read_exact_vec(len)?;
                match alloc::string::String::from_utf8(str_buf) {
                    Ok(s) => Ok(Tag::String(s.into())),
                    Err(err) => Err(Error::InvalidUtf8(err.utf8_error())),
                }
            }
            STR8_MARKER => {
                self.read_exact(&mut buf)?;
                let len = buf[0] as usize;
                let str_buf = self.read_exact_vec(len)?;
                match alloc::string::String::from_utf8(str_buf) {
                    Ok(s) => Ok(Tag::String(s.into())),
                    Err(err) => Err(Error::InvalidUtf8(err.utf8_error())),
                }
            }
            STR16_MARKER => {
                let mut buf = [0u8; 2];
                self.read_exact(&mut buf)?;
                let len = u16::from_be_bytes(buf) as usize;
                let str_buf = self.read_exact_vec(len)?;
                match alloc::string::String::from_utf8(str_buf) {
                    Ok(s) => Ok(Tag::String(s.into())),
                    Err(err) => Err(Error::InvalidUtf8(err.utf8_error())),
                }
            }
            STR32_MARKER => {
                let mut buf = [0u8; 4];
                self.read_exact(&mut buf)?;
                let len = u32::from_be_bytes(buf) as usize;
                let str_buf = self.read_exact_vec(len)?;
                match alloc::string::String::from_utf8(str_buf) {
                    Ok(s) => Ok(Tag::String(s.into())),
                    Err(err) => Err(Error::InvalidUtf8(err.utf8_error())),
                }
            }
            _ => Err(Error::InvalidMarker(byte)),
        }
    }
}
