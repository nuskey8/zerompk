use zerompk::{ToMessagePack, Write};

fn assert_cross_path(f: impl Fn(&mut dyn zerompk::Write) -> zerompk::Result<()>, expected: &[u8]) {
    struct W<F>(F);
    impl<F: Fn(&mut dyn zerompk::Write) -> zerompk::Result<()>> ToMessagePack for W<F> {
        fn write<Wr: zerompk::Write>(&self, writer: &mut Wr) -> zerompk::Result<()> {
            (self.0)(writer)
        }
    }
    assert_cross_path_value(&W(f), expected);
}

fn assert_cross_path_value(value: &impl ToMessagePack, expected: &[u8]) {
    let vec_result = zerompk::to_msgpack_vec(value).unwrap();
    assert_eq!(vec_result, expected, "VecWriter mismatch");

    let mut buf = vec![0u8; expected.len() + 64];
    let n = zerompk::to_msgpack(value, &mut buf).unwrap();
    assert_eq!(&buf[..n], expected, "SliceWriter mismatch");

    #[cfg(feature = "std")]
    {
        let mut io_buf = Vec::new();
        zerompk::write_msgpack(&mut io_buf, value).unwrap();
        assert_eq!(io_buf, expected, "IOWriter mismatch");
    }
}

#[test]
fn cross_path_nil() {
    assert_cross_path(|w| w.write_nil(), &[0xc0]);
}

#[test]
fn cross_path_true() {
    assert_cross_path(|w| w.write_boolean(true), &[0xc3]);
}

#[test]
fn cross_path_false() {
    assert_cross_path(|w| w.write_boolean(false), &[0xc2]);
}

#[test]
fn cross_path_u8() {
    // positive fixint
    assert_cross_path(|w| w.write_u8(0), &[0x00]);
    assert_cross_path(|w| w.write_u8(127), &[0x7f]);

    // uint8
    assert_cross_path(|w| w.write_u8(128), &[0xcc, 0x80]);
    assert_cross_path(|w| w.write_u8(255), &[0xcc, 0xff]);
}

#[test]
fn cross_path_u16() {
    // still encoded in the smallest unsigned integer format
    assert_cross_path(|w| w.write_u16(255), &[0xcc, 0xff]);

    // uint16
    assert_cross_path(|w| w.write_u16(256), &[0xcd, 0x01, 0x00]);
    assert_cross_path(|w| w.write_u16(65535), &[0xcd, 0xff, 0xff]);
}

#[test]
fn cross_path_u32() {
    // still encoded in the smallest unsigned integer format
    assert_cross_path(|w| w.write_u32(65535), &[0xcd, 0xff, 0xff]);

    // uint32
    assert_cross_path(|w| w.write_u32(65536), &[0xce, 0x00, 0x01, 0x00, 0x00]);
    assert_cross_path(|w| w.write_u32(u32::MAX), &[0xce, 0xff, 0xff, 0xff, 0xff]);
}

