#![feature(test)]

extern crate test;

const ELEMENTS: usize = 1024;
const N: usize = 100;

fn bench_values(b: &mut test::Bencher, values: Vec<i32>) {
    let msgpack = zerompk::to_msgpack_vec(&values).unwrap();
    b.bytes = msgpack.len() as u64 * N as u64;
    b.iter(|| {
        let data = test::black_box(&msgpack);
        for _ in 0..N {
            let decoded: Vec<i32> = zerompk::from_msgpack(data).unwrap();
            test::black_box(decoded);
        }
    });
}

#[bench]
fn deserialize_i32_positive_fixint(b: &mut test::Bencher) {
    bench_values(b, (0..ELEMENTS).map(|i| (i % 128) as i32).collect());
}

#[bench]
fn deserialize_i32_negative_fixint(b: &mut test::Bencher) {
    bench_values(b, (0..ELEMENTS).map(|i| -1 - (i % 32) as i32).collect());
}

#[bench]
fn deserialize_i32_int8(b: &mut test::Bencher) {
    bench_values(b, (0..ELEMENTS).map(|i| -33 - (i % 96) as i32).collect());
}

#[bench]
fn deserialize_i32_int16(b: &mut test::Bencher) {
    bench_values(
        b,
        (0..ELEMENTS)
            .map(|i| {
                if i % 2 == 0 {
                    128 + (i % 32_640) as i32
                } else {
                    -129 - (i % 32_640) as i32
                }
            })
            .collect(),
    );
}

#[bench]
fn deserialize_i32_int32(b: &mut test::Bencher) {
    bench_values(
        b,
        (0..ELEMENTS)
            .map(|i| {
                if i % 2 == 0 {
                    32_768 + i as i32
                } else {
                    -32_769 - i as i32
                }
            })
            .collect(),
    );
}

#[bench]
fn deserialize_i32_mixed(b: &mut test::Bencher) {
    const VALUES: [i32; 10] = [0, 127, -1, -32, -33, -128, 128, -129, 32_768, -32_769];
    bench_values(b, (0..ELEMENTS).map(|i| VALUES[i % VALUES.len()]).collect());
}
