use crate::error::EntityParseError;
use crate::{Angles, Vector};
use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;
use vdf_reader::VdfError;

#[cfg(feature = "basic")]
pub mod basic;

#[derive(Clone)]
pub struct Entities {
    pub entities: String,
}

impl fmt::Debug for Entities {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        struct Entities<'a> {
            #[allow(dead_code)]
            entities: Vec<RawEntity<'a>>,
        }

        Entities {
            entities: self.iter().collect(),
        }
        .fmt(f)
    }
}
pub struct EntitiesIter<'a> {
    buf: &'a str,
}

impl<'a> Iterator for EntitiesIter<'a> {
    type Item = RawEntity<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.buf.find('{')?;
        let end = start + self.buf[start..].find('}')?;

        let out = &self.buf[start..end + 1];

        self.buf = &self.buf[end + 1..];

        Some(RawEntity { buf: out })
    }
}

impl<'a> IntoIterator for &'a Entities {
    type Item = RawEntity<'a>;

    type IntoIter = EntitiesIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Entities {
    pub fn iter(&self) -> EntitiesIter {
        EntitiesIter {
            buf: &self.entities,
        }
    }
}

#[derive(Clone)]
pub struct RawEntity<'a> {
    buf: &'a str,
}

impl fmt::Debug for RawEntity<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::collections::HashMap;

        self.properties().collect::<HashMap<_, _>>().fmt(f)
    }
}

impl<'a> RawEntity<'a> {
    pub fn as_str(&self) -> &'a str {
        self.buf
    }

    pub fn properties(&self) -> impl Iterator<Item = (&'a str, &'a str)> {
        struct Iter<'a> {
            buf: &'a str,
        }

        impl<'a> Iterator for Iter<'a> {
            type Item = (&'a str, &'a str);

            fn next(&mut self) -> Option<Self::Item> {
                let start = self.buf.find('"')? + 1;
                let end = start + self.buf[start..].find('"')?;

                let key = &self.buf[start..end];

                let rest = &self.buf[end + 1..];

                let start = rest.find('"')? + 1;
                let end = start + rest[start..].find('"')?;

                let value = &rest[start..end];

                self.buf = &rest[end + 1..];

                Some((key, value))
            }
        }

        Iter { buf: self.buf }
    }

    pub fn prop(&self, key: &'static str) -> Option<&'a str> {
        self.properties()
            .find_map(|(prop_key, value)| (key == prop_key).then_some(value))
    }

    pub fn prop_parse<T: EntityProp<'a>>(
        &self,
        key: &'static str,
    ) -> Option<Result<T, EntityParseError>> {
        self.prop(key).map(T::parse)
    }

    pub fn parse<E: Deserialize<'a>>(&self) -> Result<E, VdfError> {
        vdf_reader::from_str(self.buf)
    }
}

pub trait EntityProp<'a>: Sized {
    fn parse(raw: &'a str) -> Result<Self, EntityParseError>;
}

trait FromStrProp: FromStr {}

impl FromStrProp for u8 {}
impl FromStrProp for u16 {}
impl FromStrProp for f32 {}
impl FromStrProp for u32 {}
impl FromStrProp for i32 {}
impl FromStrProp for Color {}
impl FromStrProp for Angles {}
impl FromStrProp for Vector {}
impl FromStrProp for LightColor {}

impl<T: FromStrProp> EntityProp<'_> for T
where
    EntityParseError: From<<T as FromStr>::Err>,
{
    fn parse(raw: &'_ str) -> Result<Self, EntityParseError> {
        Ok(raw.parse()?)
    }
}

impl<T: FromStrProp, const N: usize> EntityProp<'_> for [T; N]
where
    EntityParseError: From<<T as FromStr>::Err>,
    [T; N]: Default,
{
    fn parse(raw: &'_ str) -> Result<Self, EntityParseError> {
        let mut values = raw.split_whitespace().map(T::from_str);
        let mut result = <[T; N]>::default();
        for item in result.iter_mut() {
            *item = values.next().ok_or(EntityParseError::ElementCount)??;
        }
        Ok(result)
    }
}

impl<'a> EntityProp<'a> for &'a str {
    fn parse(raw: &'a str) -> Result<Self, EntityParseError> {
        Ok(raw)
    }
}

impl EntityProp<'_> for bool {
    fn parse(raw: &'_ str) -> Result<Self, EntityParseError> {
        Ok(raw != "0")
    }
}

impl<'a, T: EntityProp<'a>> EntityProp<'a> for Option<T> {
    fn parse(raw: &'a str) -> Result<Self, EntityParseError> {
        Ok(Some(T::parse(raw)?))
    }
}

#[derive(Debug, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl FromStr for Color {
    type Err = EntityParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut floats = s.split_whitespace().map(u8::from_str);
        let r = floats.next().ok_or(EntityParseError::ElementCount)??;
        let g = floats.next().ok_or(EntityParseError::ElementCount)??;
        let b = floats.next().ok_or(EntityParseError::ElementCount)??;
        if floats.next().is_some() {
            return Err(EntityParseError::ElementCount);
        }
        Ok(Self { r, g, b })
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = <&str>::deserialize(deserializer)?;
        let [r, g, b] = <[u8; 3]>::parse(str)
            .map_err(|_| D::Error::invalid_value(Unexpected::Other(str), &"a list of 3 numbers"))?;
        Ok(Color { r, g, b })
    }
}

#[derive(Debug, Clone)]
pub struct LightColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub intensity: u16,
}

impl FromStr for LightColor {
    type Err = EntityParseError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let mut values = str.split_whitespace();
        let r = values
            .next()
            .ok_or(EntityParseError::ElementCount)?
            .parse()
            .map_err(EntityParseError::Int)?;
        let g = values
            .next()
            .ok_or(EntityParseError::ElementCount)?
            .parse()
            .map_err(EntityParseError::Int)?;
        let b = values
            .next()
            .ok_or(EntityParseError::ElementCount)?
            .parse()
            .map_err(EntityParseError::Int)?;
        let intensity = values
            .next()
            .ok_or(EntityParseError::ElementCount)?
            .parse()
            .map_err(EntityParseError::Int)?;
        if values.next().is_some() {
            return Err(EntityParseError::ElementCount);
        }
        Ok(LightColor { r, g, b, intensity })
    }
}

impl<'de> Deserialize<'de> for LightColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = <&str>::deserialize(deserializer)?;
        str.parse()
            .map_err(|_| D::Error::invalid_value(Unexpected::Str(str), &"a list of 4 integers"))
    }
}

#[allow(dead_code)]
pub(crate) fn bool_from_int<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    let int = u8::deserialize(deserializer)?;
    Ok(int != 0)
}
