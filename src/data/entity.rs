use crate::error::EntityParseError;
use crate::Vector;
use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;
use vbsp_derive::Entity;

#[derive(Clone)]
pub struct Entities {
    pub entities: String,
}

impl fmt::Debug for Entities {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        struct Entities<'a> {
            #[allow(dead_code)]
            entities: Vec<RawEntity<'a>>,
        }

        Entities {
            entities: self.iter().collect(),
        }
        .fmt(f)
    }
}

impl Entities {
    pub fn iter(&self) -> impl Iterator<Item = RawEntity<'_>> {
        struct Iter<'a> {
            buf: &'a str,
        }

        impl<'a> Iterator for Iter<'a> {
            type Item = RawEntity<'a>;

            fn next(&mut self) -> Option<Self::Item> {
                let start = self.buf.find('{')? + 1;
                let end = start + self.buf[start..].find('}')?;

                let out = &self.buf[start..end];

                self.buf = &self.buf[end + 1..];

                Some(RawEntity { buf: out })
            }
        }

        Iter {
            buf: &self.entities,
        }
    }
}

#[derive(Clone)]
pub struct RawEntity<'a> {
    buf: &'a str,
}

impl fmt::Debug for RawEntity<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::collections::HashMap;

        self.properties().collect::<HashMap<_, _>>().fmt(f)
    }
}

impl<'a> RawEntity<'a> {
    pub fn properties(&self) -> impl Iterator<Item = (&'a str, &'a str)> {
        struct Iter<'a> {
            buf: &'a str,
        }

        impl<'a> Iterator for Iter<'a> {
            type Item = (&'a str, &'a str);

            fn next(&mut self) -> Option<Self::Item> {
                let start = self.buf.find('"')? + 1;
                let end = start + self.buf[start..].find('"')?;

                let key = &self.buf[start..end];

                let rest = &self.buf[end + 1..];

                let start = rest.find('"')? + 1;
                let end = start + rest[start..].find('"')?;

                let value = &rest[start..end];

                self.buf = &rest[end + 1..];

                Some((key, value))
            }
        }

        Iter { buf: self.buf }
    }

    pub fn prop(&self, key: &'static str) -> Result<&'a str, EntityParseError> {
        self.properties()
            .find_map(|(prop_key, value)| (key == prop_key).then(|| value))
            .ok_or(EntityParseError::NoSuchProperty(key))
    }

    fn prop_parse<T: EntityProp<'a>>(&self, key: &'static str) -> Result<T, EntityParseError> {
        T::parse(self.prop(key)?)
    }

    pub fn parse(&self) -> Result<Entity<'a>, EntityParseError> {
        self.clone().try_into()
    }
}

trait EntityProp<'a>: Sized {
    fn parse(raw: &'a str) -> Result<Self, EntityParseError>;
}

trait FromStrProp: FromStr {}

impl FromStrProp for u8 {}
impl FromStrProp for f32 {}
impl FromStrProp for u32 {}
impl FromStrProp for Vector {}

impl<T: FromStrProp> EntityProp<'_> for T
where
    EntityParseError: From<<T as FromStr>::Err>,
{
    fn parse(raw: &'_ str) -> Result<Self, EntityParseError> {
        Ok(raw.parse()?)
    }
}

impl<T: FromStrProp, const N: usize> EntityProp<'_> for [T; N]
where
    EntityParseError: From<<T as FromStr>::Err>,
    [T; N]: Default,
{
    fn parse(raw: &'_ str) -> Result<Self, EntityParseError> {
        let mut values = raw.split(" ").map(T::from_str);
        let mut result = <[T; N]>::default();
        for i in 0..N {
            result[i] = values.next().ok_or(EntityParseError::ElementCount)??;
        }
        Ok(result)
    }
}

impl<'a> EntityProp<'a> for &'a str {
    fn parse(raw: &'a str) -> Result<Self, EntityParseError> {
        Ok(raw)
    }
}

impl EntityProp<'_> for bool {
    fn parse(raw: &'_ str) -> Result<Self, EntityParseError> {
        Ok(raw != "0")
    }
}

impl<'a, T: EntityProp<'a>> EntityProp<'a> for Option<T> {
    fn parse(raw: &'a str) -> Result<Self, EntityParseError> {
        Ok(Some(T::parse(raw)?))
    }
}

