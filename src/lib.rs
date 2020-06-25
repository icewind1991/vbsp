#![allow(dead_code)]
#![allow(unreachable_code)]

mod bspfile;
mod reader;

use crate::bspfile::LumpType;
use arrayvec::ArrayString;
use binread::io::{Cursor, SeekFrom};
use binread::{BinRead, BinResult, ReadOptions};
use bitflags::bitflags;
use bspfile::BspFile;
use bv::BitVec;
use itertools::{GroupBy, Itertools};
use parse_display::Display;
use reader::LumpReader;
use std::ops::Index;
use std::{
    convert::{TryFrom, TryInto},
    fmt,
    io::{self, Error, ErrorKind, Read},
    iter::once,
    mem::size_of,
    ops::Deref,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BspError {
    #[error("unexpected magic numbers or version, is this a valve bsp?")]
    UnexpectedHeader(Header),
    #[error("bsp lump is out of bounds of the bsp file")]
    LumpOutOfBounds(LumpEntry),
    #[error("unexpected length of uncompressed lump, got {got} but expected {expected}")]
    UnexpectedUncompressedLumpSize { got: u32, expected: u32 },
    #[error("error while decompressing lump")]
    LumpDecompressError(lzma_rs::error::Error),
    #[error("malformed utf8 data")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Directory entry length isn't a multiple of element size")]
    MalformedLump,
    #[error("invalid surface flag in {0}")]
    InvalidSurfaceFlag(Name),
    #[error("invalid content flag in {0}")]
    InvalidContentFlag(Name),
    #[error("non null-terminated name")]
    InvalidName,
    #[error("unexpected eof while reading data")]
    UnexpectedEOF,
    #[error("extra data at the end of the lump")]
    UnexpectedExtraData,
    #[error("error while reading data: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("error while reading data: {0}")]
    BinReadError(#[from] binread::Error),
}

pub type BspResult<T> = Result<T, BspError>;

#[derive(Debug, Clone, BinRead)]
#[repr(u32)]
pub enum FaceType {
    Polygon = 1,
    Patch = 2,
    Mesh = 3,
    Billboard = 4,
}

#[derive(Debug)]
pub struct FaceTypeError;

impl fmt::Display for FaceTypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid face type ID")
    }
}

impl std::error::Error for FaceTypeError {}

impl TryFrom<u32> for FaceType {
    type Error = FaceTypeError;

    fn try_from(other: u32) -> Result<Self, FaceTypeError> {
        const POLYGON: u32 = FaceType::Polygon as u32;
        const PATCH: u32 = FaceType::Patch as u32;
        const MESH: u32 = FaceType::Mesh as u32;
        const BILLBOARD: u32 = FaceType::Billboard as u32;

        match other {
            POLYGON => Ok(FaceType::Polygon),
            PATCH => Ok(FaceType::Patch),
            MESH => Ok(FaceType::Mesh),
            BILLBOARD => Ok(FaceType::Billboard),
            _ => Err(FaceTypeError),
        }
    }
}

#[derive(Clone)]
pub struct Directories {
    entries: [LumpEntry; 64],
}

impl BinRead for Directories {
    type Args = <LumpEntry as BinRead>::Args;

    fn read_options<R: binread::io::Read + binread::io::Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        let mut entries = [LumpEntry::default(); 64];
        for i in 0..64 {
            entries[i] = LumpEntry::read_options(reader, options, args)?;
        }

        Ok(Directories { entries })
    }
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
    offset: u32,
    length: u32,
    version: u32,
    ident: u32,
}

#[derive(Debug, Clone, BinRead)]
pub struct LeafFace {
    pub face: u32,
}

#[derive(Clone)]
pub struct Entities {
    entities: String,
}

impl fmt::Debug for Entities {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        struct Entities<'a> {
            entities: Vec<Entity<'a>>,
        }

        Entities {
            entities: self.iter().collect(),
        }
        .fmt(f)
    }
}

