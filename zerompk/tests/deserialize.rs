use crate::common::{Nested, Point};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};

#[cfg(feature = "std")]
use std::io::{Cursor, ErrorKind};
use std::rc::Rc;

mod common;

#[test]
fn test_point_deserialization_from_bytes() {
    let data = [0x92, 0x0a, 0x14];
    let point: Point = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(point, Point { x: 10, y: 20 });
}

#[test]
fn test_point_deserialization_wrong_array_len() {
    let data = [0x91, 0x0a];
    let err = zerompk::from_msgpack::<Point>(&data).unwrap_err();
    assert!(matches!(
        err,
        zerompk::Error::ArrayLengthMismatch {
            expected: 2,
            actual: 1
        }
    ));
}

#[test]
fn test_point_deserialization_wrong_marker() {
    let data = [0x82];
    let err = zerompk::from_msgpack::<Point>(&data).unwrap_err();
    assert!(matches!(err, zerompk::Error::InvalidMarker(0x82)));
}

#[test]
fn test_nested_deserialization_with_some() {
    let data = [
        0x94, 0xa4, b'T', b'e', b's', b't', 0x92, 0x0a, 0x14, 0x92, 0x1e, 0x28, 0x95, 0x01, 0x02,
        0x03, 0x04, 0x05,
    ];

    let value: Nested = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(
        value,
        Nested {
            name: "Test".to_string(),
            p1: Point { x: 10, y: 20 },
            p2: Some(Point { x: 30, y: 40 }),
            params: vec![1, 2, 3, 4, 5],
        }
    );
}

#[test]
fn test_nested_deserialization_with_none() {
    let data = [0x94, 0xa1, b'X', 0x92, 0x01, 0x02, 0xc0, 0x90];
    let value: Nested = zerompk::from_msgpack(&data).unwrap();

    assert_eq!(
        value,
        Nested {
            name: "X".to_string(),
            p1: Point { x: 1, y: 2 },
            p2: None,
            params: vec![],
        }
    );
}

#[test]
fn test_bool_deserialization() {
    let t: bool = zerompk::from_msgpack(&[0xc3]).unwrap();
    let f: bool = zerompk::from_msgpack(&[0xc2]).unwrap();
    assert!(t);
    assert!(!f);
}

#[test]
fn test_unit_deserialization() {
    let unit: () = zerompk::from_msgpack(&[0xc0]).unwrap();
    assert_eq!(unit, ());
}

#[test]
fn test_option_deserialization() {
    let some: Option<i32> = zerompk::from_msgpack(&[0x2a]).unwrap();
    let none: Option<i32> = zerompk::from_msgpack(&[0xc0]).unwrap();

    assert_eq!(some, Some(42));
    assert_eq!(none, None);
}

#[test]
fn test_result_deserialization() {
    let ok_data = [0x92, 0xc3, 0x2a];
    let err_data = [0x92, 0xc2, 0xa2, b'n', b'g'];

    let ok: Result<i32, String> = zerompk::from_msgpack(&ok_data).unwrap();
    let err: Result<i32, String> = zerompk::from_msgpack(&err_data).unwrap();

    assert_eq!(ok, Ok(42));
    assert_eq!(err, Err("ng".to_string()));
}

#[test]
fn test_result_deserialization_invalid_len() {
    let data = [0x91, 0xc3];
    let err = zerompk::from_msgpack::<Result<i32, String>>(&data).unwrap_err();
    assert!(matches!(
        err,
        zerompk::Error::ArrayLengthMismatch {
            expected: 2,
            actual: 1
        }
    ));
}

