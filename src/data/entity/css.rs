use crate::bool_from_int;
use crate::{Angles, Color, LightColor, Negated, Vector};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
#[serde(tag = "classname")]
pub enum Entity<'a> {
    #[serde(rename = "env_beam")]
    #[serde(borrow)]
    EnvBeam(EnvBeam<'a>),
    #[serde(rename = "env_detail_controller")]
    EnvDetailController(EnvDetailController),
    #[serde(rename = "env_embers")]
    #[serde(borrow)]
    EnvEmbers(EnvEmbers<'a>),
    #[serde(rename = "env_entity_maker")]
    #[serde(borrow)]
    EnvEntityMaker(EnvEntityMaker<'a>),
    #[serde(rename = "env_fade")]
    #[serde(borrow)]
    EnvFade(EnvFade<'a>),
    #[serde(rename = "env_fire")]
    EnvFire(EnvFire),
    #[serde(rename = "env_fire_trail")]
    #[serde(borrow)]
    EnvFireTrail(EnvFireTrail<'a>),
    #[serde(rename = "env_fog_controller")]
    EnvFogController(EnvFogController),
    #[serde(rename = "env_hudhint")]
    #[serde(borrow)]
    EnvHudhint(EnvHudhint<'a>),
    #[serde(rename = "env_laser")]
    #[serde(borrow)]
    EnvLaser(EnvLaser<'a>),
    #[serde(rename = "env_lightglow")]
    EnvLightglow(EnvLightglow),
    #[serde(rename = "env_shake")]
    #[serde(borrow)]
    EnvShake(EnvShake<'a>),
    #[serde(rename = "env_shooter")]
    #[serde(borrow)]
    EnvShooter(EnvShooter<'a>),
    #[serde(rename = "env_smokestack")]
    #[serde(borrow)]
    EnvSmokestack(EnvSmokestack<'a>),
    #[serde(rename = "env_soundscape")]
    #[serde(borrow)]
    EnvSoundscape(EnvSoundscape<'a>),
    #[serde(rename = "env_soundscape_triggerable")]
    #[serde(borrow)]
    EnvSoundscapeTriggerable(EnvSoundscapeTriggerable<'a>),
    #[serde(rename = "env_spark")]
    #[serde(borrow)]
    EnvSpark(EnvSpark<'a>),
    #[serde(rename = "env_sprite")]
    #[serde(borrow)]
    EnvSprite(EnvSprite<'a>),
    #[serde(rename = "env_spritetrail")]
    #[serde(borrow)]
    EnvSpritetrail(EnvSpritetrail<'a>),
    #[serde(rename = "env_steam")]
    #[serde(borrow)]
    EnvSteam(EnvSteam<'a>),
    #[serde(rename = "env_sun")]
    #[serde(borrow)]
    EnvSun(EnvSun<'a>),
    #[serde(rename = "env_tonemap_controller")]
    EnvTonemapController(EnvTonemapController),
    #[serde(rename = "env_wind")]
    EnvWind(EnvWind),
    #[serde(rename = "filter_activator_class")]
    #[serde(borrow)]
    FilterActivatorClass(FilterActivatorClass<'a>),
    #[serde(rename = "filter_activator_name")]
    #[serde(borrow)]
    FilterActivatorName(FilterActivatorName<'a>),
    #[serde(rename = "filter_damage_type")]
    #[serde(borrow)]
    FilterDamageType(FilterDamageType<'a>),
    #[serde(rename = "filter_multi")]
    #[serde(borrow)]
    FilterMulti(FilterMulti<'a>),
    #[serde(rename = "func_areaportalwindow")]
    #[serde(borrow)]
    FuncAreaportalwindow(FuncAreaportalwindow<'a>),
    #[serde(rename = "func_breakable")]
    #[serde(borrow)]
    FuncBreakable(FuncBreakable<'a>),
    #[serde(rename = "func_button")]
    #[serde(borrow)]
    FuncButton(FuncButton<'a>),
    #[serde(rename = "func_buyzone")]
    #[serde(borrow)]
    FuncBuyzone(FuncBuyzone<'a>),
    #[serde(rename = "func_conveyor")]
    #[serde(borrow)]
    FuncConveyor(FuncConveyor<'a>),
    #[serde(rename = "func_door_rotating")]
    #[serde(borrow)]
    FuncDoorRotating(FuncDoorRotating<'a>),
    #[serde(rename = "func_dustcloud")]
    #[serde(borrow)]
    FuncDustcloud(FuncDustcloud<'a>),
    #[serde(rename = "func_movelinear")]
    #[serde(borrow)]
    FuncMovelinear(FuncMovelinear<'a>),
    #[serde(rename = "func_physbox")]
    #[serde(borrow)]
    FuncPhysbox(FuncPhysbox<'a>),
    #[serde(rename = "func_physbox_multiplayer")]
    #[serde(borrow)]
    FuncPhysboxMultiplayer(FuncPhysboxMultiplayer<'a>),
    #[serde(rename = "func_precipitation")]
    #[serde(borrow)]
    FuncPrecipitation(FuncPrecipitation<'a>),
    #[serde(rename = "func_rot_button")]
    #[serde(borrow)]
    FuncRotButton(FuncRotButton<'a>),
    #[serde(rename = "func_rotating")]
    #[serde(borrow)]
    FuncRotating(FuncRotating<'a>),
    #[serde(rename = "func_smokevolume")]
    #[serde(borrow)]
    FuncSmokevolume(FuncSmokevolume<'a>),
    #[serde(rename = "func_tracktrain")]
    #[serde(borrow)]
    FuncTracktrain(FuncTracktrain<'a>),
    #[serde(rename = "func_wall")]
    #[serde(borrow)]
    FuncWall(FuncWall<'a>),
    #[serde(rename = "func_wall_toggle")]
    #[serde(borrow)]
    FuncWallToggle(FuncWallToggle<'a>),
    #[serde(rename = "func_water_analog")]
    #[serde(borrow)]
    FuncWaterAnalog(FuncWaterAnalog<'a>),
    #[serde(rename = "game_player_equip")]
    GamePlayerEquip(GamePlayerEquip),
    #[serde(rename = "game_ui")]
    #[serde(borrow)]
    GameUi(GameUi<'a>),
    #[serde(rename = "game_weapon_manager")]
    #[serde(borrow)]
    GameWeaponManager(GameWeaponManager<'a>),
    #[serde(rename = "info_ladder")]
    InfoLadder(InfoLadder),
    #[serde(rename = "info_player_start")]
    InfoPlayerStart(InfoPlayerStart),
    #[serde(rename = "info_player_counterterrorist")]
    CounterTerroristSpawn(CounterTerroristSpawn),
    #[serde(rename = "info_player_terrorist")]
    InfoPlayerTerrorist(InfoPlayerTerrorist),
    #[serde(rename = "info_target")]
    #[serde(borrow)]
    InfoTarget(InfoTarget<'a>),
    #[serde(rename = "info_teleport_destination")]
    #[serde(borrow)]
    InfoTeleportDestination(InfoTeleportDestination<'a>),
    #[serde(rename = "infodecal")]
    #[serde(borrow)]
    Infodecal(Infodecal<'a>),
    #[serde(rename = "keyframe_rope")]
    #[serde(borrow)]
    KeyframeRope(KeyframeRope<'a>),
    #[serde(rename = "light")]
    Light(Light),
    #[serde(rename = "light_environment")]
    #[serde(borrow)]
    LightEnvironment(LightEnvironment<'a>),
    #[serde(rename = "logic_auto")]
    #[serde(borrow)]
    LogicAuto(LogicAuto<'a>),
    #[serde(rename = "logic_relay")]
    #[serde(borrow)]
    LogicRelay(LogicRelay<'a>),
    #[serde(rename = "logic_timer")]
    #[serde(borrow)]
    LogicTimer(LogicTimer<'a>),
    #[serde(rename = "math_counter")]
    #[serde(borrow)]
    MathCounter(MathCounter<'a>),
    #[serde(rename = "move_rope")]
    #[serde(borrow)]
    MoveRope(MoveRope<'a>),
    #[serde(rename = "path_track")]
    #[serde(borrow)]
    PathTrack(PathTrack<'a>),
    #[serde(rename = "phys_ballsocket")]
    #[serde(borrow)]
    PhysBallsocket(PhysBallsocket<'a>),
    #[serde(rename = "player_speedmod")]
    #[serde(borrow)]
    PlayerSpeedmod(PlayerSpeedmod<'a>),
    #[serde(rename = "player_weaponstrip")]
    #[serde(borrow)]
    PlayerWeaponstrip(PlayerWeaponstrip<'a>),
    #[serde(rename = "point_clientcommand")]
    #[serde(borrow)]
    PointClientcommand(PointClientcommand<'a>),
    #[serde(rename = "point_servercommand")]
    #[serde(borrow)]
    PointServercommand(PointServercommand<'a>),
    #[serde(rename = "point_spotlight")]
    #[serde(borrow)]
    PointSpotlight(PointSpotlight<'a>),
    #[serde(rename = "point_template")]
    #[serde(borrow)]
    PointTemplate(PointTemplate<'a>),
    #[serde(rename = "point_viewcontrol")]
    #[serde(borrow)]
    PointViewcontrol(PointViewcontrol<'a>),
    #[serde(rename = "prop_dynamic")]
    #[serde(borrow)]
    PropDynamic(PropDynamic<'a>),
    #[serde(rename = "prop_physics")]
    #[serde(borrow)]
    PropPhysics(PropPhysics<'a>),
    #[serde(rename = "prop_physics_multiplayer")]
    #[serde(borrow)]
    PropPhysicsMultiplayer(PropPhysicsMultiplayer<'a>),
    #[serde(rename = "prop_physics_override")]
    #[serde(borrow)]
    PropPhysicsOverride(PropPhysicsOverride<'a>),
    #[serde(rename = "prop_ragdoll")]
    #[serde(borrow)]
    PropRagdoll(PropRagdoll<'a>),
    #[serde(rename = "shadow_control")]
    ShadowControl(ShadowControl),
    #[serde(rename = "sky_camera")]
    SkyCamera(SkyCamera),
    #[serde(rename = "trigger_gravity")]
    #[serde(borrow)]
    TriggerGravity(TriggerGravity<'a>),
    #[serde(rename = "trigger_hurt")]
    #[serde(borrow)]
    TriggerHurt(TriggerHurt<'a>),
    #[serde(rename = "trigger_look")]
    #[serde(borrow)]
    TriggerLook(TriggerLook<'a>),
    #[serde(rename = "trigger_multiple")]
    #[serde(borrow)]
    TriggerMultiple(TriggerMultiple<'a>),
    #[serde(rename = "trigger_once")]
    #[serde(borrow)]
    TriggerOnce(TriggerOnce<'a>),
    #[serde(rename = "trigger_push")]
    #[serde(borrow)]
    TriggerPush(TriggerPush<'a>),
    #[serde(rename = "trigger_soundscape")]
    #[serde(borrow)]
    TriggerSoundscape(TriggerSoundscape<'a>),
    #[serde(rename = "trigger_teleport")]
    #[serde(borrow)]
    TriggerTeleport(TriggerTeleport<'a>),
    #[serde(rename = "water_lod_control")]
    WaterLodControl(WaterLodControl),
    #[serde(rename = "weapon_ak47")]
    #[serde(borrow)]
    WeaponAk47(WeaponAk47<'a>),
    #[serde(rename = "weapon_awp")]
    #[serde(borrow)]
    WeaponAwp(WeaponAwp<'a>),
    #[serde(rename = "weapon_deagle")]
    WeaponDeagle(WeaponDeagle),
    #[serde(rename = "weapon_elite")]
    #[serde(borrow)]
    WeaponElite(WeaponElite<'a>),
    #[serde(rename = "weapon_famas")]
    WeaponFamas(WeaponFamas),
    #[serde(rename = "weapon_fiveseven")]
    #[serde(borrow)]
    WeaponFiveseven(WeaponFiveseven<'a>),
    #[serde(rename = "weapon_flashbang")]
    #[serde(borrow)]
    WeaponFlashbang(WeaponFlashbang<'a>),
    #[serde(rename = "weapon_g3sg1")]
    WeaponG3Sg1(WeaponG3Sg1),
    #[serde(rename = "weapon_glock")]
    #[serde(borrow)]
    WeaponGlock(WeaponGlock<'a>),
    #[serde(rename = "weapon_hegrenade")]
    #[serde(borrow)]
    WeaponHegrenade(WeaponHegrenade<'a>),
    #[serde(rename = "weapon_knife")]
    #[serde(borrow)]
    WeaponKnife(WeaponKnife<'a>),
    #[serde(rename = "weapon_m249")]
    #[serde(borrow)]
    WeaponM249(WeaponM249<'a>),
    #[serde(rename = "weapon_m3")]
    #[serde(borrow)]
    WeaponM3(WeaponM3<'a>),
    #[serde(rename = "weapon_m4a1")]
    WeaponM4A1(WeaponM4A1),
    #[serde(rename = "weapon_mac10")]
    WeaponMac10(WeaponMac10),
    #[serde(rename = "weapon_p90")]
    #[serde(borrow)]
    WeaponP90(WeaponP90<'a>),
    #[serde(rename = "weapon_scout")]
    #[serde(borrow)]
    WeaponScout(WeaponScout<'a>),
    #[serde(rename = "weapon_sg550")]
    WeaponSg550(WeaponSg550),
    #[serde(rename = "weapon_smokegrenade")]
    #[serde(borrow)]
    WeaponSmokegrenade(WeaponSmokegrenade<'a>),
    #[serde(rename = "weapon_tmp")]
    WeaponTmp(WeaponTmp),
    #[serde(rename = "weapon_ump45")]
    WeaponUmp45(WeaponUmp45),
    #[serde(rename = "weapon_usp")]
    #[serde(borrow)]
    WeaponUsp(WeaponUsp<'a>),
    #[serde(rename = "weapon_xm1014")]
    WeaponXm1014(WeaponXm1014),
    #[serde(rename = "worldspawn")]
    #[serde(borrow)]
    Worldspawn(Worldspawn<'a>),
}

#[derive(Debug, Clone, Deserialize)]
pub struct CounterTerroristSpawn {
    pub angles: Angles,
    pub origin: Vector,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvBeam<'a> {
    pub boltwidth: f32,
    pub decalname: &'a str,
    pub hdrcolorscale: f32,
    pub life: f32,
    pub lightningend: &'a str,
    pub lightningstart: &'a str,
    pub noiseamplitude: f32,
    pub origin: Vector,
    pub radius: f32,
    pub renderamt: u8,
    pub rendercolor: Color,
    pub spawnflags: u32,
    pub striketime: f32,
    pub targetname: &'a str,
    pub texture: &'a str,
    pub texturescroll: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvDetailController {
    pub angles: Vector,
    pub fademaxdist: f32,
    pub fademindist: f32,
    pub origin: Vector,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvEmbers<'a> {
    pub angles: Vector,
    pub density: f32,
    pub lifetime: f32,
    pub model: &'a str,
    pub rendercolor: Color,
    pub spawnflags: u32,
    pub speed: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvEntityMaker<'a> {
    pub angles: Vector,
    pub entitytemplate: &'a str,
    pub origin: Vector,
    pub postspawndirection: Vector,
    pub postspawndirectionvariance: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub postspawninheritangles: bool,
    pub postspawnspeed: f32,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvFade<'a> {
    pub duration: f32,
    pub holdtime: f32,
    pub origin: Vector,
    pub renderamt: u8,
    pub rendercolor: Color,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvFire {
    pub damagescale: f32,
    pub fireattack: u8,
    pub firesize: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub firetype: bool,
    pub health: u8,
    pub ignitionpoint: f32,
    pub origin: Vector,
    pub spawnflags: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub startdisabled: bool,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvFireTrail<'a> {
    pub origin: Vector,
    pub parentname: &'a str,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvFogController {
    pub angles: Vector,
    pub farz: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub fogblend: bool,
    pub fogcolor: Color,
    pub fogcolor2: Color,
    pub fogdir: Vector,
    #[serde(deserialize_with = "bool_from_int")]
    pub fogenable: bool,
    pub fogend: f32,
    pub foglerptime: f32,
    pub fogmaxdensity: f32,
    pub fogstart: f32,
    pub maxdxlevel: u8,
    pub mindxlevel: u8,
    pub origin: Vector,
    pub spawnflags: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub use_angles: bool,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvHudhint<'a> {
    pub message: &'a str,
    pub origin: Vector,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvLaser<'a> {
    pub damage: i32,
    pub dissolvetype: &'a str,
    pub lasertarget: &'a str,
    pub origin: Vector,
    pub renderamt: u8,
    pub rendercolor: Color,
    pub spawnflags: u32,
    pub texture: &'a str,
    pub texturescroll: f32,
    pub width: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvLightglow {
    pub angles: Vector,
    pub glowproxysize: f32,
    pub hdrcolorscale: f32,
    pub horizontalglowsize: f32,
    pub maxdist: f32,
    pub mindist: f32,
    pub origin: Vector,
    pub outermaxdist: f32,
    pub rendercolor: Color,
    pub spawnflags: u32,
    pub verticalglowsize: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvShake<'a> {
    pub amplitude: f32,
    pub duration: f32,
    pub frequency: f32,
    pub origin: Vector,
    pub radius: f32,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvShooter<'a> {
    pub angles: Vector,
    pub delay: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub disablereceiveshadows: bool,
    pub gibangles: Vector,
    pub gibgravityscale: f32,
    pub m_flgiblife: f32,
    pub m_flvariance: f32,
    pub m_flvelocity: f32,
    pub m_igibs: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub massoverride: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub nogibshadows: bool,
    pub origin: Vector,
    pub parentname: &'a str,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    pub shootmodel: &'a str,
    pub shootsounds: i8,
    #[serde(deserialize_with = "bool_from_int")]
    pub simulation: bool,
    pub skin: u16,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvSmokestack<'a> {
    pub angles: Vector,
    pub basespread: f32,
    pub endsize: f32,
    pub initialstate: u8,
    pub jetlength: f32,
    pub origin: Vector,
    pub rate: f32,
    pub renderamt: u8,
    pub rendercolor: Color,
    pub roll: f32,
    pub smokematerial: &'a str,
    pub speed: f32,
    pub spreadspeed: f32,
    pub startsize: f32,
    pub twist: f32,
    pub windangle: f32,
    pub windspeed: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvSoundscape<'a> {
    pub origin: Vector,
    pub radius: f32,
    pub soundscape: &'a str,
    #[serde(deserialize_with = "bool_from_int")]
    pub startdisabled: bool,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvSoundscapeTriggerable<'a> {
    pub origin: Vector,
    pub radius: f32,
    pub soundscape: &'a str,
    #[serde(deserialize_with = "bool_from_int")]
    pub startdisabled: bool,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvSpark<'a> {
    pub angles: Vector,
    pub magnitude: f32,
    pub maxdelay: f32,
    pub origin: Vector,
    pub spawnflags: u32,
    pub targetname: &'a str,
    pub traillength: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvSprite<'a> {
    #[serde(deserialize_with = "bool_from_int")]
    pub disablereceiveshadows: bool,
    pub framerate: f32,
    pub glowproxysize: f32,
    pub hdrcolorscale: f32,
    pub maxdxlevel: u8,
    pub mindxlevel: u8,
    pub model: &'a str,
    pub origin: Vector,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    pub spawnflags: u32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvSpritetrail<'a> {
    pub endwidth: f32,
    pub lifetime: f32,
    pub origin: Vector,
    pub parentname: &'a str,
    pub renderamt: u8,
    pub rendercolor: Color,
    pub rendermode: u32,
    pub spritename: &'a str,
    pub startwidth: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvSteam<'a> {
    pub angles: Vector,
    pub endsize: f32,
    pub jetlength: f32,
    pub origin: Vector,
    pub rate: f32,
    pub renderamt: u8,
    pub rendercolor: Color,
    pub rollspeed: f32,
    pub spawnflags: u32,
    pub speed: f32,
    pub spreadspeed: f32,
    pub startsize: f32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvSun<'a> {
    pub angles: Vector,
    pub hdrcolorscale: f32,
    pub material: &'a str,
    pub origin: Vector,
    pub overlaycolor: Color,
    pub overlaymaterial: &'a str,
    pub overlaysize: i32,
    pub rendercolor: Color,
    pub size: f32,
    pub target: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvTonemapController {
    pub origin: Vector,
}
#[derive(Debug, Clone, Deserialize)]
pub struct EnvWind {
    pub angles: Vector,
    pub gustdirchange: f32,
    pub gustduration: f32,
    pub maxgust: f32,
    pub maxgustdelay: f32,
    pub maxwind: f32,
    pub mingust: f32,
    pub mingustdelay: f32,
    pub minwind: f32,
    pub origin: Vector,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FilterActivatorClass<'a> {
    pub filterclass: &'a str,
    pub negated: Negated,
    pub origin: Vector,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FilterActivatorName<'a> {
    pub filtername: &'a str,
    pub negated: Negated,
    pub origin: Vector,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FilterDamageType<'a> {
    pub damagetype: u32,
    pub negated: Negated,
    pub origin: Vector,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FilterMulti<'a> {
    pub filter01: &'a str,
    pub filter02: &'a str,
    #[serde(deserialize_with = "bool_from_int")]
    pub filtertype: bool,
    pub negated: Negated,
    pub origin: Vector,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FuncAreaportalwindow<'a> {
    pub fadedist: f32,
    pub fadestartdist: f32,
    pub portalnumber: u16,
    pub portalversion: u8,
    pub target: &'a str,
    pub translucencylimit: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FuncBreakable<'a> {
    #[serde(deserialize_with = "bool_from_int")]
    pub disablereceiveshadows: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub disableshadows: bool,
    pub explodedamage: u8,
    pub explodemagnitude: f32,
    pub exploderadius: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub explosion: bool,
    pub gibdir: Vector,
    pub health: u8,
    pub material: u32,
    pub minhealthdmg: u8,
    pub model: &'a str,
    #[serde(deserialize_with = "bool_from_int")]
    pub nodamageforces: bool,
    pub origin: Vector,
    #[serde(deserialize_with = "bool_from_int")]
    pub performancemode: bool,
    pub physdamagescale: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub pressuredelay: bool,
    pub propdata: u32,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    pub spawnflags: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub spawnobject: bool,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FuncButton<'a> {
    #[serde(deserialize_with = "bool_from_int")]
    pub disablereceiveshadows: bool,
    pub health: u8,
    #[serde(deserialize_with = "bool_from_int")]
    pub lip: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub locked_sentence: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub locked_sound: bool,
    pub model: &'a str,
    pub movedir: Vector,
    pub onpressed: &'a str,
    pub origin: Vector,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    pub sounds: u32,
    pub spawnflags: u32,
    pub speed: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub unlocked_sentence: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub unlocked_sound: bool,
    pub wait: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FuncBuyzone<'a> {
    pub model: &'a str,
    pub teamnum: u8,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FuncConveyor<'a> {
    pub angles: Vector,
    #[serde(deserialize_with = "bool_from_int")]
    pub disablereceiveshadows: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub disableshadows: bool,
    pub model: &'a str,
    pub movedir: Vector,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    pub spawnflags: u32,
    pub speed: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FuncDoorRotating<'a> {
    pub angles: Vector,
    #[serde(deserialize_with = "bool_from_int")]
    pub disablereceiveshadows: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub disableshadows: bool,
    pub distance: f32,
    pub dmg: i32,
    #[serde(deserialize_with = "bool_from_int")]
    pub forceclosed: bool,
    pub health: u8,
    #[serde(deserialize_with = "bool_from_int")]
    pub ignoredebris: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub lip: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub locked_sentence: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub loopmovesound: bool,
    pub model: &'a str,
    pub origin: Vector,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub solidbsp: bool,
    pub spawnflags: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub spawnpos: bool,
    pub speed: f32,
    pub targetname: &'a str,
    #[serde(deserialize_with = "bool_from_int")]
    pub unlocked_sentence: bool,
    pub wait: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FuncDustcloud<'a> {
    pub alpha: u8,
    pub color: Color,
    pub distmax: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub frozen: bool,
    pub lifetimemax: f32,
    pub lifetimemin: f32,
    pub model: &'a str,
    pub sizemax: f32,
    pub sizemin: f32,
    pub spawnrate: f32,
    pub speedmax: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub startdisabled: bool,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FuncMovelinear<'a> {
    #[serde(deserialize_with = "bool_from_int")]
    pub blockdamage: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub disablereceiveshadows: bool,
    pub model: &'a str,
    pub movedir: Vector,
    pub movedistance: f32,
    pub origin: Vector,
    pub parentname: &'a str,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    pub spawnflags: u32,
    pub speed: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub startposition: bool,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FuncPhysbox<'a> {
    #[serde(deserialize_with = "bool_from_int")]
    pub damagetoenablemotion: bool,
    pub damagetype: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub disablereceiveshadows: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub disableshadows: bool,
    pub explodedamage: u8,
    pub explodemagnitude: f32,
    pub exploderadius: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub explosion: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub forcetoenablemotion: bool,
    pub gibdir: Vector,
    pub health: u8,
    pub massscale: f32,
    pub material: u32,
    pub model: &'a str,
    #[serde(deserialize_with = "bool_from_int")]
    pub nodamageforces: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub notsolid: bool,
    pub origin: Vector,
    pub parentname: &'a str,
    #[serde(deserialize_with = "bool_from_int")]
    pub performancemode: bool,
    pub preferredcarryangles: Angles,
    #[serde(deserialize_with = "bool_from_int")]
    pub pressuredelay: bool,
    pub propdata: u32,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    pub spawnflags: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub spawnobject: bool,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FuncPhysboxMultiplayer<'a> {
    pub _minlight: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub damagetoenablemotion: bool,
    pub damagetype: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub disablereceiveshadows: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub disableshadows: bool,
    pub explodedamage: u8,
    pub explodemagnitude: f32,
    pub exploderadius: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub explosion: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub forcetoenablemotion: bool,
    pub gibdir: Vector,
    pub health: u8,
    pub massscale: f32,
    pub material: u32,
    pub model: &'a str,
    #[serde(deserialize_with = "bool_from_int")]
    pub nodamageforces: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub notsolid: bool,
    pub origin: Vector,
    #[serde(deserialize_with = "bool_from_int")]
    pub performancemode: bool,
    pub preferredcarryangles: Angles,
    #[serde(deserialize_with = "bool_from_int")]
    pub pressuredelay: bool,
    pub propdata: u32,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    pub spawnflags: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub spawnobject: bool,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FuncPrecipitation<'a> {
    pub model: &'a str,
    pub preciptype: u8,
    pub renderamt: u8,
    pub rendercolor: Color,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FuncRotButton<'a> {
    pub angles: Vector,
    pub distance: f32,
    pub health: u8,
    pub model: &'a str,
    pub onpressed: &'a str,
    pub origin: Vector,
    pub sounds: u32,
    pub spawnflags: u32,
    pub speed: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub startdisabled: bool,
    pub targetname: &'a str,
    pub wait: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FuncRotating<'a> {
    pub angles: Vector,
    #[serde(deserialize_with = "bool_from_int")]
    pub disablereceiveshadows: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub disableshadows: bool,
    pub dmg: i32,
    #[serde(deserialize_with = "bool_from_int")]
    pub fanfriction: bool,
    pub maxspeed: f32,
    pub model: &'a str,
    pub origin: Vector,
    pub parentname: &'a str,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub solidbsp: bool,
    pub spawnflags: u32,
    pub volume: u8,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FuncSmokevolume<'a> {
    pub color1: Color,
    pub color2: Color,
    pub density: f32,
    pub densityrampspeed: f32,
    pub material: &'a str,
    pub model: &'a str,
    pub movementspeed: f32,
    pub particledrawwidth: f32,
    pub particlespacingdistance: f32,
    pub rotationspeed: f32,
    pub spawnflags: u32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FuncTracktrain<'a> {
    pub bank: i16,
    #[serde(deserialize_with = "bool_from_int")]
    pub disablereceiveshadows: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub disableshadows: bool,
    pub dmg: i32,
    pub height: f32,
    pub model: &'a str,
    pub movesound: &'a str,
    pub movesoundmaxpitch: u8,
    pub movesoundmaxtime: f32,
    pub movesoundminpitch: u8,
    pub movesoundmintime: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub orientationtype: bool,
    pub origin: Vector,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    pub spawnflags: u32,
    pub speed: f32,
    pub startspeed: f32,
    pub target: &'a str,
    pub targetname: &'a str,
    #[serde(deserialize_with = "bool_from_int")]
    pub velocitytype: bool,
    pub volume: u8,
    pub wheels: u8,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FuncWall<'a> {
    #[serde(deserialize_with = "bool_from_int")]
    pub disablereceiveshadows: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub disableshadows: bool,
    pub model: &'a str,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FuncWallToggle<'a> {
    #[serde(deserialize_with = "bool_from_int")]
    pub disablereceiveshadows: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub disableshadows: bool,
    pub model: &'a str,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FuncWaterAnalog<'a> {
    pub _minlight: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub disablereceiveshadows: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub disableshadows: bool,
    pub model: &'a str,
    pub movedir: Vector,
    pub movedistance: f32,
    pub origin: Vector,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    pub speed: f32,
    pub waveheight: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct GamePlayerEquip {
    pub origin: Vector,
    #[serde(deserialize_with = "bool_from_int")]
    pub weapon_knife: bool,
}
#[derive(Debug, Clone, Deserialize)]
pub struct GameUi<'a> {
    pub fieldofview: f32,
    pub origin: Vector,
    pub playeroff: &'a str,
    pub pressedback: &'a str,
    pub pressedforward: &'a str,
    pub pressedmoveleft: &'a str,
    pub pressedmoveright: &'a str,
    pub spawnflags: u32,
    pub targetname: &'a str,
    pub unpressedback: &'a str,
    pub unpressedforward: &'a str,
    pub unpressedmoveleft: &'a str,
    pub unpressedmoveright: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct GameWeaponManager<'a> {
    #[serde(deserialize_with = "bool_from_int")]
    pub ammomod: bool,
    pub maxpieces: u8,
    pub origin: Vector,
    pub weaponname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct InfoLadder {
    #[serde(rename = "maxs.x")]
    pub maxs_x: f32,
    #[serde(rename = "maxs.y")]
    pub maxs_y: f32,
    #[serde(rename = "maxs.z")]
    pub maxs_z: f32,
    #[serde(rename = "mins.x")]
    pub mins_x: f32,
    #[serde(rename = "mins.y")]
    pub mins_y: f32,
    #[serde(rename = "mins.z")]
    pub mins_z: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct InfoPlayerStart {
    pub angles: Vector,
    pub origin: Vector,
    pub spawnflags: u32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct InfoPlayerTerrorist {
    pub angles: Vector,
    pub origin: Vector,
}
#[derive(Debug, Clone, Deserialize)]
pub struct InfoTarget<'a> {
    pub angles: Vector,
    pub origin: Vector,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct InfoTeleportDestination<'a> {
    pub angles: Vector,
    pub origin: Vector,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Infodecal<'a> {
    pub angles: Vector,
    pub origin: Vector,
    pub texture: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct KeyframeRope<'a> {
    pub angles: Vector,
    pub movespeed: f32,
    pub origin: Vector,
    pub ropematerial: &'a str,
    pub slack: f32,
    pub subdiv: u8,
    pub targetname: &'a str,
    pub texturescale: f32,
    pub width: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Light {
    pub _light: Color,
    pub _lighthdr: LightColor,
    #[serde(deserialize_with = "bool_from_int")]
    pub _lightscalehdr: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub _quadratic_attn: bool,
    pub origin: Vector,
}
#[derive(Debug, Clone, Deserialize)]
pub struct LightEnvironment<'a> {
    pub _ambient: LightColor,
    pub _ambienthdr: &'a str,
    pub _light: LightColor,
    pub _lighthdr: LightColor,
    pub angles: Vector,
    pub origin: Vector,
    pub pitch: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct LogicAuto<'a> {
    pub onmapspawn: &'a str,
    pub origin: Vector,
    pub spawnflags: u32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct LogicRelay<'a> {
    pub ontrigger: &'a str,
    pub origin: Vector,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct LogicTimer<'a> {
    pub lowerrandombound: i32,
    pub ontimer: &'a str,
    pub origin: Vector,
    pub spawnflags: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub startdisabled: bool,
    pub upperrandombound: i32,
    #[serde(deserialize_with = "bool_from_int")]
    pub userandomtime: bool,
}
#[derive(Debug, Clone, Deserialize)]
pub struct MathCounter<'a> {
    pub max: i32,
    pub min: i32,
    pub onhitmax: &'a str,
    pub origin: Vector,
    #[serde(deserialize_with = "bool_from_int")]
    pub startdisabled: bool,
    pub startvalue: i32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct MoveRope<'a> {
    pub angles: Vector,
    #[serde(deserialize_with = "bool_from_int")]
    pub barbed: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub breakable: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub collide: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub dangling: bool,
    pub maxdxlevel: u8,
    pub mindxlevel: u8,
    pub movespeed: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub nowind: bool,
    pub origin: Vector,
    pub positioninterpolator: u8,
    #[serde(deserialize_with = "bool_from_int")]
    pub r#type: bool,
    pub ropematerial: &'a str,
    pub slack: f32,
    pub spawnflags: u32,
    pub subdiv: u8,
    pub targetname: &'a str,
    pub texturescale: f32,
    pub width: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct PathTrack<'a> {
    pub angles: Vector,
    #[serde(deserialize_with = "bool_from_int")]
    pub orientationtype: bool,
    pub origin: Vector,
    pub spawnflags: u32,
    pub target: &'a str,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct PhysBallsocket<'a> {
    pub attach1: &'a str,
    pub origin: Vector,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct PlayerSpeedmod<'a> {
    pub origin: Vector,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct PlayerWeaponstrip<'a> {
    pub origin: Vector,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct PointClientcommand<'a> {
    pub origin: Vector,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct PointServercommand<'a> {
    pub origin: Vector,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct PointSpotlight<'a> {
    pub angles: Vector,
    pub hdrcolorscale: f32,
    pub origin: Vector,
    pub renderamt: u8,
    pub rendercolor: Color,
    pub spawnflags: u32,
    pub spotlightlength: f32,
    pub spotlightwidth: f32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct PointTemplate<'a> {
    pub origin: Vector,
    pub spawnflags: u32,
    pub targetname: &'a str,
    pub template01: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct PointViewcontrol<'a> {
    pub acceleration: f32,
    pub angles: Vector,
    pub deceleration: f32,
    pub origin: Vector,
    pub spawnflags: u32,
    pub target: &'a str,
    pub wait: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct PropDynamic<'a> {
    pub angles: Vector,
    #[serde(deserialize_with = "bool_from_int")]
    pub disablebonefollowers: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub disablereceiveshadows: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub disableshadows: bool,
    pub explodedamage: u8,
    pub exploderadius: f32,
    pub fademaxdist: f32,
    pub fademindist: f32,
    pub fadescale: f32,
    pub maxanimtime: f32,
    pub maxdxlevel: u8,
    pub minanimtime: f32,
    pub mindxlevel: u8,
    pub model: &'a str,
    pub origin: Vector,
    #[serde(deserialize_with = "bool_from_int")]
    pub performancemode: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub pressuredelay: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub randomanimation: bool,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub setbodygroup: bool,
    pub skin: u16,
    #[serde(deserialize_with = "bool_from_int")]
    pub solid: bool,
    pub spawnflags: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub startdisabled: bool,
}
#[derive(Debug, Clone, Deserialize)]
pub struct PropPhysics<'a> {
    pub angles: Vector,
    #[serde(deserialize_with = "bool_from_int")]
    pub damagetoenablemotion: bool,
    pub damagetype: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub disablereceiveshadows: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub disableshadows: bool,
    pub explodedamage: u8,
    pub exploderadius: f32,
    pub fademaxdist: f32,
    pub fademindist: f32,
    pub fadescale: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub forcetoenablemotion: bool,
    pub inertiascale: f32,
    pub massscale: f32,
    pub maxdxlevel: u8,
    pub mindxlevel: u8,
    pub minhealthdmg: u8,
    pub model: &'a str,
    #[serde(deserialize_with = "bool_from_int")]
    pub nodamageforces: bool,
    pub origin: Vector,
    #[serde(deserialize_with = "bool_from_int")]
    pub performancemode: bool,
    pub physdamagescale: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub pressuredelay: bool,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    pub shadowcastdist: f32,
    pub skin: u16,
    pub spawnflags: u32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct PropPhysicsMultiplayer<'a> {
    pub angles: Vector,
    #[serde(deserialize_with = "bool_from_int")]
    pub damagetoenablemotion: bool,
    pub damagetype: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub disablereceiveshadows: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub disableshadows: bool,
    pub explodedamage: u8,
    pub exploderadius: f32,
    pub fademaxdist: f32,
    pub fademindist: f32,
    pub fadescale: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub forcetoenablemotion: bool,
    pub inertiascale: f32,
    pub massscale: f32,
    pub maxdxlevel: u8,
    pub mindxlevel: u8,
    pub minhealthdmg: u8,
    pub model: &'a str,
    #[serde(deserialize_with = "bool_from_int")]
    pub nodamageforces: bool,
    pub origin: Vector,
    #[serde(deserialize_with = "bool_from_int")]
    pub performancemode: bool,
    pub physdamagescale: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub physicsmode: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub pressuredelay: bool,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    pub shadowcastdist: f32,
    pub skin: u16,
    pub spawnflags: u32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct PropPhysicsOverride<'a> {
    pub angles: Vector,
    #[serde(deserialize_with = "bool_from_int")]
    pub damagetoenablemotion: bool,
    pub damagetype: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub disableshadows: bool,
    pub explodedamage: u8,
    pub exploderadius: f32,
    pub fademaxdist: f32,
    pub fademindist: f32,
    pub fadescale: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub forcetoenablemotion: bool,
    pub health: u8,
    pub inertiascale: f32,
    pub massscale: f32,
    pub maxdxlevel: u8,
    pub mindxlevel: u8,
    pub minhealthdmg: u8,
    pub model: &'a str,
    #[serde(deserialize_with = "bool_from_int")]
    pub nodamageforces: bool,
    pub origin: Vector,
    pub parentname: &'a str,
    #[serde(deserialize_with = "bool_from_int")]
    pub performancemode: bool,
    pub physdamagescale: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub pressuredelay: bool,
    pub shadowcastdist: f32,
    pub skin: u16,
    pub spawnflags: u32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct PropRagdoll<'a> {
    pub angles: Vector,
    pub fademindist: f32,
    pub fadescale: f32,
    pub model: &'a str,
    pub modelscale: f32,
    pub origin: Vector,
    pub skin: u16,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct ShadowControl {
    pub angles: Vector,
    pub color: Color,
    pub distance: f32,
    pub origin: Vector,
}
#[derive(Debug, Clone, Deserialize)]
pub struct SkyCamera {
    pub angles: Vector,
    pub fogcolor: Color,
    pub fogcolor2: Color,
    pub fogdir: Vector,
    pub fogend: f32,
    pub fogstart: f32,
    pub origin: Vector,
    pub scale: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct TriggerGravity<'a> {
    pub gravity: f32,
    pub model: &'a str,
    pub origin: Vector,
    pub spawnflags: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub startdisabled: bool,
}
#[derive(Debug, Clone, Deserialize)]
pub struct TriggerHurt<'a> {
    pub damage: i32,
    pub damagecap: i32,
    #[serde(deserialize_with = "bool_from_int")]
    pub damagemodel: bool,
    pub damagetype: u32,
    pub model: &'a str,
    pub origin: Vector,
    pub spawnflags: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub startdisabled: bool,
}
#[derive(Debug, Clone, Deserialize)]
pub struct TriggerLook<'a> {
    pub fieldofview: f32,
    pub looktime: f32,
    pub model: &'a str,
    pub ontrigger: &'a str,
    pub origin: Vector,
    pub spawnflags: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub startdisabled: bool,
    pub target: &'a str,
    pub timeout: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct TriggerMultiple<'a> {
    pub model: &'a str,
    pub ontrigger: &'a str,
    pub origin: Vector,
    pub spawnflags: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub startdisabled: bool,
    pub wait: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct TriggerOnce<'a> {
    pub angles: Vector,
    pub model: &'a str,
    pub ontrigger: &'a str,
    pub origin: Vector,
    pub spawnflags: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub startdisabled: bool,
}
#[derive(Debug, Clone, Deserialize)]
pub struct TriggerPush<'a> {
    #[serde(deserialize_with = "bool_from_int")]
    pub alternateticksfix: bool,
    pub model: &'a str,
    pub origin: Vector,
    pub pushdir: Vector,
    pub spawnflags: u32,
    pub speed: f32,
    #[serde(deserialize_with = "bool_from_int")]
    pub startdisabled: bool,
}
#[derive(Debug, Clone, Deserialize)]
pub struct TriggerSoundscape<'a> {
    pub model: &'a str,
    pub origin: Vector,
    pub soundscape: &'a str,
    pub spawnflags: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub startdisabled: bool,
}
#[derive(Debug, Clone, Deserialize)]
pub struct TriggerTeleport<'a> {
    pub model: &'a str,
    pub origin: Vector,
    pub spawnflags: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub startdisabled: bool,
    pub target: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WaterLodControl {
    pub cheapwaterenddistance: f32,
    pub cheapwaterstartdistance: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponAk47<'a> {
    pub ammo: u32,
    pub angles: Vector,
    pub origin: Vector,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponAwp<'a> {
    pub angles: Vector,
    pub origin: Vector,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponDeagle {
    pub ammo: u32,
    pub angles: Vector,
    pub origin: Vector,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponElite<'a> {
    pub ammo: u32,
    pub angles: Vector,
    pub origin: Vector,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponFamas {
    pub ammo: u32,
    pub angles: Vector,
    pub fademaxdist: f32,
    pub fademindist: f32,
    pub fadescale: f32,
    pub maxdxlevel: u8,
    pub mindxlevel: u8,
    #[serde(deserialize_with = "bool_from_int")]
    pub nodamageforces: bool,
    pub origin: Vector,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    pub shadowcastdist: f32,
    pub spawnflags: u32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponFiveseven<'a> {
    pub ammo: u32,
    pub angles: Vector,
    pub origin: Vector,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponFlashbang<'a> {
    pub angles: Vector,
    pub fademaxdist: f32,
    pub fademindist: f32,
    pub fadescale: f32,
    pub origin: Vector,
    pub renderamt: u8,
    pub rendercolor: Color,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponG3Sg1 {
    pub ammo: u32,
    pub angles: Vector,
    pub fademaxdist: f32,
    pub fademindist: f32,
    pub fadescale: f32,
    pub maxdxlevel: u8,
    pub mindxlevel: u8,
    #[serde(deserialize_with = "bool_from_int")]
    pub nodamageforces: bool,
    pub origin: Vector,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    pub shadowcastdist: f32,
    pub spawnflags: u32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponGlock<'a> {
    pub ammo: u32,
    pub angles: Vector,
    pub origin: Vector,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponHegrenade<'a> {
    pub angles: Vector,
    pub origin: Vector,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponKnife<'a> {
    pub angles: Vector,
    pub origin: Vector,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponM249<'a> {
    pub ammo: u32,
    pub angles: Vector,
    pub origin: Vector,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponM3<'a> {
    pub ammo: u32,
    pub angles: Vector,
    pub origin: Vector,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponM4A1 {
    pub angles: Vector,
    pub origin: Vector,
    pub spawnflags: u32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponMac10 {
    pub ammo: u32,
    pub angles: Vector,
    pub fademaxdist: f32,
    pub fademindist: f32,
    pub fadescale: f32,
    pub maxdxlevel: u8,
    pub mindxlevel: u8,
    #[serde(deserialize_with = "bool_from_int")]
    pub nodamageforces: bool,
    pub origin: Vector,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    pub shadowcastdist: f32,
    pub spawnflags: u32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponP90<'a> {
    pub ammo: u32,
    pub angles: Vector,
    pub origin: Vector,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponScout<'a> {
    pub angles: Vector,
    pub origin: Vector,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponSg550 {
    pub ammo: u32,
    pub angles: Vector,
    pub fademaxdist: f32,
    pub fademindist: f32,
    pub fadescale: f32,
    pub maxdxlevel: u8,
    pub mindxlevel: u8,
    #[serde(deserialize_with = "bool_from_int")]
    pub nodamageforces: bool,
    pub origin: Vector,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    pub shadowcastdist: f32,
    pub spawnflags: u32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponSmokegrenade<'a> {
    pub angles: Vector,
    pub origin: Vector,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponTmp {
    pub ammo: u32,
    pub angles: Vector,
    pub fademaxdist: f32,
    pub fademindist: f32,
    pub fadescale: f32,
    pub maxdxlevel: u8,
    pub mindxlevel: u8,
    #[serde(deserialize_with = "bool_from_int")]
    pub nodamageforces: bool,
    pub origin: Vector,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    pub shadowcastdist: f32,
    pub spawnflags: u32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponUmp45 {
    pub ammo: u32,
    pub angles: Vector,
    pub fademaxdist: f32,
    pub fademindist: f32,
    pub fadescale: f32,
    pub maxdxlevel: u8,
    pub mindxlevel: u8,
    #[serde(deserialize_with = "bool_from_int")]
    pub nodamageforces: bool,
    pub origin: Vector,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    pub shadowcastdist: f32,
    pub spawnflags: u32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponUsp<'a> {
    pub ammo: u32,
    pub angles: Vector,
    pub origin: Vector,
    pub spawnflags: u32,
    pub targetname: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponXm1014 {
    pub ammo: u32,
    pub angles: Vector,
    pub fademaxdist: f32,
    pub fademindist: f32,
    pub fadescale: f32,
    pub maxdxlevel: u8,
    pub mindxlevel: u8,
    #[serde(deserialize_with = "bool_from_int")]
    pub nodamageforces: bool,
    pub origin: Vector,
    pub renderamt: u8,
    pub rendercolor: Color,
    #[serde(deserialize_with = "bool_from_int")]
    pub renderfx: bool,
    pub rendermode: u32,
    pub shadowcastdist: f32,
    pub spawnflags: u32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Worldspawn<'a> {
    pub detailmaterial: &'a str,
    pub detailvbsp: &'a str,
    pub maxpropscreenwidth: i32,
    pub skyname: &'a str,
    pub world_maxs: Vector,
    pub world_mins: Vector,
}
