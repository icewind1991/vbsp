use crate::EntityParseError;
use binrw::BinRead;
use cgmath::{Deg, Quaternion, Rotation3};
use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, BinRead, Default)]
pub struct Angles {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

impl<'de> Deserialize<'de> for Angles {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = <&str>::deserialize(deserializer)?;
        str.parse()
            .map_err(|_| D::Error::invalid_value(Unexpected::Other(str), &"a list of angles"))
    }
}

impl FromStr for Angles {
    type Err = EntityParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut floats = s.split_whitespace().map(f32::from_str);
        let pitch = floats.next().ok_or(EntityParseError::ElementCount)??;
        let yaw = floats.next().ok_or(EntityParseError::ElementCount)??;
        let roll = floats.next().ok_or(EntityParseError::ElementCount)??;
        Ok(Angles { pitch, yaw, roll })
    }
}

impl Angles {
    pub fn as_quaternion(&self) -> Quaternion<f32> {
        // angles are applied in roll, pitch, yaw order
        Quaternion::from_angle_y(Deg(self.yaw))
            * Quaternion::from_angle_x(Deg(self.pitch))
            * Quaternion::from_angle_z(Deg(self.roll))
    }
}
