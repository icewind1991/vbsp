use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Negated {
    Yes,
    No,
    MatchingCriteria,
}
pub struct NegatedParseErr;
impl FromStr for Negated {
    type Err = NegatedParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(Negated::Yes),
            "0" => Ok(Negated::No),
            "allow entities that match criteria" => Ok(Negated::MatchingCriteria),
            _ => Err(NegatedParseErr),
        }
    }
}

struct NegatedVisitor;
impl serde::de::Visitor<'_> for NegatedVisitor {
    type Value = Negated;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Negated value")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match v {
            0 => Ok(Negated::No),
            1 => Ok(Negated::Yes),
            _ => Err(E::invalid_value(Unexpected::Signed(v), &"0 or 1")),
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        v.parse()
            .map_err(|_| E::invalid_value(Unexpected::Str(v), &"Negated"))
    }
}

impl<'de> Deserialize<'de> for Negated {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(NegatedVisitor)
    }
}