#[derive(Debug, Clone, Entity)]
pub enum Entity<'a> {
    #[entity(name = "point_spotlight")]
    SpotLight(SpotLight),
    #[entity(name = "light_spot")]
    LightSpot(LightSpot),
    #[entity(name = "prop_dynamic")]
    PropDynamic(PropDynamic<'a>),
    #[entity(name = "prop_physics_multiplayer")]
    PropPhysics(PropDynamic<'a>),
    #[entity(name = "env_sprite")]
    EnvSprite(EnvSprite<'a>),
    #[entity(name = "info_player_teamspawn")]
    Spawn(Spawn<'a>),
    #[entity(name = "func_door")]
    Door(Door<'a>),
    #[entity(name = "worldspawn")]
    WorldSpawn(WorldSpawn<'a>),
    #[entity(name = "info_observer_point")]
    ObserverPoint(ObserverPoint<'a>),
    #[entity(name = "func_brush")]
    Brush(BrushEntity<'a>),
    #[entity(name = "item_ammopack_small")]
    AmmoPackSmall(AmmoPack),
    #[entity(name = "item_ammopack_medium")]
    AmmoPackMedium(AmmoPack),
    #[entity(name = "item_ammopack_large")]
    HealthPackLarge(HealthPack),
    #[entity(name = "item_healthkit_small")]
    HealthPackSmall(HealthPack),
    #[entity(name = "item_healthkit_medium")]
    HealthPackMedium(HealthPack),
    #[entity(name = "item_healthkit_large")]
    AmmoPackLarge(AmmoPack),
    #[entity(default)]
    Unknown(RawEntity<'a>),
}

#[derive(Debug, Clone, Entity)]
pub struct SpotLight {
    pub origin: Vector,
    pub angles: [f32; 3],
    #[entity(name = "rendercolor")]
    pub color: [u8; 3],
    #[entity(name = "spotlightwidth")]
    pub cone: u8,
}

#[derive(Debug, Clone, Entity)]
pub struct LightSpot {
    pub origin: Vector,
    pub angles: [f32; 3],
    #[entity(name = "_light")]
    pub color: [u8; 3],
    #[entity(name = "_cone")]
    pub cone: u8,
}

#[derive(Debug, Clone, Entity)]
pub struct PropDynamic<'a> {
    pub angles: [f32; 3],
    #[entity(name = "disablereceiveshadows", default)]
    pub disable_receive_shadows: bool,
    #[entity(name = "disableshadows", default)]
    pub disable_shadows: bool,
    #[entity(name = "modelscale")]
    pub scale: f32,
    pub model: &'a str,
    pub origin: Vector,
    #[entity(name = "rendercolor")]
    pub color: [u8; 3],
    #[entity(name = "targetname", default)]
    pub name: Option<&'a str>,
    #[entity(name = "parentname", default)]
    pub parent: Option<&'a str>,
}

#[derive(Debug, Clone, Entity)]
pub struct EnvSprite<'a> {
    pub origin: Vector,
    pub scale: f32,
    pub model: &'a str,
    #[entity(name = "rendercolor")]
    pub color: [u8; 3],
}

#[derive(Debug, Clone, Entity)]
pub struct Spawn<'a> {
    pub origin: Vector,
    pub angles: [f32; 3],
    #[entity(name = "targetname", default)]
    pub target: Option<&'a str>,
    #[entity(name = "controlpoint", default)]
    pub control_point: Option<&'a str>,
    #[entity(name = "StartDisabled", default)]
    pub start_disabled: bool,
    #[entity(name = "TeamNum")]
    pub team: u8,
}

#[derive(Debug, Clone, Entity)]
pub struct Door<'a> {
    pub origin: Vector,
    #[entity(name = "targetname", default)]
    pub target: &'a str,
    pub speed: f32,
    #[entity(name = "forceclosed", default)]
    pub force_closed: bool,
    #[entity(name = "movedir")]
    pub move_direction: Vector,
    pub model: &'a str,
}

#[derive(Debug, Clone, Entity)]
pub struct AmmoPack {
    pub origin: Vector,
}

#[derive(Debug, Clone, Entity)]
pub struct HealthPack {
    pub origin: Vector,
}

#[derive(Debug, Clone, Entity)]
pub struct WorldSpawn<'a> {
    #[entity(name = "world_mins")]
    pub min: Vector,
    #[entity(name = "world_mins")]
    pub max: Vector,
    #[entity(name = "detailvbsp")]
    pub detail_vbsp: &'a str,
    #[entity(name = "detailmaterial")]
    pub detail_material: &'a str,
    #[entity(default)]
    pub comment: Option<&'a str>,
    #[entity(name = "skyname")]
    pub skybox: &'a str,
    #[entity(name = "mapversion")]
    pub version: u32,
}

#[derive(Debug, Clone, Entity)]
pub struct ObserverPoint<'a> {
    #[entity(name = "StartDisabled", default)]
    pub start_disabled: bool,
    pub angles: [f32; 3],
    pub origin: Vector,
    #[entity(name = "targetname", default)]
    pub target: Option<&'a str>,
    #[entity(name = "parentname", default)]
    pub parent: Option<&'a str>,
}

#[derive(Debug, Clone, Entity)]
pub struct BrushEntity<'a> {
    pub model: &'a str,
    pub origin: Vector,
    #[entity(name = "StartDisabled", default)]
    pub start_disabled: bool,
    #[entity(name = "rendercolor")]
    pub color: [f32; 3],
}
