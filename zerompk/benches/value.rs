#![feature(test)]

extern crate test;

use serde_json::{Value as JsonValue, json};
use zerompk::Value;

const N: usize = 1_000;

fn value() -> JsonValue {
    json!({
        "name": "zerompk",
        "active": true,
        "version": 6,
        "ratio": 1.25,
        "tags": ["messagepack", "rust", "zero-copy", "no-std"],
        "authors": [
            {"name": "Alice", "commits": 127, "active": true},
            {"name": "Bob", "commits": 63, "active": false},
            {"name": "Carol", "commits": 255, "active": true}
        ],
        "metrics": {
            "downloads": [1, 127, 128, 255, 256, 65535, 65536],
            "latencies": [0.25, 1.5, 12.75, 100.125],
            "nullable": null
        }
    })
}

fn data() -> Vec<u8> {
    rmp_serde::to_vec(&value()).unwrap()
}

#[bench]
fn deserialize_value_zerompk(b: &mut test::Bencher) {
    let data = test::black_box(data());
    b.bytes = (data.len() * N) as u64;
    b.iter(|| {
        for _ in 0..N {
            test::black_box(zerompk::from_msgpack::<Value>(&data).unwrap());
        }
    });
}

#[bench]
fn deserialize_value_rmp_serde(b: &mut test::Bencher) {
    let data = test::black_box(data());
    b.bytes = (data.len() * N) as u64;
    b.iter(|| {
        for _ in 0..N {
            test::black_box(rmp_serde::from_slice::<JsonValue>(&data).unwrap());
        }
    });
}

#[bench]
fn deserialize_value_msgpacker(b: &mut test::Bencher) {
    let data = test::black_box(data());
    b.bytes = (data.len() * N) as u64;
    b.iter(|| {
        for _ in 0..N {
            test::black_box(msgpacker::serde::from_slice::<JsonValue>(&data).unwrap());
        }
    });
}

#[bench]
fn serialize_value_zerompk(b: &mut test::Bencher) {
    let data = data();
    let value: Value = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(zerompk::to_msgpack_vec(&value).unwrap(), data);
    b.bytes = (data.len() * N) as u64;
    b.iter(|| {
        for _ in 0..N {
            test::black_box(zerompk::to_msgpack_vec(&value).unwrap());
        }
    });
}

#[bench]
fn serialize_value_rmp_serde(b: &mut test::Bencher) {
    let value = value();
    let data = data();
    assert_eq!(rmp_serde::to_vec(&value).unwrap(), data);
    b.bytes = (data.len() * N) as u64;
    b.iter(|| {
        for _ in 0..N {
            test::black_box(rmp_serde::to_vec(&value).unwrap());
        }
    });
}

#[bench]
fn serialize_value_msgpacker(b: &mut test::Bencher) {
    let value = value();
    let data = data();
    assert_eq!(msgpacker::serde::to_vec(&value), data);
    b.bytes = (data.len() * N) as u64;
    b.iter(|| {
        for _ in 0..N {
            test::black_box(msgpacker::serde::to_vec(&value));
        }
    });
}