impl Entities {
    pub fn iter(&self) -> impl Iterator<Item = Entity<'_>> {
        struct Iter<'a> {
            buf: &'a str,
        }

        impl<'a> Iterator for Iter<'a> {
            type Item = Entity<'a>;

            fn next(&mut self) -> Option<Self::Item> {
                let start = self.buf.find('{')? + 1;
                let end = start + self.buf[start..].find('}')?;

                let out = &self.buf[start..end];

                self.buf = &self.buf[end + 1..];

                Some(Entity { buf: out })
            }
        }

        Iter {
            buf: &self.entities,
        }
    }
}

#[derive(Clone)]
pub struct Entity<'a> {
    buf: &'a str,
}

impl fmt::Debug for Entity<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::collections::HashMap;

        self.properties().collect::<HashMap<_, _>>().fmt(f)
    }
}

impl<'a> Entity<'a> {
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

        Iter { buf: &self.buf }
    }
}

bitflags! {
    #[derive(BinRead)]
    pub struct TextureFlags: u32 {
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

#[derive(Debug, Display, Clone)]
pub struct Name(ArrayString<[u8; 64]>);

impl BinRead for Name {
    type Args = ();

    fn read_options<R: binread::io::Read + binread::io::Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        use std::str;

        let mut name_buf: [u8; 64] = [0; 64];

        for i in 0..64 {
            name_buf[i] = u8::read_options(reader, options, args)?;
        }

        let zero_pos =
            name_buf
                .iter()
                .position(|c| *c == 0)
                .ok_or_else(|| binread::Error::AssertFail {
                    pos: reader.seek(SeekFrom::Current(0)).unwrap() as usize,
                    message: "Name not null terminated".to_string(),
                })?;
        let name = &name_buf[..zero_pos];
        Ok(Name(
            ArrayString::from(
                str::from_utf8(name).map_err(|err| Error::new(ErrorKind::InvalidData, err))?,
            )
            .expect(
                "Programmer error: it should be impossible for the string to exceed the capacity",
            ),
        ))
    }
}

#[derive(Debug, Clone, BinRead)]
pub struct Vector {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Clone, BinRead)]
pub struct TextureInfo {
    pub texture_scale: [f32; 4],
    pub texture_transform: [f32; 4],
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
    pub plane: u32,
    pub children: [i32; 2],
    pub mins: [i32; 3],
    pub maxs: [i32; 3],
}

#[derive(Debug, Clone, BinRead)]
pub struct Leaf {
    pub cluster: i32,
    pub area: u32,
    pub mins: [i32; 3],
    pub maxs: [i32; 3],
    pub leaf_face: u32,
    pub num_leaf_faces: u32,
    pub leaf_brush: u32,
    pub num_leaf_brushes: u32,
}

#[derive(Debug, Clone, BinRead)]
pub struct LeafBrush {
    pub brush: u32,
}

#[derive(Debug, Clone, BinRead)]
pub struct Model {
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub face: u32,
    pub num_faces: u32,
    pub brush: u32,
    pub num_brushes: u32,
}

#[derive(Debug, Clone, BinRead)]
pub struct Brush {
    pub brush_side: u32,
    pub num_brush_sides: u32,
    pub texture: u32,
}

#[derive(Debug, Clone, BinRead)]
pub struct BrushSide {
    pub plane: u32,
    pub texture: u32,
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

pub enum EdgeDirection {
    FirstToLast,
    LastToFirst,
}

#[derive(Debug, Clone, BinRead)]
pub struct SurfaceEdge {
    edge: i32,
}

impl SurfaceEdge {
    pub fn edge_index(&self) -> usize {
        self.edge.abs() as usize
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
pub struct MeshVert {
    pub offset: u32,
}

#[derive(Debug, Clone, BinRead)]
pub struct Effect {
    pub name: Name,
    pub brush: u32,
    pub unknown: u32,
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

static_assertions::const_assert_eq!(size_of::<Face>(), 56);

const LIGHTMAP_SIZE: usize = 128;

#[derive(Default, Clone, Copy, BinRead, Debug)]
pub struct LightColor {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Clone)]
pub struct Lightmap {
    map: [[LightColor; LIGHTMAP_SIZE]; LIGHTMAP_SIZE],
}

impl BinRead for Lightmap {
    type Args = <LightColor as BinRead>::Args;

    fn read_options<R: binread::io::Read + binread::io::Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        let mut map = [[LightColor::default(); LIGHTMAP_SIZE]; LIGHTMAP_SIZE];

        for x in 0..LIGHTMAP_SIZE {
            for y in 0..LIGHTMAP_SIZE {
                map[x][y] = LightColor::read_options(reader, options, args)?;
            }
        }

        Ok(Lightmap { map })
    }
}

impl fmt::Debug for Lightmap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        struct Lightmap {
            map: Vec<Vec<LightColor>>,
        }

