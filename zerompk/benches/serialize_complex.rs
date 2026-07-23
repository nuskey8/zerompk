#![feature(test)]
extern crate test;

mod common;

use common::{Nested, Point};
use msgpacker::Packable;
use serde::Serialize;

use crate::common::NestedArray;

const N: usize = 1000;

#[bench]
fn serialize_array_complex_zerompk(b: &mut test::Bencher) {
    let nested = NestedArray {
        name: "Test".to_string(),
        p1: Point { x: 10, y: 20 },
        p2: Some(Point { x: 30, y: 40 }),
        params: vec![1, 2, 3, 4, 5],
    };
    let mut buf = vec![0; 256];
    b.iter(|| {
        for _ in 0..N {
            let len = zerompk::to_msgpack(test::black_box(&nested), &mut buf).unwrap();
            test::black_box(&buf[..len]);
        }
    });
}

#[bench]
fn serialize_array_complex_rmp_serde(b: &mut test::Bencher) {
    let nested = NestedArray {
        name: "Test".to_string(),
        p1: Point { x: 10, y: 20 },
        p2: Some(Point { x: 30, y: 40 }),
        params: vec![1, 2, 3, 4, 5],
    };
    let mut buf = vec![0; 256];
    b.iter(|| {
        for _ in 0..N {
            buf.clear();
            nested
                .serialize(&mut rmp_serde::Serializer::new(&mut buf))
                .unwrap();
            test::black_box(&buf);
        }
    });
}

#[bench]
fn serialize_array_complex_msgpacker(b: &mut test::Bencher) {
    let nested = NestedArray {
        name: "Test".to_string(),
        p1: Point { x: 10, y: 20 },
        p2: Some(Point { x: 30, y: 40 }),
        params: vec![1, 2, 3, 4, 5],
    };
    let mut buf = vec![0; 256];
    b.iter(|| {
        for _ in 0..N {
            buf.clear();
            nested.pack(&mut buf);
            test::black_box(&buf);
        }
    });
}

#[bench]
fn serialize_map_complex_zerompk(b: &mut test::Bencher) {
    let nested = Nested {
        name: "Test".to_string(),
        p1: Point { x: 10, y: 20 },
        p2: Some(Point { x: 30, y: 40 }),
        params: vec![1, 2, 3, 4, 5],
    };
    let mut buf = vec![0; 256];
    b.iter(|| {
        for _ in 0..N {
            let len = zerompk::to_msgpack(test::black_box(&nested), &mut buf).unwrap();
            test::black_box(&buf[..len]);
        }
    });
}

#[bench]
fn serialize_map_complex_rmp_serde(b: &mut test::Bencher) {
    let nested = Nested {
        name: "Test".to_string(),
        p1: Point { x: 10, y: 20 },
        p2: Some(Point { x: 30, y: 40 }),
        params: vec![1, 2, 3, 4, 5],
    };
    let mut buf = vec![0; 256];
    b.iter(|| {
        for _ in 0..N {
            buf.clear();
            nested
                .serialize(&mut rmp_serde::Serializer::new(&mut buf).with_struct_map())
                .unwrap();
            test::black_box(&buf);
        }
    });
}

#[bench]
fn serialize_array_complex_serde_json(b: &mut test::Bencher) {
    let nested = NestedArray {
        name: "Test".to_string(),
        p1: Point { x: 10, y: 20 },
        p2: Some(Point { x: 30, y: 40 }),
        params: vec![1, 2, 3, 4, 5],
    };
    let mut buf = Vec::new();
    b.iter(|| {
        for _ in 0..N {
            buf.clear();
            serde_json::to_writer(&mut buf, &nested).unwrap();
            test::black_box(&buf);
        }
    });
}

#[bench]
fn serialize_map_complex_serde_json(b: &mut test::Bencher) {
    let nested = Nested {
        name: "Test".to_string(),
        p1: Point { x: 10, y: 20 },
        p2: Some(Point { x: 30, y: 40 }),
        params: vec![1, 2, 3, 4, 5],
    };
    let mut buf = Vec::new();
    b.iter(|| {
        for _ in 0..N {
            buf.clear();
            serde_json::to_writer(&mut buf, &nested).unwrap();
            test::black_box(&buf);
        }
    });
}