#[test]
fn test_unsigned_integer_deserialization_boundaries() {
    let v0: u8 = zerompk::from_msgpack(&[0x00]).unwrap();
    let v127: u8 = zerompk::from_msgpack(&[0x7f]).unwrap();
    let v128: u8 = zerompk::from_msgpack(&[0xcc, 0x80]).unwrap();
    assert_eq!(v0, 0);
    assert_eq!(v127, 127);
    assert_eq!(v128, 128);

    let v255: u16 = zerompk::from_msgpack(&[0xcc, 0xff]).unwrap();
    let v256: u16 = zerompk::from_msgpack(&[0xcd, 0x01, 0x00]).unwrap();
    assert_eq!(v255, 255);
    assert_eq!(v256, 256);

    let v65536: u32 = zerompk::from_msgpack(&[0xce, 0x00, 0x01, 0x00, 0x00]).unwrap();
    assert_eq!(v65536, 65536);

    let v4294967296: u64 =
        zerompk::from_msgpack(&[0xcf, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]).unwrap();
    assert_eq!(v4294967296, 4294967296);
}

#[test]
fn test_unsigned_integer_deserialization_wrong_marker() {
    let err = zerompk::from_msgpack::<u32>(&[0xcf, 0, 0, 0, 0, 0, 0, 0, 1]).unwrap_err();
    assert!(matches!(err, zerompk::Error::InvalidMarker(0xcf)));
}

#[test]
fn test_signed_integer_deserialization_boundaries() {
    let m1: i8 = zerompk::from_msgpack(&[0xff]).unwrap();
    let m32: i8 = zerompk::from_msgpack(&[0xe0]).unwrap();
    let m33: i8 = zerompk::from_msgpack(&[0xd0, 0xdf]).unwrap();
    assert_eq!(m1, -1);
    assert_eq!(m32, -32);
    assert_eq!(m33, -33);

    let m128: i16 = zerompk::from_msgpack(&[0xd0, 0x80]).unwrap();
    let m129: i16 = zerompk::from_msgpack(&[0xd1, 0xff, 0x7f]).unwrap();
    assert_eq!(m128, -128);
    assert_eq!(m129, -129);

    let m32769: i32 = zerompk::from_msgpack(&[0xd2, 0xff, 0xff, 0x7f, 0xff]).unwrap();
    assert_eq!(m32769, -32769);

    let m2147483649: i64 =
        zerompk::from_msgpack(&[0xd3, 0xff, 0xff, 0xff, 0xff, 0x7f, 0xff, 0xff, 0xff]).unwrap();
    assert_eq!(m2147483649, -2147483649);
}

#[test]
fn test_string_deserialization_fixstr_str8_str16() {
    let s31 = "a".repeat(31);
    let s32 = "b".repeat(32);
    let s256 = "x".repeat(256);

    let mut d31 = vec![0xbf];
    d31.extend_from_slice(s31.as_bytes());

    let mut d32 = vec![0xd9, 32];
    d32.extend_from_slice(s32.as_bytes());

    let mut d256 = vec![0xda, 0x01, 0x00];
    d256.extend_from_slice(s256.as_bytes());

    let r31: String = zerompk::from_msgpack(&d31).unwrap();
    let r32: String = zerompk::from_msgpack(&d32).unwrap();
    let r256: String = zerompk::from_msgpack(&d256).unwrap();

    assert_eq!(r31, s31);
    assert_eq!(r32, s32);
    assert_eq!(r256, s256);
}

#[test]
fn test_string_deserialization_invalid_utf8() {
    let err = zerompk::from_msgpack::<String>(&[0xa1, 0xff]).unwrap_err();
    assert!(matches!(err, zerompk::Error::InvalidUtf8(_)));
}

#[test]
fn test_borrowed_str_deserialization_zero_copy() {
    let data = [0xa5, b'h', b'e', b'l', b'l', b'o'];
    let value: &str = zerompk::from_msgpack(&data).unwrap();

    assert_eq!(value, "hello");
    assert_eq!(value.as_ptr(), data[1..].as_ptr());
}

#[test]
fn test_vec_deserialization() {
    let data = [0x93, 0x01, 0x02, 0x03];
    let value: Vec<i32> = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(value, vec![1, 2, 3]);
}

