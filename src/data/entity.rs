use crate::error::EntityParseError;
use crate::Vector;
use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;
use vdf_reader::VdfError;

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
pub struct EntitiesIter<'a> {
    buf: &'a str,
}

impl<'a> Iterator for EntitiesIter<'a> {
    type Item = RawEntity<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.buf.find('{')?;
        let end = start + self.buf[start..].find('}')?;

        let out = &self.buf[start..end + 1];

        self.buf = &self.buf[end + 1..];

        Some(RawEntity { buf: out })
    }
}

impl<'a> IntoIterator for &'a Entities {
    type Item = RawEntity<'a>;

    type IntoIter = EntitiesIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Entities {
    pub fn iter(&self) -> EntitiesIter {
        EntitiesIter {
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
    pub fn as_str(&self) -> &'a str {
        self.buf
    }

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
            .find_map(|(prop_key, value)| (key == prop_key).then_some(value))
            .ok_or(EntityParseError::NoSuchProperty(key))
    }

    pub fn prop_parse<T: EntityProp<'a>>(&self, key: &'static str) -> Result<T, EntityParseError> {
        T::parse(self.prop(key)?)
    }

    pub fn parse(&self) -> Result<Entity<'a>, EntityParseError> {
        match vdf_reader::from_str(self.buf) {
            Ok(entity) => Ok(entity),
            Err(VdfError::UnknownVariant(_)) => Ok(Entity::Unknown(self.clone())),
            // todo
            Err(_) => Err(EntityParseError::NoSuchProperty("unknown serde error")),
        }
    }
}

pub trait EntityProp<'a>: Sized {
    fn parse(raw: &'a str) -> Result<Self, EntityParseError>;
}

trait FromStrProp: FromStr {}

impl FromStrProp for u8 {}
impl FromStrProp for u16 {}
impl FromStrProp for f32 {}
impl FromStrProp for u32 {}
impl FromStrProp for i32 {}
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
        let mut values = raw.split(' ').map(T::from_str);
        let mut result = <[T; N]>::default();
        for item in result.iter_mut() {
            *item = values.next().ok_or(EntityParseError::ElementCount)??;
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

#[derive(Debug, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = <&str>::deserialize(deserializer)?;
        let colors = <[u8; 3]>::parse(str)
            .map_err(|_| D::Error::invalid_value(Unexpected::Other(str), &"a list of 3 numbers"))?;
        Ok(Color {
            r: colors[0],
            g: colors[1],
            b: colors[2],
        })
    }
}

#[derive(Debug, Clone)]
pub struct LightColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub intensity: u16,
}

impl FromStr for LightColor {
    type Err = EntityParseError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let mut values = str.split(' ');
        Ok(LightColor {
            r: values
                .next()
                .ok_or(EntityParseError::ElementCount)?
                .parse()
                .map_err(EntityParseError::Int)?,
            g: values
                .next()
                .ok_or(EntityParseError::ElementCount)?
                .parse()
                .map_err(EntityParseError::Int)?,
            b: values
                .next()
                .ok_or(EntityParseError::ElementCount)?
                .parse()
                .map_err(EntityParseError::Int)?,
            intensity: values
                .next()
                .ok_or(EntityParseError::ElementCount)?
                .parse()
                .map_err(EntityParseError::Int)?,
        })
    }
}

impl<'de> Deserialize<'de> for LightColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = <&str>::deserialize(deserializer)?;
        str.parse()
            .map_err(|_| D::Error::invalid_value(Unexpected::Str(str), &"a list of 4 integers"))
    }
}

pub use typed::*;

mod typed {
    use crate::{Angles, Color, LightColor, RawEntity, Vector};
    use serde::{Deserialize, Deserializer};

