use zerompk_derive::{FromMessagePack, ToMessagePack};

#[derive(ToMessagePack, FromMessagePack, Debug, PartialEq)]
struct PointArray {
    x: i32,
    y: i32,
}

#[derive(ToMessagePack, FromMessagePack, Debug, PartialEq)]
#[msgpack(map)]
struct PointMap {
    x: i32,
    y: i32,
}

#[derive(ToMessagePack, FromMessagePack, Debug, PartialEq)]
struct PointArrayWithIndex {
    #[msgpack(key = 0)]
    x: i32,
    #[msgpack(key = 2)]
    y: i32,
}

#[derive(ToMessagePack, FromMessagePack, Debug, PartialEq)]
#[msgpack(map)]
struct PointMapWithKey {
    #[msgpack(key = "px")]
    x: i32,
    #[msgpack(key = "py")]
    y: i32,
}

#[derive(ToMessagePack, FromMessagePack, Debug, PartialEq)]
#[msgpack(map)]
struct LongMapKeyPoint {
    #[msgpack(key = "abcdefghX")]
    x: i32,
    #[msgpack(key = "zzzzzzzzz")]
    y: i32,
}

#[derive(ToMessagePack, FromMessagePack, Debug, PartialEq)]
struct UnitStruct;

#[derive(ToMessagePack, FromMessagePack, Debug, PartialEq)]
struct EmptyTupleStruct();

#[derive(ToMessagePack, FromMessagePack, Debug, PartialEq)]
struct TupleStruct(i32, String);

#[derive(ToMessagePack, FromMessagePack, Debug, PartialEq)]
struct EmptyStruct {}

#[derive(ToMessagePack, FromMessagePack, Debug, PartialEq)]
#[msgpack(map)]
struct EmptyStructWithMap {}

#[derive(ToMessagePack, FromMessagePack, Debug, PartialEq)]
struct NewtypeStruct(i32);

#[derive(ToMessagePack, FromMessagePack, Debug, PartialEq)]
struct IgnoreArrayField {
    x: i32,
    #[msgpack(ignore)]
    note: String,
    y: i32,
}

#[derive(ToMessagePack, FromMessagePack, Debug, PartialEq)]
#[msgpack(map)]
struct IgnoreMapField {
    x: i32,
    #[msgpack(ignore)]
    note: String,
    y: i32,
}

#[derive(ToMessagePack, FromMessagePack, Debug, PartialEq)]
enum Event {
    A,
    #[msgpack(key = "p")]
    Point {
        x: i32,
        y: i32,
    },
    #[msgpack(key = 2)]
    Tuple(#[msgpack(key = 0)] i32, #[msgpack(key = 2)] i32),
    #[msgpack(key = "m")]
    #[msgpack(map)]
    Mapped {
        #[msgpack(key = "x1")]
        x: i32,
        y: i32,
    },
    IgnoredNamed {
        x: i32,
        #[msgpack(ignore)]
        note: String,
        y: i32,
    },
    #[msgpack(map)]
    IgnoredMapNamed {
        x: i32,
        #[msgpack(ignore)]
        note: String,
        y: i32,
    },
}

#[derive(ToMessagePack, FromMessagePack, Debug, PartialEq)]
struct RecursiveNode {
    next: Option<Box<RecursiveNode>>,
}

#[derive(ToMessagePack, FromMessagePack, Debug, PartialEq)]
struct BorrowedPayload<'a> {
    text: &'a str,
    data: &'a [u8],
}

fn recursive_node_msgpack(depth: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(depth + 1);
    for _ in 0..depth {
        out.push(0x91); // [next]
    }
    out.push(0xc0); // None
    out
}

#[test]
fn derive_array_default() {
    let point = PointArray { x: 10, y: 20 };
    let data = zerompk::to_msgpack_vec(&point).unwrap();
    assert_eq!(data, vec![0x92, 0x0a, 0x14]);

    let decoded: PointArray = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(decoded, point);
}

#[test]
fn derive_map_with_attribute() {
    let point = PointMap { x: 10, y: 20 };
    let data = zerompk::to_msgpack_vec(&point).unwrap();
    assert_eq!(data, vec![0x82, 0xa1, b'x', 0x0a, 0xa1, b'y', 0x14]);

    let decoded: PointMap = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(decoded, point);
}

