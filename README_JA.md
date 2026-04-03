# zerompk

A zero-copy, zero-dependency, no_std-compatible, extremely fast MessagePack serializer for Rust.

[![Crates.io version](https://img.shields.io/crates/v/zerompk.svg?style=flat-square)](https://crates.io/crates/zerompk)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/zerompk)

![bench](docs/images/bench.png)

## 概要

zerompkはRust向けの高速なMessagePackシリアライザです。[rmp_serde](https://github.com/3Hren/msgpack-rust)と比較して1.3〜3.0倍ほど高速に動作し、`std`を含む一切のライブラリに依存せずに実装されています。

## クイックスタート

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

## フォーマット

zerompkにおけるRustとMessagePackの型の対応は以下の通りです。MessagePackは可変長エンコーディングであるため、zerompkは値に応じて可能な限りサイズの小さい型にシリアライズします。

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

`derive`フィーチャーフラグを有効化することで、`derive`マクロを用いて`FromMessagePack`/`ToMessagePack`を実装できます。

```rust
#[derive(FromMessagePack, ToMessagePack)]
pub struct Person {
    pub name: String,
    pub age: u32,
}
```

また、`#[msgpack]`属性を用いてシリアライズ形式をカスタマイズできます。

### array/map

structやenumのバリアントのシリアライズ形式は`array`/`map`から選択できます。パフォーマンス上の理由からデフォルトは`array`に設定されています。

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

フィールドやenumのバリアントに用いるindex/keyを上書きできます。arrayの場合は整数、mapの場合は文字列が利用できます。形式がarrayかつインデックスに空白がある場合は自動的に`nil`が挿入されます。

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
> バージョニング耐性を高めるため、可能な限りkeyを明示的に設定することが推奨されます。

### ignore

シリアライズ/デシリアライズ時に無視したいフィールドには`ignore`を設定します。`ignore`を含む構造体をデシリアライズする場合、`ignore`フィールドの型は`Default`を実装している必要があります。

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

C-styleのenumに`#[msgpack(c_enum)]`を付与することで、enumを整数としてシリアライズできます。値は各バリアントの判別子(discriminant)です。

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

## 設計哲学

最もメジャーなMessagePackシリアライザである[rmp](https://github.com/3Hren/msgpack-rust)は十分に最適化されていますが、zerompkはそれ以上にパフォーマンスに注力した設計になっています。

### No Serde

Serdeは非常に優れたシリアライザの抽象化層ですが、これはパフォーマンス上の(わずかな、しかしシリアライザにとっては無視出来ない)コストがかかります。zerompkはMessagePackに特化したシリアライザであるため、Serdeのtraitを利用しません。

例として、以下のコードに対するderiveマクロの生成コードを比較してみましょう。

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

複雑なvisitorを生成するSerdeに比べ、zerompkのコードは極めてシンプルです。これは実行時パフォーマンスの利点だけでなく、副次的にバイナリサイズやコンパイル時間の削減に繋がります。

### Zero Copy

Serde同様、zerompkは元のシリアライズされたデータを直接参照するzero-copyデシリアライズをサポートしています。

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

MessagePackフォーマット上の制約から、zero-copyデシリアライズは`&str`および`&[u8]`に限定されています。そのためzerompkは[rkyv](https://github.com/rkyv/rkyv)や[bincode](https://github.com/bincode-org/bincode)などの形式に比べるとパフォーマンスは低下します。(ただし、これらの形式と比較するとMessagePackは自己記述的であり、言語間運用などの汎用性の面において優れています。)

### その他の最適化

ほかにもzerompkは様々な最適化によってパフォーマンスを向上させています。

- 積極的なインライン化による関数呼び出しの削減
- `unsafe`コードによる不要な境界チェックの排除
- `zerompk::{Read, Write}`による中間層の最小化
- オートマトンベースの文字列探索によるmap形式のデシリアライズの高速化

これらの最適化の多くはC#製の高速なMessagePackシリアライザである[MessagePack-CSharp](https://github.com/MessagePack-CSharp/MessagePack-CSharp)にインスパイアされています。

## ベンチマーク

> [!NOTE]
> [msgpacker](https://github.com/codx-dev/msgpacker)はMessagePackシリアライザと説明されていますが、実際には正しいMessagePackバイナリを生成しません。msgpackerでは構造体は常に配列として表現されますが、判別用のヘッダが省略されています。したがって、msgpackerによってシリアライズされたバイナリは正しく実装されたMessagePackシリアライザと互換性がないため、厳密な比較ではありません。

### Serialize/Deserialize Struct (with 4 fields, array format) 1000 times

| Crate               | Serialize | Deserialize |
| ------------------- | --------: | ----------: |
| `serde_json` (JSON) |  98.33 μs |   329.12 μs |
| `msgpacker`         |  25.41 μs |   134.37 μs |
| `rmp_serde`         |  56.22 μs |    97.00 μs |
| `zerompk`           |  12.38 μs |    72.27 μs |

### Serialize/Deserialize Struct (with 4 fields, map format) 1000 times

| Crate              | Serialize | Deserialize |
| ------------------ | --------: | ----------: |
| `serde_json` (JSON) |  98.33 μs |   329.12 μs |
| `rmp_serde`         |  92.63 μs |    98.31 μs |
| `zerompk`           |  18.76 μs |    71.19 μs |
| `msgpacker`         |       N/A |         N/A |

### Serialize/Deserialize Array (struct with 2 fields, 1000 elements) 1000 times

| Crate              |    Serialize |  Deserialize |
| ------------------ | -----------: | -----------: |
| `serde_json` (JSON) | 22,369.22 μs | 37,034.55 μs |
| `rmp_serde`         |  9,803.24 μs | 10,839.79 μs |
| `msgpacker`         | 10,981.52 μs |  4,608.72 μs |
| `zerompk`           |  2,632.23 μs |  3,571.90 μs |

### Serialize/Deserialize Struct (with 2 fields, no-copy) 1000 times

| Crate       | Serialize | Deserialize |
| ----------- | --------: | ----------: |
| `rmp_serde` |  15.47 μs |    16.82 μs |
| `zerompk`   |   8.57 μs |    10.33 μs |

## セキュリティ

zerompkはシリアライズ/デシリアライズに対して常に厳格な型スキーマを要求するため、信頼出来ないバイナリに対してほぼ安全です。また、zerompkは以下の攻撃に対して対策を講じています。

- 過剰なオブジェクトのネストによるスタックオーバーフロー。zerompkは`MAX_DEPTH = 500`を超えるオブジェクトのネストを拒否し、エラーを返します。
- 巨大なサイズヘッダによるメモリの消費。zerompkはメモリ確保を行う前にヘッダのサイズの検証を行い、バッファが不足している場合はエラーを返します。

ただし、これらの対策は一般的な攻撃に対するものであり、データ自体のチェックは行わないことに注意してください。信頼出来ないデータをデシリアライズする場合は、アプリケーション側で適切なバリデーションを行ってください。

## ライセンス

このライブラリは[MIT License](LICENSE)の下で公開されています。