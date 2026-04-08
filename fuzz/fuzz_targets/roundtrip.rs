#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use zerompk::{FromMessagePack, ToMessagePack};

#[derive(Debug, Clone, PartialEq, Arbitrary, ToMessagePack, FromMessagePack)]
struct FuzzValue {
    flag: bool,
    small: i8,
    signed: i64,
    unsigned: u64,
    text: String,
    bytes: Vec<u8>,
    items: Vec<i32>,
    nested: Option<Box<FuzzNested>>,
}

#[derive(Debug, Clone, PartialEq, Arbitrary, ToMessagePack, FromMessagePack)]
struct FuzzNested {
    name: String,
    value: i32,
}

fuzz_target!(|value: FuzzValue| {
    if let Ok(buf) = zerompk::to_msgpack_vec(&value) {
        let decoded = zerompk::from_msgpack::<FuzzValue>(&buf)
            .expect("roundtrip decode must succeed for encoded data");
        assert_eq!(decoded, value);
    }
});