        Lightmap {
            map: self.map.iter().map(|a| a.to_vec()).collect::<Vec<_>>(),
        }
        .fmt(f)
    }
}

#[derive(Debug, Clone, BinRead)]
pub struct Lightvol {
    ambient: [u8; 3],
    directional: [u8; 3],
    dir: [u8; 2],
}

#[derive(Default, Debug, Clone)]
pub struct VisData {
    pub n_vecs: u32,
    // Number of vectors.
    pub sz_vecs: u32,
    // Size of each vector, in bytes.
    pub vecs: BitVec<u8>, // Visibility data. One bit per cluster per vector.
}

#[derive(Debug)]
pub struct Handle<'a, T> {
    bsp: &'a Bsp,
    data: &'a T,
}

impl<T> Clone for Handle<'_, T> {
    fn clone(&self) -> Self {
        Handle { ..*self }
    }
}

impl<'a, T> Handle<'a, T> {
    pub fn as_ref(&self) -> &'a T {
        self.data
    }
}

impl<T> Deref for Handle<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

#[derive(Debug, Clone)]
pub struct Leaves {
    leaves: Vec<Leaf>,
}

impl Leaves {
    pub fn new(mut leaves: Vec<Leaf>) -> Self {
        leaves.sort_unstable_by_key(|leaf| leaf.cluster);

        Leaves { leaves }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Leaf> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Leaf> {
        self.into_iter()
    }

    pub fn into_inner(self) -> Vec<Leaf> {
        self.leaves
    }

    // TODO: There's no syntax for `-> T where &T: IntoIterator<...>` and `GroupBy`
    //       doesn't implement `IntoIterator` directly, only `&GroupBy`, so we have
    //       to explicitly specify the type.
    pub fn clusters<'this>(
        &'this self,
    ) -> GroupBy<i32, impl Iterator<Item = &'this Leaf>, impl FnMut(&&'this Leaf) -> i32> {
        self.leaves.iter().group_by(|leaf: &&Leaf| leaf.cluster)
    }
}

impl From<Vec<Leaf>> for Leaves {
    fn from(other: Vec<Leaf>) -> Self {
        Self::new(other)
    }
}

impl Deref for Leaves {
    type Target = [Leaf];

    fn deref(&self) -> &Self::Target {
        &self.leaves
    }
}

impl IntoIterator for Leaves {
    type IntoIter = <Vec<Leaf> as IntoIterator>::IntoIter;
    type Item = Leaf;

    fn into_iter(self) -> Self::IntoIter {
        self.leaves.into_iter()
    }
}

impl<'a> IntoIterator for &'a Leaves {
    type IntoIter = <&'a [Leaf] as IntoIterator>::IntoIter;
    type Item = &'a Leaf;

    fn into_iter(self) -> Self::IntoIter {
        (&self.leaves[..]).into_iter()
    }
}

impl<'a> IntoIterator for &'a mut Leaves {
    type IntoIter = <&'a mut [Leaf] as IntoIterator>::IntoIter;
    type Item = &'a mut Leaf;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.leaves[..]).into_iter()
    }
}

