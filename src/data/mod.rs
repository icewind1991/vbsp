mod displacement;
mod entity;
mod game;
mod prop;
mod vector;

pub use self::displacement::*;
pub use self::entity::*;
pub use self::game::*;
pub use self::prop::PropPlacement;
pub use self::vector::*;
use crate::bspfile::LumpType;
use crate::{BspResult, StringError};
use arrayvec::ArrayString;
use binrw::error::CustomError;
use binrw::{BinRead, BinResult, Endian};
use bitflags::bitflags;
use bv::BitVec;
use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use std::borrow::Cow;
use std::cmp::min;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::io::{Cursor, Read, Seek};
use std::mem::{align_of, size_of};
use std::ops::Index;
use std::sync::Mutex;
use zip::result::ZipError;
use zip::ZipArchive;

/// Validate that reading the type consumes `size_of::<T>()` bytes
#[cfg(test)]
fn test_read_bytes<T: BinRead>()
where
    T::Args<'static>: Default,
    <T as BinRead>::Args<'static>: Clone,
{
    use binrw::BinReaderExt;
    use std::any::type_name;

    let bytes = [0; 512];
    let mut reader = Cursor::new(bytes);

    let _ = reader.read_le::<T>().unwrap();

    assert_eq!(
        reader.position() as usize,
        size_of::<T>(),
        "Invalid number of bytes used to read {}",
        type_name::<T>()
    );
}

#[derive(Clone, BinRead)]
pub struct Directories {
    entries: [LumpEntry; 64],
}

impl Index<LumpType> for Directories {
    type Output = LumpEntry;

    fn index(&self, index: LumpType) -> &Self::Output {
        &self.entries[index as usize]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, BinRead)]
#[br(little)]
pub struct Header {
    pub v: u8,
    pub b: u8,
    pub s: u8,
    pub p: u8,
}

#[derive(Clone, Copy, Debug, Default, BinRead)]
#[br(little)]
pub struct LumpEntry {
    pub offset: u32,
    pub length: u32,
    pub version: u32,
    pub ident: u32,
}

#[derive(Debug, Clone, BinRead)]
pub struct LeafFace {
    pub face: u16,
}

#[derive(BinRead, Debug, Clone, Copy)]
pub struct TextureFlags(u32);

bitflags! {
    impl TextureFlags: u32 {
        const LIGHT      = 0b0000_0000_0000_0000_0001; // value will hold the light strength
        const SKY2D      = 0b0000_0000_0000_0000_0010; // don't draw, indicate we should skylight + draw 2d sky but don't draw the 3d skybox
        const SKY        = 0b0000_0000_0000_0000_0100; // don't draw, but add the skybox
        const WARP       = 0b0000_0000_0000_0000_1000; // turbulent water warp
        const TRANS      = 0b0000_0000_0000_0001_0000; // texture is translucent
        const NOPORTAL   = 0b0000_0000_0000_0010_0000; // the surface can't have a portal placed on it
        const TRIGGER    = 0b0000_0000_0000_0100_0000; // xbox hack to work around elimination of trigger surfaces
        const NODRAW     = 0b0000_0000_0000_1000_0000; // don't bother referencing the texture
        const HINT       = 0b0000_0000_0001_0000_0000; // make a primary bsp splitter
        const SKIP       = 0b0000_0000_0010_0000_0000; // completely ignore, allowing non-closed brushes
        const NOLIGHT    = 0b0000_0000_0100_0000_0000; // dont calculate light
        const BUMPLIGHT  = 0b0000_0000_1000_0000_0000; // calculate thee light maps for the surface for bump mapping
        const NOSHADOWS  = 0b0000_0001_0000_0000_0000; // don't receive shadows
        const NODECALS   = 0b0000_0010_0000_0000_0000; // don't receive decals
        const NOCHOP     = 0b0000_0100_0000_0000_0000; // don't subdivide patches on this surface
        const HITBOX     = 0b0000_1000_0000_0000_0000; // surface is part of a hitbox
    }
}

/// Fixed length, null-terminated string
#[derive(Debug, Clone)]
pub struct FixedString<const LEN: usize>(ArrayString<LEN>);

