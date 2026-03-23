pub const NIL_MARKER: u8 = 0xc0;
pub const TRUE_MARKER: u8 = 0xc3;
pub const FALSE_MARKER: u8 = 0xc2;
pub const POS_FIXINT_START: u8 = 0x00;
pub const POS_FIXINT_END: u8 = 0x7f;
pub const NEG_FIXINT_START: u8 = 0xe0;
pub const NEG_FIXINT_END: u8 = 0xff;

pub const UINT8_MARKER: u8 = 0xcc;
pub const UINT16_MARKER: u8 = 0xcd;
pub const UINT32_MARKER: u8 = 0xce;
pub const UINT64_MARKER: u8 = 0xcf;

pub const INT8_MARKER: u8 = 0xd0;
pub const INT16_MARKER: u8 = 0xd1;
pub const INT32_MARKER: u8 = 0xd2;
pub const INT64_MARKER: u8 = 0xd3;

pub const FLOAT32_MARKER: u8 = 0xca;
pub const FLOAT64_MARKER: u8 = 0xcb;

pub const FIXMAP_START: u8 = 0x80;
pub const FIXMAP_END: u8 = 0x8f;
pub const FIXSTR_START: u8 = 0xa0;
pub const FIXSTR_END: u8 = 0xbf;
pub const STR8_MARKER: u8 = 0xd9;
pub const STR16_MARKER: u8 = 0xda;
pub const STR32_MARKER: u8 = 0xdb;
pub const BIN8_MARKER: u8 = 0xc4;
pub const BIN16_MARKER: u8 = 0xc5;
pub const BIN32_MARKER: u8 = 0xc6;
pub const FIXARRAY_START: u8 = 0x90;
pub const FIXARRAY_END: u8 = 0x9f;
pub const ARRAY16_MARKER: u8 = 0xdc;
pub const ARRAY32_MARKER: u8 = 0xdd;
pub const MAP16_MARKER: u8 = 0xde;
pub const MAP32_MARKER: u8 = 0xdf;

pub const TIMESTAMP_EXT_TYPE: i8 = -1;
pub const TIMESTAMP32_MARKER: u8 = 0xd6;
pub const TIMESTAMP64_MARKER: u8 = 0xd7;
pub const TIMESTAMP96_MARKER: u8 = 0xc7;

pub const FIXEXT1_MARKER: u8 = 0xd4;
pub const FIXEXT2_MARKER: u8 = 0xd5;
pub const FIXEXT4_MARKER: u8 = 0xd6;
pub const FIXEXT8_MARKER: u8 = 0xd7;
pub const FIXEXT16_MARKER: u8 = 0xd8;
pub const EXT8_MARKER: u8 = 0xc7;
pub const EXT16_MARKER: u8 = 0xc8;
pub const EXT32_MARKER: u8 = 0xc9;
