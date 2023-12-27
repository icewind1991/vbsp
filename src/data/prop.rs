use crate::{Handle, PropDynamic, PropDynamicOverride, StaticPropLump, Vector};
use cgmath::{Deg, Quaternion, Rotation3};

#[derive(Debug, Clone)]
pub struct PropPlacement<'a> {
    pub model: &'a str,
    pub rotation: Quaternion<f32>,
    pub scale: f32,
    pub origin: Vector,
    pub skin: i32,
}

impl<'a> Handle<'a, StaticPropLump> {
    pub fn as_prop_placement(&self) -> PropPlacement<'a> {
        PropPlacement {
            model: self.model(),
            rotation: self.rotation(),
            scale: 1.0,
            origin: self.origin,
            skin: self.skin,
        }
    }
}

fn rotation(angles: [f32; 3]) -> Quaternion<f32> {
    Quaternion::from_angle_y(Deg(angles[1]))
        * Quaternion::from_angle_x(Deg(angles[0]))
        * Quaternion::from_angle_z(Deg(angles[2]))
}

impl<'a> PropDynamic<'a> {
    pub fn as_prop_placement(&self) -> PropPlacement<'a> {
        PropPlacement {
            model: self.model,
            rotation: rotation(self.angles),
            scale: self.scale,
            origin: self.origin,
            skin: 0,
        }
    }
}

impl<'a> PropDynamicOverride<'a> {
    pub fn as_prop_placement(&self) -> PropPlacement<'a> {
        PropPlacement {
            model: self.model,
            rotation: rotation(self.angles),
            scale: self.scale,
            origin: self.origin,
            skin: 0,
        }
    }
}
