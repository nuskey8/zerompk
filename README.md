# zerompk

A zero-copy, zero-dependency, no_std-compatible, extremely fast MessagePack serializer for Rust.

[![Crates.io version](https://img.shields.io/crates/v/zerompk.svg?style=flat-square)](https://crates.io/crates/zerompk)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/zerompk)

![bench](docs/images/bench.png)

## Overview

zerompk is a high-performance MessagePack serializer for Rust. Compared to [rmp_serde](https://github.com/3Hren/msgpack-rust), it operates approximately 1.3 to 3.0 times faster and is implemented without relying on any libraries, including `std`.

## Quick Start

```rust
use zerompk::{FromMessagePack, ToMessagePack};

#[derive(FromMessagePack, ToMessagePack)]
pub struct Person {
    pub name: String,
    pub age: u32,
}

fn main() {
    let person = Person {
        name: "Alice".to_string(),
        age: 18,
    };
    
    let msgpack: Vec<u8> = zerompk::to_msgpack_vec(&person).unwrap();
    let person: Person = zerompk::from_msgpack(&msgpack).unwrap();
}
```

## Format

The correspondence between Rust types and MessagePack types in zerompk is as follows. Since MessagePack uses variable-length encoding, zerompk serializes values into the smallest possible type based on their size.

| Rust Type                                                                                      | MessagePack Type                                                            |
| ---------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| `bool`                                                                                         | `true`, `false`                                                             |
| `u8`, `u16`, `u32`, `u64`, `usize`                                                             | `positive fixint`, `uint 8`, `uint 16`, `uint 32`, `uint 64`                |
| `i8`, `i16`, `i32`, `i64`, `isize`                                                             | `positive fixint`, `negative fixint`, `int 8`, `int 16`, `int 32`, `int 64` |
| `f32`, `f64`                                                                                   | `float 32`, `float 64`                                                      |
| `str`, `String`                                                                                | `fixstr`, `str 8`, `str 16`, `str 32`                                       |
| `[u8]`                                                                                         | `bin 8`, `bin 16`, `bin 32`                                                 |
| `&[T]`, `Vec<T>`, `VecDeque<T>`, `LinkedList<T>`, `HashSet<T>`, `BTreeSet<T>`, `BinaryHeap<T>` | `fixarray`, `array 16`, `array 32`                                          |
| `HashMap<K, V>`, `BTreeMap<K, V>`                                                              | `fixmap`, `map 16`, `map 32`                                                |
| `()`                                                                                           | `nil`                                                                       |
| `Option<T>`                                                                                    | `nil` (`None`) or `T` (`Some(T)`)                                           |
| `(T0, T1)`, `(T0, T1, T2)`, ...                                                                | `fixarray`, `array 16`, `array 32`                                          |
| `DateTime<Utc>`, `NaiveDateTime` (chrono)                                                      | `timestamp 32`, `timestamp 64`, `timestamp 96` (ext -1)                     |
| struct (default, with `#[msgpack(array)]`)                                                     | `fixarray`, `array 16`, `array 32`                                          |
| struct (with `#[msgpack(map)]`)                                                                | `fixmap`, `map 16`, `map 32`                                                |
| enum (default)                                                                                 | `fixarray` (`[tag, value]`)                                                 |
| enum (with `#[msgpack(c_enum)]`)                                                               | `positive fixint`, `uint 8`, `uint 16`, `uint 32`, `uint 64`                |

## derive

By enabling the `derive` feature flag, you can implement `FromMessagePack`/`ToMessagePack` using the `derive` macro.

```rust
#[derive(FromMessagePack, ToMessagePack)]
pub struct Person {
    pub name: String,
    pub age: u32,
}
```

You can also customize the serialization format using the `#[msgpack]` attribute.

### array/map

The serialization format of structs and enum variants can be chosen from `array` or `map`. For performance reasons, the default is set to `array`.

```rust
#[derive(FromMessagePack, ToMessagePack)]
#[msgpack(array)] // default
pub struct PersonArray {
    pub name: String,
    pub age: u32,
}

#[derive(FromMessagePack, ToMessagePack)]
#[msgpack(map)]
pub struct PersonMap {
    pub name: String,
    pub age: u32,
}
```

### key

You can override the index/key used for fields or enum variants. For `array`, integers are used, and for `map`, strings are used. If the format is `array` and there are gaps in the indices, `nil` is automatically inserted.

```rust
#[derive(FromMessagePack, ToMessagePack)]
pub struct Person {
    #[msgpack(key = 0)]
    pub name: String,

    #[msgpack(key = 2)]
    pub age: u32,
}
```

> [!NOTE]
> To enhance versioning resilience, it is recommended to explicitly set keys whenever possible.

### ignore

Set `ignore` for fields you want to exclude during serialization/deserialization. When deserializing a struct with `ignore`, the type of the `ignore` field must implement `Default`.

```rust
#[derive(FromMessagePack, ToMessagePack)]
pub struct Person {
    pub name: String,
    pub age: u32,

    #[msgpack(ignore)]
    pub meta: Metadata,
}
```

### c_enum

You can serialize a C-style enum as an integer by adding `#[msgpack(c_enum)]` to it. The value is the discriminant for each variant.

```rust
#[derive(FromMessagePack, ToMessagePack)]
#[msgpack(c_enum)]
#[repr(u8)]
pub enum Status {
    Ok = 0,
    NotFound = 4,
    InternalServerError = 5,
}
```

## Design Philosophy

The most popular MessagePack serializer, [rmp](https://github.com/3Hren/msgpack-rust), is highly optimized, but zerompk is designed with an even greater focus on performance.

### No Serde

Serde is an excellent abstraction layer for serializers, but it comes with a (slight but non-negligible for serializers) performance cost. Since zerompk is a serializer specialized for MessagePack, it does not use Serde traits.

For example, let's compare the generated code for the following derive macro:

```rust
use serde::{Deserialize, Serialize};
use zerompk::{FromMessagePack, ToMessagePack};

#[derive(Serialize, Deserialize, FromMessagePack, ToMessagePack)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
```

<details>

<summary>Serde</summary>

```rust
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths,
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for Point {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private228::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = _serde::Serializer::serialize_struct(
                __serializer,
                "Point",
                false as usize + 1 + 1,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "x",
                &self.x,
            )?;
            _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "y",
                &self.y,
            )?;
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths,
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for Point {
        fn deserialize<__D>(
            __deserializer: __D,
        ) -> _serde::__private228::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __field1,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private228::Formatter,
                ) -> _serde::__private228::fmt::Result {
                    _serde::__private228::Formatter::write_str(
                        __formatter,
                        "field identifier",
                    )
                }
                fn visit_u64<__E>(
                    self,
                    __value: u64,
                ) -> _serde::__private228::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private228::Ok(__Field::__field0),
                        1u64 => _serde::__private228::Ok(__Field::__field1),
                        _ => _serde::__private228::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private228::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "x" => _serde::__private228::Ok(__Field::__field0),
                        "y" => _serde::__private228::Ok(__Field::__field1),
                        _ => _serde::__private228::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private228::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"x" => _serde::__private228::Ok(__Field::__field0),
                        b"y" => _serde::__private228::Ok(__Field::__field1),
                        _ => _serde::__private228::Ok(__Field::__ignore),
                    }
                }
            }
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private228::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(
                        __deserializer,
                        __FieldVisitor,
                    )
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private228::PhantomData<Point>,
                lifetime: _serde::__private228::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = Point;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private228::Formatter,
                ) -> _serde::__private228::fmt::Result {
                    _serde::__private228::Formatter::write_str(
                        __formatter,
                        "struct Point",
                    )
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private228::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<
                        i32,
                    >(&mut __seq)? {
                        _serde::__private228::Some(__value) => __value,
                        _serde::__private228::None => {
                            return _serde::__private228::Err(
                                _serde::de::Error::invalid_length(
                                    0usize,
                                    &"struct Point with 2 elements",
                                ),
                            );
                        }
                    };
                    let __field1 = match _serde::de::SeqAccess::next_element::<
                        i32,
                    >(&mut __seq)? {
                        _serde::__private228::Some(__value) => __value,
                        _serde::__private228::None => {
                            return _serde::__private228::Err(
                                _serde::de::Error::invalid_length(
                                    1usize,
                                    &"struct Point with 2 elements",
                                ),
                            );
                        }
                    };
                    _serde::__private228::Ok(Point { x: __field0, y: __field1 })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private228::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private228::Option<i32> = _serde::__private228::None;
                    let mut __field1: _serde::__private228::Option<i32> = _serde::__private228::None;
                    while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                        __Field,
                    >(&mut __map)? {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private228::Option::is_some(&__field0) {
                                    return _serde::__private228::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("x"),
                                    );
                                }
                                __field0 = _serde::__private228::Some(
                                    _serde::de::MapAccess::next_value::<i32>(&mut __map)?,
                                );
                            }
                            __Field::__field1 => {
                                if _serde::__private228::Option::is_some(&__field1) {
                                    return _serde::__private228::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("y"),
                                    );
                                }
                                __field1 = _serde::__private228::Some(
                                    _serde::de::MapAccess::next_value::<i32>(&mut __map)?,
                                );
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<
                                    _serde::de::IgnoredAny,
                                >(&mut __map)?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private228::Some(__field0) => __field0,
                        _serde::__private228::None => {
                            _serde::__private228::de::missing_field("x")?
                        }
                    };
                    let __field1 = match __field1 {
                        _serde::__private228::Some(__field1) => __field1,
                        _serde::__private228::None => {
                            _serde::__private228::de::missing_field("y")?
                        }
                    };
                    _serde::__private228::Ok(Point { x: __field0, y: __field1 })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["x", "y"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "Point",
                FIELDS,
                __Visitor {
                    marker: _serde::__private228::PhantomData::<Point>,
                    lifetime: _serde::__private228::PhantomData,
                },
            )
        }
    }
};
```

</details>

<details>

<summary>zerompk</summary>

```rust
impl ::zerompk::ToMessagePack for Point {
    fn write<W: ::zerompk::Write>(
        &self,
        writer: &mut W,
    ) -> ::core::result::Result<(), ::zerompk::Error> {
        writer.write_array_len(2usize)?;
        self.x.write(writer)?;
        self.y.write(writer)?;
        Ok(())
    }
}

impl<'__msgpack_de> ::zerompk::FromMessagePack<'__msgpack_de> for Point {
    fn read<R: ::zerompk::Read<'__msgpack_de>>(
        reader: &mut R,
    ) -> ::core::result::Result<Self, ::zerompk::Error>
    where
        Self: Sized,
    {
        reader.increment_depth()?;
        let __result = {
            reader.check_array_len(2usize)?;
            Ok(Self {
                x: <i32 as ::zerompk::FromMessagePack<'__msgpack_de>>::read(reader)?,
                y: <i32 as ::zerompk::FromMessagePack<'__msgpack_de>>::read(reader)?,
            })
        };
        reader.decrement_depth();
        __result
    }
}
```

</details>

Compared to the complex visitor generated by Serde, zerompk's code is extremely simple. This not only benefits runtime performance but also reduces binary size and compile time as a side effect.

### Zero Copy

Like Serde, zerompk supports zero-copy deserialization, directly referencing the original serialized data.

```rust
#[derive(ToMessagePack, FromMessagePack)]
pub struct NoCopy<'a> {
    pub str: &'a str,
    pub bin: &'a [u8],
}

fn main() -> zerompk::Result<()> {
    let value = NoCopy {
        str: "hello",
        bin: &[0x01, 0x02, 0x03],
    };
    let msgpack = zerompk::to_msgpack_vec(&value)?;
    let value: NoCopy = zerompk::from_msgpack(&msgpack)?;
}
```

Due to constraints in the MessagePack format, zero-copy deserialization is limited to `&str` and `&[u8]`. As a result, zerompk's performance is lower compared to formats like [rkyv](https://github.com/rkyv/rkyv) or [bincode](https://github.com/bincode-org/bincode). (However, compared to these formats, MessagePack is self-descriptive and excels in versatility for inter-language operations.)

### Other Optimizations

zerompk improves performance through various optimizations:

- Aggressive inlining to reduce function calls
- Elimination of unnecessary boundary checks using `unsafe` code
- Minimization of intermediate layers with `zerompk::{Read, Write}`
- Automaton-based string search for faster deserialization of map formats

Many of these optimizations are inspired by the high-performance MessagePack serializer [MessagePack-CSharp](https://github.com/MessagePack-CSharp/MessagePack-CSharp).

## Benchmarks

> [!NOTE]
> [msgpacker](https://github.com/codx-dev/msgpacker) is described as a MessagePack serializer, but it does not produce correct MessagePack binaries. In msgpacker, structs are always represented as arrays, but the discriminating header is omitted. Therefore, binaries serialized by msgpacker are not compatible with properly implemented MessagePack serializers, making strict comparisons invalid.

### Serialize/Deserialize Struct (with 4 fields, array format) 1000 times

| Crate               | Serialize | Deserialize |
| ------------------- | --------: | ----------: |
| `serde_json` (JSON) |  98.33 μs |   329.12 μs |
| `msgpacker`         |  25.41 μs |   134.37 μs |
| `rmp_serde`         |  56.22 μs |    97.00 μs |
| `zerompk`           |  12.38 μs |    72.27 μs |

### Serialize/Deserialize Struct (with 4 fields, map format) 1000 times

| Crate               | Serialize | Deserialize |
| ------------------- | --------: | ----------: |
| `serde_json` (JSON) |  98.33 μs |   329.12 μs |
| `rmp_serde`         |  92.63 μs |    98.31 μs |
| `zerompk`           |  18.76 μs |    71.19 μs |
| `msgpacker`         |       N/A |         N/A |

### Serialize/Deserialize Array (struct with 2 fields, 1000 elements) 1000 times

| Crate               |    Serialize |  Deserialize |
| ------------------- | -----------: | -----------: |
| `serde_json` (JSON) | 22,369.22 μs | 37,034.55 μs |
| `rmp_serde`         |  9,803.24 μs | 10,839.79 μs |
| `msgpacker`         | 10,981.52 μs |  4,608.72 μs |
| `zerompk`           |  6,310.66 μs |  3,571.90 μs |

### Serialize/Deserialize Struct (with 2 fields, no-copy) 1000 times

| Crate       | Serialize | Deserialize |
| ----------- | --------: | ----------: |
| `rmp_serde` |  15.47 μs |    16.82 μs |
| `zerompk`   |   8.57 μs |    10.33 μs |

## Security

zerompk always requires strict type schemas for serialization/deserialization, making it almost safe against untrusted binaries. Additionally, zerompk implements measures against the following attacks:

- Stack overflow caused by excessive object nesting. zerompk rejects objects nested beyond `MAX_DEPTH = 500` and returns an error.
- Memory consumption due to large size headers. zerompk validates header sizes before memory allocation and returns an error if the buffer is insufficient.

However, note that these measures are for general attacks and do not validate the data itself. When deserializing untrusted data, ensure proper validation on the application side.

## License

This library is released under the [MIT License](LICENSE).
