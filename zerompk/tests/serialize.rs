use crate::common::Point;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};
use std::rc::Rc;

use crate::common::Nested;

mod common;

#[test]
fn test_point_serialization() {
    let point = Point { x: 10, y: 20 };
    let msgpack = zerompk::to_msgpack_vec(&point).unwrap();
    assert_eq!(msgpack, vec![0x92, 0x0a, 0x14]);
}

#[test]
fn test_point_roundtrip() {
    let point = Point { x: -10, y: 20 };
    let msgpack = zerompk::to_msgpack_vec(&point).unwrap();
    let decoded: Point = zerompk::from_msgpack(&msgpack).unwrap();
    assert_eq!(decoded, point);
}

#[test]
fn test_nested_serialization_with_some() {
    let value = Nested {
        name: "Test".to_string(),
        p1: Point { x: 10, y: 20 },
        p2: Some(Point { x: 30, y: 40 }),
        params: vec![1, 2, 3, 4, 5],
    };

    let msgpack = zerompk::to_msgpack_vec(&value).unwrap();
    assert_eq!(
        msgpack,
        vec![
            0x94, 0xa4, b'T', b'e', b's', b't', 0x92, 0x0a, 0x14, 0x92, 0x1e, 0x28, 0x95, 0x01,
            0x02, 0x03, 0x04, 0x05
        ]
    );
}