impl<const N: usize> AsRef<str> for FixedString<N> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> FixedString<N> {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<const LEN: usize> Display for FixedString<LEN> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<const LEN: usize> BinRead for FixedString<LEN> {
    type Args<'a> = ();

    fn read_options<R: Read + binrw::io::Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'static>,
    ) -> BinResult<Self> {
        use std::str;

        let start = reader.stream_position().unwrap();

        let name_buf = <[u8; LEN]>::read_options(reader, endian, args)?;

        let zero_pos =
            name_buf
                .iter()
                .position(|c| *c == 0)
                .ok_or_else(|| binrw::Error::Custom {
                    pos: start,
                    err: Box::new(StringError::NotNullTerminated),
                })?;
        let name = &name_buf[..zero_pos];
        Ok(FixedString(
            ArrayString::from(
                str::from_utf8(name)
                    .map_err(StringError::NonUTF8)
                    .map_err(|e| binrw::Error::Custom {
                        pos: start,
                        err: Box::new(e),
                    })?,
            )
            .expect(
                "Programmer error: it should be impossible for the string to exceed the capacity",
            ),
        ))
    }
}

#[derive(Debug, Clone, BinRead)]
pub struct TextureInfo {
    pub texture_transforms_u: [f32; 4],
    pub texture_transforms_v: [f32; 4],
    pub light_map_scale: [f32; 4],
    pub light_map_transform: [f32; 4],
    pub flags: TextureFlags,
    pub texture_data_index: i32,
}

static_assertions::const_assert_eq!(size_of::<TextureInfo>(), 72);

#[derive(Debug, Clone, BinRead)]
pub struct TextureData {
    pub reflectivity: Vector,
    pub name_string_table_id: i32,
    pub width: i32,
    pub height: i32,
    pub view_width: i32,
    pub view_height: i32,
}

#[derive(Debug, Clone, BinRead)]
pub struct Plane {
    pub normal: Vector,
    pub dist: f32,
    pub ty: i32,
}

#[derive(Debug, Clone, BinRead)]
pub struct Node {
    pub plane_index: i32,
    pub children: [i32; 2],
    pub mins: [i16; 3],
    pub maxs: [i16; 3],
    pub first_face: u16,
    pub face_cound: u16,
    pub area: i16,
    pub padding: i16,
}

static_assertions::const_assert_eq!(size_of::<Node>(), 32);

#[derive(Default, Debug, Clone, BinRead)]
pub struct Leaf {
    pub contents: i32,
    pub cluster: i16,
    pub area_and_flags: i16,
    // first 9 bits is area, last 7 bits is flags
    pub mins: [i16; 3],
    pub maxs: [i16; 3],
    pub first_leaf_face: u16,
    pub leaf_face_count: u16,
    pub first_leaf_brush: u16,
    pub leaf_brush_count: u16,
    #[br(align_after = align_of::< Leaf > ())]
    pub leaf_watter_data_id: i16,
}

static_assertions::const_assert_eq!(size_of::<Leaf>(), 32);

#[test]
fn test_leaf_bytes() {
    test_read_bytes::<Leaf>();
}

#[derive(Debug, Clone, BinRead)]
pub struct LeafBrush {
    pub brush: u16,
}

#[derive(Debug, Clone, BinRead)]
pub struct Model {
    pub mins: Vector,
    pub maxs: Vector,
    pub origin: Vector,
    pub head_node: i32,
    pub first_face: i32,
    pub face_count: i32,
}

static_assertions::const_assert_eq!(size_of::<Model>(), 48);

#[derive(Debug, Clone, BinRead)]
pub struct Brush {
    pub brush_side: u32,
    pub num_brush_sides: u32,
    pub flags: BrushFlags,
}

impl Brush {
    pub fn is_visible(&self) -> bool {
        self.flags.intersects(
            BrushFlags::SOLID
                | BrushFlags::GRATE
                | BrushFlags::OPAQUE
                | BrushFlags::TESTFOGVOLUME
                | BrushFlags::TRANSLUCENT,
        )
    }
}

#[derive(BinRead, Debug, Clone, Copy)]
pub struct BrushFlags(u32);

