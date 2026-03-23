#![feature(test)]

use msgpacker::Unpackable;

use crate::common::Point;
extern crate test;

mod common;

const N: usize = 1000;

#[bench]
fn deserialize_simple_zerompk(b: &mut test::Bencher) {
    b.iter(|| {
        let data = test::black_box(zerompk::to_msgpack_vec(&Point { x: 10, y: 20 }).unwrap());
        for _ in 0..N {
            let point = zerompk::from_msgpack::<common::Point>(&data).unwrap();
            test::black_box(point);
        }
    });
}

#[bench]
fn deserialize_simple_rmp_serde(b: &mut test::Bencher) {
    b.iter(|| {
        let data = test::black_box(rmp_serde::to_vec(&Point { x: 10, y: 20 }).unwrap());
        for _ in 0..N {
            let point = rmp_serde::from_slice::<common::Point>(&data).unwrap();
            test::black_box(point);
        }
    });
}

#[bench]
fn deserialize_simple_msgpacker(b: &mut test::Bencher) {
    b.iter(|| {
        let data = test::black_box(msgpacker::pack_to_vec(&Point { x: 10, y: 20 }));
        for _ in 0..N {
            let point = common::Point::unpack(&data).unwrap();
            test::black_box(point);
        }
    });
}

#[bench]
fn deserialize_simple_serde_json(b: &mut test::Bencher) {
    b.iter(|| {
        let data = test::black_box(serde_json::to_vec(&Point { x: 10, y: 20 }).unwrap());
        for _ in 0..N {
            let point = serde_json::from_slice::<common::Point>(&data).unwrap();
            test::black_box(point);
        }
    });
}
