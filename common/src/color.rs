use crate::{EntityParseError, EntityProp};
use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

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
