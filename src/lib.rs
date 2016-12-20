extern crate byteorder;

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::*;

#[derive(Debug)]
pub struct Header {
    pub i: char,
    pub b: char,
    pub s: char,
    pub p: char,
}

fn read_header(cursor: &mut Cursor<Vec<u8>>) -> Result<Header> {
    let i = cursor.read_u8()? as char;
    let b = cursor.read_u8()? as char;
    let s = cursor.read_u8()? as char;
    let p = cursor.read_u8()? as char;
    Ok(Header {
        i: i,
        b: b,
        s: s,
        p: p,
    })
}

fn read_version(cursor: &mut Cursor<Vec<u8>>) -> Result<i32> {
    cursor.read_i32::<LittleEndian>()
}

#[derive(Debug)]
pub struct DirEntry {
    pub offset: i32,
    pub length: i32,
}

fn read_directories(cursor: &mut Cursor<Vec<u8>>) -> Result<Vec<DirEntry>> {
    let mut dir_entries = Vec::new();
    for _ in 0..16 {
        let offset = cursor.read_i32::<LittleEndian>()?;
        let length = cursor.read_i32::<LittleEndian>()?;
        dir_entries.push(DirEntry {
            offset: offset,
            length: length,
        });
    }
    Ok(dir_entries)
}

#[derive(Debug)]
pub struct Entity {
    pub entities: String,
}

fn read_entities(cursor: &mut Cursor<Vec<u8>>, dir_entry: &DirEntry) -> Result<Entity> {
    let mut entities = Vec::with_capacity(dir_entry.length as usize);
    cursor.set_position(dir_entry.offset as u64);
    for _ in 0..dir_entry.length {
        let data = cursor.read_u8()?;
        entities.push(data);
    }
    let entities = String::from_utf8(entities).unwrap();
    Ok(Entity { entities: entities })
}

fn read_entry<F, T>(cursor: &mut Cursor<Vec<u8>>, dir_entry: &DirEntry, mut f: F) -> Result<Vec<T>>
    where F: FnMut(&mut Cursor<Vec<u8>>) -> Result<T>
{
    let mut entries = Vec::new();
    cursor.set_position(dir_entry.offset as u64);
    let end_pos = (dir_entry.offset + dir_entry.length) as u64;
    while cursor.position() < end_pos {
        let entry = f(cursor)?;
        entries.push(entry);
    }
    Ok(entries)
}

#[derive(Debug)]
pub struct Texture {
    pub name: String,
    pub flags: i32,
    pub contents: i32,
}

fn read_texture(cursor: &mut Cursor<Vec<u8>>) -> Result<Texture> {
    let mut texture = Vec::new();
    for _ in 0..64 {
        let data = cursor.read_u8()?;
        if data != 0u8 {
            texture.push(data);
        }
    }
    let texture_name = String::from_utf8(texture).unwrap();
    let flags = cursor.read_i32::<LittleEndian>()?;
    let contents = cursor.read_i32::<LittleEndian>()?;
    Ok(Texture {
        name: texture_name,
        flags: flags,
        contents: contents,
    })
}

#[derive(Debug)]
pub struct Plane {
    pub normal: [f32; 3],
    pub dist: f32,
}

fn read_plane(cursor: &mut Cursor<Vec<u8>>) -> Result<Plane> {
    let x = cursor.read_f32::<LittleEndian>()?;
    let y = cursor.read_f32::<LittleEndian>()?;
    let z = cursor.read_f32::<LittleEndian>()?;
    let dist = cursor.read_f32::<LittleEndian>()?;
    let plane = Plane {
        normal: [x, y, z],
        dist: dist,
    };
    Ok(plane)
}

#[derive(Debug)]
pub struct Node {
    pub plane: i32,
    pub children: [i32; 2],
    pub mins: [i32; 3],
    pub maxs: [i32; 3],
}

