//! Tests for `#[msgpack(default)]` (fill missing keys) and
//! `#[msgpack(allow_unknown_fields)]` (skip unknown keys), plus their
//! interaction. These two opt-ins are intentionally orthogonal.

use zerompk::{FromMessagePack, ToMessagePack};
use zerompk_derive::{
    FromMessagePack as DeriveFromMessagePack, ToMessagePack as DeriveToMessagePack,
};

fn encode<T: ToMessagePack>(value: &T) -> Vec<u8> {
    zerompk::to_msgpack_vec(value).unwrap()
}

fn decode<'a, T: FromMessagePack<'a>>(bytes: &'a [u8]) -> Result<T, zerompk::Error> {
    zerompk::from_msgpack(bytes)
}

// ---------------------------------------------------------------------------
// V1 schema (writer side): the "old" version of a message.
// ---------------------------------------------------------------------------

#[derive(DeriveToMessagePack, DeriveFromMessagePack, Debug, PartialEq)]
#[msgpack(map)]
struct V1 {
    a: i32,
    b: i32,
}

// ---------------------------------------------------------------------------
// `default` only: fill missing keys, but unknown keys must still error.
// This is the "I added a new field" evolution direction.
// ---------------------------------------------------------------------------

#[derive(DeriveFromMessagePack, Debug, PartialEq)]
#[msgpack(map)]
struct V2DefaultsOnly {
    a: i32,
    b: i32,
    #[msgpack(default)]
    c: i32,
}

#[test]
fn defaults_fill_missing_keys() {
    let v1 = V1 { a: 1, b: 2 };
    let bytes = encode(&v1);
    let v2: V2DefaultsOnly = decode(&bytes).unwrap();
    assert_eq!(v2, V2DefaultsOnly { a: 1, b: 2, c: 0 });
}

#[test]
fn defaults_alone_still_reject_unknown_keys() {
    // Writer emits an extra unknown key `z`; reader has defaults but not
    // `allow_unknown_fields`. Decode must fail loudly.
    #[derive(DeriveToMessagePack)]
    #[msgpack(map)]
    struct WithExtra {
        a: i32,
        b: i32,
        z: i32,
    }
    let bytes = encode(&WithExtra { a: 1, b: 2, z: 99 });
    let err = decode::<V2DefaultsOnly>(&bytes).unwrap_err();
    match err {
        zerompk::Error::KeyNotFound(k) => assert_eq!(k, "z"),
        other => panic!("expected KeyNotFound, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// `allow_unknown_fields` only: skip unknown keys, but missing keys must
// still error. This is the "I removed a field" evolution direction.
// ---------------------------------------------------------------------------

#[derive(DeriveFromMessagePack, Debug, PartialEq)]
#[msgpack(map, allow_unknown_fields)]
struct V0AllowUnknownOnly {
    a: i32,
}

#[test]
fn allow_unknown_skips_extra_keys() {
    let v1 = V1 { a: 1, b: 2 };
    let bytes = encode(&v1);
    let v0: V0AllowUnknownOnly = decode(&bytes).unwrap();
    assert_eq!(v0, V0AllowUnknownOnly { a: 1 });
}

#[test]
fn allow_unknown_alone_still_requires_all_keys() {
    // Writer is missing key `a`; reader allows unknowns but `a` has no default.
    #[derive(DeriveToMessagePack)]
    #[msgpack(map)]
    struct OnlyB {
        b: i32,
    }
    let bytes = encode(&OnlyB { b: 5 });
    let err = decode::<V0AllowUnknownOnly>(&bytes).unwrap_err();
    assert!(matches!(err, zerompk::Error::KeyNotFound(_)));
}

// ---------------------------------------------------------------------------
// Both: full schema evolution — accept missing keys (defaulted) and skip
// unknown keys.
// ---------------------------------------------------------------------------

#[derive(DeriveFromMessagePack, Debug, PartialEq)]
#[msgpack(map, allow_unknown_fields)]
struct VFull {
    a: i32,
    #[msgpack(default)]
    new_field: i32,
}

#[test]
fn both_modes_compose() {
    // V1 has `b` (unknown to VFull) and lacks `new_field` (defaulted).
    let v1 = V1 { a: 7, b: 2 };
    let bytes = encode(&v1);
    let v: VFull = decode(&bytes).unwrap();
    assert_eq!(v, VFull { a: 7, new_field: 0 });
}

// ---------------------------------------------------------------------------
// `default = "path"` form.
// ---------------------------------------------------------------------------

fn forty_two() -> i32 {
    42
}

#[derive(DeriveFromMessagePack, Debug, PartialEq)]
#[msgpack(map)]
struct V2DefaultPath {
    a: i32,
    b: i32,
    #[msgpack(default = "forty_two")]
    c: i32,
}

#[test]
fn default_path_invokes_named_function() {
    let bytes = encode(&V1 { a: 1, b: 2 });
    let v: V2DefaultPath = decode(&bytes).unwrap();
    assert_eq!(v, V2DefaultPath { a: 1, b: 2, c: 42 });
}

// ---------------------------------------------------------------------------
// Strict-by-default: an untouched struct still rejects missing keys and
// extra keys. This guards against the strict-mode codegen regressing.
// ---------------------------------------------------------------------------

#[test]
fn strict_default_rejects_missing_key() {
    #[derive(DeriveToMessagePack)]
    #[msgpack(map)]
    struct OnlyA {
        a: i32,
    }
    let bytes = encode(&OnlyA { a: 1 });
    let err = decode::<V1>(&bytes).unwrap_err();
    // 0.4.1 strict path uses check_map_len, which surfaces a length error.
    assert!(matches!(err, zerompk::Error::MapLengthMismatch { .. }));
}

#[test]
fn strict_default_rejects_extra_key() {
    #[derive(DeriveToMessagePack)]
    #[msgpack(map)]
    struct WithExtra {
        a: i32,
        b: i32,
        c: i32,
    }
    let bytes = encode(&WithExtra { a: 1, b: 2, c: 3 });
    let err = decode::<V1>(&bytes).unwrap_err();
    assert!(matches!(err, zerompk::Error::MapLengthMismatch { .. }));
}

// ---------------------------------------------------------------------------
// Round-trip: writer emits all declared fields, so V2-encoded → V2-decoded
// preserves values regardless of mode.
// ---------------------------------------------------------------------------

#[test]
fn round_trip_preserves_values() {
    #[derive(DeriveToMessagePack, DeriveFromMessagePack, Debug, PartialEq)]
    #[msgpack(map, allow_unknown_fields)]
    struct V {
        a: i32,
        #[msgpack(default)]
        b: i32,
    }
    let original = V { a: 10, b: 20 };
    let bytes = encode(&original);
    let decoded: V = decode(&bytes).unwrap();
    assert_eq!(original, decoded);
}
