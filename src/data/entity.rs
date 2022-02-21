use crate::error::EntityParseError;
use crate::Vector;
use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;

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

    fn prop_parse<T: FromStr>(&self, key: &'static str) -> Result<T, EntityParseError>
    where
        EntityParseError: From<<T as FromStr>::Err>,
    {
        Ok(self.prop(key)?.parse()?)
    }

    fn prop_parse_space_seperated<T: FromStr + Default, const N: usize>(
        &self,
        key: &'static str,
    ) -> Result<[T; N], EntityParseError>
    where
        EntityParseError: From<<T as FromStr>::Err>,
        [T; N]: Default,
    {
        let prop = self.prop(key)?;
        let mut values = prop.split(" ").map(T::from_str);
        let mut result = <[T; N]>::default();
        for i in 0..N {
            result[i] = values.next().ok_or(EntityParseError::ElementCount)??;
        }
        Ok(result)
    }

    pub fn parse(&self) -> Result<Entity<'a>, EntityParseError> {
        let class = self.prop("classname")?;
        match class {
            "prop_dynamic" => Ok(Entity::PropDynamic(PropDynamic {
                angles: self.prop_parse_space_seperated("angles")?,
                disable_receive_shadows: self
                    .prop_parse::<u8>("disablereceiveshadows")
                    .unwrap_or_default()
                    > 0,
                disable_shadows: self
                    .prop_parse::<u8>("disablereceiveshadows")
                    .unwrap_or_default()
                    > 0,
                scale: self.prop_parse("modelscale")?,
                model: self.prop("model")?,
                origin: self.prop_parse("angles")?,
                color: self.prop_parse_space_seperated("rendercolor")?,
                name: self.prop("targetname").ok(),
                parent: self.prop("parentname").ok(),
            })),
            "prop_physics_multiplayer" => Ok(Entity::PropPhysics(PropDynamic {
                angles: self.prop_parse_space_seperated("angles")?,
                disable_receive_shadows: self
                    .prop_parse::<u8>("disablereceiveshadows")
                    .unwrap_or_default()
                    > 0,
                disable_shadows: self
                    .prop_parse::<u8>("disablereceiveshadows")
                    .unwrap_or_default()
                    > 0,
                scale: self.prop_parse("modelscale")?,
                model: self.prop("model")?,
                origin: self.prop_parse("angles")?,
                color: self.prop_parse_space_seperated("rendercolor")?,
                name: self.prop("targetname").ok(),
                parent: self.prop("parentname").ok(),
            })),
            "light_spot" => Ok(Entity::SpotLight(SpotLight {
                origin: self.prop_parse("origin")?,
                angles: self.prop_parse_space_seperated("angles")?,
                color: self.prop_parse_space_seperated("_light")?,
                cone: self.prop_parse("_cone")?,
            })),
            "point_spotlight" => Ok(Entity::SpotLight(SpotLight {
                origin: self.prop_parse("origin")?,
                angles: self.prop_parse_space_seperated("angles")?,
                color: self.prop_parse_space_seperated("rendercolor")?,
                cone: self.prop_parse("spotlightwidth")?,
            })),
            "env_sprite" => Ok(Entity::EnvSprite(EnvSprite {
                origin: self.prop_parse("origin")?,
                scale: self.prop_parse("scale")?,
                model: self.prop("model")?,
                color: self.prop_parse_space_seperated("rendercolor")?,
            })),
            "info_player_teamspawn" => Ok(Entity::Spawn(Spawn {
                origin: self.prop_parse("origin")?,
                angles: self.prop_parse_space_seperated("angles")?,
                target: self.prop("targetname").ok(),
                control_point: self.prop("controlpoint").ok(),
                start_disabled: self.prop_parse::<u8>("StartDisabled").unwrap_or_default() > 0,
                team: self.prop_parse("TeamNum")?,
            })),
            "func_door" => Ok(Entity::Door(Door {
                origin: self.prop_parse("origin")?,
                target: self.prop("targetname")?,
                speed: self.prop_parse("speed")?,
                force_closed: self.prop_parse::<u8>("forceclosed").unwrap_or_default() > 0,
                move_direction: self.prop_parse("movedir")?,
                model: self.prop("model")?,
            })),
            "item_ammopack_small" => Ok(Entity::AmmoPack(AmmoPack {
                origin: self.prop_parse("origin")?,
                ty: PackType::Small,
            })),
            "item_ammopack_medium" => Ok(Entity::AmmoPack(AmmoPack {
                origin: self.prop_parse("origin")?,
                ty: PackType::Medium,
            })),
            "item_ammopack_large" => Ok(Entity::AmmoPack(AmmoPack {
                origin: self.prop_parse("origin")?,
                ty: PackType::Large,
            })),
            "item_healthkit_small" => Ok(Entity::HealthPack(HealthPack {
                origin: self.prop_parse("origin")?,
                ty: PackType::Small,
            })),
            "item_healthkit_medium" => Ok(Entity::HealthPack(HealthPack {
                origin: self.prop_parse("origin")?,
                ty: PackType::Medium,
            })),
            "item_healthkit_large" => Ok(Entity::HealthPack(HealthPack {
                origin: self.prop_parse("origin")?,
                ty: PackType::Large,
            })),
            "worldspawn" => Ok(Entity::WorldSpawn(WorldSpawn {
                min: self.prop_parse("world_mins")?,
                max: self.prop_parse("world_maxs")?,
                detail_vbsp: self.prop("detailvbsp")?,
                detail_material: self.prop("detailmaterial")?,
                comment: self.prop("comment").ok(),
                skybox: self.prop("skyname")?,
                version: self.prop_parse("mapversion")?,
            })),
            "info_observer_point" => Ok(Entity::ObserverPoint(ObserverPoint {
                start_disabled: self.prop_parse::<u8>("StartDisabled").unwrap_or_default() > 0,
                angles: self.prop_parse_space_seperated("angles")?,
                origin: self.prop_parse("origin")?,
                target: self.prop("targetname").ok(),
                parent: self.prop("parentname").ok(),
            })),
            "func_brush" => Ok(Entity::Brush(BrushEntity {
                model: self.prop("model")?,
                start_disabled: self.prop_parse::<u8>("StartDisabled").unwrap_or_default() > 0,
                origin: self.prop_parse("origin")?,
                color: self.prop_parse_space_seperated("rendercolor")?,
            })),
            _ => Ok(Entity::Unknown(self.clone())),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Entity<'a> {
    SpotLight(SpotLight),
    PropDynamic(PropDynamic<'a>),
    PropPhysics(PropDynamic<'a>),
    EnvSprite(EnvSprite<'a>),
    Spawn(Spawn<'a>),
    Door(Door<'a>),
    AmmoPack(AmmoPack),
    HealthPack(HealthPack),
    WorldSpawn(WorldSpawn<'a>),
    ObserverPoint(ObserverPoint<'a>),
    Brush(BrushEntity<'a>),
    Unknown(RawEntity<'a>),
}

#[derive(Debug, Clone)]
pub struct SpotLight {
    pub origin: Vector,
    pub angles: [f32; 3],
    pub color: [u8; 3],
    pub cone: u8,
}

#[derive(Debug, Clone)]
pub struct PropDynamic<'a> {
    pub angles: [f32; 3],
    pub disable_receive_shadows: bool,
    pub disable_shadows: bool,
    pub scale: f32,
    pub model: &'a str,
    pub origin: Vector,
    pub color: [u8; 3],
    pub name: Option<&'a str>,
    pub parent: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct EnvSprite<'a> {
    pub origin: Vector,
    pub scale: f32,
    pub model: &'a str,
    pub color: [u8; 3],
}

#[derive(Debug, Clone)]
pub struct Spawn<'a> {
    pub origin: Vector,
    pub angles: [f32; 3],
    pub target: Option<&'a str>,
    pub control_point: Option<&'a str>,
    pub start_disabled: bool,
    pub team: u8,
}

#[derive(Debug, Clone)]
pub struct Door<'a> {
    pub origin: Vector,
    pub target: &'a str,
    pub speed: f32,
    pub force_closed: bool,
    pub move_direction: Vector,
    pub model: &'a str,
}

#[derive(Debug, Clone)]
pub struct AmmoPack {
    pub origin: Vector,
    pub ty: PackType,
}

#[derive(Debug, Clone)]
pub struct HealthPack {
    pub origin: Vector,
    pub ty: PackType,
}

#[derive(Debug, Clone)]
pub enum PackType {
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone)]
pub struct WorldSpawn<'a> {
    pub min: Vector,
    pub max: Vector,
    pub detail_vbsp: &'a str,
    pub detail_material: &'a str,
    pub comment: Option<&'a str>,
    pub skybox: &'a str,
    pub version: u32,
}

#[derive(Debug, Clone)]
pub struct ObserverPoint<'a> {
    pub start_disabled: bool,
    pub angles: [f32; 3],
    pub origin: Vector,
    pub target: Option<&'a str>,
    pub parent: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct BrushEntity<'a> {
    pub model: &'a str,
    pub origin: Vector,
    pub start_disabled: bool,
    pub color: [f32; 3],
}