bitflags! {
    impl BrushFlags: u32 {
        const EMPTY =       	        0; // 	No contents
        const SOLID =       	        0x1; // 	an eye is never valid in a solid
        const WINDOW =      	        0x2; // 	translucent, but not watery (glass)
        const AUX =         	        0x4;
        const GRATE =       	        0x8; // 	alpha-tested "grate" textures. Bullets/sight pass through, but solids don't
        const SLIME =       	        0x10;
        const WATER =       	        0x20;
        const MIST =        	        0x40;
        const OPAQUE =      	        0x80; // 	block AI line of sight
        const TESTFOGVOLUME =          0x100; // 	things that cannot be seen through (may be non-solid though)
        const UNUSED =      	        0x200; // 	unused
        const UNUSED6 =                0x400; // 	unused
        const TEAM1 =       	        0x800; // 	per team contents used to differentiate collisions between players and objects on different teams
        const TEAM2 =       	        0x1000;
        const IGNORE_NODRAW_OPAQUE =   0x2000; // 	ignore CONTENTS_OPAQUE on surfaces that have SURF_NODRAW
        const MOVEABLE =               0x4000; // 	hits entities which are MOVETYPE_PUSH (doors, plats, etc.)
        const AREAPORTAL =             0x8000; // 	remaining contents are non-visible, and don't eat brushes
        const PLAYERCLIP =             0x10000;
        const MONSTERCLIP =            0x20000;
        const CURRENT_0 =              0x40000; // 	currents can be added to any other contents, and may be mixed
        const CURRENT_90 =             0x80000;
        const CURRENT_180 =            0x100000;
        const CURRENT_270 =            0x200000;
        const CURRENT_UP =             0x400000;
        const CURRENT_DOWN =           0x800000;
        const ORIGIN =      	        0x1000000; // 	removed before bsping an entity
        const MONSTER =                0x2000000; // 	should never be on a brush, only in game
        const DEBRIS =      	        0x4000000;
        const DETAIL =      	        0x8000000; // 	brushes to be added after vis leafs
        const TRANSLUCENT =            0x10000000; // 	auto set if any surface has trans
        const LADDER =      	        0x20000000;
        const HITBOX =      	        0x40000000; // 	use accurate hitboxes on trace
    }
}

#[derive(Debug, Clone, BinRead)]
pub struct BrushSide {
    pub plane: u16,
    pub texture_info: i16,
    pub displacement_info: i16,
    pub bevel: i16,
}

#[derive(Debug, Clone, BinRead)]
pub struct Vertex {
    pub position: Vector,
}

#[derive(Debug, Clone, BinRead)]
pub struct Edge {
    pub start_index: u16,
    pub end_index: u16,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum EdgeDirection {
    FirstToLast,
    LastToFirst,
}

#[derive(Debug, Clone, BinRead)]
pub struct SurfaceEdge {
    edge: i32,
}

impl SurfaceEdge {
    pub fn edge_index(&self) -> u32 {
        self.edge.unsigned_abs()
    }

