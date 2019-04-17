#![cfg_attr(feature = "bench", feature(test))]

#[cfg(feature = "bench")]
extern crate test;

use arrayvec::ArrayString;
use bitflags::bitflags;
use bv::BitVec;
use byteorder::{LittleEndian, ReadBytesExt};
use itertools::{GroupBy, Itertools};
use std::{
    borrow::Cow,
    convert::{TryFrom, TryInto},
    fmt,
    io::{self, Error, ErrorKind, Read, Seek, SeekFrom, Take},
    ops::Deref,
};

trait ElementSize {
    const SIZE: usize;
}

macro_rules! elsize {
    ($(#[$($any:tt)*])* $v:vis struct $name:ident { $($v0:vis $fname:ident : $fty:ty,)* }) => {
        impl ElementSize for $name {
            const SIZE: usize = {
                use std::mem;

                let mut a = 0;

                $(
                    a += mem::size_of::<$fty>();
                )*

                a
            };
        }

        $(#[$($any)*])* $v struct $name {
            $($v0 $fname : $fty,)*
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Default)]
struct Directories {
    entities: DirEntry,
    textures: DirEntry,
    planes: DirEntry,
    nodes: DirEntry,
    leaves: DirEntry,
    leaf_faces: DirEntry,
    leaf_brushes: DirEntry,
    models: DirEntry,
    brushes: DirEntry,
    brush_sides: DirEntry,
    vertices: DirEntry,
    mesh_verts: DirEntry,
    effects: DirEntry,
    faces: DirEntry,
    lightmaps: DirEntry,
    lightvols: DirEntry,
    visdata: DirEntry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub i: u8,
    pub b: u8,
    pub s: u8,
    pub p: u8,
}

#[derive(Clone, Debug, Default)]
struct DirEntry {
    offset: u32,
    length: u32,
}

elsize! {
    #[derive(Debug, Clone)]
    pub struct LeafFace {
        pub face: u32,
    }
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
    pub struct SurfaceFlags: u32 {
        const NODAMAGE    = 0b0000_0000_0000_0000_0001; // Never give falling damage
        const SLICK       = 0b0000_0000_0000_0000_0010; // Affects game physics
        const SKY         = 0b0000_0000_0000_0000_0100; // Lighting from environment map
        const LADDER      = 0b0000_0000_0000_0000_1000; // Climbable ladder
        const NOIMPACT    = 0b0000_0000_0000_0001_0000; // Don't make missile explosions
        const NOMARKS     = 0b0000_0000_0000_0010_0000; // Don't leave missile marks
        const FLESH       = 0b0000_0000_0000_0100_0000; // Make flesh sounds and effects
        const NODRAW      = 0b0000_0000_0000_1000_0000; // Don't generate a drawsurface at all
        const HINT        = 0b0000_0000_0001_0000_0000; // Make a primary bsp splitter
        const SKIP        = 0b0000_0000_0010_0000_0000; // Completely ignore, allowing non-closed brushes
        const NOLIGHTMAP  = 0b0000_0000_0100_0000_0000; // Surface doesn't need a lightmap
        const POINTLIGHT  = 0b0000_0000_1000_0000_0000; // Generate lighting info at vertices
        const METALSTEPS  = 0b0000_0001_0000_0000_0000; // Clanking footsteps
        const NOSTEPS     = 0b0000_0010_0000_0000_0000; // No footstep sounds
        const NONSOLID    = 0b0000_0100_0000_0000_0000; // Don't collide against curves with this set
        const LIGHTFILTER = 0b0000_1000_0000_0000_0000; // Act as a light filter during q3map -light
        const ALPHASHADOW = 0b0001_0000_0000_0000_0000; // Do per-pixel light shadow casting in q3map
        const NODLIGHT    = 0b0010_0000_0000_0000_0000; // Never add dynamic lights
    }
}

impl SurfaceFlags {
    pub fn should_draw(&self) -> bool {
        !self.intersects(Self::HINT | Self::SKIP | Self::NODRAW | Self::LIGHTFILTER)
    }
}

bitflags! {
    pub struct ContentFlags: u32 {
        // An eye is never valid in a solid
        const SOLID          = 0b0000_0000_0000_0000_0000_0000_0000_0001;
        const LAVA           = 0b0000_0000_0000_0000_0000_0000_0000_1000;
        const SLIME          = 0b0000_0000_0000_0000_0000_0000_0001_0000;
        const WATER          = 0b0000_0000_0000_0000_0000_0000_0010_0000;
        const FOG            = 0b0000_0000_0000_0000_0000_0000_0100_0000;
        const NOTTEAM1       = 0b0000_0000_0000_0000_0000_0000_1000_0000;
        const NOTTEAM2       = 0b0000_0000_0000_0000_0000_0001_0000_0000;
        const NOBOTCLIP      = 0b0000_0000_0000_0000_0000_0010_0000_0000;

        const AREAPORTAL     = 0b0000_0000_0000_0000_1000_0000_0000_0000;

        const PLAYERCLIP     = 0b0000_0000_0000_0001_0000_0000_0000_0000;
        const MONSTERCLIP    = 0b0000_0000_0000_0010_0000_0000_0000_0000;

        // Bot-specific contents types
        const TELEPORTER     = 0b0000_0000_0000_0100_0000_0000_0000_0000;
        const JUMPPAD        = 0b0000_0000_0000_1000_0000_0000_0000_0000;
        const CLUSTERPORTAL  = 0b0000_0000_0001_0000_0000_0000_0000_0000;
        const DONOTENTER     = 0b0000_0000_0010_0000_0000_0000_0000_0000;
        const BOTCLIP        = 0b0000_0000_0100_0000_0000_0000_0000_0000;
        const MOVER          = 0b0000_0000_1000_0000_0000_0000_0000_0000;

        // Removed before bsping an entity
        const ORIGIN         = 0b0000_0001_0000_0000_0000_0000_0000_0000;

        // Should never be on a brush, only in game
        const BODY           = 0b0000_0010_0000_0000_0000_0000_0000_0000;
        const CORPSE         = 0b0000_0100_0000_0000_0000_0000_0000_0000;
        // Brushes not used for the bsp
        const DETAIL         = 0b0000_1000_0000_0000_0000_0000_0000_0000;
        // Brushes used for the bsp
        const STRUCTURAL     = 0b0001_0000_0000_0000_0000_0000_0000_0000;
        // Don't consume surface fragments inside
        const TRANSLUCENT    = 0b0010_0000_0000_0000_0000_0000_0000_0000;
        const TRIGGER        = 0b0100_0000_0000_0000_0000_0000_0000_0000;
        // Don't leave bodies or items (death fog, lava)
        const NODROP         = 0b1000_0000_0000_0000_0000_0000_0000_0000;
    }
}

#[derive(Debug, Clone)]
pub struct Texture {
    pub name: ArrayString<[u8; 64]>,
    pub flags: SurfaceFlags,
    pub contents: ContentFlags,
}

impl ElementSize for Texture {
    const SIZE: usize = std::mem::size_of::<u32>() * 2 + std::mem::size_of::<u8>() * 64;
}

elsize! {
    #[derive(Debug, Clone)]
    pub struct Plane {
        pub normal: [f32; 3],
        pub dist: f32,
    }
}

elsize! {
    #[derive(Debug, Clone)]
    pub struct Node {
        pub plane: u32,
        pub children: [i32; 2],
        pub mins: [i32; 3],
        pub maxs: [i32; 3],
    }
}

elsize! {
    #[derive(Debug, Clone)]
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
}

elsize! {
    #[derive(Debug, Clone)]
    pub struct LeafBrush {
        pub brush: u32,
    }
}

elsize! {
    #[derive(Debug, Clone)]
    pub struct Model {
        pub mins: [f32; 3],
        pub maxs: [f32; 3],
        pub face: u32,
        pub num_faces: u32,
        pub brush: u32,
        pub num_brushes: u32,
    }
}

elsize! {
    #[derive(Debug, Clone)]
    pub struct Brush {
        pub brush_side: u32,
        pub num_brush_sides: u32,
        pub texture: u32,
    }
}

elsize! {
    #[derive(Debug, Clone)]
    pub struct BrushSide {
        pub plane: u32,
        pub texture: u32,
    }
}

elsize! {
    #[derive(Debug, Clone)]
    pub struct Vertex {
        pub position: [f32; 3],
        pub surface_texcoord: [f32; 2],
        pub lightmap_texcoord: [f32; 2],
        pub normal: [f32; 3],
        pub color: [u8; 4],
    }
}

elsize! {
    #[derive(Debug, Clone)]
    pub struct MeshVert {
        pub offset: u32,
    }
}

#[derive(Debug, Clone)]
pub struct Effect {
    pub name: ArrayString<[u8; 64]>,
    pub brush: u32,
    pub unknown: u32,
}

impl ElementSize for Effect {
    const SIZE: usize = std::mem::size_of::<u32>() * 2 + std::mem::size_of::<u8>() * 64;
}

elsize! {
    #[derive(Debug, Clone)]
    pub struct Face {
        pub texture: u32,
        pub effect: u32,
        pub face_type: FaceType,
        pub vertex: u32,
        pub num_vertexes: u32,
        pub mesh_vert: u32,
        pub num_mesh_verts: u32,
        pub lm_index: u32,
        pub lm_start: [u32; 2],
        pub lm_size: [u32; 2],
        pub lm_origin: [f32; 3],
        pub lm_vecs: [[f32; 3]; 2],
        pub normal: [f32; 3],
        pub size: [u32; 2],
    }
}

const LIGHTMAP_SIZE: usize = 128;

elsize! {
    #[derive(Clone)]
    pub struct Lightmap {
        map: [[[u8; 3]; LIGHTMAP_SIZE]; LIGHTMAP_SIZE],
    }
}

impl fmt::Debug for Lightmap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        struct Lightmap {
            map: Vec<Vec<[u8; 3]>>,
        }

        Lightmap {
            map: self.map.iter().map(|a| a.to_vec()).collect::<Vec<_>>(),
        }
        .fmt(f)
    }
}