#[test]
fn derive_array_with_field_index_and_nil_gap() {
    let point = PointArrayWithIndex { x: 10, y: 20 };
    let data = zerompk::to_msgpack_vec(&point).unwrap();
    assert_eq!(data, vec![0x93, 0x0a, 0xc0, 0x14]);

    let decoded: PointArrayWithIndex = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(decoded, point);
}

#[test]
fn derive_map_with_field_key() {
    let point = PointMapWithKey { x: 10, y: 20 };
    let data = zerompk::to_msgpack_vec(&point).unwrap();
    assert_eq!(
        data,
        vec![0x82, 0xa2, b'p', b'x', 0x0a, 0xa2, b'p', b'y', 0x14]
    );

    let decoded: PointMapWithKey = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(decoded, point);
}

#[test]
fn derive_map_rejects_unknown_key_with_same_len_and_prefix() {
    // {"abcdefghY": 1, "zzzzzzzzz": 2}
    let data = vec![
        0x82, 0xa9, b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'Y', 0x01, 0xa9, b'z', b'z',
        b'z', b'z', b'z', b'z', b'z', b'z', b'z', 0x02,
    ];

    let err = zerompk::from_msgpack::<LongMapKeyPoint>(&data).unwrap_err();
    assert!(matches!(
        err,
        zerompk::Error::UnknownKey(ref key) if key == "abcdefghY"
    ));
}

#[test]
fn derive_unit_struct() {
    let unit = UnitStruct;
    let data = zerompk::to_msgpack_vec(&unit).unwrap();
    assert_eq!(data, vec![0xc0]); // nil

    let decoded: UnitStruct = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(decoded, unit);
}

#[test]
fn derive_empty_struct() {
    let empty = EmptyStruct {};
    let data = zerompk::to_msgpack_vec(&empty).unwrap();
    assert_eq!(data, vec![0x90]); // empty array

    let decoded: EmptyStruct = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(decoded, empty);
}

#[test]
fn derive_empty_struct_with_map() {
    let empty = EmptyStructWithMap {};
    let data = zerompk::to_msgpack_vec(&empty).unwrap();
    assert_eq!(data, vec![0x80]); // empty map

    let decoded: EmptyStructWithMap = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(decoded, empty);
}

#[test]
fn derive_empty_tuple_struct() {
    let empty = EmptyTupleStruct();
    let data = zerompk::to_msgpack_vec(&empty).unwrap();
    assert_eq!(data, vec![0x90]); // empty array

    let decoded: EmptyTupleStruct = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(decoded, empty);
}

#[test]
fn derive_tuple_struct() {
    let tuple = TupleStruct(42, "hello".to_string());
    let data = zerompk::to_msgpack_vec(&tuple).unwrap();
    assert_eq!(data, vec![0x92, 0x2a, 0xa5, b'h', b'e', b'l', b'l', b'o']);

    let decoded: TupleStruct = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(decoded, tuple);
}

#[test]
fn derive_borrowed_fields_are_zero_copy_and_bin() {
    let value = BorrowedPayload {
        text: "hi",
        data: &[1, 2, 3],
    };

    let encoded = zerompk::to_msgpack_vec(&value).unwrap();
    assert_eq!(
        encoded,
        vec![0x92, 0xa2, b'h', b'i', 0xc4, 0x03, 0x01, 0x02, 0x03]
    );

    let decoded: BorrowedPayload = zerompk::from_msgpack(&encoded).unwrap();
    assert_eq!(decoded, value);

    assert_eq!(decoded.text.as_ptr(), encoded[2..4].as_ptr());
    assert_eq!(decoded.data.as_ptr(), encoded[6..9].as_ptr());
}

