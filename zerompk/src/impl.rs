use crate::{Error, FromMessagePack, Read, ToMessagePack, Write};
use alloc::string::ToString;

#[cfg(feature = "std")]
use core::hash::Hash;

// -------------------------------------------------------------------------------
// primitive types
// -------------------------------------------------------------------------------

macro_rules! impl_scalar {
    ($ty:ty, $write_fn:ident, $read_fn:ident) => {
        impl<'a> FromMessagePack<'a> for $ty {
            #[inline(always)]
            fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
            where
                Self: Sized,
            {
                reader.$read_fn()
            }
        }

        impl ToMessagePack for $ty {
            #[inline(always)]
            fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
                writer.$write_fn(*self)
            }
        }
    };
}

impl_scalar!(bool, write_boolean, read_boolean);
impl_scalar!(i8, write_i8, read_i8);
impl_scalar!(i16, write_i16, read_i16);
impl_scalar!(i32, write_i32, read_i32);
impl_scalar!(i64, write_i64, read_i64);
impl_scalar!(u8, write_u8, read_u8);
impl_scalar!(u16, write_u16, read_u16);
impl_scalar!(u32, write_u32, read_u32);
impl_scalar!(u64, write_u64, read_u64);
impl_scalar!(f32, write_f32, read_f32);
impl_scalar!(f64, write_f64, read_f64);

impl<'a> FromMessagePack<'a> for usize {
    #[inline(always)]
    fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        if usize::BITS <= 32 {
            reader.read_u32().map(|v| v as usize)
        } else {
            reader.read_u64().map(|v| v as usize)
        }
    }
}

impl ToMessagePack for usize {
    #[inline(always)]
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        if usize::BITS <= 32 {
            writer.write_u32(*self as u32)
        } else {
            writer.write_u64(*self as u64)
        }
    }
}

impl<'a> FromMessagePack<'a> for isize {
    #[inline(always)]
    fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        if isize::BITS <= 32 {
            reader.read_i32().map(|v| v as isize)
        } else {
            reader.read_i64().map(|v| v as isize)
        }
    }
}

impl ToMessagePack for isize {
    #[inline(always)]
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        if isize::BITS <= 32 {
            writer.write_i32(*self as i32)
        } else {
            writer.write_i64(*self as i64)
        }
    }
}

impl<'a> FromMessagePack<'a> for char {
    #[inline(always)]
    fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let code = reader.read_u32()?;
        match char::from_u32(code) {
            Some(c) => Ok(c),
            None => Err(Error::InvalidChar(code)),
        }
    }
}

impl ToMessagePack for char {
    #[inline(always)]
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        writer.write_u32(*self as u32)
    }
}

// -------------------------------------------------------------------------------
// PhantomData
// -------------------------------------------------------------------------------

impl<'a, T> FromMessagePack<'a> for core::marker::PhantomData<T> {
    #[inline(always)]
    fn read<R: Read<'a>>(_: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(core::marker::PhantomData)
    }
}

impl<T> ToMessagePack for core::marker::PhantomData<T> {
    #[inline(always)]
    fn write<W: Write>(&self, _: &mut W) -> crate::Result<()> {
        Ok(())
    }
}

// -------------------------------------------------------------------------------
// string, binary types
// -------------------------------------------------------------------------------

impl<'de, 'a> FromMessagePack<'de> for &'a str
where
    'de: 'a,
{
    #[inline(always)]
    fn read<R: Read<'de>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        match reader.read_string()? {
            alloc::borrow::Cow::Borrowed(s) => Ok(s),
            alloc::borrow::Cow::Owned(_) => Err(crate::Error::CannotBorrow),
        }
    }
}

impl<'a> ToMessagePack for &'a str {
    #[inline(always)]
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        writer.write_string(*self)
    }
}

impl<'de, 'a> FromMessagePack<'de> for &'a [u8]
where
    'de: 'a,
{
    #[inline(always)]
    fn read<R: Read<'de>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        match reader.read_binary()? {
            alloc::borrow::Cow::Borrowed(s) => Ok(s),
            alloc::borrow::Cow::Owned(_) => Err(crate::Error::CannotBorrow),
        }
    }
}

impl<T: ToMessagePack> ToMessagePack for [T] {
    #[inline(always)]
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        writer.write_array_len(self.len())?;
        for item in self {
            item.write(writer)?;
        }
        Ok(())
    }
}

