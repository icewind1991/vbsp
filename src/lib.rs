use arrayvec::ArrayString;
use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    fmt,
    io::{Error, ErrorKind, Read, Result, Seek, SeekFrom, Take},
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

#[derive(Debug, Default)]
pub struct Directories {
    pub entities: DirEntry,
    pub textures: DirEntry,
    pub planes: DirEntry,
    pub nodes: DirEntry,
    pub leafs: DirEntry,
    pub leaf_faces: DirEntry,
    pub leaf_brushes: DirEntry,
    pub models: DirEntry,
    pub brushes: DirEntry,
    pub brush_sides: DirEntry,
    pub vertexes: DirEntry,
    pub mesh_verts: DirEntry,
    pub effects: DirEntry,
    pub faces: DirEntry,
    pub lightmaps: DirEntry,
    pub lightvols: DirEntry,
    pub visdata: DirEntry,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    pub i: u8,
    pub b: u8,
    pub s: u8,
    pub p: u8,
}

#[derive(Debug, Default)]
pub struct DirEntry {
    pub offset: i32,
    pub length: i32,
}

elsize! {
    #[derive(Debug)]
    pub struct LeafFace {
        pub face: i32,
    }
}

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

#[derive(Debug)]
pub struct Texture {
    pub name: ArrayString<[u8; 64]>,
    pub flags: i32,
    pub contents: i32,
}

impl ElementSize for Texture {
    const SIZE: usize = std::mem::size_of::<i32>() * 2 + std::mem::size_of::<u8>() * 64;
}

elsize! {
    #[derive(Debug)]
    pub struct Plane {
        pub normal: [f32; 3],
        pub dist: f32,
    }
}

elsize! {
    #[derive(Debug)]
    pub struct Node {
        pub plane: i32,
        pub children: [i32; 2],
        pub mins: [i32; 3],
        pub maxs: [i32; 3],
    }
}

elsize! {
    #[derive(Debug)]
    pub struct Leaf {
        pub cluster: i32,
        pub area: i32,
        pub mins: [i32; 3],
        pub maxs: [i32; 3],
        pub leaf_face: i32,
        pub num_leaf_faces: i32,
        pub leaf_brush: i32,
        pub num_leaf_brushes: i32,
    }
}

elsize! {
    #[derive(Debug)]
    pub struct LeafBrush {
        pub brush: i32,
    }
}

elsize! {
    #[derive(Debug)]
    pub struct Model {
        pub mins: [f32; 3],
        pub maxs: [f32; 3],
        pub face: i32,
        pub num_faces: i32,
        pub brush: i32,
        pub num_brushes: i32,
    }
}

elsize! {
    #[derive(Debug)]
    pub struct Brush {
        pub brush_side: i32,
        pub num_brush_sides: i32,
        pub texture: i32,
    }
}

elsize! {
    #[derive(Debug)]
    pub struct BrushSide {
        pub plane: i32,
        pub texture: i32,
    }
}

elsize! {
    #[derive(Debug)]
    pub struct Vertex {
        pub position: [f32; 3],
        pub tex_coord1: [f32; 2],
        pub tex_coord2: [f32; 2],
        pub normal: [f32; 3],
        pub color: [u8; 4],
    }
}

elsize! {
    #[derive(Debug)]
    pub struct MeshVert {
        pub offset: i32,
    }
}

#[derive(Debug)]
pub struct Effect {
    pub name: ArrayString<[u8; 64]>,
    pub brush: i32,
    pub unknown: i32,
}

impl ElementSize for Effect {
    const SIZE: usize = std::mem::size_of::<i32>() * 2 + std::mem::size_of::<u8>() * 64;
}

elsize! {
    #[derive(Debug)]
    pub struct Face {
        pub texture: i32,
        pub effect: i32,
        pub face_type: i32,
        pub vertex: i32,
        pub num_vertexes: i32,
        pub mesh_vert: i32,
        pub num_mesh_verts: i32,
        pub lm_index: i32,
        pub lm_start: [i32; 2],
        pub lm_size: [i32; 2],
        pub lm_origin: [f32; 3],
        pub lm_vecs: [[f32; 3]; 2],
        pub normal: [f32; 3],
        pub size: [i32; 2],
    }
}

