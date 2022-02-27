use vbsp::RawEntity;
use vbsp_derive::Entity;

#[derive(Entity)]
pub struct SpotLight {
    pub angles: [f32; 3],
    #[entity(name = "render_color")]
    pub color: [u8; 3],
    pub cone: u8,
    #[entity(default)]
    pub optional: bool,
}

#[derive(Entity)]
pub struct Lifetime<'a> {
    pub model: &'a str,
}

#[derive(Entity)]
pub enum Entity<'a> {
    // #[entity(name = "spot")]
    // SpotLight(SpotLight),
    #[entity(name = "bar")]
    Foo(Lifetime<'a>),
    #[entity(default)]
    Unknown(RawEntity<'a>),
}
