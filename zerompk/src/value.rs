use alloc::borrow::Cow;
use alloc::vec::Vec;

use crate::consts::*;
use crate::{Error, FromMessagePack, Read, Result, ToMessagePack, Write};

/// A schema-less MessagePack value.
#[derive(Clone, Debug, PartialEq)]
pub enum Value<'de> {
    Nil,
    Boolean(bool),
    Unsigned(u64),
    Signed(i64),
    Float32(f32),
    Float64(f64),
    String(Cow<'de, str>),
    Binary(Cow<'de, [u8]>),
    Array(Vec<Self>),
    Map(Vec<(Self, Self)>),
    Extension(i8, Cow<'de, [u8]>),
}

enum Frame<'de> {
    Array {
        remaining: usize,
        values: Vec<Value<'de>>,
    },
    Map {
        remaining: usize,
        entries: Vec<(Value<'de>, Value<'de>)>,
        key: Option<Value<'de>>,
    },
}

fn read_scalar<'de, R: Read<'de>>(reader: &mut R, marker: u8) -> Result<Value<'de>> {
    match marker {
        NIL_MARKER => {
            reader.read_nil()?;
            Ok(Value::Nil)
        }
        FALSE_MARKER | TRUE_MARKER => reader.read_boolean().map(Value::Boolean),
        POS_FIXINT_START..=POS_FIXINT_END
        | UINT8_MARKER
        | UINT16_MARKER
        | UINT32_MARKER
        | UINT64_MARKER => reader.read_u64().map(Value::Unsigned),
        NEG_FIXINT_START..=NEG_FIXINT_END
        | INT8_MARKER
        | INT16_MARKER
        | INT32_MARKER
        | INT64_MARKER => reader.read_i64().map(Value::Signed),
        FLOAT32_MARKER => reader.read_f32().map(Value::Float32),
        FLOAT64_MARKER => reader.read_f64().map(Value::Float64),
        FIXSTR_START..=FIXSTR_END | STR8_MARKER | STR16_MARKER | STR32_MARKER => {
            reader.read_string().map(Value::String)
        }
        BIN8_MARKER | BIN16_MARKER | BIN32_MARKER => reader.read_binary().map(Value::Binary),
        FIXEXT1_MARKER | FIXEXT2_MARKER | FIXEXT4_MARKER | FIXEXT8_MARKER | FIXEXT16_MARKER
        | EXT8_MARKER | EXT16_MARKER | EXT32_MARKER => reader
            .read_ext()
            .map(|(type_id, data)| Value::Extension(type_id, data)),
        _ => Err(Error::InvalidMarker(marker)),
    }
}

impl<'de> FromMessagePack<'de> for Value<'de> {
    fn read<R: Read<'de>>(reader: &mut R) -> Result<Self> {
        let mut stack = Vec::<Frame<'de>>::new();
        let result = (|| {
            let mut value = None;
            loop {
                if let Some(completed) = value.take() {
                    let Some(frame) = stack.last_mut() else {
                        return Ok(completed);
                    };
                    match frame {
                        Frame::Array { remaining, values } => {
                            values.push(completed);
                            *remaining -= 1;
                            if *remaining == 0 {
                                let Frame::Array { values, .. } = stack.pop().unwrap() else {
                                    unreachable!()
                                };
                                reader.decrement_depth();
                                value = Some(Value::Array(values));
                            }
                        }
                        Frame::Map {
                            remaining,
                            entries,
                            key,
                        } => {
                            if let Some(key) = key.take() {
                                entries.push((key, completed));
                                *remaining -= 1;
                                if *remaining == 0 {
                                    let Frame::Map { entries, .. } = stack.pop().unwrap() else {
                                        unreachable!()
                                    };
                                    reader.decrement_depth();
                                    value = Some(Value::Map(entries));
                                }
                            } else {
                                *key = Some(completed);
                            }
                        }
                    }
                    continue;
                }

                let marker = reader.peek_marker()?;
                match marker {
                    FIXARRAY_START..=FIXARRAY_END | ARRAY16_MARKER | ARRAY32_MARKER => {
                        reader.increment_depth()?;
                        let len = match reader.read_array_len() {
                            Ok(len) => len,
                            Err(error) => {
                                reader.decrement_depth();
                                return Err(error);
                            }
                        };
                        if len == 0 {
                            reader.decrement_depth();
                            value = Some(Value::Array(Vec::new()));
                        } else {
                            stack.push(Frame::Array {
                                remaining: len,
                                values: Vec::with_capacity(len),
                            });
                        }
                    }
                    FIXMAP_START..=FIXMAP_END | MAP16_MARKER | MAP32_MARKER => {
                        reader.increment_depth()?;
                        let len = match reader.read_map_len() {
                            Ok(len) => len,
                            Err(error) => {
                                reader.decrement_depth();
                                return Err(error);
                            }
                        };
                        if len == 0 {
                            reader.decrement_depth();
                            value = Some(Value::Map(Vec::new()));
                        } else {
                            stack.push(Frame::Map {
                                remaining: len,
                                entries: Vec::with_capacity(len),
                                key: None,
                            });
                        }
                    }
                    _ => value = Some(read_scalar(reader, marker)?),
                }
            }
        })();

        for _ in 0..stack.len() {
            reader.decrement_depth();
        }
        result
    }
}

impl ToMessagePack for Value<'_> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        match self {
            Self::Nil => writer.write_nil(),
            Self::Boolean(value) => writer.write_boolean(*value),
            Self::Unsigned(value) => writer.write_u64(*value),
            Self::Signed(value) => writer.write_i64(*value),
            Self::Float32(value) => writer.write_f32(*value),
            Self::Float64(value) => writer.write_f64(*value),
            Self::String(value) => writer.write_string(value),
            Self::Binary(value) => writer.write_binary(value),
            Self::Array(values) => {
                writer.write_array_len(values.len())?;
                for value in values {
                    value.write(writer)?;
                }
                Ok(())
            }
            Self::Map(entries) => {
                writer.write_map_len(entries.len())?;
                for (key, value) in entries {
                    key.write(writer)?;
                    value.write(writer)?;
                }
                Ok(())
            }
            Self::Extension(type_id, data) => writer.write_ext(*type_id, data),
        }
    }
}
