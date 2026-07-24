macro_rules! impl_write_methods {
    (
        write = |$writer:ident, $data:ident| $write:expr,
        write_parts = |$parts_writer:ident, $header:ident, $payload:ident| $write_parts:expr,
        write_container =
            |$container_writer:ident, $container_header:ident, $reserve:ident| $write_container:expr
        $(,)?
    ) => {
        #[inline(always)]
        fn write_nil(&mut self) -> Result<()> {
            let bytes = [NIL_MARKER];
            let $writer = &mut *self;
            let $data = bytes.as_slice();
            $write
        }

        #[inline(always)]
        fn write_boolean(&mut self, value: bool) -> Result<()> {
            let bytes = [if value { TRUE_MARKER } else { FALSE_MARKER }];
            let $writer = &mut *self;
            let $data = bytes.as_slice();
            $write
        }

        #[inline(always)]
        fn write_u8(&mut self, value: u8) -> Result<()> {
            let bytes = [UINT8_MARKER, value];
            let len = if value <= POS_FIXINT_END { 1 } else { 2 };
            let $writer = &mut *self;
            let $data = if len == 1 { &bytes[1..] } else { &bytes };
            $write
        }

        #[inline(always)]
        fn write_u16(&mut self, value: u16) -> Result<()> {
            if value <= u8::MAX as u16 {
                return self.write_u8(value as u8);
            }
            let [a, b] = value.to_be_bytes();
            let bytes = [UINT16_MARKER, a, b];
            let $writer = &mut *self;
            let $data = bytes.as_slice();
            $write
        }

        #[inline(always)]
        fn write_u32(&mut self, value: u32) -> Result<()> {
            if value <= u16::MAX as u32 {
                return self.write_u16(value as u16);
            }
            let [a, b, c, d] = value.to_be_bytes();
            let bytes = [UINT32_MARKER, a, b, c, d];
            let $writer = &mut *self;
            let $data = bytes.as_slice();
            $write
        }

        #[inline(always)]
        fn write_u64(&mut self, value: u64) -> Result<()> {
            if value <= u32::MAX as u64 {
                return self.write_u32(value as u32);
            }
            let [a, b, c, d, e, f, g, h] = value.to_be_bytes();
            let bytes = [UINT64_MARKER, a, b, c, d, e, f, g, h];
            let $writer = &mut *self;
            let $data = bytes.as_slice();
            $write
        }

        #[inline(always)]
        fn write_i8(&mut self, value: i8) -> Result<()> {
            let bytes = [INT8_MARKER, value as u8];
            let $writer = &mut *self;
            let $data = if (-32..=127).contains(&value) {
                &bytes[1..]
            } else {
                &bytes
            };
            $write
        }

        #[inline(always)]
        fn write_i16(&mut self, value: i16) -> Result<()> {
            if (i8::MIN as i16..=i8::MAX as i16).contains(&value) {
                return self.write_i8(value as i8);
            }
            let [a, b] = value.to_be_bytes();
            let bytes = [INT16_MARKER, a, b];
            let $writer = &mut *self;
            let $data = bytes.as_slice();
            $write
        }

        #[inline(always)]
        fn write_i32(&mut self, value: i32) -> Result<()> {
            if (i16::MIN as i32..=i16::MAX as i32).contains(&value) {
                return self.write_i16(value as i16);
            }
            let [a, b, c, d] = value.to_be_bytes();
            let bytes = [INT32_MARKER, a, b, c, d];
            let $writer = &mut *self;
            let $data = bytes.as_slice();
            $write
        }

        #[inline(always)]
        fn write_i64(&mut self, value: i64) -> Result<()> {
            if (i32::MIN as i64..=i32::MAX as i64).contains(&value) {
                return self.write_i32(value as i32);
            }
            let [a, b, c, d, e, f, g, h] = value.to_be_bytes();
            let bytes = [INT64_MARKER, a, b, c, d, e, f, g, h];
            let $writer = &mut *self;
            let $data = bytes.as_slice();
            $write
        }

        #[inline(always)]
        fn write_f32(&mut self, value: f32) -> Result<()> {
            let [a, b, c, d] = value.to_be_bytes();
            let bytes = [FLOAT32_MARKER, a, b, c, d];
            let $writer = &mut *self;
            let $data = bytes.as_slice();
            $write
        }

        #[inline(always)]
        fn write_f64(&mut self, value: f64) -> Result<()> {
            let [a, b, c, d, e, f, g, h] = value.to_be_bytes();
            let bytes = [FLOAT64_MARKER, a, b, c, d, e, f, g, h];
            let $writer = &mut *self;
            let $data = bytes.as_slice();
            $write
        }

        #[inline(always)]
        fn write_timestamp(&mut self, seconds: i64, nanoseconds: u32) -> Result<()> {
            if nanoseconds >= 1_000_000_000 {
                return Err(Error::InvalidTimestamp);
            }
            if nanoseconds == 0 && (0..=u32::MAX as i64).contains(&seconds) {
                let [a, b, c, d] = (seconds as u32).to_be_bytes();
                let bytes = [TIMESTAMP32_MARKER, TIMESTAMP_EXT_TYPE as u8, a, b, c, d];
                let $writer = &mut *self;
                let $data = bytes.as_slice();
                return $write;
            }
            if (0..=(1i64 << 34) - 1).contains(&seconds) {
                let value = ((nanoseconds as u64) << 34) | seconds as u64;
                let [a, b, c, d, e, f, g, h] = value.to_be_bytes();
                let bytes = [
                    TIMESTAMP64_MARKER,
                    TIMESTAMP_EXT_TYPE as u8,
                    a,
                    b,
                    c,
                    d,
                    e,
                    f,
                    g,
                    h,
                ];
                let $writer = &mut *self;
                let $data = bytes.as_slice();
                return $write;
            }
            let [a, b, c, d] = nanoseconds.to_be_bytes();
            let [e, f, g, h, i, j, k, l] = seconds.to_be_bytes();
            let bytes = [
                TIMESTAMP96_MARKER,
                12,
                TIMESTAMP_EXT_TYPE as u8,
                a,
                b,
                c,
                d,
                e,
                f,
                g,
                h,
                i,
                j,
                k,
                l,
            ];
            let $writer = &mut *self;
            let $data = bytes.as_slice();
            $write
        }

        #[inline(always)]
        fn write_string(&mut self, value: &str) -> Result<()> {
            let len = value.len();
            let mut header = [0; 5];
            let header_len = match len {
                0..=31 => {
                    header[0] = FIXSTR_START | len as u8;
                    1
                }
                32..=255 => {
                    header[..2].copy_from_slice(&[STR8_MARKER, len as u8]);
                    2
                }
                256..=65535 => {
                    header[0] = STR16_MARKER;
                    header[1..3].copy_from_slice(&(len as u16).to_be_bytes());
                    3
                }
                _ => {
                    header[0] = STR32_MARKER;
                    header[1..].copy_from_slice(&(len as u32).to_be_bytes());
                    5
                }
            };
            let $parts_writer = &mut *self;
            let $header = &header[..header_len];
            let $payload = value.as_bytes();
            $write_parts
        }

        #[inline(always)]
        fn write_binary(&mut self, value: &[u8]) -> Result<()> {
            let len = value.len();
            let mut header = [0; 5];
            let header_len = match len {
                0..=255 => {
                    header[..2].copy_from_slice(&[BIN8_MARKER, len as u8]);
                    2
                }
                256..=65535 => {
                    header[0] = BIN16_MARKER;
                    header[1..3].copy_from_slice(&(len as u16).to_be_bytes());
                    3
                }
                _ => {
                    header[0] = BIN32_MARKER;
                    header[1..].copy_from_slice(&(len as u32).to_be_bytes());
                    5
                }
            };
            let $parts_writer = &mut *self;
            let $header = &header[..header_len];
            let $payload = value;
            $write_parts
        }

        #[inline(always)]
        fn write_ext(&mut self, type_id: i8, value: &[u8]) -> Result<()> {
            let len = value.len();
            let mut header = [0; 6];
            let header_len = match len {
                1 => {
                    header[..2].copy_from_slice(&[FIXEXT1_MARKER, type_id as u8]);
                    2
                }
                2 => {
                    header[..2].copy_from_slice(&[FIXEXT2_MARKER, type_id as u8]);
                    2
                }
                4 => {
                    header[..2].copy_from_slice(&[FIXEXT4_MARKER, type_id as u8]);
                    2
                }
                8 => {
                    header[..2].copy_from_slice(&[FIXEXT8_MARKER, type_id as u8]);
                    2
                }
                16 => {
                    header[..2].copy_from_slice(&[FIXEXT16_MARKER, type_id as u8]);
                    2
                }
                0..=255 => {
                    header[..3].copy_from_slice(&[EXT8_MARKER, len as u8, type_id as u8]);
                    3
                }
                256..=65535 => {
                    header[0] = EXT16_MARKER;
                    header[1..3].copy_from_slice(&(len as u16).to_be_bytes());
                    header[3] = type_id as u8;
                    4
                }
                _ => {
                    header[0] = EXT32_MARKER;
                    header[1..5].copy_from_slice(&(len as u32).to_be_bytes());
                    header[5] = type_id as u8;
                    6
                }
            };
            let $parts_writer = &mut *self;
            let $header = &header[..header_len];
            let $payload = value;
            $write_parts
        }

        #[inline(always)]
        fn write_array_len(&mut self, len: usize) -> Result<()> {
            let mut header = [0; 5];
            let header_len = match len {
                0..=15 => {
                    header[0] = FIXARRAY_START | len as u8;
                    1
                }
                16..=65535 => {
                    header[0] = ARRAY16_MARKER;
                    header[1..3].copy_from_slice(&(len as u16).to_be_bytes());
                    3
                }
                _ => {
                    header[0] = ARRAY32_MARKER;
                    header[1..].copy_from_slice(&(len as u32).to_be_bytes());
                    5
                }
            };
            let $container_writer = &mut *self;
            let $container_header = &header[..header_len];
            let $reserve = len.min(MAX_CONTAINER_PREALLOC);
            $write_container
        }

        #[inline(always)]
        fn write_map_len(&mut self, len: usize) -> Result<()> {
            let mut header = [0; 5];
            let header_len = match len {
                0..=15 => {
                    header[0] = FIXMAP_START | len as u8;
                    1
                }
                16..=65535 => {
                    header[0] = MAP16_MARKER;
                    header[1..3].copy_from_slice(&(len as u16).to_be_bytes());
                    3
                }
                _ => {
                    header[0] = MAP32_MARKER;
                    header[1..].copy_from_slice(&(len as u32).to_be_bytes());
                    5
                }
            };
            let $container_writer = &mut *self;
            let $container_header = &header[..header_len];
            let $reserve = len.saturating_mul(2).min(MAX_CONTAINER_PREALLOC);
            $write_container
        }
    };
}