#[test]
fn test_vec_array16_deserialization() {
    let mut data = vec![0xdc, 0x00, 0x10];
    data.extend(0u8..16u8);
    let value: Vec<u8> = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(value, (0u8..16u8).collect::<Vec<_>>());
}

#[test]
fn test_tuple_deserialization() {
    let data = [0x93, 0x01, 0xc3, 0xa1, b'a'];
    let value: (i32, bool, String) = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(value, (1, true, "a".to_string()));
}

#[test]
fn test_tuple_deserialization_length_mismatch() {
    let data = [0x92, 0x01, 0xc3];
    let err = zerompk::from_msgpack::<(i32, bool, String)>(&data).unwrap_err();
    assert!(matches!(
        err,
        zerompk::Error::ArrayLengthMismatch {
            expected: 3,
            actual: 2
        }
    ));
}

#[test]
fn test_box_and_rc_deserialization() {
    let boxed: Box<i32> = zerompk::from_msgpack(&[0x7b]).unwrap();
    let rc: Rc<i32> = zerompk::from_msgpack(&[0x64]).unwrap();
    assert_eq!(*boxed, 123);
    assert_eq!(*rc, 100);
}

#[test]
fn test_vecdeque_deserialization() {
    let data = [0x93, 0x01, 0x02, 0x03];
    let value: VecDeque<i32> = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(value.into_iter().collect::<Vec<_>>(), vec![1, 2, 3]);
}

#[test]
fn test_linked_list_deserialization() {
    let data = [0x92, 0x0a, 0x14];
    let value: LinkedList<i32> = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(value.into_iter().collect::<Vec<_>>(), vec![10, 20]);
}

#[test]
fn test_btree_set_deserialization() {
    let data = [0x93, 0x03, 0x01, 0x02];
    let value: BTreeSet<i32> = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(value.into_iter().collect::<Vec<_>>(), vec![1, 2, 3]);
}

#[test]
fn test_btree_map_deserialization() {
    let data = [0x81, 0xa1, b'a', 0x01]; // fixmap: {"a": 1}
    let value: BTreeMap<String, i32> = zerompk::from_msgpack(&data).unwrap();

    let mut expected = BTreeMap::new();
    expected.insert("a".to_string(), 1);
    assert_eq!(value, expected);
}

#[test]
fn test_binary_heap_deserialization() {
    let data = [0x93, 0x05, 0x01, 0x03];
    let value: BinaryHeap<i32> = zerompk::from_msgpack(&data).unwrap();

    let mut sorted = value.into_sorted_vec();
    sorted.reverse();
    assert_eq!(sorted, vec![5, 3, 1]);
}

#[test]
fn test_deserialization_with_trailing_bytes_is_accepted() {
    let data = [0x01, 0x02, 0x03];
    let value: i32 = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(value, 1);
}

#[test]
#[cfg(feature = "std")]
fn test_read_msgpack_std_io_success() {
    let data = [0x92, 0x0a, 0x14];
    let point: Point = zerompk::read_msgpack(Cursor::new(data)).unwrap();
    assert_eq!(point, Point { x: 10, y: 20 });
}

#[test]
#[cfg(feature = "std")]
fn test_read_msgpack_std_io_invalid_marker() {
    let data = [0x82, 0x00, 0x00];
    let err = zerompk::read_msgpack::<_, Point>(Cursor::new(data)).unwrap_err();
    assert!(matches!(err, zerompk::Error::InvalidMarker(0x82)));
}

#[test]
#[cfg(feature = "std")]
fn test_read_msgpack_std_io_unexpected_eof() {
    let data = [0x92, 0x01];
    let err = zerompk::read_msgpack::<_, Point>(Cursor::new(data)).unwrap_err();
    match err {
        zerompk::Error::IoError(io_err) => assert_eq!(io_err.kind(), ErrorKind::UnexpectedEof),
        _ => panic!("expected IoError(UnexpectedEof), got: {err:?}"),
    }
}
