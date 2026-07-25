#![feature(test)]

extern crate test;

use zerompk::{Read, SliceReader};

const VALUES: usize = 10_000;

#[bench]
fn skip_large_scalar_array(b: &mut test::Bencher) {
    let mut data = Vec::with_capacity(VALUES + 5);
    data.push(0xdd); // array32
    data.extend_from_slice(&(VALUES as u32).to_be_bytes());
    data.resize(data.len() + VALUES, 1); // positive fixints

    b.bytes = data.len() as u64;
    b.iter(|| {
        let mut reader = SliceReader::new(test::black_box(&data));
        reader.skip_value().unwrap();
        test::black_box(reader);
    });
}

#[bench]
fn skip_large_u32_array(b: &mut test::Bencher) {
    let mut data = Vec::with_capacity(VALUES * 5 + 5);
    data.push(0xdd); // array32
    data.extend_from_slice(&(VALUES as u32).to_be_bytes());
    for value in 0..VALUES as u32 {
        data.push(0xce); // uint32
        data.extend_from_slice(&value.to_be_bytes());
    }

    b.bytes = data.len() as u64;
    b.iter(|| {
        let mut reader = SliceReader::new(test::black_box(&data));
        reader.skip_value().unwrap();
        test::black_box(reader);
    });
}