elsize! {
    #[derive(Debug, Clone)]
    pub struct Lightvol {
        ambient: [u8; 3],
        directional: [u8; 3],
        dir: [u8; 2],
    }
}

#[derive(Default, Debug, Clone)]
pub struct VisData {
    pub n_vecs: u32,      // Number of vectors.
    pub sz_vecs: u32,     // Size of each vector, in bytes.
    pub vecs: BitVec<u8>, // Visibility data. One bit per cluster per vector.
}

struct BspReader<R> {
    inner: R,
}

impl<R: Read + Seek> BspReader<R> {
    fn read_entities(&mut self, dir_entry: &DirEntry) -> io::Result<Entities> {
        let mut entities = Vec::with_capacity(dir_entry.length as usize);
        self.inner.seek(SeekFrom::Start(dir_entry.offset as u64))?;
        self.inner
            .by_ref()
            .take(dir_entry.length as u64)
            .read_to_end(&mut entities)?;
        let entities =
            String::from_utf8(entities).map_err(|err| Error::new(ErrorKind::InvalidData, err))?;
        Ok(Entities { entities })
    }

    fn read_entry<F, T>(&mut self, dir_entry: &DirEntry, mut f: F) -> io::Result<Vec<T>>
    where
        F: FnMut(&mut BspReader<Take<&mut R>>) -> io::Result<T>,
        T: ElementSize,
    {
        if dir_entry.length % T::SIZE as u32 != 0 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Directory entry length isn't a multiple of element size (length: {}, element size: {})", dir_entry.length, T::SIZE),
            ));
        }

        let num_entries = dir_entry.length as usize / T::SIZE;
        let mut entries = Vec::with_capacity(num_entries);
        self.inner.seek(SeekFrom::Start(dir_entry.offset as u64))?;
        let mut reader = BspReader {
            inner: self.inner.by_ref().take(dir_entry.length as u64),
        };

        for _ in 0..num_entries {
            entries.push(f(&mut reader)?);
        }

        Ok(entries)
    }
}