fn read_node(cursor: &mut Cursor<Vec<u8>>) -> Result<Node> {
    let plane = cursor.read_i32::<LittleEndian>()?;
    let children = [cursor.read_i32::<LittleEndian>()?, cursor.read_i32::<LittleEndian>()?];
    let mins = [cursor.read_i32::<LittleEndian>()?,
                cursor.read_i32::<LittleEndian>()?,
                cursor.read_i32::<LittleEndian>()?];
    let maxs = [cursor.read_i32::<LittleEndian>()?,
                cursor.read_i32::<LittleEndian>()?,
                cursor.read_i32::<LittleEndian>()?];
    let node = Node {
        plane: plane,
        children: children,
        mins: mins,
        maxs: maxs,
    };
    Ok(node)
}

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

fn read_leaf(cursor: &mut Cursor<Vec<u8>>) -> Result<Leaf> {
    let cluster = cursor.read_i32::<LittleEndian>()?;
    let area = cursor.read_i32::<LittleEndian>()?;
    let mins = [cursor.read_i32::<LittleEndian>()?,
                cursor.read_i32::<LittleEndian>()?,
                cursor.read_i32::<LittleEndian>()?];
    let maxs = [cursor.read_i32::<LittleEndian>()?,
                cursor.read_i32::<LittleEndian>()?,
                cursor.read_i32::<LittleEndian>()?];
    let leaf_face = cursor.read_i32::<LittleEndian>()?;
    let num_leaf_faces = cursor.read_i32::<LittleEndian>()?;
    let leaf_brush = cursor.read_i32::<LittleEndian>()?;
    let num_leaf_brushes = cursor.read_i32::<LittleEndian>()?;
    let leaf = Leaf {
        cluster: cluster,
        area: area,
        mins: mins,
        maxs: maxs,
        leaf_face: leaf_face,
        num_leaf_faces: num_leaf_faces,
        leaf_brush: leaf_brush,
        num_leaf_brushes: num_leaf_brushes,
    };
    Ok(leaf)
}

#[derive(Debug)]
pub struct LeafFace {
    pub face: i32,
}

fn read_leaf_face(cursor: &mut Cursor<Vec<u8>>) -> Result<LeafFace> {
    let face = cursor.read_i32::<LittleEndian>()?;
    let leaf_face = LeafFace { face: face };
    Ok(leaf_face)
}

#[derive(Debug)]
pub struct LeafBrush {
    pub brush: i32,
}

fn read_leaf_brush(cursor: &mut Cursor<Vec<u8>>) -> Result<LeafBrush> {
    let brush = cursor.read_i32::<LittleEndian>()?;
    let leaf_brush = LeafBrush { brush: brush };
    Ok(leaf_brush)
}

#[derive(Debug)]
pub struct Model {
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub face: i32,
    pub num_faces: i32,
    pub brush: i32,
    pub num_brushes: i32,
}

fn read_model(cursor: &mut Cursor<Vec<u8>>) -> Result<Model> {
    let mins = [cursor.read_f32::<LittleEndian>()?,
                cursor.read_f32::<LittleEndian>()?,
                cursor.read_f32::<LittleEndian>()?];
    let maxs = [cursor.read_f32::<LittleEndian>()?,
                cursor.read_f32::<LittleEndian>()?,
                cursor.read_f32::<LittleEndian>()?];
    let face = cursor.read_i32::<LittleEndian>()?;
    let num_faces = cursor.read_i32::<LittleEndian>()?;
    let brush = cursor.read_i32::<LittleEndian>()?;
    let num_brushes = cursor.read_i32::<LittleEndian>()?;
    let model = Model {
        mins: mins,
        maxs: maxs,
        face: face,
        num_faces: num_faces,
        brush: brush,
        num_brushes: num_brushes,
    };
    Ok(model)
}

#[derive(Debug)]
pub struct Brush {
    pub brush_side: i32,
    pub num_brush_sides: i32,
    pub texture: i32,
}