impl<'a, T: FromMessagePack<'a>, const N: usize> FromMessagePack<'a> for [T; N] {
    #[inline(always)]
    fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self> {
        reader.check_array_len(N)?;
        let mut arr: core::mem::MaybeUninit<[T; N]> = core::mem::MaybeUninit::uninit();
        let ptr = arr.as_mut_ptr() as *mut T;
        for i in 0..N {
            unsafe {
                ptr.add(i).write(T::read(reader)?);
            }
        }
        Ok(unsafe { arr.assume_init() })
    }
}

impl<T: ToMessagePack, const N: usize> ToMessagePack for [T; N] {
    #[inline(always)]
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        writer.write_array_len(N)?;
        for item in self {
            item.write(writer)?;
        }
        Ok(())
    }
}

impl<'a> FromMessagePack<'a> for alloc::string::String {
    #[inline(always)]
    fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        match reader.read_string()? {
            alloc::borrow::Cow::Borrowed(s) => Ok(s.to_string()),
            alloc::borrow::Cow::Owned(s) => Ok(s),
        }
    }
}

impl ToMessagePack for alloc::string::String {
    #[inline(always)]
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        writer.write_string(self)
    }
}

impl<'a> FromMessagePack<'a> for alloc::borrow::Cow<'a, str> {
    #[inline(always)]
    fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        reader.read_string()
    }
}

impl ToMessagePack for alloc::borrow::Cow<'_, str> {
    #[inline(always)]
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        writer.write_string(self)
    }
}

impl<'de, 'a, T> FromMessagePack<'de> for alloc::borrow::Cow<'a, [T]>
where
    'de: 'a,
    T: Clone + FromMessagePack<'de>,
{
    #[inline(always)]
    fn read<R: Read<'de>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(alloc::borrow::Cow::Owned(reader.read_array()?))
    }
}

impl<T: Clone + ToMessagePack> ToMessagePack for alloc::borrow::Cow<'_, [T]> {
    #[inline(always)]
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        self.as_ref().write(writer)
    }
}

// -------------------------------------------------------------------------------
// reference types
// -------------------------------------------------------------------------------

impl<T: ToMessagePack + ?Sized> ToMessagePack for &T {
    #[inline(always)]
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        T::write(&self, writer)
    }
}

impl<T: ToMessagePack + ?Sized> ToMessagePack for &mut T {
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        T::write(&self, writer)
    }
}

// -------------------------------------------------------------------------------
// option and result types
// -------------------------------------------------------------------------------

impl<'a, T: FromMessagePack<'a>> FromMessagePack<'a> for Option<T> {
    #[inline(always)]
    fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        reader.read_option()
    }
}

impl<T: ToMessagePack> ToMessagePack for Option<T> {
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        match self {
            Some(value) => value.write(writer),
            None => writer.write_nil(),
        }
    }
}

impl<'a, T: FromMessagePack<'a>, E: FromMessagePack<'a>> FromMessagePack<'a>
    for core::result::Result<T, E>
{
    fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        reader.check_array_len(2)?;
        let is_ok = reader.read_boolean()?;
        if is_ok {
            Ok(core::result::Result::Ok(T::read(reader)?))
        } else {
            Ok(core::result::Result::Err(E::read(reader)?))
        }
    }
}

impl<T: ToMessagePack, E: ToMessagePack> ToMessagePack for core::result::Result<T, E> {
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        match self {
            Ok(value) => {
                writer.write_array_len(2)?;
                writer.write_boolean(true)?; // Ok variant
                value.write(writer)
            }
            Err(err) => {
                writer.write_array_len(2)?;
                writer.write_boolean(false)?; // Err variant
                err.write(writer)
            }
        }
    }
}

// -------------------------------------------------------------------------------
// collections
// -------------------------------------------------------------------------------

impl<'a, T: FromMessagePack<'a>> FromMessagePack<'a> for alloc::vec::Vec<T> {
    #[inline(always)]
    fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        reader.read_array()
    }
}

impl<T: ToMessagePack> ToMessagePack for alloc::vec::Vec<T> {
    #[inline(always)]
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        writer.write_array_len(self.len())?;
        for item in self {
            item.write(writer)?;
        }
        Ok(())
    }
}

impl<'a, T: FromMessagePack<'a>> FromMessagePack<'a> for alloc::collections::VecDeque<T> {
    fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let len = reader.read_array_len()?;

        // don't use `with_capacity` to protect against OOM attacks
        let mut vec = alloc::collections::VecDeque::new();
        for _ in 0..len {
            vec.push_back(T::read(reader)?);
        }
        Ok(vec)
    }
}