#[test]
fn cross_path_u64() {
    // still encoded in the smallest unsigned integer format
    assert_cross_path(
        |w| w.write_u64(u32::MAX as u64),
        &[0xce, 0xff, 0xff, 0xff, 0xff],
    );

    // uint64
    assert_cross_path(
        |w| w.write_u64(u32::MAX as u64 + 1),
        &[0xcf, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
    );
    assert_cross_path(
        |w| w.write_u64(u64::MAX),
        &[0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
    );
}

#[test]
fn cross_path_i8() {
    // positive fixint
    assert_cross_path(|w| w.write_i8(0), &[0x00]);

    // negative fixint
    assert_cross_path(|w| w.write_i8(-1), &[0xff]);
    assert_cross_path(|w| w.write_i8(-32), &[0xe0]);

    // int8
    assert_cross_path(|w| w.write_i8(-33), &[0xd0, 0xdf]);
    assert_cross_path(|w| w.write_i8(i8::MIN), &[0xd0, 0x80]);
}

#[test]
fn cross_path_i16() {
    // still encoded in the smallest signed integer format
    assert_cross_path(|w| w.write_i16(-128), &[0xd0, 0x80]);

    // int16
    assert_cross_path(|w| w.write_i16(-129), &[0xd1, 0xff, 0x7f]);
    assert_cross_path(|w| w.write_i16(i16::MIN), &[0xd1, 0x80, 0x00]);
}

#[test]
fn cross_path_i32() {
    // still encoded in the smallest signed integer format
    assert_cross_path(|w| w.write_i32(-32768), &[0xd1, 0x80, 0x00]);

    // int32
    assert_cross_path(|w| w.write_i32(-32769), &[0xd2, 0xff, 0xff, 0x7f, 0xff]);
    assert_cross_path(|w| w.write_i32(i32::MIN), &[0xd2, 0x80, 0x00, 0x00, 0x00]);
}

#[test]
fn cross_path_i64() {
    // still encoded in the smallest signed integer format
    assert_cross_path(
        |w| w.write_i64(i32::MIN as i64),
        &[0xd2, 0x80, 0x00, 0x00, 0x00],
    );

    // int64
    assert_cross_path(
        |w| w.write_i64(i32::MIN as i64 - 1),
        &[0xd3, 0xff, 0xff, 0xff, 0xff, 0x7f, 0xff, 0xff, 0xff],
    );
    assert_cross_path(
        |w| w.write_i64(i64::MIN),
        &[0xd3, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    );
}

#[test]
fn cross_path_f32() {
    assert_cross_path(|w| w.write_f32(0.0), &[0xca, 0x00, 0x00, 0x00, 0x00]);

    let bytes = std::f32::consts::PI.to_be_bytes();
    assert_cross_path(
        |w| w.write_f32(std::f32::consts::PI),
        &[0xca, bytes[0], bytes[1], bytes[2], bytes[3]],
    );

    let bytes = (-1.5f32).to_be_bytes();
    assert_cross_path(
        |w| w.write_f32(-1.5),
        &[0xca, bytes[0], bytes[1], bytes[2], bytes[3]],
    );
}

#[test]
fn cross_path_f64() {
    assert_cross_path(
        |w| w.write_f64(0.0),
        &[0xcb, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    );

    let bytes = std::f64::consts::PI.to_be_bytes();
    assert_cross_path(
        |w| w.write_f64(std::f64::consts::PI),
        &[
            0xcb, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ],
    );
}

#[test]
fn cross_path_string() {
    // fixstr
    assert_cross_path(|w| w.write_string(""), &[0xa0]);
    assert_cross_path(|w| w.write_string("abc"), &[0xa3, b'a', b'b', b'c']);

    let s = "a".repeat(31);
    let mut expected = vec![0xbf];
    expected.extend(s.as_bytes());
    assert_cross_path(move |w| w.write_string(&s), &expected);

    // str8
    let s = "b".repeat(32);
    let mut expected = vec![0xd9, 32];
    expected.extend(s.as_bytes());
    assert_cross_path(move |w| w.write_string(&s), &expected);

    // str16
    let s = "x".repeat(256);
    let mut expected = vec![0xda, 0x01, 0x00];
    expected.extend(s.as_bytes());
    assert_cross_path(move |w| w.write_string(&s), &expected);
}

#[test]
fn cross_path_binary() {
    // bin8
    assert_cross_path(|w| w.write_binary(&[]), &[0xc4, 0x00]);
    assert_cross_path(
        |w| w.write_binary(&[0x01, 0x02, 0x03]),
        &[0xc4, 0x03, 0x01, 0x02, 0x03],
    );

    // bin16
    let data = vec![0xab; 256];
    let mut expected = vec![0xc5, 0x01, 0x00];
    expected.extend(&data);
    assert_cross_path(move |w| w.write_binary(&data), &expected);
}

#[test]
fn cross_path_timestamp() {
    // seconds in [0, 2^32), nanoseconds == 0 -> timestamp32
    assert_cross_path(
        |w| w.write_timestamp(1, 0),
        &[0xd6, 0xff, 0x00, 0x00, 0x00, 0x01],
    );

    // seconds in [0, 2^34), nanoseconds != 0 -> timestamp64
    let secs: u64 = 1;
    let nsec: u32 = 1;
    let val64 = ((nsec as u64) << 34) | secs;
    let bytes = val64.to_be_bytes();
    let mut expected = vec![0xd7, 0xff];
    expected.extend(&bytes);
    assert_cross_path(|w| w.write_timestamp(1, 1), &expected);

    // negative seconds -> timestamp96
    let nsec_bytes = 0u32.to_be_bytes();
    let sec_bytes = (-1i64).to_be_bytes();
    let mut expected = vec![0xc7, 12, 0xff];
    expected.extend(&nsec_bytes);
    expected.extend(&sec_bytes);
    assert_cross_path(|w| w.write_timestamp(-1, 0), &expected);
}

#[test]
fn cross_path_array_len() {
    // fixarray
    assert_cross_path(|w| w.write_array_len(0), &[0x90]);
    assert_cross_path(|w| w.write_array_len(15), &[0x9f]);

    // array16
    assert_cross_path(|w| w.write_array_len(16), &[0xdc, 0x00, 0x10]);

    // array32
    assert_cross_path(
        |w| w.write_array_len(65536),
        &[0xdd, 0x00, 0x01, 0x00, 0x00],
    );
}

#[test]
fn cross_path_map_len() {
    // fixmap
    assert_cross_path(|w| w.write_map_len(0), &[0x80]);
    assert_cross_path(|w| w.write_map_len(15), &[0x8f]);

    // map16
    assert_cross_path(|w| w.write_map_len(16), &[0xde, 0x00, 0x10]);

    // map32
    assert_cross_path(|w| w.write_map_len(65536), &[0xdf, 0x00, 0x01, 0x00, 0x00]);
}

#[test]
fn cross_path_ext() {
    // fixext1
    assert_cross_path(|w| w.write_ext(5, &[0xab]), &[0xd4, 0x05, 0xab]);

    // ext8
    let data = [0xab; 7];
    let mut expected = vec![0xc7, 0x07, 0x05];
    expected.extend(&data);
    assert_cross_path(move |w| w.write_ext(5, &data), &expected);
}

#[test]
fn cross_path_nested_struct() {
    #[derive(Clone)]
    struct Point {
        x: i32,
        y: i32,
    }

    impl ToMessagePack for Point {
        fn write<W: Write>(&self, writer: &mut W) -> zerompk::Result<()> {
            writer.write_array_len(2)?;
            writer.write_i32(self.x)?;
            writer.write_i32(self.y)?;
            Ok(())
        }
    }

    struct Data {
        name: &'static str,
        point: Point,
        values: Vec<i32>,
    }

    impl ToMessagePack for Data {
        fn write<W: Write>(&self, writer: &mut W) -> zerompk::Result<()> {
            writer.write_array_len(3)?;
            writer.write_string(self.name)?;
            self.point.write(writer)?;
            self.values.write(writer)?;
            Ok(())
        }
    }

    let data = Data {
        name: "hi",
        point: Point { x: 1, y: 2 },
        values: vec![3, 4, 5],
    };

    let expected = &[
        0x93, // fixarray(3)
        0xa2, b'h', b'i', // fixstr "hi"
        0x92, 0x01, 0x02, // fixarray(2), 1, 2
        0x93, 0x03, 0x04, 0x05, // fixarray(3), 3, 4, 5
    ];

    assert_cross_path_value(&data, expected);
}

#[test]
fn cross_path_option_some() {
    assert_cross_path_value(&Some(42i32), &[0x2a]);
}

#[test]
fn cross_path_option_none() {
    assert_cross_path_value(&None::<i32>, &[0xc0]);
}

#[test]
fn cross_path_vec() {
    assert_cross_path_value(&vec![1i32, 2, 3], &[0x93, 0x01, 0x02, 0x03]);
}

#[test]
fn cross_path_map() {
    use std::collections::BTreeMap;
    let mut map = BTreeMap::new();
    map.insert("a".to_string(), 1i32);
    assert_cross_path_value(&map, &[0x81, 0xa1, b'a', 0x01]);
}
