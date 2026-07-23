use std::borrow::Cow;
use std::io::{BufReader, Cursor};

use zerompk::Value;

fn sample_value() -> Value<'static> {
    Value::Map(vec![
        (
            Value::String("scalars".into()),
            Value::Array(vec![
                Value::Nil,
                Value::Boolean(true),
                Value::Unsigned(u64::MAX),
                Value::Signed(i64::MIN),
                Value::Float32(1.5),
                Value::Float64(-2.5),
            ]),
        ),
        (
            Value::String("data".into()),
            Value::Array(vec![
                Value::String("hello".into()),
                Value::Binary(vec![0, 1, 2, 255].into()),
                Value::Extension(42, vec![1, 2, 3].into()),
            ]),
        ),
    ])
}

#[test]
fn value_debug_is_json_like_and_honors_pretty_formatting() {
    let value = Value::Map(vec![
        (
            Value::String("items".into()),
            Value::Array(vec![
                Value::Nil,
                Value::Boolean(true),
                Value::String("a\nb".into()),
            ]),
        ),
        (
            Value::String("binary".into()),
            Value::Binary(vec![0, 255].into()),
        ),
        (
            Value::String("extension".into()),
            Value::Extension(-1, vec![1, 2].into()),
        ),
    ]);

    assert_eq!(
        format!("{value:?}"),
        r#"{"items": [null, true, "a\nb"], "binary": [0, 255], "extension": {"type": -1, "data": [1, 2]}}"#
    );
    assert_eq!(
        format!("{value:#?}"),
        r#"{
    "items": [
        null,
        true,
        "a\nb",
    ],
    "binary": [
        0,
        255,
    ],
    "extension": {
        "type": -1,
        "data": [
            1,
            2,
        ],
    },
}"#
    );
}

#[test]
fn value_containers_use_the_declared_lengths() {
    let encoded = [
        0x82, 0xa1, b'a', 0x93, 0x01, 0x02, 0x03, 0xa1, b'b', 0x81, 0xa1, b'c', 0xc3,
    ];
    let decoded: Value = zerompk::from_msgpack(&encoded).unwrap();

    let Value::Map(entries) = decoded else {
        panic!("expected map");
    };
    assert_eq!(entries.len(), 2);
    assert!(matches!(&entries[0].1, Value::Array(values) if values.len() == 3));
    assert!(matches!(&entries[1].1, Value::Map(values) if values.len() == 1));
}

#[test]
fn value_roundtrip_from_slice() {
    let value = sample_value();
    let encoded = zerompk::to_msgpack_vec(&value).unwrap();
    let decoded: Value = zerompk::from_msgpack(&encoded).unwrap();
    assert_eq!(decoded, value);
    let Value::Map(entries) = decoded else {
        unreachable!()
    };
    assert!(matches!(entries[0].0, Value::String(Cow::Borrowed(_))));
}

#[test]
fn value_reads_from_io_reader() {
    let value = sample_value();
    let encoded = zerompk::to_msgpack_vec(&value).unwrap();
    let decoded: Value<'_> = zerompk::read_msgpack(Cursor::new(encoded)).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn value_reads_from_bufread_reader() {
    let value = sample_value();
    let encoded = zerompk::to_msgpack_vec(&value).unwrap();
    let mut reader = BufReader::with_capacity(1, Cursor::new(encoded));
    let decoded: Value<'_> = zerompk::read_msgpack_bufread(&mut reader).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn value_reads_all_extension_header_sizes() {
    for len in [1, 2, 3, 4, 8, 16, 255, 256, 65_536] {
        let value = Value::Extension(-42, vec![0x5a; len].into());
        let encoded = zerompk::to_msgpack_vec(&value).unwrap();
        let decoded: Value = zerompk::from_msgpack(&encoded).unwrap();
        assert_eq!(decoded, value);
    }
}

#[test]
fn timestamp_is_preserved_as_extension() {
    let encoded = [
        0xd6, 0xff, 0x65, 0x53, 0xf1, 0x00, // timestamp32
    ];
    let decoded: Value = zerompk::from_msgpack(&encoded).unwrap();
    assert_eq!(
        decoded,
        Value::Extension(-1, vec![0x65, 0x53, 0xf1, 0x00].into())
    );
    assert_eq!(zerompk::to_msgpack_vec(&decoded).unwrap(), encoded);
}

#[test]
fn value_respects_depth_limit() {
    let mut encoded = vec![0x91; 501];
    encoded.push(0xc0);
    assert!(matches!(
        zerompk::from_msgpack::<Value>(&encoded),
        Err(zerompk::Error::DepthLimitExceeded { .. })
    ));
}

#[test]
fn value_rejects_truncated_extension() {
    let encoded = [0xc7, 3, 42, 1, 2];
    assert!(zerompk::from_msgpack::<Value>(&encoded).is_err());
}

#[test]
fn value_rejects_truncated_huge_containers_without_large_preallocation() {
    for encoded in [
        [0xdd, 0xff, 0xff, 0xff, 0xff],
        [0xdf, 0xff, 0xff, 0xff, 0xff],
    ] {
        assert!(matches!(
            zerompk::from_msgpack::<Value>(&encoded),
            Err(zerompk::Error::BufferTooSmall)
        ));
    }
}
