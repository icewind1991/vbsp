use crate::{Handle, StaticPropLump, Vector};
use cgmath::Quaternion;

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