impl<R: Read> BspReader<R> {
    fn read_header(&mut self) -> io::Result<Header> {
        let i = self.inner.read_u8()?;
        let b = self.inner.read_u8()?;
        let s = self.inner.read_u8()?;
        let p = self.inner.read_u8()?;
        Ok(Header { i, b, s, p })
    }

    fn read_version(&mut self) -> io::Result<u32> {
        self.inner.read_u32::<LittleEndian>()
    }

    fn read_directories(&mut self) -> io::Result<Directories> {
        macro_rules! read_dirs {
            (@inner $out:expr,) => {
                $out
            };
            (@inner $out:expr, $name:ident $(, $rest:ident)*) => {{
                let mut out = $out;
                out.$name = {
                    let offset = self.inner.read_u32::<LittleEndian>()?;
                    let length = self.inner.read_u32::<LittleEndian>()?;
                    DirEntry {
                        offset,
                        length,
                    }
                };
                read_dirs!(@inner out, $($rest),*)
            }};
            ($($any:tt)*) => {{
                read_dirs!(@inner Directories::default(), $($any)*)
            }}
        }

        Ok(read_dirs!(
            entities,
            textures,
            planes,
            nodes,
            leaves,
            leaf_faces,
            leaf_brushes,
            models,
            brushes,
            brush_sides,
            vertices,
            mesh_verts,
            effects,
            faces,
            lightmaps,
            lightvols,
            visdata
        ))
    }

