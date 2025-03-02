use crate::{Angles, Color, LightColor, Negated, Vector};
use std::num::{ParseFloatError, ParseIntError};
use std::str::FromStr;
use thiserror::Error;

pub trait EntityProp<'a>: Sized {
    fn parse(raw: &'a str) -> Result<Self, EntityParseError>;
}

/// Properties that can be parsed with their FromStr implementation
pub trait FromStrProp: FromStr {}

impl FromStrProp for u8 {}
impl FromStrProp for u16 {}
impl FromStrProp for f32 {}
impl FromStrProp for u32 {}
impl FromStrProp for i32 {}
impl FromStrProp for Color {}
impl FromStrProp for Angles {}
impl FromStrProp for Vector {}
impl FromStrProp for LightColor {}
impl FromStrProp for Negated {}

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
        Ok(raw != "0" && raw != "no")
    }
}

impl<'a, T: EntityProp<'a>> EntityProp<'a> for Option<T> {
    fn parse(raw: &'a str) -> Result<Self, EntityParseError> {
        Ok(Some(T::parse(raw)?))
    }
}

#[derive(Debug, Error)]
pub enum EntityParseError {
    #[error("wrong number of elements")]
    ElementCount,
    #[error(transparent)]
    Float(#[from] ParseFloatError),
    #[error(transparent)]
    Int(#[from] ParseIntError),
}
