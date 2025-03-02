use crate::EntityParseError;
use binrw::BinRead;
use cgmath::Vector3;
use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer};
use std::cmp::Ordering;
use std::fmt::Debug;
use std::ops::{Add, Mul, Sub};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, BinRead, Default)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector {
    pub fn iter(&self) -> impl Iterator<Item = f32> {
        [self.x, self.y, self.z].into_iter()
    }

    pub fn length_squared(&self) -> f32 {
        self.x.powf(2.0) + self.y.powf(2.0) + self.z.powf(2.0)
    }
}

impl Add<Vector> for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Self::Output {
        Vector {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub<Vector> for Vector {
    type Output = Vector;

    fn sub(self, rhs: Vector) -> Self::Output {
        Vector {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<f32> for Vector {
    type Output = Vector;

    fn mul(self, rhs: f32) -> Self::Output {
        Vector {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl PartialEq for Vector {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl PartialOrd for Vector {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.length_squared().partial_cmp(&other.length_squared())
    }
}

impl From<Vector> for [f32; 3] {
    fn from(vector: Vector) -> Self {
        [vector.x, vector.y, vector.z]
    }
}

impl From<[f32; 3]> for Vector {
    fn from(vector: [f32; 3]) -> Self {
        Vector {
            x: vector[0],
            y: vector[1],
            z: vector[2],
        }
    }
}

impl From<&Vector> for [f32; 3] {
    fn from(vector: &Vector) -> Self {
        [vector.x, vector.y, vector.z]
    }
}

impl FromStr for Vector {
    type Err = EntityParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut floats = s.split_whitespace().map(f32::from_str);
        let x = floats.next().ok_or(EntityParseError::ElementCount)??;
        let y = floats.next().ok_or(EntityParseError::ElementCount)??;
        let z = floats.next().ok_or(EntityParseError::ElementCount)??;
        if floats.next().is_some() {
            return Err(EntityParseError::ElementCount);
        }
        Ok(Vector { x, y, z })
    }
}

impl From<Vector> for Vector3<f32> {
    fn from(v: Vector) -> Self {
        Vector3::new(v.x, v.y, v.z)
    }
}

impl<'de> Deserialize<'de> for Vector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = <&str>::deserialize(deserializer)?;
        str.parse()
            .map_err(|_| D::Error::invalid_value(Unexpected::Other(str), &"a vector"))
    }
}