const LIGHTMAP_SIZE: usize = 128;

elsize! {
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
    #[derive(Debug)]
    pub struct Lightvol {
        ambient: [u8; 3],
        directional: [u8; 3],
        dir: [u8; 2],
    }
}

#[derive(Debug)]
pub struct VisData {
    pub n_vecs: i32,   // Number of vectors.
    pub sz_vecs: i32,  // Size of each vector, in bytes.
    pub vecs: Vec<u8>, // Visibility data. One bit per cluster per vector.
}

struct BspReader<R> {
    inner: R,
}

impl<R: Read + Seek> BspReader<R> {
    fn read_entities(&mut self, dir_entry: &DirEntry) -> Result<Entities> {
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

    fn read_entry<F, T>(&mut self, dir_entry: &DirEntry, mut f: F) -> Result<Vec<T>>
    where
        F: FnMut(&mut BspReader<Take<&mut R>>) -> Result<T>,
        T: ElementSize,
    {
        if dir_entry.length % T::SIZE as i32 != 0 {
            return Err(ErrorKind::InvalidData.into());
        }

        let mut entries = Vec::with_capacity(dir_entry.length as usize / T::SIZE);
        self.inner.seek(SeekFrom::Start(dir_entry.offset as u64))?;
        let mut reader = BspReader {
            inner: self.inner.by_ref().take(dir_entry.length as u64),
        };
        loop {
            match f(&mut reader) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    if e.kind() != ErrorKind::UnexpectedEof || reader.inner.bytes().next().is_some()
                    {
                        return Err(e);
                    } else {
                        break;
                    }
                }
            }
        }
        entries.shrink_to_fit();
        Ok(entries)
    }
}

impl<R: Read> BspReader<R> {
    fn read_header(&mut self) -> Result<Header> {
        let i = self.inner.read_u8()?;
        let b = self.inner.read_u8()?;
        let s = self.inner.read_u8()?;
        let p = self.inner.read_u8()?;
        Ok(Header { i, b, s, p })
    }

    fn read_version(&mut self) -> Result<i32> {
        self.inner.read_i32::<LittleEndian>()
    }

