use crate::Vector;
use cgmath::Quaternion;

#[derive(Debug, Clone)]
pub struct PropPlacement<'a> {
    pub model: &'a str,
    pub rotation: Quaternion<f32>,
    pub scale: f32,
    pub origin: Vector,
    pub skin: i32,
}

/// Abstraction for various ways props are placed in a bsp
pub trait AsPropPlacement<'a> {
    fn as_prop_placement(&self) -> PropPlacement<'a>;
}
