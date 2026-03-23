use serde::{Deserialize, Serialize};
use zerompk::{FromMessagePack, ToMessagePack};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl ToMessagePack for Point {
    fn write<W: zerompk::Write>(&self, writer: &mut W) -> Result<(), zerompk::Error> {
        writer.write_array_len(2)?;
        writer.write_i32(self.x)?;
        writer.write_i32(self.y)?;
        Ok(())
    }
}

impl<'a> FromMessagePack<'a> for Point {
    fn read<R: zerompk::Read<'a>>(reader: &mut R) -> Result<Self, zerompk::Error>
    where
        Self: Sized,
    {
        let len = reader.read_array_len()?;
        if len != 2 {
            return Err(zerompk::Error::ArrayLengthMismatch {
                expected: 2,
                actual: len,
            });
        }

        let x = reader.read_i32()?;
        let y = reader.read_i32()?;

        Ok(Point { x, y })
    }
}

#[derive(Serialize, Debug, PartialEq, Eq, Clone)]
pub struct Nested {
    pub name: String,
    pub p1: Point,
    pub p2: Option<Point>,
    pub params: Vec<i32>,
}

impl ToMessagePack for Nested {
    fn write<W: zerompk::Write>(&self, writer: &mut W) -> Result<(), zerompk::Error> {
        writer.write_byte(0x94)?; // Array of 4 elements
        writer.write_string(&self.name)?;
        self.p1.write(writer)?;
        self.p2.write(writer)?;
        self.params.write(writer)?;
        Ok(())
    }
}

impl<'a> FromMessagePack<'a> for Nested {
    fn read<R: zerompk::Read<'a>>(reader: &mut R) -> Result<Self, zerompk::Error>
    where
        Self: Sized,
    {
        let len = reader.read_array_len()?;
        if len != 4 {
            return Err(zerompk::Error::ArrayLengthMismatch {
                expected: 4,
                actual: len,
            });
        }

        let name = reader.read_string()?;
        let p1 = Point::read(reader)?;
        let p2 = Option::<Point>::read(reader)?;
        let params = Vec::<i32>::read(reader)?;

        Ok(Nested {
            name: name.into_owned(),
            p1,
            p2,
            params,
        })
    }
}