    pub fn direction(&self) -> EdgeDirection {
        if self.edge >= 0 {
            EdgeDirection::FirstToLast
        } else {
            EdgeDirection::LastToFirst
        }
    }
}

#[derive(Debug, Clone, BinRead)]
pub struct Face {
    pub plane_num: u16,
    pub side: u8,
    pub on_node: u8,
    pub first_edge: i32,
    pub num_edges: i16,
    pub texture_info: i16,
    pub displacement_info: i16,
    pub surface_fog_volume_id: i16,
    pub styles: [u8; 4],
    pub light_offset: i32,
    pub area: f32,
    pub light_map_texture_min: [i32; 2],
    pub light_map_texture_size: [i32; 2],
    pub original_face: i32,
    pub primitive_count: u16,
    pub first_primitive_index: u16,
    pub smoothing_groups: u32,
}

impl Face {
    pub fn displacement_index(&self) -> Option<i16> {
        (self.displacement_info >= 0).then_some(self.displacement_info)
    }
}

static_assertions::const_assert_eq!(size_of::<Face>(), 56);

#[derive(Default, Debug, Clone)]
pub struct VisData {
    pub cluster_count: u32,
    pub pvs_offsets: Vec<i32>,
    pub pas_offsets: Vec<i32>,
    pub data: Vec<u8>,
}

impl VisData {
    pub fn visible_clusters(&self, cluster: i16) -> BitVec<u8> {
        let offset = self.pvs_offsets[cluster as usize] as usize;
        let pvs_buffer = &self.data[offset..];
        let mut visible_clusters = BitVec::with_capacity(min(self.cluster_count as u64, 1024));
        visible_clusters.resize(self.cluster_count as u64, false);

        let mut cluster_index = 0;
        let mut buffer_index = 0;

        while cluster_index < self.cluster_count {
            if pvs_buffer[buffer_index] == 0 {
                let skip = pvs_buffer[buffer_index + 1];
                cluster_index += skip as u32;
                buffer_index += 2;
            } else {
                let packed = pvs_buffer[buffer_index];
                for i in 0..8 {
                    let bit = 1 << i;
                    if (packed & bit) == bit {
                        visible_clusters.set(cluster_index as u64, true);
                    }
                    cluster_index += 1;
                }
                buffer_index += 1;
            }
        }

        visible_clusters
    }
}

#[derive(Debug, Clone, BinRead)]
pub struct VertNormal {
    pub normal: f32,
}

#[derive(Debug, Clone, BinRead)]
pub struct VertNormalIndex {
    pub index: i16,
}

pub struct Packfile {
    zip: Mutex<ZipArchive<Cursor<Vec<u8>>>>,
}

impl Clone for Packfile {
    fn clone(&self) -> Self {
        Packfile {
            zip: Mutex::new(self.zip.lock().unwrap().clone()),
        }
    }
}

impl Debug for Packfile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Packfile")
            .field(
                "zip",
                &self
                    .zip
                    .lock()
                    .unwrap()
                    .file_names()
                    .collect::<Vec<_>>()
                    .join(", "),
            )
            .finish()
    }
}

impl Packfile {
    pub fn read(data: Cow<[u8]>) -> BspResult<Self> {
        let reader = Cursor::new(data.into_owned());
        let zip = Mutex::new(ZipArchive::new(reader)?);
        Ok(Packfile { zip })
    }

    pub fn get(&self, name: &str) -> BspResult<Option<Vec<u8>>> {
        let mut zip = self.zip.lock().unwrap();
        let mut entry = match zip.by_name(name) {
            Ok(entry) => entry,
            Err(ZipError::FileNotFound) => {
                return Ok(None);
            }
            Err(e) => {
                return Err(e.into());
            }
        };
        let mut buff = vec![0; entry.size() as usize];
        entry.read_exact(&mut buff)?;
        Ok(Some(buff))
    }

    pub fn has(&self, name: &str) -> BspResult<bool> {
        let mut zip = self.zip.lock().unwrap();
        let result = match zip.by_name(name) {
            Ok(_) => Ok(true),
            Err(ZipError::FileNotFound) => {
                return Ok(false);
            }
            Err(e) => {
                return Err(e.into());
            }
        };
        result
    }

    pub fn into_zip(self) -> Mutex<ZipArchive<Cursor<Vec<u8>>>> {
        self.zip
    }
}

fn try_read_enum<Enum, Reader, Error, ErrorFn>(
    reader: &mut Reader,
    endian: Endian,
    args: <<Enum as TryFromPrimitive>::Primitive as BinRead>::Args<'static>,
    err_map: ErrorFn,
) -> BinResult<Enum>
where
    Reader: Read + Seek,
    Enum: TryFromPrimitive<Error = TryFromPrimitiveError<Enum>>,
    Enum::Primitive: BinRead,
    ErrorFn: FnOnce(Enum::Primitive) -> Error,
    Error: CustomError + 'static,
{
    let start = reader.stream_position().unwrap();
    let raw = <Enum::Primitive>::read_options(reader, endian, args)?;

    Enum::try_from_primitive(raw)
        .map_err(|e| err_map(e.number))
        .map_err(|e| binrw::Error::Custom {
            pos: start,
            err: Box::new(e),
        })
}