impl<T: ToMessagePack> ToMessagePack for alloc::collections::VecDeque<T> {
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        writer.write_array_len(self.len())?;
        for item in self {
            item.write(writer)?;
        }
        Ok(())
    }
}

impl<'a, T: FromMessagePack<'a>> FromMessagePack<'a> for alloc::collections::LinkedList<T> {
    fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let len = reader.read_array_len()?;
        let mut list = alloc::collections::LinkedList::new();
        for _ in 0..len {
            list.push_back(T::read(reader)?);
        }
        Ok(list)
    }
}

impl<T: ToMessagePack> ToMessagePack for alloc::collections::LinkedList<T> {
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        writer.write_array_len(self.len())?;
        for item in self {
            item.write(writer)?;
        }
        Ok(())
    }
}

impl<'a, T: Ord + FromMessagePack<'a>> FromMessagePack<'a> for alloc::collections::BTreeSet<T> {
    fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let len = reader.read_array_len()?;
        let mut set = alloc::collections::BTreeSet::new();
        for _ in 0..len {
            set.insert(T::read(reader)?);
        }
        Ok(set)
    }
}

impl<T: ToMessagePack> ToMessagePack for alloc::collections::BTreeSet<T> {
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        writer.write_array_len(self.len())?;
        for item in self {
            item.write(writer)?;
        }
        Ok(())
    }
}

impl<'a, K: Ord + FromMessagePack<'a>, V: FromMessagePack<'a>> FromMessagePack<'a>
    for alloc::collections::BTreeMap<K, V>
{
    fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let len = reader.read_map_len()?;
        let mut map = alloc::collections::BTreeMap::new();
        for _ in 0..len {
            let key = K::read(reader)?;
            let value = V::read(reader)?;
            map.insert(key, value);
        }
        Ok(map)
    }
}

impl<K: ToMessagePack, V: ToMessagePack> ToMessagePack for alloc::collections::BTreeMap<K, V> {
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        writer.write_map_len(self.len())?;
        for (key, value) in self {
            key.write(writer)?;
            value.write(writer)?;
        }
        Ok(())
    }
}

impl<'a, T: FromMessagePack<'a> + Ord> FromMessagePack<'a> for alloc::collections::BinaryHeap<T> {
    fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let len = reader.read_array_len()?;

        // don't use `with_capacity` to protect against OOM attacks
        let mut heap = alloc::collections::BinaryHeap::new();

        for _ in 0..len {
            heap.push(T::read(reader)?);
        }
        Ok(heap)
    }
}

impl<T: ToMessagePack + Ord> ToMessagePack for alloc::collections::BinaryHeap<T> {
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        writer.write_array_len(self.len())?;
        for item in self {
            item.write(writer)?;
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<'a, T: Hash + Eq + FromMessagePack<'a>> FromMessagePack<'a> for std::collections::HashSet<T> {
    fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let len = reader.read_array_len()?;

        // don't use `with_capacity` to protect against OOM attacks
        let mut set = std::collections::HashSet::new();

        for _ in 0..len {
            set.insert(T::read(reader)?);
        }
        Ok(set)
    }
}

#[cfg(feature = "std")]
impl<T: ToMessagePack> ToMessagePack for std::collections::HashSet<T> {
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        writer.write_array_len(self.len())?;
        for item in self {
            item.write(writer)?;
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<'a, K: Hash + Eq + FromMessagePack<'a>, V: FromMessagePack<'a>> FromMessagePack<'a>
    for std::collections::HashMap<K, V>
{
    fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let len = reader.read_map_len()?;

        // don't use `with_capacity` to protect against OOM attacks
        let mut map = std::collections::HashMap::new();

        for _ in 0..len {
            let key = K::read(reader)?;
            let value = V::read(reader)?;
            map.insert(key, value);
        }
        Ok(map)
    }
}

#[cfg(feature = "std")]
impl<K: ToMessagePack, V: ToMessagePack> ToMessagePack for std::collections::HashMap<K, V> {
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        writer.write_map_len(self.len())?;
        for (key, value) in self {
            key.write(writer)?;
            value.write(writer)?;
        }
        Ok(())
    }
}

// -------------------------------------------------------------------------------
// smart pointer types
// -------------------------------------------------------------------------------

impl<'a, T: FromMessagePack<'a>> FromMessagePack<'a> for alloc::boxed::Box<T> {
    #[inline(always)]
    fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(alloc::boxed::Box::new(T::read(reader)?))
    }
}

impl<T: ToMessagePack> ToMessagePack for alloc::boxed::Box<T> {
    #[inline(always)]
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        self.as_ref().write(writer)
    }
}