#[test]
fn test_nested_roundtrip_with_none() {
    let value = Nested {
        name: "X".to_string(),
        p1: Point { x: 1, y: 2 },
        p2: None,
        params: vec![],
    };

    let msgpack = zerompk::to_msgpack_vec(&value).unwrap();
    assert_eq!(
        msgpack,
        vec![0x94, 0xa1, b'X', 0x92, 0x01, 0x02, 0xc0, 0x90]
    );

    let decoded: Nested = zerompk::from_msgpack(&msgpack).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn test_bool_serialization() {
    assert_eq!(zerompk::to_msgpack_vec(&true).unwrap(), vec![0xc3]);
    assert_eq!(zerompk::to_msgpack_vec(&false).unwrap(), vec![0xc2]);
}

#[test]
fn test_unit_serialization() {
    assert_eq!(zerompk::to_msgpack_vec(&()).unwrap(), vec![0xc0]);
}

#[test]
fn test_option_serialization() {
    let some = Some(42i32);
    let none: Option<i32> = None;

    assert_eq!(zerompk::to_msgpack_vec(&some).unwrap(), vec![0x2a]);
    assert_eq!(zerompk::to_msgpack_vec(&none).unwrap(), vec![0xc0]);
}

#[test]
fn test_result_serialization() {
    let ok: Result<i32, String> = Ok(42);
    let err: Result<i32, String> = Err("ng".to_string());

    assert_eq!(
        zerompk::to_msgpack_vec(&ok).unwrap(),
        vec![0x92, 0xc3, 0x2a]
    );
    assert_eq!(
        zerompk::to_msgpack_vec(&err).unwrap(),
        vec![0x92, 0xc2, 0xa2, b'n', b'g']
    );

    let ok_bytes = zerompk::to_msgpack_vec(&ok).unwrap();
    let err_bytes = zerompk::to_msgpack_vec(&err).unwrap();
    let ok_decoded: Result<i32, String> = zerompk::from_msgpack(&ok_bytes).unwrap();
    let err_decoded: Result<i32, String> = zerompk::from_msgpack(&err_bytes).unwrap();
    assert_eq!(ok_decoded, ok);
    assert_eq!(err_decoded, err);
}

#[test]
fn test_unsigned_integer_boundaries() {
    assert_eq!(zerompk::to_msgpack_vec(&0u8).unwrap(), vec![0x00]);
    assert_eq!(zerompk::to_msgpack_vec(&127u8).unwrap(), vec![0x7f]);
    assert_eq!(zerompk::to_msgpack_vec(&128u8).unwrap(), vec![0xcc, 0x80]);

    assert_eq!(zerompk::to_msgpack_vec(&255u16).unwrap(), vec![0xcc, 0xff]);
    assert_eq!(
        zerompk::to_msgpack_vec(&256u16).unwrap(),
        vec![0xcd, 0x01, 0x00]
    );

    assert_eq!(
        zerompk::to_msgpack_vec(&65536u32).unwrap(),
        vec![0xce, 0x00, 0x01, 0x00, 0x00]
    );

    assert_eq!(
        zerompk::to_msgpack_vec(&4294967296u64).unwrap(),
        vec![0xcf, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]
    );
}

#[test]
fn test_signed_integer_boundaries() {
    assert_eq!(zerompk::to_msgpack_vec(&-1i8).unwrap(), vec![0xff]);
    assert_eq!(zerompk::to_msgpack_vec(&-32i8).unwrap(), vec![0xe0]);
    assert_eq!(zerompk::to_msgpack_vec(&-33i8).unwrap(), vec![0xd0, 0xdf]);

    assert_eq!(zerompk::to_msgpack_vec(&-128i16).unwrap(), vec![0xd0, 0x80]);
    assert_eq!(
        zerompk::to_msgpack_vec(&-129i16).unwrap(),
        vec![0xd1, 0xff, 0x7f]
    );

    assert_eq!(
        zerompk::to_msgpack_vec(&-32769i32).unwrap(),
        vec![0xd2, 0xff, 0xff, 0x7f, 0xff]
    );

    assert_eq!(
        zerompk::to_msgpack_vec(&-2147483649i64).unwrap(),
        vec![0xd3, 0xff, 0xff, 0xff, 0xff, 0x7f, 0xff, 0xff, 0xff]
    );
}

#[test]
fn test_signed_integer_positive_regression_roundtrip() {
    let i32_value = 32768i32;
    let i32_bytes = zerompk::to_msgpack_vec(&i32_value).unwrap();
    assert_eq!(i32_bytes, vec![0xd2, 0x00, 0x00, 0x80, 0x00]);
    let i32_decoded: i32 = zerompk::from_msgpack(&i32_bytes).unwrap();
    assert_eq!(i32_decoded, i32_value);

    let i64_value = 27394048i64;
    let i64_bytes = zerompk::to_msgpack_vec(&i64_value).unwrap();
    assert_eq!(i64_bytes, vec![0xd2, 0x01, 0xa2, 0x00, 0x00]);
    let i64_decoded: i64 = zerompk::from_msgpack(&i64_bytes).unwrap();
    assert_eq!(i64_decoded, i64_value);
}

#[test]
fn test_string_serialization_fixstr_and_str8() {
    let s31 = "a".repeat(31);
    let s32 = "b".repeat(32);

    let bytes31 = zerompk::to_msgpack_vec(&s31).unwrap();
    let bytes32 = zerompk::to_msgpack_vec(&s32).unwrap();

    assert_eq!(bytes31[0], 0xbf);
    assert_eq!(bytes31.len(), 32);

    assert_eq!(bytes32[0], 0xd9);
    assert_eq!(bytes32[1], 32);
    assert_eq!(bytes32.len(), 34);
}

#[test]
fn test_string_serialization_str16() {
    let s = "x".repeat(256);
    let bytes = zerompk::to_msgpack_vec(&s).unwrap();
    assert_eq!(bytes[0], 0xda);
    assert_eq!(bytes[1], 0x01);
    assert_eq!(bytes[2], 0x00);
    assert_eq!(bytes.len(), 259);
}

#[test]
fn test_vec_serialization_and_roundtrip() {
    let v = vec![1i32, 2, 3];
    let bytes = zerompk::to_msgpack_vec(&v).unwrap();
    assert_eq!(bytes, vec![0x93, 0x01, 0x02, 0x03]);

    let decoded: Vec<i32> = zerompk::from_msgpack(&bytes).unwrap();
    assert_eq!(decoded, v);
}

#[test]
fn test_generic_slice_serialization_as_array() {
    let value: &[i32] = &[1, 2, 3];
    let bytes = zerompk::to_msgpack_vec(&value).unwrap();
    assert_eq!(bytes, vec![0x93, 0x01, 0x02, 0x03]);
}

#[test]
fn test_cow_slice_serialization_as_array() {
    let value: Cow<'_, [i32]> = Cow::Borrowed(&[1, 2, 3]);
    let bytes = zerompk::to_msgpack_vec(&value).unwrap();
    assert_eq!(bytes, vec![0x93, 0x01, 0x02, 0x03]);
}

#[test]
fn test_vec_array16_boundary_header() {
    let v: Vec<i32> = (0..16).collect();
    let bytes = zerompk::to_msgpack_vec(&v).unwrap();
    assert_eq!(bytes[0], 0xdc);
    assert_eq!(bytes[1], 0x00);
    assert_eq!(bytes[2], 0x10);
}

#[test]
fn test_tuple_serialization_and_roundtrip() {
    let value = (1i32, true, "a".to_string());
    let bytes = zerompk::to_msgpack_vec(&value).unwrap();
    assert_eq!(bytes, vec![0x93, 0x01, 0xc3, 0xa1, b'a']);

    let decoded: (i32, bool, String) = zerompk::from_msgpack(&bytes).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn test_box_and_rc_roundtrip() {
    let boxed = Box::new(123i32);
    let rc = Rc::new(100i32);

    let boxed_bytes = zerompk::to_msgpack_vec(&boxed).unwrap();
    let rc_bytes = zerompk::to_msgpack_vec(&rc).unwrap();

    let boxed_decoded: Box<i32> = zerompk::from_msgpack(&boxed_bytes).unwrap();
    let rc_decoded: Rc<i32> = zerompk::from_msgpack(&rc_bytes).unwrap();

    assert_eq!(*boxed_decoded, 123);
    assert_eq!(*rc_decoded, 100);
}

#[test]
fn test_vecdeque_roundtrip() {
    let mut deque = VecDeque::new();
    deque.push_back(1i32);
    deque.push_back(2i32);
    deque.push_back(3i32);

    let bytes = zerompk::to_msgpack_vec(&deque).unwrap();
    let decoded: VecDeque<i32> = zerompk::from_msgpack(&bytes).unwrap();
    assert_eq!(decoded, deque);
}

#[test]
fn test_linked_list_roundtrip() {
    let mut list = LinkedList::new();
    list.push_back(10i32);
    list.push_back(20i32);

    let bytes = zerompk::to_msgpack_vec(&list).unwrap();
    let decoded: LinkedList<i32> = zerompk::from_msgpack(&bytes).unwrap();
    assert_eq!(decoded, list);
}

#[test]
fn test_btree_set_roundtrip() {
    let mut set = BTreeSet::new();
    set.insert(3i32);
    set.insert(1i32);
    set.insert(2i32);

    let bytes = zerompk::to_msgpack_vec(&set).unwrap();
    assert_eq!(bytes, vec![0x93, 0x01, 0x02, 0x03]);

    let decoded: BTreeSet<i32> = zerompk::from_msgpack(&bytes).unwrap();
    assert_eq!(decoded, set);
}

#[test]
fn test_btree_map_serialization_and_roundtrip() {
    let mut map = BTreeMap::new();
    map.insert("a".to_string(), 1i32);

    let bytes = zerompk::to_msgpack_vec(&map).unwrap();
    assert_eq!(bytes, vec![0x81, 0xa1, b'a', 0x01]); // fixmap: {"a": 1}

    let decoded: BTreeMap<String, i32> = zerompk::from_msgpack(&bytes).unwrap();
    assert_eq!(decoded, map);
}

#[test]
fn test_binary_heap_roundtrip() {
    let mut heap = BinaryHeap::new();
    heap.push(5i32);
    heap.push(1i32);
    heap.push(3i32);

    let bytes = zerompk::to_msgpack_vec(&heap).unwrap();
    let decoded: BinaryHeap<i32> = zerompk::from_msgpack(&bytes).unwrap();

    let mut original_sorted = heap.into_sorted_vec();
    let mut decoded_sorted = decoded.into_sorted_vec();
    original_sorted.reverse();
    decoded_sorted.reverse();
    assert_eq!(decoded_sorted, original_sorted);
}

#[test]
fn test_from_msgpack_slice_error_for_wrong_type() {
    let err = zerompk::from_msgpack::<i32>(&[0xc0]).unwrap_err();
    assert!(matches!(err, zerompk::Error::InvalidMarker(0xc0)));
}

#[test]
fn test_to_msgpack_with_exact_buffer() {
    let point = Point { x: 10, y: 20 };
    let mut buf = [0u8; 8];
    let written = zerompk::to_msgpack(&point, &mut buf).unwrap();
    assert_eq!(written, 3);
    assert_eq!(&buf[..3], &[0x92, 0x0a, 0x14]);
}

#[test]
fn test_to_msgpack_with_small_buffer() {
    let point = Point { x: 10, y: 20 };
    let mut buf = [0u8; 2];
    let err = zerompk::to_msgpack(&point, &mut buf).unwrap_err();
    assert!(matches!(err, zerompk::Error::BufferTooSmall));
}

#[test]
#[cfg(feature = "std")]
fn test_write_read_msgpack_std_io() {
    let point = Point { x: 7, y: 9 };
    let mut out = Vec::<u8>::new();
    zerompk::write_msgpack(&mut out, &point).unwrap();

    let decoded: Point = zerompk::read_msgpack(out.as_slice()).unwrap();
    assert_eq!(decoded, point);
}
