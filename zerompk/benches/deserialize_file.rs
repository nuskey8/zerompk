#![feature(test)]
extern crate test;

mod common;

use common::Point;
use std::io::{Seek, Write};

#[bench]
fn deserialize_rmp_serde_file(b: &mut test::Bencher) {
    let points: Vec<Point> = (0..1000).map(|i| Point { x: i, y: i * 2 }).collect();
    let tmp_file = std::env::temp_dir().join("points.msgpack");
    let mut file = std::fs::File::create(&tmp_file).unwrap();
    let mut buf_writer = std::io::BufWriter::with_capacity(4096, &mut file);
    rmp_serde::encode::write(&mut buf_writer, &points).unwrap();
    buf_writer.flush().unwrap();

    let file = std::fs::File::open(&tmp_file).unwrap();
    let mut buf_reader = std::io::BufReader::with_capacity(4096, file);

    b.iter(|| {
        buf_reader.seek(std::io::SeekFrom::Start(0)).unwrap();
        rmp_serde::decode::from_read::<_, Vec<Point>>(&mut buf_reader).unwrap()
    });
}

#[bench]
#[cfg(feature = "std")]
fn deserialize_zerompk_file(b: &mut test::Bencher) {
    let points: Vec<Point> = (0..1000).map(|i| Point { x: i, y: i * 2 }).collect();
    let tmp_file = std::env::temp_dir().join("points_zerompk.msgpack");
    let mut file = std::fs::File::create(&tmp_file).unwrap();
    let mut buf_writer = std::io::BufWriter::with_capacity(4096, &mut file);
    zerompk::write_msgpack(&mut buf_writer, &points).unwrap();
    buf_writer.flush().unwrap();

    let file = std::fs::File::open(&tmp_file).unwrap();
    let mut buf_reader = std::io::BufReader::with_capacity(4096, file);
    b.iter(|| {
        buf_reader.seek(std::io::SeekFrom::Start(0)).unwrap();
        zerompk::read_msgpack::<_, Vec<Point>>(&mut buf_reader).unwrap()
    });
}

#[bench]
fn deserialize_serde_json_file(b: &mut test::Bencher) {
    let points: Vec<Point> = (0..1000).map(|i| Point { x: i, y: i * 2 }).collect();
    let tmp_file = std::env::temp_dir().join("points.json");
    let mut file = std::fs::File::create(&tmp_file).unwrap();
    let mut buf_writer = std::io::BufWriter::with_capacity(4096, &mut file);
    serde_json::to_writer(&mut buf_writer, &points).unwrap();
    buf_writer.flush().unwrap();

    let file = std::fs::File::open(&tmp_file).unwrap();
    let mut buf_reader = std::io::BufReader::with_capacity(4096, file);

    b.iter(|| {
        buf_reader.seek(std::io::SeekFrom::Start(0)).unwrap();
        serde_json::from_reader::<_, Vec<Point>>(&mut buf_reader).unwrap()
    });
}
