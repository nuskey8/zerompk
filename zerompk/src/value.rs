use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::fmt;

use crate::consts::*;
use crate::{Error, FromMessagePack, Read, Result, ToMessagePack, Write};

/// A schema-less MessagePack value.
#[derive(Clone, PartialEq)]
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

impl fmt::Debug for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nil => f.write_str("null"),
            Self::Boolean(value) => value.fmt(f),
            Self::Unsigned(value) => value.fmt(f),
            Self::Signed(value) => value.fmt(f),
            Self::Float32(value) => value.fmt(f),
            Self::Float64(value) => value.fmt(f),
            Self::String(value) => value.fmt(f),
            Self::Binary(value) => value.fmt(f),
            Self::Array(values) => f.debug_list().entries(values).finish(),
            Self::Map(entries) => {
                let mut map = f.debug_map();
                for (key, value) in entries {
                    map.entry(key, value);
                }
                map.finish()
            }
            Self::Extension(type_id, data) => f
                .debug_map()
                .entry(&"type", type_id)
                .entry(&"data", &data.as_ref())
                .finish(),
        }
    }
}

impl From<()> for Value<'_> {
    fn from((): ()) -> Self {
        Self::Nil
    }
}

impl From<bool> for Value<'_> {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

macro_rules! impl_from_unsigned {
    ($($type:ty),* $(,)?) => {
        $(
            impl From<$type> for Value<'_> {
                fn from(value: $type) -> Self {
                    Self::Unsigned(value as u64)
                }
            }
        )*
    };
}

macro_rules! impl_from_signed {
    ($($type:ty),* $(,)?) => {
        $(
            impl From<$type> for Value<'_> {
                fn from(value: $type) -> Self {
                    Self::Signed(value as i64)
                }
            }
        )*
    };
}

impl_from_unsigned!(u8, u16, u32, u64, usize);
impl_from_signed!(i8, i16, i32, i64, isize);

impl From<f32> for Value<'_> {
    fn from(value: f32) -> Self {
        Self::Float32(value)
    }
}

impl From<f64> for Value<'_> {
    fn from(value: f64) -> Self {
        Self::Float64(value)
    }
}

impl<'de> From<&'de str> for Value<'de> {
    fn from(value: &'de str) -> Self {
        Self::String(Cow::Borrowed(value))
    }
}

impl From<alloc::string::String> for Value<'_> {
    fn from(value: alloc::string::String) -> Self {
        Self::String(Cow::Owned(value))
    }
}

impl<'de> From<Cow<'de, str>> for Value<'de> {
    fn from(value: Cow<'de, str>) -> Self {
        Self::String(value)
    }
}

impl<'de> From<&'de [u8]> for Value<'de> {
    fn from(value: &'de [u8]) -> Self {
        Self::Binary(Cow::Borrowed(value))
    }
}

impl From<Vec<u8>> for Value<'_> {
    fn from(value: Vec<u8>) -> Self {
        Self::Binary(Cow::Owned(value))
    }
}

impl<'de> From<Cow<'de, [u8]>> for Value<'de> {
    fn from(value: Cow<'de, [u8]>) -> Self {
        Self::Binary(value)
    }
}

impl<'de> From<Vec<Value<'de>>> for Value<'de> {
    fn from(value: Vec<Value<'de>>) -> Self {
        Self::Array(value)
    }
}

impl<'de> From<Vec<(Value<'de>, Value<'de>)>> for Value<'de> {
    fn from(value: Vec<(Value<'de>, Value<'de>)>) -> Self {
        Self::Map(value)
    }
}

impl<'de, T> From<Option<T>> for Value<'de>
where
    Value<'de>: From<T>,
{
    fn from(value: Option<T>) -> Self {
        value.map_or(Self::Nil, Self::from)
    }
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
                                values: Vec::with_capacity(len.min(32)),
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
                                entries: Vec::with_capacity(len.min(32)),
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
