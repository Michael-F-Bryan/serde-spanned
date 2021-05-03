use core::{
    fmt::{self, Formatter},
    marker::PhantomData,
};
use serde::{
    de::{Error, MapAccess},
    Deserialize, Deserializer, Serialize, Serializer,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Spanned<T> {
    value: T,
    start: usize,
    end: usize,
}

impl<T> Spanned<T> {
    pub const fn new(value: T, start: usize, end: usize) -> Self {
        Spanned { value, start, end }
    }

    pub const fn start(&self) -> usize { self.start }

    pub const fn end(&self) -> usize { self.end }

    pub const fn span(&self) -> (usize, usize) { (self.start(), self.end()) }

    pub const fn len(&self) -> usize { self.end() - self.start() }

    pub const fn is_empty(&self) -> bool { self.len() == 0 }

    pub const fn value(&self) -> &T { &self.value }

    pub fn value_mut(&mut self) -> &mut T { &mut self.value }

    pub fn into_value(self) -> T { self.value }
}

impl<T: Serialize> Serialize for Spanned<T> {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(ser)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Spanned<T> {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        de.deserialize_struct(crate::NAME, &crate::FIELDS, Visitor(PhantomData))
    }
}

struct Visitor<T>(PhantomData<T>);

impl<'de, T> serde::de::Visitor<'de> for Visitor<T>
where
    T: Deserialize<'de>,
{
    type Value = Spanned<T>;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "A spanned {}", core::any::type_name::<T>())
    }

    fn visit_map<A>(self, mut visitor: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        if visitor.next_key()? != Some(crate::START) {
            return Err(Error::custom("spanned start key not found"));
        }

        let start: usize = visitor.next_value()?;

        if visitor.next_key()? != Some(crate::END) {
            return Err(Error::custom("spanned end key not found"));
        }

        let end: usize = visitor.next_value()?;

        if visitor.next_key()? != Some(crate::VALUE) {
            return Err(Error::custom("spanned value key not found"));
        }

        let value: T = visitor.next_value()?;

        Ok(Spanned { start, end, value })
    }
}