    fn read_texture(&mut self) -> io::Result<Texture> {
        let name = self.read_name()?;
        let flags =
            SurfaceFlags::from_bits(self.inner.read_u32::<LittleEndian>()?).ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidData,
                    format!("Invalid surface flag in texture: {}", name),
                )
            })?;
        let contents =
            ContentFlags::from_bits(self.inner.read_u32::<LittleEndian>()?).ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidData,
                    format!("Invalid content flag in texture: {}", name),
                )
            })?;
        Ok(Texture {
            name,
            flags,
            contents,
        })
    }

    fn read_plane(&mut self) -> io::Result<Plane> {
        let x = self.inner.read_f32::<LittleEndian>()?;
        let y = self.inner.read_f32::<LittleEndian>()?;
        let z = self.inner.read_f32::<LittleEndian>()?;
        let dist = self.inner.read_f32::<LittleEndian>()?;
        let plane = Plane {
            normal: [x, y, z],
            dist,
        };
        Ok(plane)
    }

    fn read_node(&mut self) -> io::Result<Node> {
        let plane = self.inner.read_u32::<LittleEndian>()?;
        let children = [
            self.inner.read_i32::<LittleEndian>()?,
            self.inner.read_i32::<LittleEndian>()?,
        ];
        let mins = [
            self.inner.read_i32::<LittleEndian>()?,
            self.inner.read_i32::<LittleEndian>()?,
            self.inner.read_i32::<LittleEndian>()?,
        ];
        let maxs = [
            self.inner.read_i32::<LittleEndian>()?,
            self.inner.read_i32::<LittleEndian>()?,
            self.inner.read_i32::<LittleEndian>()?,
        ];
        let node = Node {
            plane,
            children,
            mins,
            maxs,
        };
        Ok(node)
    }

    fn read_leaf(&mut self) -> io::Result<Leaf> {
        let cluster = self.inner.read_i32::<LittleEndian>()?;
        let area = self.inner.read_u32::<LittleEndian>()?;
        let mins = [
            self.inner.read_i32::<LittleEndian>()?,
            self.inner.read_i32::<LittleEndian>()?,
            self.inner.read_i32::<LittleEndian>()?,
        ];
        let maxs = [
            self.inner.read_i32::<LittleEndian>()?,
            self.inner.read_i32::<LittleEndian>()?,
            self.inner.read_i32::<LittleEndian>()?,
        ];
        let leaf_face = self.inner.read_u32::<LittleEndian>()?;
        let num_leaf_faces = self.inner.read_u32::<LittleEndian>()?;
        let leaf_brush = self.inner.read_u32::<LittleEndian>()?;
        let num_leaf_brushes = self.inner.read_u32::<LittleEndian>()?;
        let leaf = Leaf {
            cluster,
            area,
            mins,
            maxs,
            leaf_face,
            num_leaf_faces,
            leaf_brush,
            num_leaf_brushes,
        };
        Ok(leaf)
    }

    fn read_leaf_face(&mut self) -> io::Result<LeafFace> {
        let face = self.inner.read_u32::<LittleEndian>()?;
        let leaf_face = LeafFace { face };
        Ok(leaf_face)
    }

    fn read_leaf_brush(&mut self) -> io::Result<LeafBrush> {
        let brush = self.inner.read_u32::<LittleEndian>()?;
        let leaf_brush = LeafBrush { brush };
        Ok(leaf_brush)
    }

    fn read_model(&mut self) -> io::Result<Model> {
        let mins = [
            self.inner.read_f32::<LittleEndian>()?,
            self.inner.read_f32::<LittleEndian>()?,
            self.inner.read_f32::<LittleEndian>()?,
        ];
        let maxs = [
            self.inner.read_f32::<LittleEndian>()?,
            self.inner.read_f32::<LittleEndian>()?,
            self.inner.read_f32::<LittleEndian>()?,
        ];
        let face = self.inner.read_u32::<LittleEndian>()?;
        let num_faces = self.inner.read_u32::<LittleEndian>()?;
        let brush = self.inner.read_u32::<LittleEndian>()?;
        let num_brushes = self.inner.read_u32::<LittleEndian>()?;
        let model = Model {
            mins,
            maxs,
            face,
            num_faces,
            brush,
            num_brushes,
        };
        Ok(model)
    }

    fn read_brush(&mut self) -> io::Result<Brush> {
        let brush_side = self.inner.read_u32::<LittleEndian>()?;
        let num_brush_sides = self.inner.read_u32::<LittleEndian>()?;
        let texture = self.inner.read_u32::<LittleEndian>()?;
        let brush = Brush {
            brush_side,
            num_brush_sides,
            texture,
        };
        Ok(brush)
    }

    fn read_brush_side(&mut self) -> io::Result<BrushSide> {
        let plane = self.inner.read_u32::<LittleEndian>()?;
        let texture = self.inner.read_u32::<LittleEndian>()?;
        let brush_side = BrushSide { plane, texture };
        Ok(brush_side)
    }

    fn read_vertex(&mut self) -> io::Result<Vertex> {
        let position = [
            self.inner.read_f32::<LittleEndian>()?,
            self.inner.read_f32::<LittleEndian>()?,
            self.inner.read_f32::<LittleEndian>()?,
        ];
        let surface_texcoord = [
            self.inner.read_f32::<LittleEndian>()?,
            self.inner.read_f32::<LittleEndian>()?,
        ];
        let lightmap_texcoord = [
            self.inner.read_f32::<LittleEndian>()?,
            self.inner.read_f32::<LittleEndian>()?,
        ];
        let normal = [
            self.inner.read_f32::<LittleEndian>()?,
            self.inner.read_f32::<LittleEndian>()?,
            self.inner.read_f32::<LittleEndian>()?,
        ];
        let color = [
            self.inner.read_u8()?,
            self.inner.read_u8()?,
            self.inner.read_u8()?,
            self.inner.read_u8()?,
        ];
        let vertex = Vertex {
            position,
            surface_texcoord,
            lightmap_texcoord,
            normal,
            color,
        };
        Ok(vertex)
    }

    fn read_mesh_vert(&mut self) -> io::Result<MeshVert> {
        let offset = self.inner.read_u32::<LittleEndian>()?;
        let mesh_vert = MeshVert { offset };
        Ok(mesh_vert)
    }

    fn read_name(&mut self) -> io::Result<ArrayString<[u8; 64]>> {
        use std::str;

        let mut name_buf = [0u8; 64];
        self.inner.read_exact(&mut name_buf)?;
        let zero_pos = name_buf.iter().position(|c| *c == 0).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Name isn't null-terminated"),
            )
        })?;
        let name = &name_buf[..zero_pos];
        Ok(ArrayString::from(
            str::from_utf8(name).map_err(|err| Error::new(ErrorKind::InvalidData, err))?,
        )
        .expect("Programmer error: it should be impossible for the string to exceed the capacity"))
    }

    fn read_effect(&mut self) -> io::Result<Effect> {
        let name = self.read_name()?;
        let brush = self.inner.read_u32::<LittleEndian>()?;
        let unknown = self.inner.read_u32::<LittleEndian>()?;
        Ok(Effect {
            name,
            brush,
            unknown,
        })
    }

    fn read_face(&mut self) -> io::Result<Face> {
        let texture = self.inner.read_u32::<LittleEndian>()?;
        let effect = self.inner.read_u32::<LittleEndian>()?;
        let face_type = self
            .inner
            .read_u32::<LittleEndian>()?
            .try_into()
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        let vertex = self.inner.read_u32::<LittleEndian>()?;
        let num_vertexes = self.inner.read_u32::<LittleEndian>()?;
        let mesh_vert = self.inner.read_u32::<LittleEndian>()?;
        let num_mesh_verts = self.inner.read_u32::<LittleEndian>()?;
        let lm_index = self.inner.read_u32::<LittleEndian>()?;
        let lm_start = [
            self.inner.read_u32::<LittleEndian>()?,
            self.inner.read_u32::<LittleEndian>()?,
        ];
        let lm_size = [
            self.inner.read_u32::<LittleEndian>()?,
            self.inner.read_u32::<LittleEndian>()?,
        ];
        let lm_origin = [
            self.inner.read_f32::<LittleEndian>()?,
            self.inner.read_f32::<LittleEndian>()?,
            self.inner.read_f32::<LittleEndian>()?,
        ];
        let lm_vec1 = [
            self.inner.read_f32::<LittleEndian>()?,
            self.inner.read_f32::<LittleEndian>()?,
            self.inner.read_f32::<LittleEndian>()?,
        ];
        let lm_vec2 = [
            self.inner.read_f32::<LittleEndian>()?,
            self.inner.read_f32::<LittleEndian>()?,
            self.inner.read_f32::<LittleEndian>()?,
        ];
        let lm_vecs = [lm_vec1, lm_vec2];
        let normal = [
            self.inner.read_f32::<LittleEndian>()?,
            self.inner.read_f32::<LittleEndian>()?,
            self.inner.read_f32::<LittleEndian>()?,
        ];
        let size = [
            self.inner.read_u32::<LittleEndian>()?,
            self.inner.read_u32::<LittleEndian>()?,
        ];
        let face = Face {
            texture,
            effect,
            face_type,
            vertex,
            num_vertexes,
            mesh_vert,
            num_mesh_verts,
            lm_index,
            lm_start,
            lm_size,
            lm_origin,
            lm_vecs,
            normal,
            size,
        };
        Ok(face)
    }

    fn read_lightmap(&mut self) -> io::Result<Lightmap> {
        use std::ptr;

        const NUM_BYTES: usize = LIGHTMAP_SIZE * LIGHTMAP_SIZE * 3;

        let mut vec = Vec::with_capacity(NUM_BYTES);
        self.inner
            .by_ref()
            .take(NUM_BYTES as u64)
            .read_to_end(&mut vec)?;

        if vec.len() != NUM_BYTES {
            return Err(ErrorKind::UnexpectedEof.into());
        }

        let ptr = vec[..].as_ptr();

        let ptr = ptr as *const [[[u8; 3]; LIGHTMAP_SIZE]; LIGHTMAP_SIZE];

        Ok(Lightmap {
            map: unsafe { ptr::read(ptr) },
        })
    }

    fn read_lightvol(&mut self) -> io::Result<Lightvol> {
        let ambient = [
            self.inner.read_u8()?,
            self.inner.read_u8()?,
            self.inner.read_u8()?,
        ];
        let directional = [
            self.inner.read_u8()?,
            self.inner.read_u8()?,
            self.inner.read_u8()?,
        ];
        let dir = [self.inner.read_u8()?, self.inner.read_u8()?];
        Ok(Lightvol {
            ambient,
            directional,
            dir,
        })
    }

    fn read_visdata(&mut self, entry: &DirEntry) -> io::Result<VisData> {
        if (entry.length as usize) < std::mem::size_of::<u32>() * 2 {
            return Ok(VisData::default());
        }

        let n_vecs = self.inner.read_u32::<LittleEndian>()?;
        let sz_vecs = self.inner.read_u32::<LittleEndian>()?;
        let vecs_size = n_vecs as u64 * sz_vecs as u64;
        let mut vecs = Vec::with_capacity(
            vecs_size
                .try_into()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        );
        self.inner
            .by_ref()
            .take(vecs_size as u64)
            .read_to_end(&mut vecs)?;

        if (vecs.len() as u64) < vecs_size {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Unexpected EOF while reading VisData"),
            ));
        }

        if (vecs.len() as u64) > vecs_size {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Extra data at end of file"),
            ));
        }

        let vecs = BitVec::from_bits(vecs);

        let vis_data = VisData {
            n_vecs,
            sz_vecs,
            vecs,
        };
        Ok(vis_data)
    }
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
    pub textures: Vec<Texture>,
    pub planes: Vec<Plane>,
    pub nodes: Vec<Node>,
    pub leaves: Leaves,
    pub leaf_faces: Vec<LeafFace>,
    pub leaf_brushes: Vec<LeafBrush>,
    pub models: Vec<Model>,
    pub brushes: Vec<Brush>,
    pub brush_sides: Vec<BrushSide>,
    pub vertices: Vec<Vertex>,
    pub mesh_verts: Vec<MeshVert>,
    pub effects: Vec<Effect>,
    pub faces: Vec<Face>,
    pub lightmaps: Vec<Lightmap>,
    pub lightvols: Vec<Lightvol>,
    pub vis_data: VisData,
}

