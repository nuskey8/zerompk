use alloc::vec::Vec;

use crate::{Error, Result, ToMessagePack};

pub trait Write {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<()>;
    fn write_byte(&mut self, byte: u8) -> Result<()>;

    #[inline(always)]
    fn write_nil(&mut self) -> Result<()> {
        self.write_byte(0xc0)
    }

    #[inline(always)]
    fn write_pos_fixint(&mut self, i: u8) -> Result<()> {
        self.write_byte(i)
    }

    #[inline(always)]
    fn write_neg_fixint(&mut self, i: i8) -> Result<()> {
        self.write_byte((0xe0 | ((i + 32) as u8)) as u8)
    }

    #[inline(always)]
    fn write_u8(&mut self, u: u8) -> Result<()> {
        if u <= 127 {
            self.write_pos_fixint(u)
        } else {
            let mut slice = [0u8; 2];
            slice[0] = 0xcc;
            slice[1] = u;
            self.write_bytes(&slice)
        }
    }

    #[inline(always)]
    fn write_u16(&mut self, u: u16) -> Result<()> {
        match u {
            0..=127 => self.write_pos_fixint(u as u8),
            128..=255 => {
                let mut slice = [0u8; 2];
                slice[0] = 0xcc;
                slice[1] = u as u8;
                self.write_bytes(&slice)
            }
            _ => {
                let mut slice = [0u8; 3];
                slice[0] = 0xcd;
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        (u as u16).to_be_bytes().as_ptr(),
                        slice.as_mut_ptr().add(1),
                        2,
                    );
                }
                self.write_bytes(&slice)
            }
        }
    }

    #[inline(always)]
    fn write_u32(&mut self, u: u32) -> Result<()> {
        match u {
            0..=127 => self.write_pos_fixint(u as u8),
            128..=255 => {
                let mut slice = [0u8; 2];
                slice[0] = 0xcc;
                slice[1] = u as u8;
                self.write_bytes(&slice)
            }
            256..=65535 => {
                let mut slice = [0u8; 3];
                slice[0] = 0xcd;
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        (u as u16).to_be_bytes().as_ptr(),
                        slice.as_mut_ptr().add(1),
                        2,
                    );
                }
                self.write_bytes(&slice)
            }
            _ => {
                let mut slice = [0u8; 5];
                slice[0] = 0xce;
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        (u as u32).to_be_bytes().as_ptr(),
                        slice.as_mut_ptr().add(1),
                        4,
                    );
                }
                self.write_bytes(&slice)
            }
        }
    }

    #[inline(always)]
    fn write_u64(&mut self, u: u64) -> Result<()> {
        match u {
            0..=127 => self.write_pos_fixint(u as u8),
            128..=255 => {
                let slice = [0xcc, u as u8];
                self.write_bytes(&slice)
            }
            256..=65535 => {
                let mut slice = [0u8; 3];
                slice[0] = 0xcd;
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        (u as u16).to_be_bytes().as_ptr(),
                        slice.as_mut_ptr().add(1),
                        2,
                    );
                }
                self.write_bytes(&slice)
            }
            65536..=4294967295 => {
                let mut slice = [0u8; 5];
                slice[0] = 0xce;
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        (u as u32).to_be_bytes().as_ptr(),
                        slice.as_mut_ptr().add(1),
                        4,
                    );
                }
                self.write_bytes(&slice)
            }
            _ => {
                let mut slice = [0u8; 9];
                slice[0] = 0xcf;
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        (u as u64).to_be_bytes().as_ptr(),
                        slice.as_mut_ptr().add(1),
                        8,
                    );
                }
                self.write_bytes(&slice)
            }
        }
    }

    #[inline(always)]
    fn write_i8(&mut self, i: i8) -> Result<()> {
        match i {
            0..=127 => self.write_pos_fixint(i as u8),
            -32..=-1 => self.write_neg_fixint(i as i8),
            _ => {
                let mut slice = [0u8; 2];
                slice[0] = 0xd0;
                slice[1] = i as u8;
                self.write_bytes(&slice)
            }
        }
    }

    #[inline(always)]
    fn write_i16(&mut self, i: i16) -> Result<()> {
        match i {
            0..=127 => self.write_pos_fixint(i as u8),
            -32..=-1 => self.write_neg_fixint(i as i8),
            -128..=127 => {
                let slice = [0xd0, i as u8];
                self.write_bytes(&slice)
            }
            _ => {
                let mut slice = [0u8; 3];
                slice[0] = 0xd1;
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        i.to_be_bytes().as_ptr(),
                        slice.as_mut_ptr().add(1),
                        2,
                    );
                }
                self.write_bytes(&slice)
            }
        }
    }

    #[inline(always)]
    fn write_i32(&mut self, i: i32) -> Result<()> {
        match i {
            0..=127 => self.write_pos_fixint(i as u8),
            -32..=-1 => self.write_neg_fixint(i as i8),
            -128..=127 => {
                let slice = [0xd0, i as u8];
                self.write_bytes(&slice)
            }
            -32768..=32767 => {
                let mut slice = [0u8; 3];
                slice[0] = 0xd1;
                let be = (i as i16).to_be_bytes();
                unsafe {
                    core::ptr::copy_nonoverlapping(be.as_ptr(), slice.as_mut_ptr().add(1), 2);
                }
                self.write_bytes(&slice)
            }
            _ => {
                let mut slice = [0u8; 5];
                slice[0] = 0xd2;
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        i.to_be_bytes().as_ptr(),
                        slice.as_mut_ptr().add(1),
                        4,
                    );
                }
                self.write_bytes(&slice)
            }
        }
    }

    #[inline(always)]
    fn write_i64(&mut self, i: i64) -> Result<()> {
        match i {
            0..=127 => self.write_pos_fixint(i as u8),
            -32..=-1 => self.write_neg_fixint(i as i8),
            -128..=127 => {
                let slice = [0xd0, i as u8];
                self.write_bytes(&slice)
            }
            -32768..=32767 => {
                let mut slice = [0u8; 3];
                slice[0] = 0xd1;
                let be = (i as i16).to_be_bytes();
                unsafe {
                    core::ptr::copy_nonoverlapping(be.as_ptr(), slice.as_mut_ptr().add(1), 2);
                }
                self.write_bytes(&slice)
            }
            -2147483648..=2147483647 => {
                let mut slice = [0u8; 5];
                slice[0] = 0xd2;
                let be = (i as i32).to_be_bytes();
                unsafe {
                    core::ptr::copy_nonoverlapping(be.as_ptr(), slice.as_mut_ptr().add(1), 4);
                }
                self.write_bytes(&slice)
            }
            _ => {
                let mut slice = [0u8; 9];
                slice[0] = 0xd3;
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        i.to_be_bytes().as_ptr(),
                        slice.as_mut_ptr().add(1),
                        8,
                    );
                }
                self.write_bytes(&slice)
            }
        }
    }

    #[inline(always)]
    fn write_boolean(&mut self, b: bool) -> Result<()> {
        self.write_byte(if b { 0xc3 } else { 0xc2 })
    }

    #[inline(always)]
    fn write_f32(&mut self, f: f32) -> Result<()> {
        let mut slice = [0u8; 5];
        slice[0] = 0xca;
        unsafe {
            core::ptr::copy_nonoverlapping(f.to_be_bytes().as_ptr(), slice.as_mut_ptr().add(1), 4);
        }
        self.write_bytes(&slice)
    }

    #[inline(always)]
    fn write_f64(&mut self, f: f64) -> Result<()> {
        let mut slice = [0u8; 9];
        slice[0] = 0xcb;
        unsafe {
            core::ptr::copy_nonoverlapping(f.to_be_bytes().as_ptr(), slice.as_mut_ptr().add(1), 8);
        }
        self.write_bytes(&slice)
    }

    fn write_string(&mut self, s: &str) -> Result<()> {
        let len = s.len();
        match len {
            // FixStr
            0..=31 => {
                self.write_byte(0xa0 | (len as u8))?;
                self.write_bytes(s.as_bytes())?;
            }
            // Str8
            32..=255 => {
                let header = [0xd9, len as u8];
                self.write_bytes(&header)?;
                self.write_bytes(s.as_bytes())?;
            }
            // Str16
            256..=65535 => {
                let len_bytes = (len as u16).to_be_bytes();
                let header = [0xda, len_bytes[0], len_bytes[1]];
                self.write_bytes(&header)?;
                self.write_bytes(s.as_bytes())?;
            }
            // Str32
            _ => {
                let len_bytes = (len as u32).to_be_bytes();
                let header = [0xdb, len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3]];
                self.write_bytes(&header)?;
                self.write_bytes(s.as_bytes())?;
            }
        }
        Ok(())
    }

    fn write_binary(&mut self, data: &[u8]) -> Result<()> {
        let len = data.len();
        match len {
            // Bin8
            0..=255 => {
                let header = [0xc4, len as u8];
                self.write_bytes(&header)?;
                self.write_bytes(data)?;
                Ok(())
            }
            // Bin16
            256..=65535 => {
                let len_bytes = (len as u16).to_be_bytes();
                let header = [0xc5, len_bytes[0], len_bytes[1]];
                self.write_bytes(&header)?;
                self.write_bytes(data)?;
                Ok(())
            }
            // Bin32
            _ => {
                let len_bytes = (len as u32).to_be_bytes();
                let header = [0xc6, len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3]];
                self.write_bytes(&header)?;
                self.write_bytes(data)?;
                Ok(())
            }
        }
    }

    fn write_timestamp(&mut self, seconds: i64, nanoseconds: u32) -> Result<()> {
        if nanoseconds >= 1_000_000_000 {
            return Err(Error::InvalidTimestamp);
        }

        // timestamp 32: sec in [0, 2^32-1], nsec == 0
        if nanoseconds == 0 && (0..=u32::MAX as i64).contains(&seconds) {
            let mut buf = [0u8; 6];
            buf[0] = 0xd6;
            buf[1] = 0xff;
            unsafe {
                core::ptr::copy_nonoverlapping(
                    (seconds as u32).to_be_bytes().as_ptr(),
                    buf.as_mut_ptr().add(2),
                    4,
                );
            }
            return self.write_bytes(&buf);
        }

        // timestamp 64: sec in [0, 2^34-1]
        if (0..=(1i64 << 34) - 1).contains(&seconds) {
            let data = ((nanoseconds as u64) << 34) | (seconds as u64);
            let mut buf = [0u8; 10];
            buf[0] = 0xd7;
            buf[1] = 0xff;
            unsafe {
                core::ptr::copy_nonoverlapping(
                    data.to_be_bytes().as_ptr(),
                    buf.as_mut_ptr().add(2),
                    8,
                );
            }
            return self.write_bytes(&buf);
        }

        // timestamp 96
        let mut buf = [0u8; 15];
        buf[0] = 0xc7;
        buf[1] = 12;
        buf[2] = 0xff;
        unsafe {
            core::ptr::copy_nonoverlapping(
                nanoseconds.to_be_bytes().as_ptr(),
                buf.as_mut_ptr().add(3),
                4,
            );
            core::ptr::copy_nonoverlapping(
                seconds.to_be_bytes().as_ptr(),
                buf.as_mut_ptr().add(7),
                8,
            );
        }
        self.write_bytes(&buf)
    }

    #[inline(always)]
    fn write_array_len(&mut self, len: usize) -> Result<()> {
        match len {
            // FixArray
            0..=15 => self.write_byte(0x90 | (len as u8)),
            // Array16
            16..=65535 => {
                let mut slice = [0u8; 3];
                slice[0] = 0xdc;
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        (len as u16).to_be_bytes().as_ptr(),
                        slice.as_mut_ptr().add(1),
                        2,
                    );
                }
                self.write_bytes(&slice)
            }
            // Array32
            _ => {
                let mut slice = [0u8; 5];
                slice[0] = 0xdd;
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        (len as u32).to_be_bytes().as_ptr(),
                        slice.as_mut_ptr().add(1),
                        4,
                    );
                }
                self.write_bytes(&slice)
            }
        }
    }

    #[inline(always)]
    fn write_map_len(&mut self, len: usize) -> Result<()> {
        match len {
            // FixMap
            0..=15 => self.write_byte(0x80 | (len as u8)),
            // Map16
            16..=65535 => {
                let mut slice = [0u8; 3];
                slice[0] = 0xde;
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        (len as u16).to_be_bytes().as_ptr(),
                        slice.as_mut_ptr().add(1),
                        2,
                    );
                }
                self.write_bytes(&slice)
            }
            // Map32
            _ => {
                let mut slice = [0u8; 5];
                slice[0] = 0xdf;
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        (len as u32).to_be_bytes().as_ptr(),
                        slice.as_mut_ptr().add(1),
                        4,
                    );
                }
                self.write_bytes(&slice)
            }
        }
    }

    #[inline(always)]
    fn write_array_from_slice<T: ToMessagePack>(&mut self, slice: &[T]) -> Result<()>
    where
        Self: Sized,
    {
        self.write_array_len(slice.len())?;
        for item in slice {
            item.write(self)?;
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<T: std::io::Write> Write for T {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        self.write_all(bytes).map_err(|e| Error::IoError(e))
    }

    fn write_byte(&mut self, byte: u8) -> Result<()> {
        self.write_all(&[byte]).map_err(|e| Error::IoError(e))
    }
}

pub struct VecWriter {
    buffer: Vec<u8>,
}

impl Write for VecWriter {
    #[inline(always)]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        self.buffer.extend_from_slice(bytes);
        Ok(())
    }

    #[inline(always)]
    fn write_byte(&mut self, byte: u8) -> Result<()> {
        self.buffer.push(byte);
        Ok(())
    }

    #[inline(always)]
    fn write_u8(&mut self, u: u8) -> Result<()> {
        self.buffer.reserve_exact(2);
        unsafe {
            let len = self.buffer.len();
            let dst = self.buffer.as_mut_ptr().add(len);
            if u <= 127 {
                *dst = u;
                self.buffer.set_len(len + 1);
            } else {
                *dst = 0xcc;
                *dst.add(1) = u;
                self.buffer.set_len(len + 2);
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn write_u16(&mut self, u: u16) -> Result<()> {
        self.buffer.reserve_exact(3);
        unsafe {
            let len = self.buffer.len();
            let dst = self.buffer.as_mut_ptr().add(len);
            match u {
                0..=127 => {
                    *dst = u as u8;
                    self.buffer.set_len(len + 1);
                }
                128..=255 => {
                    *dst = 0xcc;
                    *dst.add(1) = u as u8;
                    self.buffer.set_len(len + 2);
                }
                _ => {
                    *dst = 0xcd;
                    core::ptr::copy_nonoverlapping(
                        (u as u16).to_be_bytes().as_ptr(),
                        dst.add(1),
                        2,
                    );
                    self.buffer.set_len(len + 3);
                }
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn write_u32(&mut self, u: u32) -> Result<()> {
        self.buffer.reserve_exact(5);
        unsafe {
            let len = self.buffer.len();
            let dst = self.buffer.as_mut_ptr().add(len);
            match u {
                0..=127 => {
                    *dst = u as u8;
                    self.buffer.set_len(len + 1);
                }
                128..=255 => {
                    *dst = 0xcc;
                    *dst.add(1) = u as u8;
                    self.buffer.set_len(len + 2);
                }
                256..=65535 => {
                    *dst = 0xcd;
                    core::ptr::copy_nonoverlapping(
                        (u as u16).to_be_bytes().as_ptr(),
                        dst.add(1),
                        2,
                    );
                    self.buffer.set_len(len + 3);
                }
                _ => {
                    *dst = 0xce;
                    core::ptr::copy_nonoverlapping(
                        (u as u32).to_be_bytes().as_ptr(),
                        dst.add(1),
                        4,
                    );
                    self.buffer.set_len(len + 5);
                }
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn write_u64(&mut self, u: u64) -> Result<()> {
        self.buffer.reserve_exact(9);
        unsafe {
            let len = self.buffer.len();
            let dst = self.buffer.as_mut_ptr().add(len);
            match u {
                0..=127 => {
                    *dst = u as u8;
                    self.buffer.set_len(len + 1);
                }
                128..=255 => {
                    *dst = 0xcc;
                    *dst.add(1) = u as u8;
                    self.buffer.set_len(len + 2);
                }
                256..=65535 => {
                    *dst = 0xcd;
                    core::ptr::copy_nonoverlapping(
                        (u as u16).to_be_bytes().as_ptr(),
                        dst.add(1),
                        2,
                    );
                    self.buffer.set_len(len + 3);
                }
                65536..=4294967295 => {
                    *dst = 0xce;
                    core::ptr::copy_nonoverlapping(
                        (u as u32).to_be_bytes().as_ptr(),
                        dst.add(1),
                        4,
                    );
                    self.buffer.set_len(len + 5);
                }
                _ => {
                    *dst = 0xcf;
                    core::ptr::copy_nonoverlapping(
                        (u as u64).to_be_bytes().as_ptr(),
                        dst.add(1),
                        8,
                    );
                    self.buffer.set_len(len + 9);
                }
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn write_i8(&mut self, i: i8) -> Result<()> {
        self.buffer.reserve_exact(2);
        unsafe {
            let dst = self.buffer.as_mut_ptr().add(self.buffer.len());
            match i {
                0..=127 => {
                    *dst = i as u8;
                    self.buffer.set_len(self.buffer.len() + 1);
                }
                -32..=-1 => {
                    *dst = (0xe0 | ((i + 32) as u8)) as u8;
                    self.buffer.set_len(self.buffer.len() + 1);
                }
                _ => {
                    *dst = 0xd0;
                    *dst.add(1) = i as u8;
                    self.buffer.set_len(self.buffer.len() + 2);
                }
            }
            Ok(())
        }
    }

    #[inline(always)]
    fn write_i16(&mut self, i: i16) -> Result<()> {
        self.buffer.reserve_exact(3);
        unsafe {
            let dst = self.buffer.as_mut_ptr().add(self.buffer.len());
            match i {
                0..=127 => {
                    *dst = i as u8;
                    self.buffer.set_len(self.buffer.len() + 1);
                }
                -32..=-1 => {
                    *dst = (0xe0 | ((i + 32) as u8)) as u8;
                    self.buffer.set_len(self.buffer.len() + 1);
                }
                -128..=127 => {
                    *dst = 0xd0;
                    *dst.add(1) = i as u8;
                    self.buffer.set_len(self.buffer.len() + 2);
                }
                _ => {
                    *dst = 0xd1;
                    core::ptr::copy_nonoverlapping(i.to_be_bytes().as_ptr(), dst.add(1), 2);
                    self.buffer.set_len(self.buffer.len() + 3);
                }
            }
            Ok(())
        }
    }

    #[inline(always)]
    fn write_i32(&mut self, i: i32) -> Result<()> {
        self.buffer.reserve_exact(5);
        unsafe {
            let dst = self.buffer.as_mut_ptr().add(self.buffer.len());
            match i {
                0..=127 => {
                    *dst = i as u8;
                    self.buffer.set_len(self.buffer.len() + 1);
                }
                -32..=-1 => {
                    *dst = (0xe0 | ((i + 32) as u8)) as u8;
                    self.buffer.set_len(self.buffer.len() + 1);
                }
                -128..=127 => {
                    *dst = 0xd0;
                    *dst.add(1) = i as u8;
                    self.buffer.set_len(self.buffer.len() + 2);
                }
                -32768..=32767 => {
                    *dst = 0xd1;
                    core::ptr::copy_nonoverlapping(
                        (i as i16).to_be_bytes().as_ptr(),
                        dst.add(1),
                        2,
                    );
                    self.buffer.set_len(self.buffer.len() + 3);
                }
                _ => {
                    *dst = 0xd2;
                    core::ptr::copy_nonoverlapping(i.to_be_bytes().as_ptr(), dst.add(1), 4);
                    self.buffer.set_len(self.buffer.len() + 5);
                }
            }
            Ok(())
        }
    }

    #[inline(always)]
    fn write_i64(&mut self, i: i64) -> Result<()> {
        self.buffer.reserve_exact(9);
        unsafe {
            let dst = self.buffer.as_mut_ptr().add(self.buffer.len());
            match i {
                0..=127 => {
                    *dst = i as u8;
                    self.buffer.set_len(self.buffer.len() + 1);
                }
                -32..=-1 => {
                    *dst = (0xe0 | ((i + 32) as u8)) as u8;
                    self.buffer.set_len(self.buffer.len() + 1);
                }
                -128..=127 => {
                    *dst = 0xd0;
                    *dst.add(1) = i as u8;
                    self.buffer.set_len(self.buffer.len() + 2);
                }
                -32768..=32767 => {
                    *dst = 0xd1;
                    core::ptr::copy_nonoverlapping(
                        (i as i16).to_be_bytes().as_ptr(),
                        dst.add(1),
                        2,
                    );
                    self.buffer.set_len(self.buffer.len() + 3);
                }
                -2147483648..=2147483647 => {
                    *dst = 0xd2;
                    core::ptr::copy_nonoverlapping(
                        (i as i32).to_be_bytes().as_ptr(),
                        dst.add(1),
                        4,
                    );
                    self.buffer.set_len(self.buffer.len() + 5);
                }
                _ => {
                    *dst = 0xd3;
                    core::ptr::copy_nonoverlapping(i.to_be_bytes().as_ptr(), dst.add(1), 8);
                    self.buffer.set_len(self.buffer.len() + 9);
                }
            }
            Ok(())
        }
    }

    #[inline(always)]
    fn write_f32(&mut self, f: f32) -> Result<()> {
        self.buffer.reserve_exact(5);
        unsafe {
            let len = self.buffer.len();
            let dst = self.buffer.as_mut_ptr().add(len);
            *dst = 0xca;
            core::ptr::copy_nonoverlapping(f.to_be_bytes().as_ptr(), dst.add(1), 4);
            self.buffer.set_len(len + 5);
        }
        Ok(())
    }

    #[inline(always)]
    fn write_f64(&mut self, f: f64) -> Result<()> {
        self.buffer.reserve_exact(9);
        unsafe {
            let len = self.buffer.len();
            let dst = self.buffer.as_mut_ptr().add(len);
            *dst = 0xcb;
            core::ptr::copy_nonoverlapping(f.to_be_bytes().as_ptr(), dst.add(1), 8);
            self.buffer.set_len(len + 9);
        }
        Ok(())
    }

    #[inline(always)]
    fn write_timestamp(&mut self, seconds: i64, nanoseconds: u32) -> Result<()> {
        if nanoseconds >= 1_000_000_000 {
            return Err(Error::InvalidTimestamp);
        }

        // timestamp 32: sec in [0, 2^32-1], nsec == 0
        if nanoseconds == 0 && (0..=u32::MAX as i64).contains(&seconds) {
            self.buffer.reserve_exact(6);
            unsafe {
                let len = self.buffer.len();
                let dst = self.buffer.as_mut_ptr().add(len);
                *dst = 0xd6;
                *dst.add(1) = 0xff;
                core::ptr::copy_nonoverlapping(
                    (seconds as u32).to_be_bytes().as_ptr(),
                    dst.add(2),
                    4,
                );
                self.buffer.set_len(len + 6);
            }
            return Ok(());
        }

        // timestamp 64: sec in [0, 2^34-1]
        if (0..=(1i64 << 34) - 1).contains(&seconds) {
            let data = ((nanoseconds as u64) << 34) | (seconds as u64);
            self.buffer.reserve_exact(10);
            unsafe {
                let len = self.buffer.len();
                let dst = self.buffer.as_mut_ptr().add(len);
                *dst = 0xd7;
                *dst.add(1) = 0xff;
                core::ptr::copy_nonoverlapping(data.to_be_bytes().as_ptr(), dst.add(2), 8);
                self.buffer.set_len(len + 10);
            }
            return Ok(());
        }

        // timestamp 96
        self.buffer.reserve_exact(15);
        unsafe {
            let len = self.buffer.len();
            let dst = self.buffer.as_mut_ptr().add(len);
            *dst = 0xc7;
            *dst.add(1) = 12;
            *dst.add(2) = 0xff;
            core::ptr::copy_nonoverlapping(nanoseconds.to_be_bytes().as_ptr(), dst.add(3), 4);
            core::ptr::copy_nonoverlapping(seconds.to_be_bytes().as_ptr(), dst.add(7), 8);
            self.buffer.set_len(len + 15);
        }
        Ok(())
    }

    fn write_array_len(&mut self, len: usize) -> Result<()> {
        self.buffer.reserve_exact(5);
        unsafe {
            let dst = self.buffer.as_mut_ptr().add(self.buffer.len());
            match len {
                0..=15 => {
                    *dst = 0x90 | (len as u8);
                    self.buffer.set_len(self.buffer.len() + 1);
                }
                16..=65535 => {
                    *dst = 0xdc;
                    core::ptr::copy_nonoverlapping(
                        (len as u16).to_be_bytes().as_ptr(),
                        dst.add(1),
                        2,
                    );
                    self.buffer.set_len(self.buffer.len() + 3);
                }
                _ => {
                    *dst = 0xdd;
                    core::ptr::copy_nonoverlapping(
                        (len as u32).to_be_bytes().as_ptr(),
                        dst.add(1),
                        4,
                    );
                    self.buffer.set_len(self.buffer.len() + 5);
                }
            }
            Ok(())
        }
    }
}

impl VecWriter {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.buffer
    }
}

pub struct SliceWriter<'a> {
    buffer: &'a mut [u8],
    pos: usize,
}

impl<'a> SliceWriter<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self { buffer, pos: 0 }
    }

    pub fn written(&self) -> usize {
        self.pos
    }

    #[inline(always)]
    fn check_buffer_size(&self, additional: usize) -> Result<()> {
        if self.pos + additional > self.buffer.len() {
            Err(Error::BufferTooSmall)
        } else {
            Ok(())
        }
    }
}

impl<'a> Write for SliceWriter<'a> {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.is_empty() {
            return Ok(());
        }

        self.check_buffer_size(bytes.len())?;
        unsafe {
            let dst = self.buffer.as_mut_ptr().add(self.pos);
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), dst, bytes.len());
        }
        self.pos += bytes.len();
        Ok(())
    }

    fn write_byte(&mut self, byte: u8) -> Result<()> {
        self.check_buffer_size(1)?;
        self.buffer[self.pos] = byte;
        self.pos += 1;
        Ok(())
    }

    fn write_i8(&mut self, i: i8) -> Result<()> {
        match i {
            0..=127 => self.write_pos_fixint(i as u8),
            -32..=-1 => self.write_neg_fixint(i as i8),
            _ => {
                self.check_buffer_size(2)?;
                unsafe {
                    let dst = self.buffer.as_mut_ptr().add(self.pos);
                    *dst = 0xd0;
                    *dst.add(1) = i as u8;
                }
                self.pos += 2;
                Ok(())
            }
        }
    }

    fn write_i16(&mut self, i: i16) -> Result<()> {
        match i {
            0..=127 => self.write_pos_fixint(i as u8),
            -32..=-1 => self.write_neg_fixint(i as i8),
            -128..=127 => {
                self.check_buffer_size(2)?;
                unsafe {
                    let dst = self.buffer.as_mut_ptr().add(self.pos);
                    *dst = 0xd0;
                    *dst.add(1) = i as u8;
                }
                self.pos += 2;
                Ok(())
            }
            _ => {
                self.check_buffer_size(3)?;
                unsafe {
                    let dst = self.buffer.as_mut_ptr().add(self.pos);
                    *dst = 0xd1;
                    core::ptr::copy_nonoverlapping(i.to_be_bytes().as_ptr(), dst.add(1), 2);
                }
                self.pos += 3;
                Ok(())
            }
        }
    }

    fn write_i32(&mut self, i: i32) -> Result<()> {
        unsafe {
            let dst = self.buffer.as_mut_ptr().add(self.pos);
            match i {
                0..=127 => self.write_pos_fixint(i as u8)?,
                -32..=-1 => self.write_neg_fixint(i as i8)?,
                -128..=127 => {
                    self.check_buffer_size(2)?;
                    *dst = 0xd0;
                    *dst.add(1) = i as u8;
                    self.pos += 2;
                }
                -32768..=32767 => {
                    self.check_buffer_size(3)?;
                    *dst = 0xd1;
                    core::ptr::copy_nonoverlapping(
                        (i as i16).to_be_bytes().as_ptr(),
                        dst.add(1),
                        2,
                    );
                    self.pos += 3;
                }
                _ => {
                    self.check_buffer_size(5)?;
                    *dst = 0xd2;
                    core::ptr::copy_nonoverlapping(i.to_be_bytes().as_ptr(), dst.add(1), 4);
                    self.pos += 5;
                }
            }
            Ok(())
        }
    }

    fn write_i64(&mut self, i: i64) -> Result<()> {
        unsafe {
            let dst = self.buffer.as_mut_ptr().add(self.pos);
            match i {
                0..=127 => self.write_pos_fixint(i as u8)?,
                -32..=-1 => self.write_neg_fixint(i as i8)?,
                -128..=127 => {
                    self.check_buffer_size(2)?;
                    *dst = 0xd0;
                    *dst.add(1) = i as u8;
                    self.pos += 2;
                }
                -32768..=32767 => {
                    self.check_buffer_size(3)?;
                    *dst = 0xd1;
                    core::ptr::copy_nonoverlapping(
                        (i as i16).to_be_bytes().as_ptr(),
                        dst.add(1),
                        2,
                    );
                    self.pos += 3;
                }
                -2147483648..=2147483647 => {
                    self.check_buffer_size(5)?;
                    *dst = 0xd2;
                    core::ptr::copy_nonoverlapping(
                        (i as i32).to_be_bytes().as_ptr(),
                        dst.add(1),
                        4,
                    );
                    self.pos += 5;
                }
                _ => {
                    self.check_buffer_size(9)?;
                    *dst = 0xd3;
                    core::ptr::copy_nonoverlapping(i.to_be_bytes().as_ptr(), dst.add(1), 8);
                    self.pos += 9;
                }
            }
            Ok(())
        }
    }

    fn write_u8(&mut self, u: u8) -> Result<()> {
        if u <= 127 {
            self.write_pos_fixint(u)
        } else {
            self.check_buffer_size(2)?;
            unsafe {
                let dst = self.buffer.as_mut_ptr().add(self.pos);
                *dst = 0xcc;
                *dst.add(1) = u;
            }
            self.pos += 2;
            Ok(())
        }
    }

    fn write_u16(&mut self, u: u16) -> Result<()> {
        match u {
            0..=127 => self.write_pos_fixint(u as u8),
            128..=255 => {
                self.check_buffer_size(2)?;
                unsafe {
                    let dst = self.buffer.as_mut_ptr().add(self.pos);
                    *dst = 0xcc;
                    *dst.add(1) = u as u8;
                }
                self.pos += 2;
                Ok(())
            }
            _ => {
                self.check_buffer_size(3)?;
                unsafe {
                    let dst = self.buffer.as_mut_ptr().add(self.pos);
                    *dst = 0xcd;
                    core::ptr::copy_nonoverlapping(
                        (u as u16).to_be_bytes().as_ptr(),
                        dst.add(1),
                        2,
                    );
                }
                self.pos += 3;
                Ok(())
            }
        }
    }

    fn write_u32(&mut self, u: u32) -> Result<()> {
        match u {
            0..=127 => self.write_pos_fixint(u as u8),
            128..=255 => {
                self.check_buffer_size(2)?;
                unsafe {
                    let dst = self.buffer.as_mut_ptr().add(self.pos);
                    *dst = 0xcc;
                    *dst.add(1) = u as u8;
                }
                self.pos += 2;
                Ok(())
            }
            256..=65535 => {
                self.check_buffer_size(3)?;
                unsafe {
                    let dst = self.buffer.as_mut_ptr().add(self.pos);
                    *dst = 0xcd;
                    core::ptr::copy_nonoverlapping(
                        (u as u16).to_be_bytes().as_ptr(),
                        dst.add(1),
                        2,
                    );
                }
                self.pos += 3;
                Ok(())
            }
            _ => {
                self.check_buffer_size(5)?;
                unsafe {
                    let dst = self.buffer.as_mut_ptr().add(self.pos);
                    *dst = 0xce;
                    core::ptr::copy_nonoverlapping(
                        (u as u32).to_be_bytes().as_ptr(),
                        dst.add(1),
                        4,
                    );
                }
                self.pos += 5;
                Ok(())
            }
        }
    }

    fn write_u64(&mut self, u: u64) -> Result<()> {
        match u {
            0..=127 => self.write_pos_fixint(u as u8),
            128..=255 => {
                self.check_buffer_size(2)?;
                unsafe {
                    let dst = self.buffer.as_mut_ptr().add(self.pos);
                    *dst = 0xcc;
                    *dst.add(1) = u as u8;
                }
                self.pos += 2;
                Ok(())
            }
            256..=65535 => {
                self.check_buffer_size(3)?;
                unsafe {
                    let dst = self.buffer.as_mut_ptr().add(self.pos);
                    *dst = 0xcd;
                    core::ptr::copy_nonoverlapping(
                        (u as u16).to_be_bytes().as_ptr(),
                        dst.add(1),
                        2,
                    );
                }
                self.pos += 3;
                Ok(())
            }
            65536..=4294967295 => {
                self.check_buffer_size(5)?;
                unsafe {
                    let dst = self.buffer.as_mut_ptr().add(self.pos);
                    *dst = 0xce;
                    core::ptr::copy_nonoverlapping(
                        (u as u32).to_be_bytes().as_ptr(),
                        dst.add(1),
                        4,
                    );
                }
                self.pos += 5;
                Ok(())
            }
            _ => {
                self.check_buffer_size(9)?;
                unsafe {
                    let dst = self.buffer.as_mut_ptr().add(self.pos);
                    *dst = 0xcf;
                    core::ptr::copy_nonoverlapping(
                        (u as u64).to_be_bytes().as_ptr(),
                        dst.add(1),
                        8,
                    );
                }
                self.pos += 9;
                Ok(())
            }
        }
    }

    fn write_f32(&mut self, f: f32) -> Result<()> {
        self.check_buffer_size(5)?;
        unsafe {
            let dst = self.buffer.as_mut_ptr().add(self.pos);
            *dst = 0xca;
            core::ptr::copy_nonoverlapping(f.to_be_bytes().as_ptr(), dst.add(1), 4);
        }
        self.pos += 5;
        Ok(())
    }

    fn write_f64(&mut self, f: f64) -> Result<()> {
        self.check_buffer_size(9)?;
        unsafe {
            let dst = self.buffer.as_mut_ptr().add(self.pos);
            *dst = 0xcb;
            core::ptr::copy_nonoverlapping(f.to_be_bytes().as_ptr(), dst.add(1), 8);
        }
        self.pos += 9;
        Ok(())
    }

    fn write_binary(&mut self, data: &[u8]) -> Result<()> {
        let len = data.len();
        match len {
            0..=255 => {
                self.check_buffer_size(1 + len)?;
                unsafe {
                    let dst = self.buffer.as_mut_ptr().add(self.pos);
                    *dst = 0xc4;
                    *dst.add(1) = len as u8;
                    core::ptr::copy_nonoverlapping(data.as_ptr(), dst.add(2), len);
                }
                self.pos += 2 + len;
                Ok(())
            }
            256..=65535 => {
                self.check_buffer_size(3 + len)?;
                unsafe {
                    let dst = self.buffer.as_mut_ptr().add(self.pos);
                    *dst = 0xc5;
                    core::ptr::copy_nonoverlapping(
                        (len as u16).to_be_bytes().as_ptr(),
                        dst.add(1),
                        2,
                    );
                    core::ptr::copy_nonoverlapping(data.as_ptr(), dst.add(3), len);
                }
                self.pos += 3 + len;
                Ok(())
            }
            _ => {
                self.check_buffer_size(5 + len)?;
                unsafe {
                    let dst = self.buffer.as_mut_ptr().add(self.pos);
                    *dst = 0xc6;
                    core::ptr::copy_nonoverlapping(
                        (len as u32).to_be_bytes().as_ptr(),
                        dst.add(1),
                        4,
                    );
                    core::ptr::copy_nonoverlapping(data.as_ptr(), dst.add(5), len);
                }
                self.pos += 5 + len;
                Ok(())
            }
        }
    }

    fn write_string(&mut self, s: &str) -> Result<()> {
        match s.len() {
            0..=31 => {
                self.check_buffer_size(1 + s.len())?;
                unsafe {
                    let dst = self.buffer.as_mut_ptr().add(self.pos);
                    *dst = 0xa0 | (s.len() as u8);
                    core::ptr::copy_nonoverlapping(s.as_ptr(), dst.add(1), s.len());
                }
                self.pos += 1 + s.len();
                Ok(())
            }
            32..=255 => {
                self.check_buffer_size(2 + s.len())?;
                unsafe {
                    let dst = self.buffer.as_mut_ptr().add(self.pos);
                    *dst = 0xd9;
                    *dst.add(1) = s.len() as u8;
                    core::ptr::copy_nonoverlapping(s.as_ptr(), dst.add(2), s.len());
                }
                self.pos += 2 + s.len();
                Ok(())
            }
            256..=65535 => {
                self.check_buffer_size(3 + s.len())?;
                unsafe {
                    let dst = self.buffer.as_mut_ptr().add(self.pos);
                    *dst = 0xda;
                    core::ptr::copy_nonoverlapping(
                        (s.len() as u16).to_be_bytes().as_ptr(),
                        dst.add(1),
                        2,
                    );
                    core::ptr::copy_nonoverlapping(s.as_ptr(), dst.add(3), s.len());
                }
                self.pos += 3 + s.len();
                Ok(())
            }
            _ => {
                self.check_buffer_size(5 + s.len())?;
                unsafe {
                    let dst = self.buffer.as_mut_ptr().add(self.pos);
                    *dst = 0xdb;
                    core::ptr::copy_nonoverlapping(
                        (s.len() as u32).to_be_bytes().as_ptr(),
                        dst.add(1),
                        4,
                    );
                    core::ptr::copy_nonoverlapping(s.as_ptr(), dst.add(5), s.len());
                }
                self.pos += 5 + s.len();
                Ok(())
            }
        }
    }

    fn write_array_len(&mut self, len: usize) -> Result<()> {
        unsafe {
            let dst = self.buffer.as_mut_ptr().add(self.pos);
            match len {
                0..=15 => {
                    self.check_buffer_size(1)?;
                    *dst = 0x90 | (len as u8);
                    self.pos += 1;
                }
                16..=65535 => {
                    self.check_buffer_size(3)?;
                    *dst = 0xdc;
                    core::ptr::copy_nonoverlapping(
                        (len as u16).to_be_bytes().as_ptr(),
                        dst.add(1),
                        2,
                    );
                    self.pos += 3;
                }
                _ => {
                    self.check_buffer_size(5)?;
                    *dst = 0xdd;
                    core::ptr::copy_nonoverlapping(
                        (len as u32).to_be_bytes().as_ptr(),
                        dst.add(1),
                        4,
                    );
                    self.pos += 5;
                }
            }
            Ok(())
        }
    }

    fn write_map_len(&mut self, len: usize) -> Result<()> {
        match len {
            0..=15 => self.write_byte(0x80 | (len as u8)),
            16..=65535 => {
                self.check_buffer_size(3)?;
                unsafe {
                    let dst = self.buffer.as_mut_ptr().add(self.pos);
                    *dst = 0xde;
                    core::ptr::copy_nonoverlapping(
                        (len as u16).to_be_bytes().as_ptr(),
                        dst.add(1),
                        2,
                    );
                }
                self.pos += 3;
                Ok(())
            }
            _ => {
                self.check_buffer_size(5)?;
                unsafe {
                    let dst = self.buffer.as_mut_ptr().add(self.pos);
                    *dst = 0xdf;
                    core::ptr::copy_nonoverlapping(
                        (len as u32).to_be_bytes().as_ptr(),
                        dst.add(1),
                        4,
                    );
                }
                self.pos += 5;
                Ok(())
            }
        }
    }
}
