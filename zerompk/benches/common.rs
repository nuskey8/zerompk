use msgpacker::MsgPacker;
use serde::{Deserialize, Serialize};
use zerompk_derive::{FromMessagePack, ToMessagePack};

#[derive(ToMessagePack, FromMessagePack, Serialize, Deserialize, MsgPacker)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(ToMessagePack, FromMessagePack, Serialize, Deserialize)]
#[msgpack(map)]
#[allow(unused)]
pub struct Nested {
    pub name: String,
    pub p1: Point,
    pub p2: Option<Point>,
    pub params: Vec<i32>,
}

#[derive(ToMessagePack, FromMessagePack, Serialize, Deserialize)]
#[msgpack(map)]
#[allow(unused)]
pub struct NestedLongKey {
    pub my_property_identifier_alpha: String,
    pub my_property_identifier_beta_point1: Point,
    pub my_property_identifier_gamma_point2: Option<Point>,
    pub my_property_identifier_delta_parameters: Vec<i32>,
}

#[derive(ToMessagePack, FromMessagePack, Serialize, Deserialize, MsgPacker)]
#[msgpack(array)]
#[allow(unused)]
pub struct NestedArray {
    pub name: String,
    pub p1: Point,
    pub p2: Option<Point>,
    pub params: Vec<i32>,
}
