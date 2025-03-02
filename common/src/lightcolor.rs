use crate::EntityParseError;
use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

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
