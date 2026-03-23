use zerompk::{FromMessagePack, ToMessagePack};

#[derive(Debug, PartialEq, Eq)]
struct Timestamp {
    seconds: i64,
    nanoseconds: u32,
}

impl ToMessagePack for Timestamp {
    fn write<W: zerompk::Write>(&self, writer: &mut W) -> Result<(), zerompk::Error> {
        writer.write_timestamp(self.seconds, self.nanoseconds)
    }
}

impl<'a> FromMessagePack<'a> for Timestamp {
    fn read<R: zerompk::Read<'a>>(reader: &mut R) -> Result<Self, zerompk::Error>
    where
        Self: Sized,
    {
        let (seconds, nanoseconds) = reader.read_timestamp()?;
        Ok(Self {
            seconds,
            nanoseconds,
        })
    }
}

#[test]
fn timestamp32_roundtrip() {
    let ts = Timestamp {
        seconds: 1_700_000_000,
        nanoseconds: 0,
    };

    let encoded = zerompk::to_msgpack_vec(&ts).unwrap();
    assert_eq!(encoded, vec![0xd6, 0xff, 0x65, 0x53, 0xf1, 0x00]);

    let decoded: Timestamp = zerompk::from_msgpack(&encoded).unwrap();
    assert_eq!(decoded, ts);
}

#[test]
fn timestamp64_roundtrip() {
    let ts = Timestamp {
        seconds: 4_294_967_296,
        nanoseconds: 1,
    };

    let encoded = zerompk::to_msgpack_vec(&ts).unwrap();
    assert_eq!(
        encoded,
        vec![0xd7, 0xff, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00]
    );

    let decoded: Timestamp = zerompk::from_msgpack(&encoded).unwrap();
    assert_eq!(decoded, ts);
}

#[test]
fn timestamp96_roundtrip() {
    let ts = Timestamp {
        seconds: -1,
        nanoseconds: 123_456_789,
    };

    let encoded = zerompk::to_msgpack_vec(&ts).unwrap();
    assert_eq!(
        encoded,
        vec![
            0xc7, 0x0c, 0xff, 0x07, 0x5b, 0xcd, 0x15, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff,
        ]
    );

    let decoded: Timestamp = zerompk::from_msgpack(&encoded).unwrap();
    assert_eq!(decoded, ts);
}

#[test]
fn timestamp_rejects_invalid_nanoseconds_on_write() {
    let ts = Timestamp {
        seconds: 1,
        nanoseconds: 1_000_000_000,
    };

    let err = zerompk::to_msgpack_vec(&ts).unwrap_err();
    assert!(matches!(err, zerompk::Error::InvalidTimestamp));
}

#[test]
fn timestamp_rejects_invalid_nanoseconds_on_read() {
    let data = [
        0xc7, 0x0c, 0xff, 0x3b, 0x9a, 0xca, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    let err = zerompk::from_msgpack::<Timestamp>(&data).unwrap_err();
    assert!(matches!(err, zerompk::Error::InvalidTimestamp));
}

#[cfg(feature = "chrono")]
#[test]
fn chrono_datetime_roundtrip() {
    let value = chrono::DateTime::<chrono::Utc>::from_timestamp(1_700_000_000, 0).unwrap();

    let encoded = zerompk::to_msgpack_vec(&value).unwrap();
    assert_eq!(encoded, vec![0xd6, 0xff, 0x65, 0x53, 0xf1, 0x00]);

    let decoded: chrono::DateTime<chrono::Utc> = zerompk::from_msgpack(&encoded).unwrap();
    assert_eq!(decoded, value);
}

#[cfg(feature = "chrono")]
#[test]
fn chrono_naive_datetime_roundtrip() {
    let value = chrono::DateTime::<chrono::Utc>::from_timestamp(-1, 42)
        .unwrap()
        .naive_utc();

    let encoded = zerompk::to_msgpack_vec(&value).unwrap();
    let decoded: chrono::NaiveDateTime = zerompk::from_msgpack(&encoded).unwrap();
    assert_eq!(decoded, value);
}