// TODO: Store all the allocated objects inline to improve cache usage
#[derive(Debug)]
pub struct Bsp {
    pub header: Header,
    pub entities: Entities,
    pub textures_data: Vec<TextureData>,
    pub textures_info: Vec<TextureInfo>,
    pub planes: Vec<Plane>,
    pub nodes: Vec<Node>,
    pub leaves: Leaves,
    pub leaf_faces: Vec<LeafFace>,
    pub leaf_brushes: Vec<LeafBrush>,
    pub models: Vec<Model>,
    pub brushes: Vec<Brush>,
    pub brush_sides: Vec<BrushSide>,
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
    pub surface_edges: Vec<SurfaceEdge>,
    pub faces: Vec<Face>,
    pub vis_data: VisData,
}

impl Bsp {
    pub fn read(data: &[u8]) -> BspResult<Self> {
        let bsp_file = BspFile::new(data)?;

        let entities = bsp_file.lump_reader(LumpType::Entities)?.read_entities()?;
        let textures_data = bsp_file
            .lump_reader(LumpType::TextureData)?
            .read_vec(|r| r.read())?;
        let textures_info = bsp_file
            .lump_reader(LumpType::TextureInfo)?
            .read_vec(|r| r.read())?;
        let planes = bsp_file
            .lump_reader(LumpType::Planes)?
            .read_vec(|r| r.read())?;
        let nodes = bsp_file
            .lump_reader(LumpType::Nodes)?
            .read_vec(|r| r.read())?;
        let leaves = bsp_file
            .lump_reader(LumpType::Leaves)?
            .read_vec(|r| r.read())?
            .into();
        let leaf_faces = bsp_file
            .lump_reader(LumpType::LeafFaces)?
            .read_vec(|r| r.read())?;
        let leaf_brushes = bsp_file
            .lump_reader(LumpType::LeafBrushes)?
            .read_vec(|r| r.read())?;
        let models = bsp_file
            .lump_reader(LumpType::Models)?
            .read_vec(|r| r.read())?;
        let brushes = bsp_file
            .lump_reader(LumpType::Brushes)?
            .read_vec(|r| r.read())?;
        let brush_sides = bsp_file
            .lump_reader(LumpType::BrushSides)?
            .read_vec(|r| r.read())?;
        let vertices = bsp_file
            .lump_reader(LumpType::Vertices)?
            .read_vec(|r| r.read())?;
        let edges = bsp_file
            .lump_reader(LumpType::Edges)?
            .read_vec(|r| r.read())?;
        let surface_edges = bsp_file
            .lump_reader(LumpType::SurfaceEdges)?
            .read_vec(|r| r.read())?;
        let faces = bsp_file
            .lump_reader(LumpType::Faces)?
            .read_vec(|r| r.read())?;
        let vis_data = bsp_file.lump_reader(LumpType::Visibility)?.read_visdata()?;

        Ok({
            Bsp {
                header: bsp_file.header().clone(),
                entities,
                textures_data,
                textures_info,
                planes,
                nodes,
                leaves,
                leaf_faces,
                leaf_brushes,
                models,
                brushes,
                brush_sides,
                vertices,
                edges,
                surface_edges,
                faces,
                vis_data,
            }
        })
    }

    pub fn leaf(&self, n: usize) -> Option<Handle<'_, Leaf>> {
        self.leaves.get(n).map(|leaf| Handle {
            bsp: self,
            data: leaf,
        })
    }

    pub fn plane(&self, n: usize) -> Option<Handle<'_, Plane>> {
        self.planes.get(n).map(|plane| Handle {
            bsp: self,
            data: plane,
        })
    }

    pub fn face(&self, n: usize) -> Option<Handle<'_, Face>> {
        self.faces.get(n).map(|face| Handle {
            bsp: self,
            data: face,
        })
    }

    pub fn node(&self, n: usize) -> Option<Handle<'_, Node>> {
        self.nodes.get(n).map(|node| Handle {
            bsp: self,
            data: node,
        })
    }

    pub fn root_node(&self) -> Option<Handle<'_, Node>> {
        self.node(0)
    }

    pub fn models(&self) -> impl Iterator<Item = Handle<'_, Model>> {
        self.models.iter().map(move |m| Handle::new(self, m))
    }

    pub fn leaf_at(&self, point: [f32; 3]) -> Option<Handle<'_, Leaf>> {
        let mut current = self.root_node()?;
        None

        // loop {
        //     let plane = current.plane()?;
        //     let dot: f32 = point
        //         .iter()
        //         .zip(plane.normal.iter())
        //         .map(|(a, b)| a * b)
        //         .sum();
        //
        //     let [front, back] = current.children;
        //
        //     let next = if dot < plane.dist { back } else { front };
        //
        //     if next < 0 {
        //         return self.leaf((!next) as usize);
        //     } else {
        //         current = self.node(next as usize)?;
        //     }
        // }
    }
}

