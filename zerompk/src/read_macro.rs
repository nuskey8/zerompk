macro_rules! impl_read_methods {
    (
        read_byte = |$byte_reader:ident| $read_byte:expr,
        read_2 = |$reader_2:ident| $read_2:expr,
        read_4 = |$reader_4:ident| $read_4:expr,
        read_5 = |$reader_5:ident| $read_5:expr,
        read_8 = |$reader_8:ident| $read_8:expr,
        read_9 = |$reader_9:ident| $read_9:expr,
        read_13 = |$reader_13:ident| $read_13:expr,
        read_bytes = |$bytes_reader:ident, $bytes_len:ident| $read_bytes:expr,
        invalid = |$reader:ident, $bad_marker:ident| $invalid:expr $(,)?
    ) => {
        #[inline(always)]
        fn read_nil(&mut self) -> Result<()> {
            let marker = {
                let $byte_reader = &mut *self;
                $read_byte
            };
            if marker == NIL_MARKER {
                Ok(())
            } else {
                {
                    let $reader = &mut *self;
                    let $bad_marker = marker;
                    $invalid
                }
            }
        }

        #[inline(always)]
        fn read_boolean(&mut self) -> Result<bool> {
            let marker = {
                let $byte_reader = &mut *self;
                $read_byte
            };
            match marker {
                FALSE_MARKER => Ok(false),
                TRUE_MARKER => Ok(true),
                _ => {
                    let $reader = &mut *self;
                    let $bad_marker = marker;
                    $invalid
                }
            }
        }

        #[inline(always)]
        fn read_f32(&mut self) -> Result<f32> {
            let marker = {
                let $byte_reader = &mut *self;
                $read_byte
            };
            if marker == FLOAT32_MARKER {
                Ok(f32::from_bits(u32::from_be_bytes({
                    let $reader_4 = &mut *self;
                    $read_4
                })))
            } else {
                {
                    let $reader = &mut *self;
                    let $bad_marker = marker;
                    $invalid
                }
            }
        }

        #[inline(always)]
        fn read_f64(&mut self) -> Result<f64> {
            let marker = {
                let $byte_reader = &mut *self;
                $read_byte
            };
            if marker == FLOAT64_MARKER {
                Ok(f64::from_bits(u64::from_be_bytes({
                    let $reader_8 = &mut *self;
                    $read_8
                })))
            } else {
                {
                    let $reader = &mut *self;
                    let $bad_marker = marker;
                    $invalid
                }
            }
        }

        #[inline(always)]
        fn read_array_len(&mut self) -> Result<usize> {
            let marker = {
                let $byte_reader = &mut *self;
                $read_byte
            };
            match marker {
                FIXARRAY_START..=FIXARRAY_END => Ok((marker - FIXARRAY_START) as usize),
                ARRAY16_MARKER => Ok(u16::from_be_bytes({
                    let $reader_2 = &mut *self;
                    $read_2
                }) as usize),
                ARRAY32_MARKER => Ok(u32::from_be_bytes({
                    let $reader_4 = &mut *self;
                    $read_4
                }) as usize),
                _ => {
                    let $reader = &mut *self;
                    let $bad_marker = marker;
                    $invalid
                }
            }
        }

        #[inline(always)]
        fn read_map_len(&mut self) -> Result<usize> {
            let marker = {
                let $byte_reader = &mut *self;
                $read_byte
            };
            match marker {
                FIXMAP_START..=FIXMAP_END => Ok((marker - FIXMAP_START) as usize),
                MAP16_MARKER => Ok(u16::from_be_bytes({
                    let $reader_2 = &mut *self;
                    $read_2
                }) as usize),
                MAP32_MARKER => Ok(u32::from_be_bytes({
                    let $reader_4 = &mut *self;
                    $read_4
                }) as usize),
                _ => {
                    let $reader = &mut *self;
                    let $bad_marker = marker;
                    $invalid
                }
            }
        }

        #[inline(always)]
        fn read_ext_len(&mut self) -> Result<(i8, usize)> {
            let marker = {
                let $byte_reader = &mut *self;
                $read_byte
            };
            let len = match marker {
                FIXEXT1_MARKER => 1,
                FIXEXT2_MARKER => 2,
                FIXEXT4_MARKER => 4,
                FIXEXT8_MARKER => 8,
                FIXEXT16_MARKER => 16,
                EXT8_MARKER => {
                    ({
                        let $byte_reader = &mut *self;
                        $read_byte
                    }) as usize
                }
                EXT16_MARKER => u16::from_be_bytes({
                    let $reader_2 = &mut *self;
                    $read_2
                }) as usize,
                EXT32_MARKER => u32::from_be_bytes({
                    let $reader_4 = &mut *self;
                    $read_4
                }) as usize,
                _ => {
                    return {
                        let $reader = &mut *self;
                        let $bad_marker = marker;
                        $invalid
                    }
                }
            };
            Ok((
                {
                    let $byte_reader = &mut *self;
                    $read_byte
                } as i8,
                len,
            ))
        }

        #[inline(always)]
        fn read_ext(&mut self) -> Result<(i8, alloc::borrow::Cow<'de, [u8]>)> {
            let (type_id, len) = self.read_ext_len()?;
            let bytes = {
                let $bytes_reader = &mut *self;
                let $bytes_len = len;
                $read_bytes
            };
            Ok((type_id, bytes))
        }

        #[inline(always)]
        fn read_string(&mut self) -> Result<alloc::borrow::Cow<'de, str>> {
            let marker = {
                let $byte_reader = &mut *self;
                $read_byte
            };
            let len = match marker {
                FIXSTR_START..=FIXSTR_END => (marker - FIXSTR_START) as usize,
                STR8_MARKER => {
                    let $byte_reader = &mut *self;
                    $read_byte as usize
                }
                STR16_MARKER => u16::from_be_bytes({
                    let $reader_2 = &mut *self;
                    $read_2
                }) as usize,
                STR32_MARKER => u32::from_be_bytes({
                    let $reader_4 = &mut *self;
                    $read_4
                }) as usize,
                _ => {
                    return {
                        let $reader = &mut *self;
                        let $bad_marker = marker;
                        $invalid
                    }
                }
            };
            let bytes = {
                let $bytes_reader = &mut *self;
                let $bytes_len = len;
                $read_bytes
            };
            match bytes {
                alloc::borrow::Cow::Borrowed(bytes) => core::str::from_utf8(bytes)
                    .map(alloc::borrow::Cow::Borrowed)
                    .map_err(Error::InvalidUtf8),
                alloc::borrow::Cow::Owned(bytes) => alloc::string::String::from_utf8(bytes)
                    .map(alloc::borrow::Cow::Owned)
                    .map_err(|error| Error::InvalidUtf8(error.utf8_error())),
            }
        }

        #[inline(always)]
        fn read_string_bytes(&mut self) -> Result<alloc::borrow::Cow<'de, [u8]>> {
            let marker = {
                let $byte_reader = &mut *self;
                $read_byte
            };
            let len = match marker {
                FIXSTR_START..=FIXSTR_END => (marker - FIXSTR_START) as usize,
                STR8_MARKER => {
                    let $byte_reader = &mut *self;
                    $read_byte as usize
                }
                STR16_MARKER => u16::from_be_bytes({
                    let $reader_2 = &mut *self;
                    $read_2
                }) as usize,
                STR32_MARKER => u32::from_be_bytes({
                    let $reader_4 = &mut *self;
                    $read_4
                }) as usize,
                _ => {
                    return {
                        let $reader = &mut *self;
                        let $bad_marker = marker;
                        $invalid
                    }
                }
            };
            let bytes = {
                let $bytes_reader = &mut *self;
                let $bytes_len = len;
                $read_bytes
            };
            Ok(bytes)
        }

        #[inline(always)]
        fn read_binary(&mut self) -> Result<alloc::borrow::Cow<'de, [u8]>> {
            let marker = {
                let $byte_reader = &mut *self;
                $read_byte
            };
            let len = match marker {
                BIN8_MARKER => {
                    let $byte_reader = &mut *self;
                    $read_byte as usize
                }
                BIN16_MARKER => u16::from_be_bytes({
                    let $reader_2 = &mut *self;
                    $read_2
                }) as usize,
                BIN32_MARKER => u32::from_be_bytes({
                    let $reader_4 = &mut *self;
                    $read_4
                }) as usize,
                _ => {
                    return {
                        let $reader = &mut *self;
                        let $bad_marker = marker;
                        $invalid
                    }
                }
            };
            let bytes = {
                let $bytes_reader = &mut *self;
                let $bytes_len = len;
                $read_bytes
            };
            Ok(bytes)
        }

        #[inline(always)]
        fn read_option<T: FromMessagePack<'de>>(&mut self) -> Result<Option<T>> {
            if self.peek_marker()? == NIL_MARKER {
                self.read_nil()?;
                Ok(None)
            } else {
                T::read(self).map(Some)
            }
        }

        #[inline(always)]
        fn read_timestamp(&mut self) -> Result<(i64, u32)> {
            let marker = {
                let $byte_reader = &mut *self;
                $read_byte
            };
            match marker {
                TIMESTAMP32_MARKER => {
                    let [ext, tail @ ..] = {
                        let $reader_5 = &mut *self;
                        $read_5
                    };
                    if ext as i8 != TIMESTAMP_EXT_TYPE {
                        return Err(Error::InvalidMarker(ext));
                    }
                    Ok((u32::from_be_bytes(tail) as i64, 0))
                }
                TIMESTAMP64_MARKER => {
                    let [ext, tail @ ..] = {
                        let $reader_9 = &mut *self;
                        $read_9
                    };
                    if ext as i8 != TIMESTAMP_EXT_TYPE {
                        return Err(Error::InvalidMarker(ext));
                    }
                    let data = u64::from_be_bytes(tail);
                    let nanoseconds = (data >> 34) as u32;
                    if nanoseconds >= 1_000_000_000 {
                        return Err(Error::InvalidTimestamp);
                    }
                    Ok(((data & 0x0000_0003_ffff_ffff) as i64, nanoseconds))
                }
                TIMESTAMP96_MARKER => {
                    let len = {
                        let $byte_reader = &mut *self;
                        $read_byte
                    };
                    if len != 12 {
                        return Err(Error::InvalidMarker(len));
                    }
                    let [ext, tail @ ..] = {
                        let $reader_13 = &mut *self;
                        $read_13
                    };
                    if ext as i8 != TIMESTAMP_EXT_TYPE {
                        return Err(Error::InvalidMarker(ext));
                    }
                    let nanoseconds = u32::from_be_bytes(tail[..4].try_into().unwrap());
                    if nanoseconds >= 1_000_000_000 {
                        return Err(Error::InvalidTimestamp);
                    }
                    let seconds = i64::from_be_bytes(tail[4..].try_into().unwrap());
                    Ok((seconds, nanoseconds))
                }
                _ => {
                    let $reader = &mut *self;
                    let $bad_marker = marker;
                    $invalid
                }
            }
        }

        #[inline(always)]
        fn read_tag(&mut self) -> Result<Tag<'de>> {
            let marker = {
                let $byte_reader = &mut *self;
                $read_byte
            };
            match marker {
                POS_FIXINT_START..=POS_FIXINT_END => Ok(Tag::Int(marker as u64)),
                UINT8_MARKER => Ok(Tag::Int({
                    let $byte_reader = &mut *self;
                    $read_byte
                } as u64)),
                UINT16_MARKER => Ok(Tag::Int(u16::from_be_bytes({
                    let $reader_2 = &mut *self;
                    $read_2
                }) as u64)),
                UINT32_MARKER => Ok(Tag::Int(u32::from_be_bytes({
                    let $reader_4 = &mut *self;
                    $read_4
                }) as u64)),
                UINT64_MARKER => Ok(Tag::Int(u64::from_be_bytes({
                    let $reader_8 = &mut *self;
                    $read_8
                }))),
                FIXSTR_START..=FIXSTR_END | STR8_MARKER | STR16_MARKER | STR32_MARKER => {
                    let len = match marker {
                        FIXSTR_START..=FIXSTR_END => (marker - FIXSTR_START) as usize,
                        STR8_MARKER => {
                            let $byte_reader = &mut *self;
                            $read_byte as usize
                        }
                        STR16_MARKER => u16::from_be_bytes({
                            let $reader_2 = &mut *self;
                            $read_2
                        }) as usize,
                        STR32_MARKER => u32::from_be_bytes({
                            let $reader_4 = &mut *self;
                            $read_4
                        }) as usize,
                        _ => unreachable!(),
                    };
                    let bytes = {
                        let $bytes_reader = &mut *self;
                        let $bytes_len = len;
                        $read_bytes
                    };
                    match bytes {
                        alloc::borrow::Cow::Borrowed(bytes) => core::str::from_utf8(bytes)
                            .map(|value| Tag::String(alloc::borrow::Cow::Borrowed(value)))
                            .map_err(Error::InvalidUtf8),
                        alloc::borrow::Cow::Owned(bytes) => alloc::string::String::from_utf8(bytes)
                            .map(|value| Tag::String(alloc::borrow::Cow::Owned(value)))
                            .map_err(|error| Error::InvalidUtf8(error.utf8_error())),
                    }
                }
                _ => {
                    let $reader = &mut *self;
                    let $bad_marker = marker;
                    $invalid
                }
            }
        }

        #[inline(always)]
        fn read_u8(&mut self) -> Result<u8> {
            let marker = {
                let $byte_reader = &mut *self;
                $read_byte
            };
            match marker {
                POS_FIXINT_START..=POS_FIXINT_END => Ok(marker),
                UINT8_MARKER => Ok({
                    let $byte_reader = &mut *self;
                    $read_byte
                }),
                _ => {
                    let $reader = &mut *self;
                    let $bad_marker = marker;
                    $invalid
                }
            }
        }

        #[inline(always)]
        fn read_u16(&mut self) -> Result<u16> {
            let marker = {
                let $byte_reader = &mut *self;
                $read_byte
            };
            match marker {
                POS_FIXINT_START..=POS_FIXINT_END => Ok(marker as u16),
                UINT8_MARKER => Ok({
                    let $byte_reader = &mut *self;
                    $read_byte
                } as u16),
                UINT16_MARKER => Ok(u16::from_be_bytes({
                    let $reader_2 = &mut *self;
                    $read_2
                })),
                _ => {
                    let $reader = &mut *self;
                    let $bad_marker = marker;
                    $invalid
                }
            }
        }

        #[inline(always)]
        fn read_u32(&mut self) -> Result<u32> {
            let marker = {
                let $byte_reader = &mut *self;
                $read_byte
            };
            match marker {
                POS_FIXINT_START..=POS_FIXINT_END => Ok(marker as u32),
                UINT8_MARKER => Ok({
                    let $byte_reader = &mut *self;
                    $read_byte
                } as u32),
                UINT16_MARKER => Ok(u16::from_be_bytes({
                    let $reader_2 = &mut *self;
                    $read_2
                }) as u32),
                UINT32_MARKER => Ok(u32::from_be_bytes({
                    let $reader_4 = &mut *self;
                    $read_4
                })),
                _ => {
                    let $reader = &mut *self;
                    let $bad_marker = marker;
                    $invalid
                }
            }
        }

        #[inline(always)]
        fn read_u64(&mut self) -> Result<u64> {
            let marker = {
                let $byte_reader = &mut *self;
                $read_byte
            };
            match marker {
                POS_FIXINT_START..=POS_FIXINT_END => Ok(marker as u64),
                UINT8_MARKER => Ok({
                    let $byte_reader = &mut *self;
                    $read_byte
                } as u64),
                UINT16_MARKER => Ok(u16::from_be_bytes({
                    let $reader_2 = &mut *self;
                    $read_2
                }) as u64),
                UINT32_MARKER => Ok(u32::from_be_bytes({
                    let $reader_4 = &mut *self;
                    $read_4
                }) as u64),
                UINT64_MARKER => Ok(u64::from_be_bytes({
                    let $reader_8 = &mut *self;
                    $read_8
                })),
                _ => {
                    let $reader = &mut *self;
                    let $bad_marker = marker;
                    $invalid
                }
            }
        }

        #[inline(always)]
        fn read_i8(&mut self) -> Result<i8> {
            let marker = {
                let $byte_reader = &mut *self;
                $read_byte
            };
            match marker {
                POS_FIXINT_START..=POS_FIXINT_END | NEG_FIXINT_START..=NEG_FIXINT_END => {
                    Ok(marker as i8)
                }
                INT8_MARKER => Ok({
                    let $byte_reader = &mut *self;
                    $read_byte
                } as i8),
                _ => {
                    let $reader = &mut *self;
                    let $bad_marker = marker;
                    $invalid
                }
            }
        }

        #[inline(always)]
        fn read_i16(&mut self) -> Result<i16> {
            let marker = {
                let $byte_reader = &mut *self;
                $read_byte
            };
            match marker {
                POS_FIXINT_START..=POS_FIXINT_END => Ok(marker as i16),
                NEG_FIXINT_START..=NEG_FIXINT_END => Ok(marker as i8 as i16),
                INT8_MARKER => Ok({
                    let $byte_reader = &mut *self;
                    $read_byte
                } as i8 as i16),
                INT16_MARKER => Ok(i16::from_be_bytes({
                    let $reader_2 = &mut *self;
                    $read_2
                })),
                _ => {
                    let $reader = &mut *self;
                    let $bad_marker = marker;
                    $invalid
                }
            }
        }

        #[inline(always)]
        fn read_i32(&mut self) -> Result<i32> {
            let marker = {
                let $byte_reader = &mut *self;
                $read_byte
            };
            match marker {
                POS_FIXINT_START..=POS_FIXINT_END => Ok(marker as i32),
                NEG_FIXINT_START..=NEG_FIXINT_END => Ok(marker as i8 as i32),
                INT8_MARKER => Ok({
                    let $byte_reader = &mut *self;
                    $read_byte
                } as i8 as i32),
                INT16_MARKER => Ok(i16::from_be_bytes({
                    let $reader_2 = &mut *self;
                    $read_2
                }) as i32),
                INT32_MARKER => Ok(i32::from_be_bytes({
                    let $reader_4 = &mut *self;
                    $read_4
                })),
                _ => {
                    let $reader = &mut *self;
                    let $bad_marker = marker;
                    $invalid
                }
            }
        }

        #[inline(always)]
        fn read_i64(&mut self) -> Result<i64> {
            let marker = {
                let $byte_reader = &mut *self;
                $read_byte
            };
            match marker {
                POS_FIXINT_START..=POS_FIXINT_END => Ok(marker as i64),
                NEG_FIXINT_START..=NEG_FIXINT_END => Ok(marker as i8 as i64),
                INT8_MARKER => Ok({
                    let $byte_reader = &mut *self;
                    $read_byte
                } as i8 as i64),
                INT16_MARKER => Ok(i16::from_be_bytes({
                    let $reader_2 = &mut *self;
                    $read_2
                }) as i64),
                INT32_MARKER => Ok(i32::from_be_bytes({
                    let $reader_4 = &mut *self;
                    $read_4
                }) as i64),
                INT64_MARKER => Ok(i64::from_be_bytes({
                    let $reader_8 = &mut *self;
                    $read_8
                })),
                _ => {
                    let $reader = &mut *self;
                    let $bad_marker = marker;
                    $invalid
                }
            }
        }
    };
}
