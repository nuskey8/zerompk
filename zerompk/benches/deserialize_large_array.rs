#![feature(test)]

use crate::common::Point;
extern crate test;

mod common;

const N: usize = 1000;

#[bench]
fn deserialize_large_array_zerompk(b: &mut test::Bencher) {
    let points: Vec<Point> = (0..1000).map(|i| Point { x: i, y: i * 2 }).collect();
    let msgpack = zerompk::to_msgpack_vec(&points).unwrap();
    b.iter(|| {
        let data = test::black_box(&msgpack);
        for _ in 0..N {
            let deserialized: Vec<Point> = zerompk::from_msgpack(data).unwrap();
            test::black_box(deserialized);
        }
    });
}

#[bench]
fn deserialize_large_array_rmp_serde(b: &mut test::Bencher) {
    let points: Vec<Point> = (0..1000).map(|i| Point { x: i, y: i * 2 }).collect();
    let msgpack = rmp_serde::to_vec(&points).unwrap();
    b.iter(|| {
        let data = test::black_box(&msgpack);
        for _ in 0..N {
            let deserialized: Vec<Point> = rmp_serde::from_slice(data).unwrap();
            test::black_box(deserialized);
        }
    });
}

#[bench]
fn deserialize_large_array_msgpacker(b: &mut test::Bencher) {
    let points: Vec<Point> = (0..1000).map(|i| Point { x: i, y: i * 2 }).collect();
    let mut msgpack = Vec::new();
    msgpacker::pack_array(&mut msgpack, points.iter());
    b.iter(|| {
        let data = test::black_box(&msgpack);
        for _ in 0..N {
            let mut buf = data.clone();
            let (_, deserialized): (usize, Vec<Point>) = msgpacker::unpack_array(&mut buf).unwrap();
            test::black_box(deserialized);
        }
    });
}

#[bench]
fn deserialize_large_array_serde_json(b: &mut test::Bencher) {
    let points: Vec<Point> = (0..1000).map(|i| Point { x: i, y: i * 2 }).collect();
    let json = serde_json::to_vec(&points).unwrap();
    b.iter(|| {
        let data = test::black_box(&json);
        for _ in 0..N {
            let deserialized: Vec<Point> = serde_json::from_slice(data).unwrap();
            test::black_box(deserialized);
        }
    });
}