impl Bsp {
    pub fn read<R: Read + Seek>(reader: R) -> io::Result<Self> {
        const EXPECTED_HEADER: Header = Header {
            i: b'I',
            b: b'B',
            s: b'S',
            p: b'P',
        };
        // TODO: Use this to decide on the version to parse it as
        const EXPECTED_VERSION: u32 = 0x2e;

        let mut reader = BspReader { inner: reader };
        let header = reader.read_header()?;
        let version = reader.read_version()?;

        if header != EXPECTED_HEADER || version != EXPECTED_VERSION {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Invalid header or version (expected {:?}, got {:?})",
                    (EXPECTED_HEADER, EXPECTED_VERSION),
                    (header, version)
                ),
            ));
        }

        let dir_entries = reader.read_directories()?;
        let entities = reader.read_entities(&dir_entries.entities)?;
        let textures = reader.read_entry(&dir_entries.textures, |r| r.read_texture())?;
        let planes = reader.read_entry(&dir_entries.planes, |r| r.read_plane())?;
        let nodes = reader.read_entry(&dir_entries.nodes, |r| r.read_node())?;
        let leaves = reader
            .read_entry(&dir_entries.leaves, |r| r.read_leaf())?
            .into();
        let leaf_faces = reader.read_entry(&dir_entries.leaf_faces, |r| r.read_leaf_face())?;
        let leaf_brushes = reader.read_entry(&dir_entries.leaf_brushes, |r| r.read_leaf_brush())?;
        let models = reader.read_entry(&dir_entries.models, |r| r.read_model())?;
        let brushes = reader.read_entry(&dir_entries.brushes, |r| r.read_brush())?;
        let brush_sides = reader.read_entry(&dir_entries.brush_sides, |r| r.read_brush_side())?;
        let vertices = reader.read_entry(&dir_entries.vertices, |r| r.read_vertex())?;
        let mesh_verts = reader.read_entry(&dir_entries.mesh_verts, |r| r.read_mesh_vert())?;
        let effects = reader.read_entry(&dir_entries.effects, |r| r.read_effect())?;
        let faces = reader.read_entry(&dir_entries.faces, |r| r.read_face())?;
        let lightmaps = reader.read_entry(&dir_entries.lightmaps, |r| r.read_lightmap())?;
        let lightvols = reader.read_entry(&dir_entries.lightvols, |r| r.read_lightvol())?;

        reader
            .inner
            .seek(SeekFrom::Start(dir_entries.visdata.offset as u64))?;
        let vis_data = reader.read_visdata(&dir_entries.visdata)?;

        Ok({
            Bsp {
                header,
                entities,
                textures,
                planes,
                nodes,
                leaves,
                leaf_faces,
                leaf_brushes,
                models,
                brushes,
                brush_sides,
                vertices,
                mesh_verts,
                effects,
                faces,
                lightmaps,
                lightvols,
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

    pub fn texture(&self, n: usize) -> Option<&Texture> {
        self.textures.get(n)
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

        loop {
            let plane = current.plane()?;
            let dot: f32 = point
                .iter()
                .zip(plane.normal.iter())
                .map(|(a, b)| a * b)
                .sum();

            let [front, back] = current.children;

            let next = if dot < plane.dist { back } else { front };

            if next < 0 {
                return self.leaf((!next) as usize);
            } else {
                current = self.node(next as usize)?;
            }
        }
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

impl<'a> Handle<'a, Face> {
    pub fn texture(&self) -> Option<&Texture> {
        self.bsp.texture(self.texture as _)
    }

    pub fn vertices(&self) -> impl Iterator<Item = Cow<'a, Vertex>> {
        use itertools::Either;

        match self.face_type {
            FaceType::Polygon | FaceType::Mesh => {
                let start = self.mesh_vert as usize;
                let end = start + self.num_mesh_verts as usize;
                let bsp = self.bsp;
                let vertex = self.vertex;

                Either::Left(
                    bsp.mesh_verts[start..end]
                        .iter()
                        .map(move |mv| Cow::Borrowed(&bsp.vertices[(mv.offset + vertex) as usize])),
                )
            }
            // TODO
            _ => Either::Right(std::iter::empty()),
        }
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
    fn random_file() {
        use std::fs::File;

        Bsp::read(&mut File::open("test.bsp").expect("Cannot open file")).unwrap();
    }
}

#[cfg(feature = "bench")]
mod benches {
    use super::Bsp;
    use test::Bencher;

    const MAP_BYTES: &[u8] = include_bytes!("../test.bsp");

    #[bench]
    fn from_bytes(b: &mut Bencher) {
        use std::io::Cursor;

        b.iter(|| {
            Bsp::read(&mut Cursor::new(MAP_BYTES)).unwrap();
        });
    }

    #[bench]
    fn leaf_at(b: &mut Bencher) {
        use std::io::Cursor;

        let bsp = Bsp::read(&mut Cursor::new(MAP_BYTES)).unwrap();

        b.iter(|| {
            test::black_box(bsp.leaf_at(test::black_box([0., 0., 0.])));
        });
    }
}
