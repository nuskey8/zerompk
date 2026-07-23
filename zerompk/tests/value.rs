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
    let decoded = zerompk::read_msgpack_value(Cursor::new(encoded)).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn value_reads_from_bufread_reader() {
    let value = sample_value();
    let encoded = zerompk::to_msgpack_vec(&value).unwrap();
    let mut reader = BufReader::with_capacity(1, Cursor::new(encoded));
    let decoded = zerompk::read_msgpack_value_bufread(&mut reader).unwrap();
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
