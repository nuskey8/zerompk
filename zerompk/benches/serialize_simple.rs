#![feature(test)]
extern crate test;

mod common;

use common::Point;
use msgpacker::Packable;
use serde::Serialize;

const N: usize = 1000;

#[bench]
fn serialize_simple_zerompk(b: &mut test::Bencher) {
    let point = Point { x: 10, y: 20 };
    let mut buf = vec![0; 12];
    b.iter(|| {
        for _ in 0..N {
            zerompk::to_msgpack(&point, &mut buf).unwrap();
        }
    });
}

#[bench]
fn serialize_simple_rmp_serde(b: &mut test::Bencher) {
    let point = Point { x: 10, y: 20 };
    let mut buf = vec![0; 12];
    b.iter(|| {
        for _ in 0..N {
            point
                .serialize(&mut rmp_serde::Serializer::new(&mut buf))
                .unwrap();
        }
    });
}

#[bench]
fn serialize_simple_msgpacker(b: &mut test::Bencher) {
    let point = Point { x: 10, y: 20 };
    let mut buf = vec![0; 12];
    b.iter(|| {
        for _ in 0..N {
            point.pack(&mut buf);
        }
    });
}

#[bench]
fn serialize_simple_serde_json(b: &mut test::Bencher) {
    let point = Point { x: 10, y: 20 };
    let mut buf = Vec::new();
    b.iter(|| {
        for _ in 0..N {
            buf.clear();
            serde_json::to_writer(&mut buf, &point).unwrap();
        }
    });
}