#[test]
fn derive_struct_newtype() {
    let value = NewtypeStruct(42);
    let data = zerompk::to_msgpack_vec(&value).unwrap();
    assert_eq!(data, vec![0x2a]); // 42

    let decoded: NewtypeStruct = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn derive_array_ignores_field() {
    let value = IgnoreArrayField {
        x: 10,
        note: "ignored".to_string(),
        y: 20,
    };

    let data = zerompk::to_msgpack_vec(&value).unwrap();
    assert_eq!(data, vec![0x92, 0x0a, 0x14]);

    let decoded: IgnoreArrayField = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(
        decoded,
        IgnoreArrayField {
            x: 10,
            note: String::new(),
            y: 20,
        }
    );
}

#[test]
fn derive_map_ignores_field() {
    let value = IgnoreMapField {
        x: 10,
        note: "ignored".to_string(),
        y: 20,
    };

    let data = zerompk::to_msgpack_vec(&value).unwrap();
    assert_eq!(data, vec![0x82, 0xa1, b'x', 0x0a, 0xa1, b'y', 0x14]);

    let decoded: IgnoreMapField = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(
        decoded,
        IgnoreMapField {
            x: 10,
            note: String::new(),
            y: 20,
        }
    );
}

#[test]
fn derive_enum_unit_variant() {
    let value = Event::A;
    let data = zerompk::to_msgpack_vec(&value).unwrap();
    assert_eq!(data, vec![0x92, 0xa1, b'A', 0xc0]); // ["A", nil]

    let decoded: Event = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn derive_enum_named_array_variant() {
    let value = Event::Point { x: 10, y: 20 };
    let data = zerompk::to_msgpack_vec(&value).unwrap();
    assert_eq!(data, vec![0x92, 0xa1, b'p', 0x92, 0x0a, 0x14]);

    let decoded: Event = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn derive_enum_tuple_variant_with_gap() {
    let value = Event::Tuple(10, 20);
    let data = zerompk::to_msgpack_vec(&value).unwrap();
    assert_eq!(data, vec![0x92, 0x02, 0x93, 0x0a, 0xc0, 0x14]);

    let decoded: Event = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn derive_enum_named_map_variant() {
    let value = Event::Mapped { x: 10, y: 20 };
    let data = zerompk::to_msgpack_vec(&value).unwrap();
    assert_eq!(
        data,
        vec![
            0x92, 0xa1, b'm', 0x82, 0xa2, b'x', b'1', 0x0a, 0xa1, b'y', 0x14
        ]
    );

    let decoded: Event = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn derive_enum_named_array_variant_ignores_field() {
    let value = Event::IgnoredNamed {
        x: 10,
        note: "ignored".to_string(),
        y: 20,
    };
    let data = zerompk::to_msgpack_vec(&value).unwrap();
    assert_eq!(
        data,
        vec![
            0x92, 0xac, b'I', b'g', b'n', b'o', b'r', b'e', b'd', b'N', b'a', b'm', b'e', b'd',
            0x92, 0x0a, 0x14,
        ]
    );

    let decoded: Event = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(
        decoded,
        Event::IgnoredNamed {
            x: 10,
            note: String::new(),
            y: 20,
        }
    );
}

#[test]
fn derive_enum_named_map_variant_ignores_field() {
    let value = Event::IgnoredMapNamed {
        x: 10,
        note: "ignored".to_string(),
        y: 20,
    };
    let data = zerompk::to_msgpack_vec(&value).unwrap();
    assert_eq!(
        data,
        vec![
            0x92, 0xaf, b'I', b'g', b'n', b'o', b'r', b'e', b'd', b'M', b'a', b'p', b'N', b'a',
            b'm', b'e', b'd', 0x82, 0xa1, b'x', 0x0a, 0xa1, b'y', 0x14,
        ]
    );

    let decoded: Event = zerompk::from_msgpack(&data).unwrap();
    assert_eq!(
        decoded,
        Event::IgnoredMapNamed {
            x: 10,
            note: String::new(),
            y: 20,
        }
    );
}

#[test]
fn derive_deserialize_depth_limit_max_is_ok() {
    let data = recursive_node_msgpack(500);
    let decoded: RecursiveNode = zerompk::from_msgpack(&data).unwrap();

    let mut depth = 0usize;
    let mut cur = &decoded;
    while let Some(next) = &cur.next {
        depth += 1;
        cur = next;
    }
    assert_eq!(depth + 1, 500);
}

#[test]
fn derive_deserialize_depth_limit_exceeded() {
    let data = recursive_node_msgpack(501);
    let err = zerompk::from_msgpack::<RecursiveNode>(&data).unwrap_err();
    assert!(matches!(
        err,
        zerompk::Error::DepthLimitExceeded { max: 500 }
    ));
}