fn read_brush(cursor: &mut Cursor<Vec<u8>>) -> Result<Brush> {
    let brush_side = cursor.read_i32::<LittleEndian>()?;
    let num_brush_sides = cursor.read_i32::<LittleEndian>()?;
    let texture = cursor.read_i32::<LittleEndian>()?;
    let brush = Brush {
        brush_side: brush_side,
        num_brush_sides: num_brush_sides,
        texture: texture,
    };
    Ok(brush)
}

#[derive(Debug)]
pub struct BrushSide {
    pub plane: i32,
    pub texture: i32,
}

fn read_brush_side(cursor: &mut Cursor<Vec<u8>>) -> Result<BrushSide> {
    let plane = cursor.read_i32::<LittleEndian>()?;
    let texture = cursor.read_i32::<LittleEndian>()?;
    let brush_side = BrushSide {
        plane: plane,
        texture: texture,
    };
    Ok(brush_side)
}

#[derive(Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coord1: [f32; 2],
    pub tex_coord2: [f32; 2],
    pub normal: [f32; 3],
    pub color: [u8; 4],
}

fn read_vertex(cursor: &mut Cursor<Vec<u8>>) -> Result<Vertex> {
    let position = [cursor.read_f32::<LittleEndian>()?,
                    cursor.read_f32::<LittleEndian>()?,
                    cursor.read_f32::<LittleEndian>()?];
    let tex_coord1 = [cursor.read_f32::<LittleEndian>()?, cursor.read_f32::<LittleEndian>()?];
    let tex_coord2 = [cursor.read_f32::<LittleEndian>()?, cursor.read_f32::<LittleEndian>()?];
    let normal = [cursor.read_f32::<LittleEndian>()?,
                  cursor.read_f32::<LittleEndian>()?,
                  cursor.read_f32::<LittleEndian>()?];
    let color = [cursor.read_u8()?, cursor.read_u8()?, cursor.read_u8()?, cursor.read_u8()?];
    let vertex = Vertex {
        position: position,
        tex_coord1: tex_coord1,
        tex_coord2: tex_coord2,
        normal: normal,
        color: color,
    };
    Ok(vertex)
}

#[derive(Debug)]
pub struct MeshVert {
    pub offset: i32,
}

fn read_mesh_vert(cursor: &mut Cursor<Vec<u8>>) -> Result<MeshVert> {
    let offset = cursor.read_i32::<LittleEndian>()?;
    let mesh_vert = MeshVert { offset: offset };
    Ok(mesh_vert)
}

#[derive(Debug)]
pub struct Effect {
    pub name: String,
    pub brush: i32,
    pub unk: i32,
}

fn read_effect(cursor: &mut Cursor<Vec<u8>>) -> Result<Effect> {
    let mut name = Vec::new();
    for _ in 0..64 {
        let data = cursor.read_u8()?;
        if data != 0u8 {
            name.push(data);
        }
    }
    let name = String::from_utf8(name).unwrap();
    let brush = cursor.read_i32::<LittleEndian>()?;
    let unk = cursor.read_i32::<LittleEndian>()?;
    Ok(Effect {
        name: name,
        brush: brush,
        unk: unk,
    })
}

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

