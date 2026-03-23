#![feature(test)]
extern crate test;

mod common;

use common::Point;
use serde::Serialize;

const N: usize = 1000;

#[bench]
fn serialize_large_array_zerompk(b: &mut test::Bencher) {
    let points: Vec<Point> = (0..1000).map(|i| Point { x: i, y: i * 2 }).collect();
    let mut buf = vec![0; 16 * 1024];
    b.iter(|| {
        for _ in 0..N {
            zerompk::to_msgpack(&points, &mut buf).unwrap();
        }
    });
}

#[bench]
fn serialize_large_array_rmp_serde(b: &mut test::Bencher) {
    let points: Vec<Point> = (0..1000).map(|i| Point { x: i, y: i * 2 }).collect();
    let mut buf = vec![0; 16 * 1024];
    b.iter(|| {
        for _ in 0..N {
            points
                .serialize(&mut rmp_serde::Serializer::new(&mut buf))
                .unwrap();
        }
    });
}

#[bench]
fn serialize_large_array_msgpacker(b: &mut test::Bencher) {
    let points: Vec<Point> = (0..1000).map(|i| Point { x: i, y: i * 2 }).collect();
    let mut buf = vec![0; 16 * 1024];
    b.iter(|| {
        for _ in 0..N {
            msgpacker::pack_array(&mut buf, &points);
        }
    });
}

#[bench]
fn serialize_large_array_serde_json(b: &mut test::Bencher) {
    let points: Vec<Point> = (0..1000).map(|i| Point { x: i, y: i * 2 }).collect();
    let mut buf = Vec::new();
    b.iter(|| {
        for _ in 0..N {
            buf.clear();
            serde_json::to_writer(&mut buf, &points).unwrap();
        }
    });
}