impl<'a, T> Handle<'a, T> {
    pub fn new(bsp: &'a Bsp, data: &'a T) -> Self {
        Handle { bsp, data }
    }
}

impl<'a> Handle<'a, Model> {
    pub fn faces(&self) -> impl Iterator<Item = Handle<'a, Face>> {
        let start = self.face as usize;
        let end = start + self.num_faces as usize;
        let bsp = self.bsp;

        bsp.faces[start..end]
            .iter()
            .map(move |face| Handle::new(bsp, face))
    }
}

impl<'a> Handle<'a, TextureInfo> {
    pub fn texture(&self) -> Option<&TextureData> {
        self.bsp
            .textures_data
            .get(self.data.texture_data_index as usize)
    }
}

impl<'a> Handle<'a, Face> {
    pub fn texture(&self) -> Option<Handle<TextureInfo>> {
        self.bsp
            .textures_info
            .get(self.texture_info as usize)
            .map(|texture_info| Handle {
                bsp: self.bsp,
                data: texture_info,
            })
    }

    pub fn vertices(&'a self) -> impl Iterator<Item = &'a Vertex> + 'a {
        (self.data.first_edge..(self.data.first_edge + self.data.num_edges as i32))
            .flat_map(move |surface_edge| self.bsp.surface_edges.get(surface_edge as usize))
            .flat_map(move |surface_edge| {
                self.bsp
                    .edges
                    .get(surface_edge.edge_index())
                    .map(|edge| (edge, surface_edge.direction()))
            })
            .flat_map(|(edge, direction)| match direction {
                EdgeDirection::FirstToLast => once(edge.start_index).chain(once(edge.end_index)),
                EdgeDirection::LastToFirst => once(edge.end_index).chain(once(edge.start_index)),
            })
            .flat_map(move |vert_index| self.bsp.vertices.get(vert_index as usize))
    }
}

impl Handle<'_, Node> {
    pub fn plane(&self) -> Option<Handle<'_, Plane>> {
        self.bsp.plane(self.plane as _)
    }
}

impl<'a> Handle<'a, Leaf> {
    pub fn visible_set(&self) -> Option<impl Iterator<Item = Handle<'a, Leaf>>> {
        // TODO: Use `itertools::Either`?
        let cluster = self.cluster;
        let bsp = self.bsp;

        if cluster < 0 {
            None
        } else {
            Some(
                bsp.leaves
                    .iter()
                    .filter(move |leaf| {
                        if leaf.cluster == cluster {
                            true
                        } else if leaf.cluster > 0 {
                            let cluster_vis_start =
                                leaf.cluster as u64 * bsp.vis_data.sz_vecs as u64 * 8;
                            bsp.vis_data.vecs[cluster_vis_start + cluster as u64]
                        } else {
                            false
                        }
                    })
                    .map(move |leaf| Handle { bsp, data: leaf }),
            )
        }
    }

    pub fn faces(&self) -> impl Iterator<Item = Handle<'a, Face>> {
        let start = self.leaf_face as usize;
        let end = start + self.num_leaf_faces as usize;
        let bsp = self.bsp;
        bsp.leaf_faces[start..end]
            .iter()
            .filter_map(move |leaf_face| bsp.face(leaf_face.face as usize))
    }
}

#[cfg(test)]
mod tests {
    use super::Bsp;

    #[test]
    fn tf2_file() {
        use std::fs::read;

        let data = read("koth_bagel_rc2a.bsp").unwrap();

        Bsp::read(&data).unwrap();
    }
}