#[cfg(feature = "std")]
impl<'a, T: FromMessagePack<'a>> FromMessagePack<'a> for std::sync::Arc<T> {
    fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(std::sync::Arc::new(T::read(reader)?))
    }
}

#[cfg(feature = "std")]
impl<T: ToMessagePack> ToMessagePack for std::sync::Arc<T> {
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        self.as_ref().write(writer)
    }
}

impl<'a, T: FromMessagePack<'a>> FromMessagePack<'a> for alloc::rc::Rc<T> {
    fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(alloc::rc::Rc::new(T::read(reader)?))
    }
}

impl<T: ToMessagePack> ToMessagePack for alloc::rc::Rc<T> {
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        self.as_ref().write(writer)
    }
}

// -------------------------------------------------------------------------------
// tuples
// -------------------------------------------------------------------------------

impl<'a> FromMessagePack<'a> for () {
    fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        reader.read_nil()
    }
}

impl ToMessagePack for () {
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        writer.write_nil()
    }
}

macro_rules! impl_tuple_message_packable {
    ($len:expr; $($t:ident : $idx:tt),+ $(,)?) => {
        impl<'a, $($t: FromMessagePack<'a>),+> FromMessagePack<'a> for ($($t,)+) {
            fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
            where
                Self: Sized,
            {
                reader.check_array_len($len)?;
                Ok(($($t::read(reader)?,)+))
            }
        }

        impl<$($t: ToMessagePack),+> ToMessagePack for ($($t,)+) {
            fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
                writer.write_array_len($len)?;
                $(self.$idx.write(writer)?;)+
                Ok(())
            }
        }
    };
}

impl_tuple_message_packable!(2; T0:0, T1:1);
impl_tuple_message_packable!(3; T0:0, T1:1, T2:2);
impl_tuple_message_packable!(4; T0:0, T1:1, T2:2, T3:3);
impl_tuple_message_packable!(5; T0:0, T1:1, T2:2, T3:3, T4:4);
impl_tuple_message_packable!(6; T0:0, T1:1, T2:2, T3:3, T4:4, T5:5);
impl_tuple_message_packable!(7; T0:0, T1:1, T2:2, T3:3, T4:4, T5:5, T6:6);
impl_tuple_message_packable!(8; T0:0, T1:1, T2:2, T3:3, T4:4, T5:5, T6:6, T7:7);
impl_tuple_message_packable!(9; T0:0, T1:1, T2:2, T3:3, T4:4, T5:5, T6:6, T7:7, T8:8);
impl_tuple_message_packable!(10; T0:0, T1:1, T2:2, T3:3, T4:4, T5:5, T6:6, T7:7, T8:8, T9:9);
impl_tuple_message_packable!(11; T0:0, T1:1, T2:2, T3:3, T4:4, T5:5, T6:6, T7:7, T8:8, T9:9, T10:10);
impl_tuple_message_packable!(12; T0:0, T1:1, T2:2, T3:3, T4:4, T5:5, T6:6, T7:7, T8:8, T9:9, T10:10, T11:11);

// -------------------------------------------------------------------------------
// chrono types
// -------------------------------------------------------------------------------

#[cfg(feature = "chrono")]
impl<'a> FromMessagePack<'a> for chrono::DateTime<chrono::Utc> {
    fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let (seconds, nanoseconds) = reader.read_timestamp()?;
        chrono::DateTime::<chrono::Utc>::from_timestamp(seconds, nanoseconds)
            .ok_or(crate::Error::InvalidTimestamp)
    }
}

#[cfg(feature = "chrono")]
impl ToMessagePack for chrono::DateTime<chrono::Utc> {
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        writer.write_timestamp(self.timestamp(), self.timestamp_subsec_nanos())
    }
}

#[cfg(feature = "chrono")]
impl<'a> FromMessagePack<'a> for chrono::NaiveDateTime {
    fn read<R: Read<'a>>(reader: &mut R) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let (seconds, nanoseconds) = reader.read_timestamp()?;
        chrono::DateTime::<chrono::Utc>::from_timestamp(seconds, nanoseconds)
            .map(|v| v.naive_utc())
            .ok_or(crate::Error::InvalidTimestamp)
    }
}

#[cfg(feature = "chrono")]
impl ToMessagePack for chrono::NaiveDateTime {
    fn write<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        let utc = self.and_utc();
        writer.write_timestamp(utc.timestamp(), utc.timestamp_subsec_nanos())
    }
}