    fn read_directories(&mut self) -> Result<Directories> {
        macro_rules! read_dirs {
            (@inner $out:expr,) => {
                $out
            };
            (@inner $out:expr, $name:ident $(, $rest:ident)*) => {{
                let mut out = $out;
                out.$name = {
                    let offset = self.inner.read_i32::<LittleEndian>()?;
                    let length = self.inner.read_i32::<LittleEndian>()?;
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
            leafs,
            leaf_faces,
            leaf_brushes,
            models,
            brushes,
            brush_sides,
            vertexes,
            mesh_verts,
            effects,
            faces,
            lightmaps,
            lightvols,
            visdata
        ))
    }

    fn read_texture(&mut self) -> Result<Texture> {
        let name = self.read_name()?;
        let flags = self.inner.read_i32::<LittleEndian>()?;
        let contents = self.inner.read_i32::<LittleEndian>()?;
        Ok(Texture {
            name,
            flags,
            contents,
        })
    }

    fn read_plane(&mut self) -> Result<Plane> {
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

    fn read_node(&mut self) -> Result<Node> {
        let plane = self.inner.read_i32::<LittleEndian>()?;
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

    fn read_leaf(&mut self) -> Result<Leaf> {
        let cluster = self.inner.read_i32::<LittleEndian>()?;
        let area = self.inner.read_i32::<LittleEndian>()?;
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
        let leaf_face = self.inner.read_i32::<LittleEndian>()?;
        let num_leaf_faces = self.inner.read_i32::<LittleEndian>()?;
        let leaf_brush = self.inner.read_i32::<LittleEndian>()?;
        let num_leaf_brushes = self.inner.read_i32::<LittleEndian>()?;
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

    fn read_leaf_face(&mut self) -> Result<LeafFace> {
        let face = self.inner.read_i32::<LittleEndian>()?;
        let leaf_face = LeafFace { face };
        Ok(leaf_face)
    }

    fn read_leaf_brush(&mut self) -> Result<LeafBrush> {
        let brush = self.inner.read_i32::<LittleEndian>()?;
        let leaf_brush = LeafBrush { brush };
        Ok(leaf_brush)
    }

    fn read_model(&mut self) -> Result<Model> {
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
        let face = self.inner.read_i32::<LittleEndian>()?;
        let num_faces = self.inner.read_i32::<LittleEndian>()?;
        let brush = self.inner.read_i32::<LittleEndian>()?;
        let num_brushes = self.inner.read_i32::<LittleEndian>()?;
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

    fn read_brush(&mut self) -> Result<Brush> {
        let brush_side = self.inner.read_i32::<LittleEndian>()?;
        let num_brush_sides = self.inner.read_i32::<LittleEndian>()?;
        let texture = self.inner.read_i32::<LittleEndian>()?;
        let brush = Brush {
            brush_side,
            num_brush_sides,
            texture,
        };
        Ok(brush)
    }

    fn read_brush_side(&mut self) -> Result<BrushSide> {
        let plane = self.inner.read_i32::<LittleEndian>()?;
        let texture = self.inner.read_i32::<LittleEndian>()?;
        let brush_side = BrushSide { plane, texture };
        Ok(brush_side)
    }

    fn read_vertex(&mut self) -> Result<Vertex> {
        let position = [
            self.inner.read_f32::<LittleEndian>()?,
            self.inner.read_f32::<LittleEndian>()?,
            self.inner.read_f32::<LittleEndian>()?,
        ];
        let tex_coord1 = [
            self.inner.read_f32::<LittleEndian>()?,
            self.inner.read_f32::<LittleEndian>()?,
        ];
        let tex_coord2 = [
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
            tex_coord1,
            tex_coord2,
            normal,
            color,
        };
        Ok(vertex)
    }

    fn read_mesh_vert(&mut self) -> Result<MeshVert> {
        let offset = self.inner.read_i32::<LittleEndian>()?;
        let mesh_vert = MeshVert { offset };
        Ok(mesh_vert)
    }

    fn read_name(&mut self) -> Result<ArrayString<[u8; 64]>> {
        let mut name_buf = [0u8; 64];
        self.inner.read_exact(&mut name_buf)?;
        ArrayString::from_byte_string(&name_buf)
            .map_err(|err| Error::new(ErrorKind::InvalidData, err))
    }

    fn read_effect(&mut self) -> Result<Effect> {
        let name = self.read_name()?;
        let brush = self.inner.read_i32::<LittleEndian>()?;
        let unknown = self.inner.read_i32::<LittleEndian>()?;
        Ok(Effect {
            name,
            brush,
            unknown,
        })
    }

    fn read_face(&mut self) -> Result<Face> {
        let texture = self.inner.read_i32::<LittleEndian>()?;
        let effect = self.inner.read_i32::<LittleEndian>()?;
        let face_type = self.inner.read_i32::<LittleEndian>()?;
        let vertex = self.inner.read_i32::<LittleEndian>()?;
        let num_vertexes = self.inner.read_i32::<LittleEndian>()?;
        let mesh_vert = self.inner.read_i32::<LittleEndian>()?;
        let num_mesh_verts = self.inner.read_i32::<LittleEndian>()?;
        let lm_index = self.inner.read_i32::<LittleEndian>()?;
        let lm_start = [
            self.inner.read_i32::<LittleEndian>()?,
            self.inner.read_i32::<LittleEndian>()?,
        ];
        let lm_size = [
            self.inner.read_i32::<LittleEndian>()?,
            self.inner.read_i32::<LittleEndian>()?,
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
            self.inner.read_i32::<LittleEndian>()?,
            self.inner.read_i32::<LittleEndian>()?,
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

    fn read_lightmap(&mut self) -> Result<Lightmap> {
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

    fn read_lightvol(&mut self) -> Result<Lightvol> {
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

    fn read_visdata(&mut self) -> Result<VisData> {
        let n_vecs = self.inner.read_i32::<LittleEndian>()?;
        let sz_vecs = self.inner.read_i32::<LittleEndian>()?;
        let vecs_size = n_vecs as u64 * sz_vecs as u64;
        let mut vecs = Vec::with_capacity(vecs_size as usize);
        self.inner.by_ref().take(vecs_size).read_to_end(&mut vecs)?;
        let vis_data = VisData {
            n_vecs,
            sz_vecs,
            vecs,
        };
        Ok(vis_data)
    }
}

// TODO: Store all the allocated objects inline to improve cache usage
#[derive(Debug)]
pub struct Bsp {
    pub header: Header,
    pub dir_entries: Directories,
    pub entities: Entities,
    pub textures: Vec<Texture>,
    pub planes: Vec<Plane>,
    pub nodes: Vec<Node>,
    pub leafs: Vec<Leaf>,
    pub leaf_faces: Vec<LeafFace>,
    pub leaf_brushes: Vec<LeafBrush>,
    pub models: Vec<Model>,
    pub brushes: Vec<Brush>,
    pub brush_sides: Vec<BrushSide>,
    pub vertexes: Vec<Vertex>,
    pub mesh_verts: Vec<MeshVert>,
    pub effects: Vec<Effect>,
    pub faces: Vec<Face>,
    pub lightmaps: Vec<Lightmap>,
    pub lightvols: Vec<Lightvol>,
    pub vis_data: VisData,
}

pub fn read_bsp<R: Read + Seek>(reader: R) -> Result<Bsp> {
    const EXPECTED_HEADER: Header = Header {
        i: b'I',
        b: b'B',
        s: b'S',
        p: b'P',
    };
    const EXPECTED_VERSION: i32 = 0x2e;

    let mut reader = BspReader { inner: reader };
    let header = reader.read_header()?;
    let version = reader.read_version()?;

    if header != EXPECTED_HEADER || version != EXPECTED_VERSION {
        return Err(ErrorKind::InvalidData.into());
    }

    let dir_entries = reader.read_directories()?;
    let entities = reader.read_entities(&dir_entries.entities)?;
    let textures = reader.read_entry(&dir_entries.textures, |r| r.read_texture())?;
    let planes = reader.read_entry(&dir_entries.planes, |r| r.read_plane())?;
    let nodes = reader.read_entry(&dir_entries.nodes, |r| r.read_node())?;
    let leafs = reader.read_entry(&dir_entries.leafs, |r| r.read_leaf())?;
    let leaf_faces = reader.read_entry(&dir_entries.leaf_faces, |r| r.read_leaf_face())?;
    let leaf_brushes = reader.read_entry(&dir_entries.leaf_brushes, |r| r.read_leaf_brush())?;
    let models = reader.read_entry(&dir_entries.models, |r| r.read_model())?;
    let brushes = reader.read_entry(&dir_entries.brushes, |r| r.read_brush())?;
    let brush_sides = reader.read_entry(&dir_entries.brush_sides, |r| r.read_brush_side())?;
    let vertexes = reader.read_entry(&dir_entries.vertexes, |r| r.read_vertex())?;
    let mesh_verts = reader.read_entry(&dir_entries.mesh_verts, |r| r.read_mesh_vert())?;
    let effects = reader.read_entry(&dir_entries.effects, |r| r.read_effect())?;
    let faces = reader.read_entry(&dir_entries.faces, |r| r.read_face())?;
    let lightmaps = reader.read_entry(&dir_entries.lightmaps, |r| r.read_lightmap())?;
    let lightvols = reader.read_entry(&dir_entries.lightvols, |r| r.read_lightvol())?;
    reader
        .inner
        .seek(SeekFrom::Start(dir_entries.visdata.offset as u64))?;
    let vis_data = reader.read_visdata()?;

    Ok({
        Bsp {
            header,
            dir_entries,
            entities,
            textures,
            planes,
            nodes,
            leafs,
            leaf_faces,
            leaf_brushes,
            models,
            brushes,
            brush_sides,
            vertexes,
            mesh_verts,
            effects,
            faces,
            lightmaps,
            lightvols,
            vis_data,
        }
    })
}

#[test]
fn random_file() {
    use std::fs::File;

    read_bsp(&mut File::open("test.bsp").expect("Cannot open file")).unwrap();
}
