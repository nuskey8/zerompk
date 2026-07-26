use alloc::borrow::Cow;
use alloc::vec::Vec;

use crate::consts::*;
use crate::read::MAX_DEPTH;
use crate::{Error, FromMessagePack, Read, Result, Tag};

pub(crate) struct BufReadReader<'a, R: std::io::BufRead> {
    reader: &'a mut R,
    depth: usize,
    ptr: *const u8,
    len: usize,
    pos: usize,
}

impl<'a, R: std::io::BufRead> BufReadReader<'a, R> {
    pub(crate) fn new(reader: &'a mut R) -> Self {
        Self {
            reader,
            depth: 0,
            ptr: core::ptr::null(),
            len: 0,
            pos: 0,
        }
    }

    #[cold]
    #[inline(never)]
    fn refill(&mut self) -> Result<()> {
        if self.pos > 0 {
            self.reader.consume(self.pos);
        }
        let buffer = self.reader.fill_buf().map_err(Error::IoError)?;
        self.ptr = buffer.as_ptr();
        self.len = buffer.len();
        self.pos = 0;
        if self.len == 0 {
            Err(Error::BufferTooSmall)
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    fn window(&mut self) -> Result<&[u8]> {
        if self.pos == self.len {
            self.refill()?;
        }
        // SAFETY: `ptr` and `len` come from `BufRead::fill_buf`. The reader is
        // exclusively borrowed for this adapter's lifetime, and we do not call
        // into it again until the current window has been consumed.
        Ok(unsafe { core::slice::from_raw_parts(self.ptr.add(self.pos), self.len - self.pos) })
    }

    #[inline(always)]
    fn advance(&mut self, count: usize) {
        debug_assert!(count <= self.len - self.pos);
        self.pos += count;
    }

    #[inline(always)]
    fn finish<T>(&mut self, value: T, consumed: usize) -> Result<T> {
        self.advance(consumed);
        Ok(value)
    }

    #[inline(always)]
    fn take<const N: usize>(&mut self) -> Result<[u8; N]> {
        let mut out = [0; N];
        let mut written = 0;
        while written < N {
            let buffer = self.window()?;
            let count = core::cmp::min(N - written, buffer.len());
            out[written..written + count].copy_from_slice(&buffer[..count]);
            self.advance(count);
            written += count;
        }
        Ok(out)
    }

    #[inline(always)]
    fn take_vec(&mut self, len: usize) -> Result<Vec<u8>> {
        const CHUNK_SIZE: usize = 8192;
        let mut out = Vec::with_capacity(len.min(CHUNK_SIZE));
        while out.len() < len {
            let buffer = self.window()?;
            let count = core::cmp::min(len - out.len(), buffer.len());
            out.extend_from_slice(&buffer[..count]);
            self.advance(count);
        }
        Ok(out)
    }

    #[inline(always)]
    fn discard(&mut self, mut len: usize) -> Result<()> {
        while len > 0 {
            let buffer = self.window()?;
            let count = core::cmp::min(len, buffer.len());
            self.advance(count);
            len -= count;
        }
        Ok(())
    }

    #[inline(always)]
    fn read_string_len(&mut self) -> Result<usize> {
        match self.peek_marker()? {
            marker @ FIXSTR_START..=FIXSTR_END => {
                self.take::<1>()?;
                Ok((marker - FIXSTR_START) as usize)
            }
            STR8_MARKER => Ok(self.take::<2>()?[1] as usize),
            STR16_MARKER => {
                let b = self.take::<3>()?;
                Ok(u16::from_be_bytes([b[1], b[2]]) as usize)
            }
            STR32_MARKER => {
                let b = self.take::<5>()?;
                Ok(u32::from_be_bytes([b[1], b[2], b[3], b[4]]) as usize)
            }
            marker => Err(Error::InvalidMarker(marker)),
        }
    }

    #[inline(always)]
    fn read_binary_len(&mut self) -> Result<usize> {
        match self.peek_marker()? {
            BIN8_MARKER => Ok(self.take::<2>()?[1] as usize),
            BIN16_MARKER => {
                let b = self.take::<3>()?;
                Ok(u16::from_be_bytes([b[1], b[2]]) as usize)
            }
            BIN32_MARKER => {
                let b = self.take::<5>()?;
                Ok(u32::from_be_bytes([b[1], b[2], b[3], b[4]]) as usize)
            }
            marker => Err(Error::InvalidMarker(marker)),
        }
    }
}

impl<R: std::io::BufRead> Drop for BufReadReader<'_, R> {
    fn drop(&mut self) {
        if self.pos > 0 {
            self.reader.consume(self.pos);
        }
    }
}

impl<'de, R: std::io::BufRead> Read<'de> for BufReadReader<'_, R> {
    #[inline(always)]
    fn peek_marker(&mut self) -> Result<u8> {
        self.window()?.first().copied().ok_or(Error::BufferTooSmall)
    }

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
        let marker = self.peek_marker()?;
        if marker != NIL_MARKER {
            return Err(Error::InvalidMarker(marker));
        }
        self.take::<1>()?;
        Ok(())
    }

    #[inline(always)]
    fn read_boolean(&mut self) -> Result<bool> {
        match self.peek_marker()? {
            FALSE_MARKER => {
                self.take::<1>()?;
                Ok(false)
            }
            TRUE_MARKER => {
                self.take::<1>()?;
                Ok(true)
            }
            marker => Err(Error::InvalidMarker(marker)),
        }
    }

    #[inline(always)]
    fn read_u8(&mut self) -> Result<u8> {
        match self.peek_marker()? {
            marker @ POS_FIXINT_START..=POS_FIXINT_END => {
                self.take::<1>()?;
                Ok(marker)
            }
            UINT8_MARKER => Ok(self.take::<2>()?[1]),
            marker => Err(Error::InvalidMarker(marker)),
        }
    }

    #[inline(always)]
    fn read_u16(&mut self) -> Result<u16> {
        match self.peek_marker()? {
            marker @ POS_FIXINT_START..=POS_FIXINT_END => {
                self.take::<1>()?;
                Ok(marker as u16)
            }
            UINT8_MARKER => Ok(self.take::<2>()?[1] as u16),
            UINT16_MARKER => {
                let bytes = self.take::<3>()?;
                Ok(u16::from_be_bytes([bytes[1], bytes[2]]))
            }
            marker => Err(Error::InvalidMarker(marker)),
        }
    }

    #[inline(always)]
    fn read_u32(&mut self) -> Result<u32> {
        match self.peek_marker()? {
            marker @ POS_FIXINT_START..=POS_FIXINT_END => {
                self.take::<1>()?;
                Ok(marker as u32)
            }
            UINT8_MARKER => Ok(self.take::<2>()?[1] as u32),
            UINT16_MARKER => {
                let b = self.take::<3>()?;
                Ok(u16::from_be_bytes([b[1], b[2]]) as u32)
            }
            UINT32_MARKER => {
                let b = self.take::<5>()?;
                Ok(u32::from_be_bytes([b[1], b[2], b[3], b[4]]))
            }
            marker => Err(Error::InvalidMarker(marker)),
        }
    }

    #[inline(always)]
    fn read_u64(&mut self) -> Result<u64> {
        match self.peek_marker()? {
            marker @ POS_FIXINT_START..=POS_FIXINT_END => {
                self.take::<1>()?;
                Ok(marker as u64)
            }
            UINT8_MARKER => Ok(self.take::<2>()?[1] as u64),
            UINT16_MARKER => {
                let b = self.take::<3>()?;
                Ok(u16::from_be_bytes([b[1], b[2]]) as u64)
            }
            UINT32_MARKER => {
                let b = self.take::<5>()?;
                Ok(u32::from_be_bytes([b[1], b[2], b[3], b[4]]) as u64)
            }
            UINT64_MARKER => {
                let b = self.take::<9>()?;
                Ok(u64::from_be_bytes(b[1..].try_into().unwrap()))
            }
            marker => Err(Error::InvalidMarker(marker)),
        }
    }

    #[inline(always)]
    fn read_i8(&mut self) -> Result<i8> {
        match self.peek_marker()? {
            marker @ POS_FIXINT_START..=POS_FIXINT_END
            | marker @ NEG_FIXINT_START..=NEG_FIXINT_END => {
                self.take::<1>()?;
                Ok(marker as i8)
            }
            INT8_MARKER => Ok(self.take::<2>()?[1] as i8),
            marker => Err(Error::InvalidMarker(marker)),
        }
    }

    #[inline(always)]
    fn read_i16(&mut self) -> Result<i16> {
        match self.peek_marker()? {
            marker @ POS_FIXINT_START..=POS_FIXINT_END => {
                self.take::<1>()?;
                Ok(marker as i16)
            }
            marker @ NEG_FIXINT_START..=NEG_FIXINT_END => {
                self.take::<1>()?;
                Ok((marker as i8) as i16)
            }
            INT8_MARKER => Ok((self.take::<2>()?[1] as i8) as i16),
            INT16_MARKER => {
                let b = self.take::<3>()?;
                Ok(i16::from_be_bytes([b[1], b[2]]))
            }
            marker => Err(Error::InvalidMarker(marker)),
        }
    }

    #[inline(always)]
    fn read_i32(&mut self) -> Result<i32> {
        if self.pos == self.len {
            self.refill()?;
        }
        let remaining = self.len - self.pos;
        // SAFETY: refill guarantees at least one available byte. Wider reads
        // are guarded by `remaining` and use unaligned loads.
        let ptr = unsafe { self.ptr.add(self.pos) };
        let marker = unsafe { *ptr };
        match marker {
            POS_FIXINT_START..=POS_FIXINT_END => {
                self.advance(1);
                return Ok(marker as i32);
            }
            NEG_FIXINT_START..=NEG_FIXINT_END => {
                self.advance(1);
                return Ok((marker as i8) as i32);
            }
            INT8_MARKER if remaining >= 2 => {
                let value = unsafe { *ptr.add(1) } as i8 as i32;
                self.advance(2);
                return Ok(value);
            }
            INT16_MARKER if remaining >= 3 => {
                let value = i16::from_be(unsafe { (ptr.add(1) as *const i16).read_unaligned() });
                self.advance(3);
                return Ok(value as i32);
            }
            INT32_MARKER if remaining >= 5 => {
                let value = i32::from_be(unsafe { (ptr.add(1) as *const i32).read_unaligned() });
                self.advance(5);
                return Ok(value);
            }
            INT8_MARKER | INT16_MARKER | INT32_MARKER => {}
            _ => return Err(Error::InvalidMarker(marker)),
        }
        match marker {
            INT8_MARKER => Ok((self.take::<2>()?[1] as i8) as i32),
            INT16_MARKER => {
                let b = self.take::<3>()?;
                Ok(i16::from_be_bytes([b[1], b[2]]) as i32)
            }
            INT32_MARKER => {
                let b = self.take::<5>()?;
                Ok(i32::from_be_bytes([b[1], b[2], b[3], b[4]]))
            }
            marker => Err(Error::InvalidMarker(marker)),
        }
    }

    #[inline(always)]
    fn read_i64(&mut self) -> Result<i64> {
        match self.peek_marker()? {
            marker @ POS_FIXINT_START..=POS_FIXINT_END => {
                self.take::<1>()?;
                Ok(marker as i64)
            }
            marker @ NEG_FIXINT_START..=NEG_FIXINT_END => {
                self.take::<1>()?;
                Ok((marker as i8) as i64)
            }
            INT8_MARKER => Ok((self.take::<2>()?[1] as i8) as i64),
            INT16_MARKER => {
                let b = self.take::<3>()?;
                Ok(i16::from_be_bytes([b[1], b[2]]) as i64)
            }
            INT32_MARKER => {
                let b = self.take::<5>()?;
                Ok(i32::from_be_bytes([b[1], b[2], b[3], b[4]]) as i64)
            }
            INT64_MARKER => {
                let b = self.take::<9>()?;
                Ok(i64::from_be_bytes(b[1..].try_into().unwrap()))
            }
            marker => Err(Error::InvalidMarker(marker)),
        }
    }

    #[inline(always)]
    fn read_f32(&mut self) -> Result<f32> {
        let marker = self.peek_marker()?;
        if marker != FLOAT32_MARKER {
            return Err(Error::InvalidMarker(marker));
        }
        let b = self.take::<5>()?;
        Ok(f32::from_bits(u32::from_be_bytes([b[1], b[2], b[3], b[4]])))
    }

    #[inline(always)]
    fn read_f64(&mut self) -> Result<f64> {
        let marker = self.peek_marker()?;
        if marker != FLOAT64_MARKER {
            return Err(Error::InvalidMarker(marker));
        }
        let b = self.take::<9>()?;
        Ok(f64::from_bits(u64::from_be_bytes(
            b[1..].try_into().unwrap(),
        )))
    }

    #[inline(always)]
    fn read_timestamp(&mut self) -> Result<(i64, u32)> {
        match self.peek_marker()? {
            TIMESTAMP32_MARKER => {
                let b = self.take::<6>()?;
                if b[1] as i8 != TIMESTAMP_EXT_TYPE {
                    return Err(Error::InvalidMarker(b[1]));
                }
                Ok((u32::from_be_bytes(b[2..6].try_into().unwrap()) as i64, 0))
            }
            TIMESTAMP64_MARKER => {
                let b = self.take::<10>()?;
                if b[1] as i8 != TIMESTAMP_EXT_TYPE {
                    return Err(Error::InvalidMarker(b[1]));
                }
                let data = u64::from_be_bytes(b[2..10].try_into().unwrap());
                let nanoseconds = (data >> 34) as u32;
                if nanoseconds >= 1_000_000_000 {
                    return Err(Error::InvalidTimestamp);
                }
                Ok(((data & 0x0000_0003_ffff_ffff) as i64, nanoseconds))
            }
            TIMESTAMP96_MARKER => {
                let b = self.take::<15>()?;
                if b[1] != 12 || b[2] as i8 != TIMESTAMP_EXT_TYPE {
                    return Err(Error::InvalidMarker(b[1]));
                }
                let nanoseconds = u32::from_be_bytes(b[3..7].try_into().unwrap());
                if nanoseconds >= 1_000_000_000 {
                    return Err(Error::InvalidTimestamp);
                }
                Ok((
                    i64::from_be_bytes(b[7..15].try_into().unwrap()),
                    nanoseconds,
                ))
            }
            marker => Err(Error::InvalidMarker(marker)),
        }
    }

    #[inline(always)]
    fn read_array_len(&mut self) -> Result<usize> {
        let parsed = {
            let buffer = self.window()?;
            let Some(&marker) = buffer.first() else {
                return Err(Error::BufferTooSmall);
            };
            match marker {
                FIXARRAY_START..=FIXARRAY_END => Some(((marker - FIXARRAY_START) as usize, 1)),
                ARRAY16_MARKER if buffer.len() >= 3 => {
                    Some((u16::from_be_bytes([buffer[1], buffer[2]]) as usize, 3))
                }
                ARRAY32_MARKER if buffer.len() >= 5 => Some((
                    u32::from_be_bytes([buffer[1], buffer[2], buffer[3], buffer[4]]) as usize,
                    5,
                )),
                ARRAY16_MARKER | ARRAY32_MARKER => None,
                _ => return Err(Error::InvalidMarker(marker)),
            }
        };
        if let Some((value, consumed)) = parsed {
            return self.finish(value, consumed);
        }
        match self.peek_marker()? {
            ARRAY16_MARKER => {
                let b = self.take::<3>()?;
                Ok(u16::from_be_bytes([b[1], b[2]]) as usize)
            }
            ARRAY32_MARKER => {
                let b = self.take::<5>()?;
                Ok(u32::from_be_bytes([b[1], b[2], b[3], b[4]]) as usize)
            }
            marker => Err(Error::InvalidMarker(marker)),
        }
    }

    #[inline(always)]
    fn read_map_len(&mut self) -> Result<usize> {
        let parsed = {
            let buffer = self.window()?;
            let Some(&marker) = buffer.first() else {
                return Err(Error::BufferTooSmall);
            };
            match marker {
                FIXMAP_START..=FIXMAP_END => Some(((marker - FIXMAP_START) as usize, 1)),
                MAP16_MARKER if buffer.len() >= 3 => {
                    Some((u16::from_be_bytes([buffer[1], buffer[2]]) as usize, 3))
                }
                MAP32_MARKER if buffer.len() >= 5 => Some((
                    u32::from_be_bytes([buffer[1], buffer[2], buffer[3], buffer[4]]) as usize,
                    5,
                )),
                MAP16_MARKER | MAP32_MARKER => None,
                _ => return Err(Error::InvalidMarker(marker)),
            }
        };
        if let Some((value, consumed)) = parsed {
            return self.finish(value, consumed);
        }
        match self.peek_marker()? {
            MAP16_MARKER => {
                let b = self.take::<3>()?;
                Ok(u16::from_be_bytes([b[1], b[2]]) as usize)
            }
            MAP32_MARKER => {
                let b = self.take::<5>()?;
                Ok(u32::from_be_bytes([b[1], b[2], b[3], b[4]]) as usize)
            }
            marker => Err(Error::InvalidMarker(marker)),
        }
    }

    #[inline(always)]
    fn read_ext_len(&mut self) -> Result<(i8, usize)> {
        let len = match self.peek_marker()? {
            FIXEXT1_MARKER => {
                let b = self.take::<2>()?;
                return Ok((b[1] as i8, 1));
            }
            FIXEXT2_MARKER => {
                let b = self.take::<2>()?;
                return Ok((b[1] as i8, 2));
            }
            FIXEXT4_MARKER => {
                let b = self.take::<2>()?;
                return Ok((b[1] as i8, 4));
            }
            FIXEXT8_MARKER => {
                let b = self.take::<2>()?;
                return Ok((b[1] as i8, 8));
            }
            FIXEXT16_MARKER => {
                let b = self.take::<2>()?;
                return Ok((b[1] as i8, 16));
            }
            EXT8_MARKER => self.take::<2>()?[1] as usize,
            EXT16_MARKER => {
                let b = self.take::<3>()?;
                u16::from_be_bytes([b[1], b[2]]) as usize
            }
            EXT32_MARKER => {
                let b = self.take::<5>()?;
                u32::from_be_bytes([b[1], b[2], b[3], b[4]]) as usize
            }
            marker => return Err(Error::InvalidMarker(marker)),
        };
        Ok((self.take::<1>()?[0] as i8, len))
    }

    #[inline(always)]
    fn read_ext(&mut self) -> Result<(i8, Cow<'de, [u8]>)> {
        let (type_id, len) = self.read_ext_len()?;
        Ok((type_id, Cow::Owned(self.take_vec(len)?)))
    }

    #[inline(always)]
    fn read_string(&mut self) -> Result<Cow<'de, str>> {
        let len = self.read_string_len()?;
        let bytes = self.take_vec(len)?;
        let string = alloc::string::String::from_utf8(bytes)
            .map_err(|error| Error::InvalidUtf8(error.utf8_error()))?;
        Ok(Cow::Owned(string))
    }

    #[inline(always)]
    fn read_string_bytes(&mut self) -> Result<Cow<'de, [u8]>> {
        let len = self.read_string_len()?;
        Ok(Cow::Owned(self.take_vec(len)?))
    }

    #[inline(always)]
    fn read_binary(&mut self) -> Result<Cow<'de, [u8]>> {
        let len = self.read_binary_len()?;
        Ok(Cow::Owned(self.take_vec(len)?))
    }

    #[inline(always)]
    fn read_option<T: FromMessagePack<'de>>(&mut self) -> Result<Option<T>> {
        if self.peek_marker()? == crate::consts::NIL_MARKER {
            self.read_nil()?;
            Ok(None)
        } else {
            T::read(self).map(Some)
        }
    }

    #[inline(always)]
    fn read_array<T: FromMessagePack<'de>>(&mut self, out: &mut Vec<T>) -> Result<()>
    where
        Self: Sized,
    {
        out.clear();
        let len = self.read_array_len()?;
        out.reserve(len.min(32));
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

    #[inline(always)]
    fn read_tag(&mut self) -> Result<Tag<'de>> {
        match self.peek_marker()? {
            FIXSTR_START..=FIXSTR_END | STR8_MARKER | STR16_MARKER | STR32_MARKER => {
                self.read_string().map(Tag::String)
            }
            POS_FIXINT_START..=POS_FIXINT_END
            | UINT8_MARKER
            | UINT16_MARKER
            | UINT32_MARKER
            | UINT64_MARKER => self.read_u64().map(Tag::Int),
            marker => Err(Error::InvalidMarker(marker)),
        }
    }

    #[inline(always)]
    fn check_array_len(&mut self, expected: usize) -> Result<()> {
        if expected <= 15 {
            let matched = {
                let buffer = self.window()?;
                buffer.first() == Some(&(FIXARRAY_START | expected as u8))
            };
            if matched {
                self.advance(1);
                return Ok(());
            }
        }
        let actual = self.read_array_len()?;
        if actual == expected {
            Ok(())
        } else {
            Err(Error::ArrayLengthMismatch { expected, actual })
        }
    }

    #[inline(always)]
    fn check_map_len(&mut self, expected: usize) -> Result<()> {
        if expected <= 15 {
            let matched = {
                let buffer = self.window()?;
                buffer.first() == Some(&(FIXMAP_START | expected as u8))
            };
            if matched {
                self.advance(1);
                return Ok(());
            }
        }
        let actual = self.read_map_len()?;
        if actual == expected {
            Ok(())
        } else {
            Err(Error::MapLengthMismatch { expected, actual })
        }
    }

    #[inline(always)]
    fn skip_value(&mut self) -> Result<()> {
        let marker = self.peek_marker()?;
        match marker {
            POS_FIXINT_START..=POS_FIXINT_END
            | NEG_FIXINT_START..=NEG_FIXINT_END
            | NIL_MARKER
            | FALSE_MARKER
            | TRUE_MARKER => self.discard(1),
            UINT8_MARKER | INT8_MARKER => self.discard(2),
            UINT16_MARKER | INT16_MARKER => self.discard(3),
            UINT32_MARKER | INT32_MARKER | FLOAT32_MARKER => self.discard(5),
            UINT64_MARKER | INT64_MARKER | FLOAT64_MARKER => self.discard(9),
            FIXSTR_START..=FIXSTR_END | STR8_MARKER | STR16_MARKER | STR32_MARKER => {
                let len = self.read_string_len()?;
                self.discard(len)
            }
            BIN8_MARKER | BIN16_MARKER | BIN32_MARKER => {
                let len = self.read_binary_len()?;
                self.discard(len)
            }
            FIXARRAY_START..=FIXARRAY_END | ARRAY16_MARKER | ARRAY32_MARKER => {
                self.increment_depth()?;
                let len = self.read_array_len()?;
                let result = (0..len).try_for_each(|_| self.skip_value());
                self.decrement_depth();
                result
            }
            FIXMAP_START..=FIXMAP_END | MAP16_MARKER | MAP32_MARKER => {
                self.increment_depth()?;
                let len = self.read_map_len()?;
                let result = (0..len).try_for_each(|_| {
                    self.skip_value()?;
                    self.skip_value()
                });
                self.decrement_depth();
                result
            }
            FIXEXT1_MARKER | FIXEXT2_MARKER | FIXEXT4_MARKER | FIXEXT8_MARKER | FIXEXT16_MARKER
            | EXT8_MARKER | EXT16_MARKER | EXT32_MARKER => {
                let (_, len) = self.read_ext_len()?;
                self.discard(len)
            }
            _ => Err(Error::InvalidMarker(marker)),
        }
    }
}
