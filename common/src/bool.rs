use serde::Deserializer;
use serde::de::{Error, Unexpected};
use std::fmt;

struct BoolVisitor;
impl serde::de::Visitor<'_> for BoolVisitor {
    type Value = bool;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "bool value")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match v {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(E::invalid_value(Unexpected::Signed(v), &"0 or 1")),
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match v {
            "0" | "no" => Ok(false),
            "1" | "yes" => Ok(true),
            other => Err(E::invalid_value(Unexpected::Str(other), &"bool")),
        }
    }
}

pub fn deserialize_bool<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    deserializer.deserialize_any(BoolVisitor)
}
