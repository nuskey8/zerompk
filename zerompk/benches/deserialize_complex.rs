#![feature(test)]

use msgpacker::{Packable, Unpackable};

use crate::common::Point;
extern crate test;

mod common;

const N: usize = 1000;

#[bench]
fn deserialize_array_complex_zerompk(b: &mut test::Bencher) {
    let data = test::black_box(
        zerompk::to_msgpack_vec(&common::NestedArray {
            name: "Test".to_string(),
            p1: Point { x: 10, y: 20 },
            p2: Some(Point { x: 30, y: 40 }),
            params: vec![1, 2, 3, 4, 5],
        })
        .unwrap(),
    );
    b.iter(|| {
        for _ in 0..N {
            let _ = zerompk::from_msgpack::<common::NestedArray>(&data).unwrap();
        }
    });
}

#[bench]
fn deserialize_array_complex_rmp_serde(b: &mut test::Bencher) {
    let data = test::black_box(
        rmp_serde::to_vec(&common::NestedArray {
            name: "Test".to_string(),
            p1: Point { x: 10, y: 20 },
            p2: Some(Point { x: 30, y: 40 }),
            params: vec![1, 2, 3, 4, 5],
        })
        .unwrap(),
    );
    b.iter(|| {
        for _ in 0..N {
            let _ = rmp_serde::from_slice::<common::NestedArray>(&data).unwrap();
        }
    });
}

#[bench]
fn deserialize_array_complex_msgpacker(b: &mut test::Bencher) {
    let data = test::black_box(
        common::NestedArray {
            name: "Test".to_string(),
            p1: Point { x: 10, y: 20 },
            p2: Some(Point { x: 30, y: 40 }),
            params: vec![1, 2, 3, 4, 5],
        }
        .pack_to_vec(),
    );
    b.iter(|| {
        for _ in 0..N {
            let _ = common::NestedArray::unpack(&data).unwrap();
        }
    });
}

#[bench]
fn deserialize_map_complex_zerompk(b: &mut test::Bencher) {
    let data = test::black_box(
        zerompk::to_msgpack_vec(&common::Nested {
            name: "Test".to_string(),
            p1: Point { x: 10, y: 20 },
            p2: Some(Point { x: 30, y: 40 }),
            params: vec![1, 2, 3, 4, 5],
        })
        .unwrap(),
    );
    b.iter(|| {
        for _ in 0..N {
            let _ = zerompk::from_msgpack::<common::Nested>(&data).unwrap();
        }
    });
}

#[bench]
fn deserialize_map_complex_rmp_serde(b: &mut test::Bencher) {
    let data = test::black_box(
        rmp_serde::to_vec(&common::Nested {
            name: "Test".to_string(),
            p1: Point { x: 10, y: 20 },
            p2: Some(Point { x: 30, y: 40 }),
            params: vec![1, 2, 3, 4, 5],
        })
        .unwrap(),
    );
    b.iter(|| {
        for _ in 0..N {
            let _ = rmp_serde::from_slice::<common::Nested>(&data).unwrap();
        }
    });
}

#[bench]
fn deserialize_map_long_key_complex_zerompk(b: &mut test::Bencher) {
    let data = test::black_box(
        zerompk::to_msgpack_vec(&common::NestedLongKey {
            my_property_identifier_alpha: "Test".to_string(),
            my_property_identifier_beta_point1: Point { x: 10, y: 20 },
            my_property_identifier_gamma_point2: Some(Point { x: 30, y: 40 }),
            my_property_identifier_delta_parameters: vec![1, 2, 3, 4, 5],
        })
        .unwrap(),
    );
    b.iter(|| {
        for _ in 0..N {
            let _ = zerompk::from_msgpack::<common::NestedLongKey>(&data).unwrap();
        }
    });
}

#[bench]
fn deserialize_map_long_key_complex_rmp_serde(b: &mut test::Bencher) {
    let data = test::black_box(
        rmp_serde::to_vec(&common::NestedLongKey {
            my_property_identifier_alpha: "Test".to_string(),
            my_property_identifier_beta_point1: Point { x: 10, y: 20 },
            my_property_identifier_gamma_point2: Some(Point { x: 30, y: 40 }),
            my_property_identifier_delta_parameters: vec![1, 2, 3, 4, 5],
        })
        .unwrap(),
    );
    b.iter(|| {
        for _ in 0..N {
            let _ = rmp_serde::from_slice::<common::NestedLongKey>(&data).unwrap();
        }
    });
}

#[bench]
fn deserialize_complex_serde_json(b: &mut test::Bencher) {
    let data = test::black_box(
        serde_json::to_vec(&common::NestedArray {
            name: "Test".to_string(),
            p1: Point { x: 10, y: 20 },
            p2: Some(Point { x: 30, y: 40 }),
            params: vec![1, 2, 3, 4, 5],
        })
        .unwrap(),
    );
    b.iter(|| {
        for _ in 0..N {
            let _ = serde_json::from_slice::<common::NestedArray>(&data).unwrap();
        }
    });
}

#[bench]
fn deserialize_map_complex_serde_json(b: &mut test::Bencher) {
    let data = test::black_box(
        serde_json::to_vec(&common::Nested {
            name: "Test".to_string(),
            p1: Point { x: 10, y: 20 },
            p2: Some(Point { x: 30, y: 40 }),
            params: vec![1, 2, 3, 4, 5],
        })
        .unwrap(),
    );
    b.iter(|| {
        for _ in 0..N {
            let _ = serde_json::from_slice::<common::Nested>(&data).unwrap();
        }
    });
}