    #[derive(Debug, Clone, Deserialize)]
    #[non_exhaustive]
    #[serde(tag = "classname")]
    pub enum Entity<'a> {
        #[serde(rename = "point_spotlight")]
        SpotLight(SpotLight),
        #[serde(rename = "light")]
        Light(Light),
        #[serde(rename = "light_spot")]
        LightSpot(LightSpot),
        #[serde(rename = "prop_dynamic")]
        #[serde(borrow)]
        PropDynamic(PropDynamic<'a>),
        #[serde(rename = "prop_dynamic_override")]
        #[serde(borrow)]
        PropDynamicOverride(PropDynamicOverride<'a>),
        #[serde(rename = "prop_physics_multiplayer")]
        #[serde(borrow)]
        PropPhysics(PropDynamic<'a>),
        #[serde(rename = "env_sprite")]
        #[serde(borrow)]
        EnvSprite(EnvSprite<'a>),
        #[serde(rename = "info_player_teamspawn")]
        #[serde(borrow)]
        Spawn(Spawn<'a>),
        #[serde(rename = "func_regenerate")]
        #[serde(borrow)]
        Regenerate(Regenerate<'a>),
        #[serde(rename = "func_respawnroom")]
        RespawnRoom(RespawnRoom<'a>),
        #[serde(rename = "func_door")]
        Door(Door<'a>),
        #[serde(rename = "worldspawn")]
        WorldSpawn(WorldSpawn<'a>),
        #[serde(rename = "info_observer_point")]
        #[serde(borrow)]
        ObserverPoint(ObserverPoint<'a>),
        #[serde(rename = "func_brush")]
        #[serde(borrow)]
        Brush(BrushEntity<'a>),
        #[serde(rename = "func_illusionary")]
        #[serde(borrow)]
        BrushIllusionary(BrushEntity<'a>),
        #[serde(rename = "func_wall")]
        #[serde(borrow)]
        BrushWall(BrushEntity<'a>),
        #[serde(rename = "func_wall_toggle")]
        #[serde(borrow)]
        BrushWallToggle(BrushEntity<'a>),
        #[serde(rename = "item_ammopack_small")]
        AmmoPackSmall(AmmoPack),
        #[serde(rename = "item_ammopack_medium")]
        AmmoPackMedium(AmmoPack),
        #[serde(rename = "item_ammopack_full")]
        HealthPackFull(HealthPack),
        #[serde(rename = "item_healthkit_small")]
        HealthPackSmall(HealthPack),
        #[serde(rename = "item_healthkit_medium")]
        HealthPackMedium(HealthPack),
        #[serde(rename = "item_healthkit_full")]
        AmmoPackFull(AmmoPack),
        #[serde(rename = "env_lightglow")]
        LightGlow(LightGlow),
        #[serde(rename = "trigger_multiple")]
        #[serde(borrow)]
        TriggerMultiple(TriggerMultiple<'a>),
        #[serde(rename = "logic_relay")]
        LogicRelay(LogicRelay<'a>),
        #[serde(rename = "filter_activator_tfteam")]
        #[serde(borrow)]
        FilterActivatorTeam(FilterActivatorTeam<'a>),
        #[serde(rename = "logic_auto")]
        LogicAuto(LogicAuto<'a>),
        #[serde(rename = "func_dustmotes")]
        DustMotes(DustMotes<'a>),
        #[serde(rename = "sky_camera")]
        SkyCamera(SkyCamera),
        #[serde(rename = "path_track")]
        PathTrack(PathTrack<'a>),
        #[serde(rename = "env_soundscape_proxy")]
        #[serde(borrow)]
        SoundScapeProxy(SoundScapeProxy<'a>),
        #[serde(rename = "func_respawnroomvisualizer")]
        #[serde(borrow)]
        RespawnVisualizer(RespawnVisualizer<'a>),
        #[serde(rename = "info_particle_system")]
        #[serde(borrow)]
        ParticleSystem(ParticleSystem<'a>),
        #[serde(rename = "team_control_point")]
        #[serde(borrow)]
        TeamControlPoint(TeamControlPoint<'a>),
        #[serde(rename = "func_areaportal")]
        AreaPortal(AreaPortal),
        #[serde(rename = "game_text")]
        #[serde(borrow)]
        GameText(GameText<'a>),
        #[serde(rename = "keyframe_rope")]
        #[serde(borrow)]
        RopeKeyFrame(RopeKeyFrame<'a>),
        #[serde(rename = "move_rope")]
        RopeMove(RopeMove<'a>),
        #[serde(rename = "tf_gamerules")]
        #[serde(borrow)]
        GameRules(GameRules<'a>),
        #[serde(rename = "tf_logic_koth")]
        KothLogic(KothLogic),
        #[serde(rename = "ambient_generic")]
        #[serde(borrow)]
        AmbientGeneric(AmbientGeneric<'a>),
        #[serde(rename = "logic_case")]
        #[serde(borrow)]
        LogicCase(LogicCase<'a>),
        #[serde(rename = "func_occluder")]
        #[serde(borrow)]
        Occluder(Occluder<'a>),
        #[serde(skip)]
        Unknown(RawEntity<'a>),
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct Light {
        pub origin: Vector,
        #[serde(rename = "_light")]
        pub light: LightColor,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct SpotLight {
        pub origin: Vector,
        pub angles: Angles,
        #[serde(rename = "rendercolor")]
        pub color: Color,
        #[serde(rename = "spotlightwidth")]
        pub cone: u8,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct LightSpot {
        pub origin: Vector,
        pub angles: Angles,
        #[serde(rename = "_light")]
        pub light: LightColor,
        #[serde(rename = "_cone")]
        pub cone: u8,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct PropDynamic<'a> {
        pub angles: Angles,
        #[serde(rename = "disablereceiveshadows", default)]
        #[serde(deserialize_with = "bool_from_int")]
        pub disable_receive_shadows: bool,
        #[serde(rename = "disableshadows", default)]
        #[serde(deserialize_with = "bool_from_int")]
        pub disable_shadows: bool,
        #[serde(rename = "modelscale")]
        pub scale: f32,
        pub model: &'a str,
        pub origin: Vector,
        #[serde(rename = "rendercolor")]
        pub color: Color,
        #[serde(rename = "targetname", default)]
        pub name: Option<&'a str>,
        #[serde(rename = "parentname", default)]
        pub parent: Option<&'a str>,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct PropDynamicOverride<'a> {
        pub angles: Angles,
        #[serde(rename = "disablereceiveshadows", default)]
        #[serde(deserialize_with = "bool_from_int")]
        pub disable_receive_shadows: bool,
        #[serde(rename = "disableshadows", default)]
        #[serde(deserialize_with = "bool_from_int")]
        pub disable_shadows: bool,
        #[serde(rename = "modelscale")]
        pub scale: f32,
        pub model: &'a str,
        pub origin: Vector,
        #[serde(rename = "rendercolor")]
        pub color: Color,
        #[serde(rename = "targetname", default)]
        pub name: Option<&'a str>,
        #[serde(rename = "parentname", default)]
        pub parent: Option<&'a str>,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct EnvSprite<'a> {
        pub origin: Vector,
        pub scale: f32,
        pub model: &'a str,
        #[serde(rename = "rendercolor")]
        pub color: Color,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct Spawn<'a> {
        pub origin: Vector,
        pub angles: Angles,
        #[serde(rename = "targetname", default)]
        pub target: Option<&'a str>,
        #[serde(rename = "controlpoint", default)]
        pub control_point: Option<&'a str>,
        #[serde(rename = "startdisabled", default)]
        #[serde(deserialize_with = "bool_from_int")]
        pub start_disabled: bool,
        #[serde(rename = "teamnum")]
        pub team: u8,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct RespawnRoom<'a> {
        #[serde(rename = "targetname", default)]
        pub target: Option<&'a str>,
        pub model: &'a str,
        #[serde(rename = "startdisabled", default)]
        #[serde(deserialize_with = "bool_from_int")]
        pub start_disabled: bool,
        #[serde(rename = "teamnum")]
        pub team: u8,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct Regenerate<'a> {
        #[serde(rename = "associatedmodel")]
        pub associated_model: &'a str,
        pub model: &'a str,
        #[serde(rename = "teamnum")]
        pub team: u8,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct Door<'a> {
        pub origin: Vector,
        #[serde(rename = "targetname", default)]
        pub target: &'a str,
        pub speed: f32,
        #[serde(rename = "forceclosed", default)]
        #[serde(deserialize_with = "bool_from_int")]
        pub force_closed: bool,
        #[serde(rename = "movedir")]
        pub move_direction: Vector,
        pub model: &'a str,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct AmmoPack {
        pub origin: Vector,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct HealthPack {
        pub origin: Vector,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct WorldSpawn<'a> {
        #[serde(rename = "world_mins")]
        pub min: Vector,
        #[serde(rename = "world_maxs")]
        pub max: Vector,
        #[serde(rename = "detailvbsp")]
        pub detail_vbsp: &'a str,
        #[serde(rename = "detailmaterial")]
        pub detail_material: &'a str,
        #[serde(default)]
        pub comment: Option<&'a str>,
        #[serde(rename = "skyname")]
        pub skybox: &'a str,
        #[serde(rename = "mapversion")]
        pub version: u32,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct ObserverPoint<'a> {
        #[serde(rename = "startdisabled", default)]
        #[serde(deserialize_with = "bool_from_int")]
        pub start_disabled: bool,
        pub angles: Angles,
        pub origin: Vector,
        #[serde(rename = "targetname", default)]
        pub target: Option<&'a str>,
        #[serde(rename = "parentname", default)]
        pub parent: Option<&'a str>,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct BrushEntity<'a> {
        pub model: &'a str,
        pub origin: Vector,
        #[serde(rename = "startdisabled", default)]
        #[serde(deserialize_with = "bool_from_int")]
        pub start_disabled: bool,
        #[serde(rename = "rendercolor")]
        pub color: Color,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct LightGlow {
        pub origin: Vector,
        #[serde(rename = "verticalglowsize")]
        pub vertical_size: u32,
        #[serde(rename = "horizontalhlowsize")]
        pub horizontal_size: u32,
        #[serde(rename = "startdisabled", default)]
        #[serde(deserialize_with = "bool_from_int")]
        pub start_disabled: bool,
        #[serde(rename = "rendercolor")]
        pub color: Color,
        #[serde(rename = "mindist")]
        pub min_distance: u32,
        #[serde(rename = "maxdist")]
        pub max_distance: u32,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct TriggerMultiple<'a> {
        pub model: &'a str,
        pub origin: Vector,
        #[serde(rename = "onstarttouch", default)]
        pub start_touch: Option<&'a str>,
        #[serde(rename = "onstarttouchall", default)]
        pub start_touch_all: Option<&'a str>,
        #[serde(rename = "onendtouch", default)]
        pub end_touch: Option<&'a str>,
        #[serde(rename = "onendtouchall", default)]
        pub end_touch_all: Option<&'a str>,
        #[serde(rename = "onnottouching", default)]
        pub not_touching: Option<&'a str>,
        #[serde(rename = "targetname", default)]
        pub target_name: Option<&'a str>,
        #[serde(rename = "filtername", default)]
        pub filter: Option<&'a str>,
        pub wait: Option<u32>,
        #[serde(rename = "startdisabled", default)]
        #[serde(deserialize_with = "bool_from_int")]
        pub start_disabled: bool,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct FilterActivatorTeam<'a> {
        pub origin: Vector,
        #[serde(rename = "targetname", default)]
        pub target_name: Option<&'a str>,
        #[serde(rename = "negated", default)]
        pub negated: Option<&'a str>,
        #[serde(rename = "teamnum", default)]
        pub team: u8,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct LogicRelay<'a> {
        pub origin: Vector,
        #[serde(rename = "targetname", default)]
        pub target_name: Option<&'a str>,
        #[serde(rename = "ontrigger", default)]
        pub on_trigger: Option<&'a str>,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct LogicAuto<'a> {
        pub origin: Vector,
        #[serde(rename = "onmapspawn", default)]
        pub on_map_spawn: Option<&'a str>,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct DustMotes<'a> {
        #[serde(default)]
        pub origin: Vector,
        pub model: &'a str,
        #[serde(rename = "startdisabled", default)]
        #[serde(deserialize_with = "bool_from_int")]
        pub start_disabled: bool,
        #[serde(rename = "color")]
        pub color: Color,
        #[serde(rename = "spawnrate")]
        pub spawn_rate: u32,
        #[serde(rename = "sizemin")]
        pub size_min: u32,
        #[serde(rename = "sizemax")]
        pub size_max: u32,
        #[serde(rename = "alpha")]
        pub alpha: u8,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct SkyCamera {
        pub origin: Vector,
        #[serde(rename = "fogenable")]
        #[serde(deserialize_with = "bool_from_int")]
        pub fog: bool,
        #[serde(deserialize_with = "bool_from_int")]
        pub use_angles: bool,
        #[serde(rename = "fogstart")]
        pub fog_start: f32,
        #[serde(rename = "fogend")]
        pub fog_end: f32,
        pub angles: Angles,
        #[serde(rename = "fogdir")]
        pub direction: Vector,
        pub scale: u32,
        #[serde(rename = "fogcolor")]
        pub color: Color,
        #[serde(rename = "fogcolor2", default)]
        pub color2: Option<Color>,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct PathTrack<'a> {
        pub origin: Vector,
        #[serde(default)]
        pub target: Option<&'a str>,
        #[serde(rename = "targetname", default)]
        pub target_name: Option<&'a str>,
        #[serde(rename = "orientationtype", default)]
        pub orientation_type: u8,
        pub angles: Angles,
        pub radius: f32,
        pub speed: f32,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct SoundScapeProxy<'a> {
        pub origin: Vector,
        pub radius: f32,
        #[serde(rename = "mainsoundscapename")]
        pub main_name: &'a str,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct RespawnVisualizer<'a> {
        pub origin: Vector,
        #[serde(rename = "respawnroomname")]
        pub room_name: &'a str,
        #[serde(rename = "rendercolor")]
        pub color: Color,
        #[serde(deserialize_with = "bool_from_int")]
        pub solid_to_enemies: bool,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct ParticleSystem<'a> {
        pub origin: Vector,
        pub angles: Angles,
        #[serde(rename = "targetname")]
        pub target_name: Option<&'a str>,
        pub effect_name: &'a str,
        #[serde(default)]
        #[serde(deserialize_with = "bool_from_int")]
        pub start_active: bool,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct TeamControlPoint<'a> {
        pub origin: Vector,
        pub angles: Angles,
        #[serde(rename = "targetname")]
        pub target_name: &'a str,
        pub point_warn_sound: &'a str,
        pub team_model_0: &'a str,
        pub team_model_2: &'a str,
        pub team_model_3: &'a str,
        pub team_icon_0: &'a str,
        pub team_icon_2: &'a str,
        pub team_icon_3: &'a str,
        #[serde(default)]
        pub point_default_owner: u8,
        #[serde(rename = "startdisabled", default)]
        #[serde(deserialize_with = "bool_from_int")]
        pub start_disabled: bool,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct AreaPortal {
        #[serde(rename = "portalversion")]
        pub version: u8,
        #[serde(rename = "portalnumber")]
        pub number: u8,
        #[serde(rename = "startopen")]
        #[serde(deserialize_with = "bool_from_int")]
        pub start_open: bool,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct GameText<'a> {
        pub origin: Vector,
        #[serde(rename = "targetname", default)]
        pub target_name: Option<&'a str>,
        pub message: &'a str,
        pub fadeout: f32,
        pub color: Color,
        #[serde(rename = "fadein")]
        pub fade_in: f32,
        #[serde(rename = "fadeout")]
        pub fade_out: f32,
        pub x: f32,
        pub y: f32,
        #[serde(rename = "holdtime")]
        pub hold_time: f32,
        #[serde(rename = "fxtime")]
        pub fx_time: f32,
        pub channel: u8,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct RopeKeyFrame<'a> {
        pub origin: Vector,
        #[serde(rename = "targetname", default)]
        pub target_name: Option<&'a str>,
        #[serde(rename = "ropematerial")]
        pub material: &'a str,
        #[serde(rename = "dangling")]
        #[serde(deserialize_with = "bool_from_int")]
        pub dangling: bool,
        #[serde(rename = "barbed")]
        #[serde(deserialize_with = "bool_from_int")]
        pub barbed: bool,
        #[serde(rename = "breakable")]
        #[serde(deserialize_with = "bool_from_int")]
        pub breakable: bool,
        #[serde(rename = "texturescale")]
        pub texture_scale: f32,
        #[serde(rename = "collide")]
        #[serde(deserialize_with = "bool_from_int")]
        pub collide: bool,
        #[serde(rename = "width")]
        pub width: f32,
        #[serde(rename = "slack")]
        pub slack: f32,
        #[serde(rename = "movespeed")]
        pub move_speed: f32,
        #[serde(rename = "subdiv")]
        pub sub_div: u8,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct RopeMove<'a> {
        pub origin: Vector,
        #[serde(rename = "ropematerial")]
        pub material: &'a str,
        #[serde(rename = "texturescale")]
        pub texture_scale: f32,
        #[serde(rename = "slack")]
        pub slack: f32,
        #[serde(rename = "width")]
        pub width: f32,
        #[serde(rename = "dangling")]
        #[serde(deserialize_with = "bool_from_int")]
        pub dangling: bool,
        #[serde(rename = "barbed")]
        #[serde(deserialize_with = "bool_from_int")]
        pub barbed: bool,
        #[serde(rename = "breakable")]
        #[serde(deserialize_with = "bool_from_int")]
        pub breakable: bool,
        #[serde(rename = "positioninterpolator")]
        pub interpolator: u8,
        #[serde(rename = "movespeed")]
        pub move_speed: f32,
        #[serde(rename = "type")]
        pub ty: u8,
        #[serde(rename = "nextkey")]
        pub next_key: &'a str,
        #[serde(rename = "subdiv")]
        pub sub_div: u8,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct GameRules<'a> {
        pub origin: Vector,
        #[serde(rename = "targetname", default)]
        pub target_name: Option<&'a str>,
        #[serde(default)]
        #[serde(deserialize_with = "bool_from_int")]
        pub ctf_overtime: bool,
        #[serde(default)]
        pub hud_type: u32,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct KothLogic {
        pub origin: Vector,
        pub unlock_point: u32,
        pub timer_length: u32,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub struct AmbientGeneric<'a> {
        pub origin: Vector,
        #[serde(rename = "volstart", default)]
        pub volume_start: f32,
        #[serde(rename = "spin_up", default)]
        pub spin_up: f32,
        #[serde(rename = "spin_down", default)]
        pub spin_down: f32,
        #[serde(rename = "spawn_flags", default)]
        pub spawn_flags: u32,
        #[serde(rename = "radius", default)]
        pub radius: f32,
        #[serde(rename = "preset", default)]
        pub preset: u32,
        #[serde(rename = "pitch_start", default)]
        pub pitch_start: f32,
        #[serde(rename = "pitch", default)]
        pub pitch: f32,
        pub message: &'a str,
        #[serde(rename = "lfo_type", default)]
        pub lfo_type: u32,
        #[serde(rename = "lfo_rate", default)]
        pub lfo_rate: u32,
        #[serde(rename = "lfo_mod_vol", default)]
        pub lfo_mod_vol: f32,
        #[serde(rename = "lfomodpitch", default)]
        pub lfo_mod_pitch: f32,
        #[serde(rename = "health", default)]
        pub health: u8,
        #[serde(rename = "fadeoutsec", default)]
        pub fade_out_secs: f32,
        #[serde(rename = "fadeinsec", default)]
        pub fade_in_secs: f32,
        #[serde(rename = "cspinup", default)]
        pub c_spin_up: f32,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub struct LogicCase<'a> {
        pub origin: Vector,
        #[serde(rename = "targetname")]
        pub target_name: Option<&'a str>,
        #[serde(rename = "oncase01")]
        pub oncase_01: Option<&'a str>,
        #[serde(rename = "oncase02")]
        pub oncase_02: Option<&'a str>,
        #[serde(rename = "oncase03")]
        pub oncase_03: Option<&'a str>,
        #[serde(rename = "oncase04")]
        pub oncase_04: Option<&'a str>,
        #[serde(rename = "oncase05")]
        pub oncase_05: Option<&'a str>,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub struct Occluder<'a> {
        #[serde(rename = "occludernumber", default)]
        pub occluder_number: u32,
        pub model: &'a str,
        #[serde(rename = "startactive", deserialize_with = "bool_from_int")]
        pub start_active: bool,
    }

    fn bool_from_int<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
        let int = u8::deserialize(deserializer)?;
        Ok(int > 0)
    }
}
