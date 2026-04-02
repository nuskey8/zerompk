#![feature(test)]
extern crate test;

mod common;

use common::Point;
use std::io::Write;

#[bench]
fn serialize_rmp_serde_file(b: &mut test::Bencher) {
    let points: Vec<Point> = (0..1000).map(|i| Point { x: i, y: i * 2 }).collect();
    let tmp_file = std::env::temp_dir().join("points.msgpack");
    let mut file = std::fs::File::create(&tmp_file).unwrap();
    let mut buf_writer = std::io::BufWriter::with_capacity(4096, &mut file);

    b.iter(|| {
        rmp_serde::encode::write(&mut buf_writer, &points).unwrap();
    });

    buf_writer.flush().unwrap();
}

#[bench]
#[cfg(feature = "std")]
fn serialize_zerompk_file(b: &mut test::Bencher) {
    let points: Vec<Point> = (0..1000).map(|i| Point { x: i, y: i * 2 }).collect();
    let tmp_file = std::env::temp_dir().join("points_zerompk.msgpack");
    let mut file = std::fs::File::create(&tmp_file).unwrap();
    let mut buf_writer = std::io::BufWriter::with_capacity(4096, &mut file);

    b.iter(|| {
        zerompk::write_msgpack(&mut buf_writer, &points).unwrap();
    });

    buf_writer.flush().unwrap();
}

#[bench]
fn serialize_serde_json_file(b: &mut test::Bencher) {
    let points: Vec<Point> = (0..1000).map(|i| Point { x: i, y: i * 2 }).collect();
    let tmp_file = std::env::temp_dir().join("points.json");
    let mut file = std::fs::File::create(&tmp_file).unwrap();
    let mut buf_writer = std::io::BufWriter::with_capacity(4096, &mut file);

    b.iter(|| {
        serde_json::to_writer(&mut buf_writer, &points).unwrap();
    });

    buf_writer.flush().unwrap();
}