fn read_face(cursor: &mut Cursor<Vec<u8>>) -> Result<Face> {
    let texture = cursor.read_i32::<LittleEndian>()?;
    let effect = cursor.read_i32::<LittleEndian>()?;
    let face_type = cursor.read_i32::<LittleEndian>()?;
    let vertex = cursor.read_i32::<LittleEndian>()?;
    let num_vertexes = cursor.read_i32::<LittleEndian>()?;
    let mesh_vert = cursor.read_i32::<LittleEndian>()?;
    let num_mesh_verts = cursor.read_i32::<LittleEndian>()?;
    let lm_index = cursor.read_i32::<LittleEndian>()?;
    let lm_start = [cursor.read_i32::<LittleEndian>()?, cursor.read_i32::<LittleEndian>()?];
    let lm_size = [cursor.read_i32::<LittleEndian>()?, cursor.read_i32::<LittleEndian>()?];
    let lm_origin = [cursor.read_f32::<LittleEndian>()?,
                     cursor.read_f32::<LittleEndian>()?,
                     cursor.read_f32::<LittleEndian>()?];
    let lm_vec1 = [cursor.read_f32::<LittleEndian>()?,
                   cursor.read_f32::<LittleEndian>()?,
                   cursor.read_f32::<LittleEndian>()?];
    let lm_vec2 = [cursor.read_f32::<LittleEndian>()?,
                   cursor.read_f32::<LittleEndian>()?,
                   cursor.read_f32::<LittleEndian>()?];
    let lm_vecs = [lm_vec1, lm_vec2];
    let normal = [cursor.read_f32::<LittleEndian>()?,
                  cursor.read_f32::<LittleEndian>()?,
                  cursor.read_f32::<LittleEndian>()?];
    let size = [cursor.read_i32::<LittleEndian>()?, cursor.read_i32::<LittleEndian>()?];
    let face = Face {
        texture: texture,
        effect: effect,
        face_type: face_type,
        vertex: vertex,
        num_vertexes: num_vertexes,
        mesh_vert: mesh_vert,
        num_mesh_verts: num_mesh_verts,
        lm_index: lm_index,
        lm_start: lm_start,
        lm_size: lm_size,
        lm_origin: lm_origin,
        lm_vecs: lm_vecs,
        normal: normal,
        size: size,
    };
    Ok(face)
}

#[derive(Debug)]
pub struct BSP {
    pub header: Header,
    pub dir_entries: Vec<DirEntry>,
    pub entities: Entity,
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
}

pub fn read_bsp(bytes: Vec<u8>) -> Result<BSP> {
    let mut cursor = Cursor::new(bytes);
    let header = read_header(&mut cursor)?;
    let version = read_version(&mut cursor)?;
    assert_eq!(version, 0x2e);
    let dir_entries = read_directories(&mut cursor)?;
    let entities = read_entities(&mut cursor, &dir_entries[0])?;
    let textures = read_entry(&mut cursor, &dir_entries[1], read_texture)?;
    let planes = read_entry(&mut cursor, &dir_entries[2], read_plane)?;
    let nodes = read_entry(&mut cursor, &dir_entries[3], read_node)?;
    let leafs = read_entry(&mut cursor, &dir_entries[4], read_leaf)?;
    let leaf_faces = read_entry(&mut cursor, &dir_entries[5], read_leaf_face)?;
    let leaf_brushes = read_entry(&mut cursor, &dir_entries[6], read_leaf_brush)?;
    let models = read_entry(&mut cursor, &dir_entries[7], read_model)?;
    let brushes = read_entry(&mut cursor, &dir_entries[8], read_brush)?;
    let brush_sides = read_entry(&mut cursor, &dir_entries[9], read_brush_side)?;
    let vertexes = read_entry(&mut cursor, &dir_entries[10], read_vertex)?;
    let mesh_verts = read_entry(&mut cursor, &dir_entries[11], read_mesh_vert)?;
    let effects = read_entry(&mut cursor, &dir_entries[12], read_effect)?;
    let faces = read_entry(&mut cursor, &dir_entries[13], read_face)?;
    Ok({
        BSP {
            header: header,
            dir_entries: dir_entries,
            entities: entities,
            textures: textures,
            planes: planes,
            nodes: nodes,
            leafs: leafs,
            leaf_faces: leaf_faces,
            leaf_brushes: leaf_brushes,
            models: models,
            brushes: brushes,
            brush_sides: brush_sides,
            vertexes: vertexes,
            mesh_verts: mesh_verts,
            effects: effects,
            faces: faces,
        }
    })
}
