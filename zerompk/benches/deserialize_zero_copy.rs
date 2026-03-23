#![feature(test)]
extern crate test;

use serde::{Deserialize, Serialize};
use zerompk_derive::{FromMessagePack, ToMessagePack};

#[derive(Serialize, Deserialize, ToMessagePack, FromMessagePack)]
pub struct NoCopy<'a> {
    pub str: &'a str,
    #[serde(with = "serde_bytes")]
    pub bin: &'a [u8],
}

#[bench]
fn serialize_zero_copy_zerompk(b: &mut test::Bencher) {
    let value = NoCopy {
        str: "hello, world!!",
        bin: &[1, 2, 3, 4, 5, 6, 7, 8, 9, 0],
    };
    let mut buf = vec![0; 64];
    b.iter(|| {
        for _ in 0..1000 {
            zerompk::to_msgpack(&value, &mut buf).unwrap();
        }
    });
}

#[bench]
fn serialize_zero_copy_rmp_serde(b: &mut test::Bencher) {
    let value = NoCopy {
        str: "hello, world!!",
        bin: &[1, 2, 3, 4, 5, 6, 7, 8, 9, 0],
    };
    let mut buf = vec![0; 64];
    b.iter(|| {
        for _ in 0..1000 {
            value
                .serialize(&mut rmp_serde::Serializer::new(&mut buf))
                .unwrap();
        }
    });
}

#[bench]
fn deserialize_zero_copy_zerompk(b: &mut test::Bencher) {
    let value = NoCopy {
        str: "hello, world!!",
        bin: &[1, 2, 3, 4, 5, 6, 7, 8, 9, 0],
    };
    let msgpack = zerompk::to_msgpack_vec(&value).unwrap();
    b.iter(|| {
        let data = test::black_box(&msgpack);
        for _ in 0..1000 {
            let deserialized: NoCopy = zerompk::from_msgpack(data).unwrap();
            test::black_box(deserialized);
        }
    });
}

#[bench]
fn deserialize_zero_copy_rmp_serde(b: &mut test::Bencher) {
    let value = NoCopy {
        str: "hello, world!!",
        bin: &[1, 2, 3, 4, 5, 6, 7, 8, 9, 0],
    };
    let msgpack = rmp_serde::to_vec(&value).unwrap();
    b.iter(|| {
        let data = test::black_box(&msgpack);
        for _ in 0..1000 {
            let deserialized: NoCopy = rmp_serde::from_slice(data).unwrap();
            test::black_box(deserialized);
        }
    });
}
