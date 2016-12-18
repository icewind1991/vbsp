
extern crate byteorder;

use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::{File};
use std::io::*;
use std::path::{Path};

#[derive(Debug)]
pub struct Header {
    pub i : char,
    pub b : char, 
    pub s : char,
    pub p : char
}

#[derive(Debug)]
pub struct DirEntry {
    pub offset: i32,
    pub length : i32
}

#[derive(Debug)]
pub struct Entity {
    pub entities: String
}

#[derive(Debug)]
pub struct Texture {
    pub name : String,
    pub flags : i32,
    pub contents : i32
}
#[derive(Debug)]
pub struct Plane {
    pub normal: [f32; 3],
    pub dist : f32
}

#[derive(Debug)]
pub struct BSP { 
    pub header : Header,
    pub dir_entries : Vec<DirEntry>,
    pub entities : Entity,
    pub textures : Vec<Texture>,
    pub planes : Vec<Plane>,
}

fn read_header(cursor : &mut Cursor<Vec<u8>>) -> Result<Header> {
    let i = cursor.read_u8()? as char;
    let b = cursor.read_u8()? as char;
    let s = cursor.read_u8()? as char;
    let p = cursor.read_u8()? as char;
    Ok(Header {i : i, b : b, s : s, p : p})
}

fn read_version(cursor : &mut Cursor<Vec<u8>>) -> Result<i32> {
    cursor.read_i32::<LittleEndian>()
}

fn read_directories(cursor : &mut Cursor<Vec<u8>>) -> Result<Vec<DirEntry>> {
    let mut dir_entries = Vec::new();
    for _ in 0 .. 16 {
        let offset = cursor.read_i32::<LittleEndian>()?;
        let length = cursor.read_i32::<LittleEndian>()?;
        dir_entries.push( DirEntry { offset : offset, length : length });
    }
    Ok(dir_entries)
}

fn read_entities(cursor : &mut Cursor<Vec<u8>>, dir_entry : &DirEntry) -> Result<Entity> {
    let mut entities = Vec::with_capacity(dir_entry.length as usize);
    cursor.set_position(dir_entry.offset as u64);
    for _ in 0 .. dir_entry.length {
        let data = cursor.read_u8()?;
        entities.push(data);
    }
    
    let entities = String::from_utf8(entities).unwrap();

    Ok (Entity { entities : entities})
}

fn read_entry<F, T>(cursor : &mut Cursor<Vec<u8>>, dir_entry : &DirEntry, mut f : F ) -> Result<Vec<T>> where F : FnMut(&mut Cursor<Vec<u8>>) -> Result<T> {
    let mut entries = Vec::new();
    cursor.set_position(dir_entry.offset as u64);
    let end_pos = (dir_entry.offset + dir_entry.length) as u64;
    while cursor.position() < end_pos {
        let entry = f(cursor)?;
        entries.push(entry);    
    }
    Ok (entries)
}

fn read_texture(cursor : &mut Cursor<Vec<u8>>) -> Result<Texture> {
    let mut texture = Vec::new();
    for _ in 0 .. 64 {
        let data = cursor.read_u8()?;
        if data != 0u8 {
            texture.push(data);
        }
    }
    let texture_name = String::from_utf8(texture).unwrap();
    let flags = cursor.read_i32::<LittleEndian>()?;
    let contents = cursor.read_i32::<LittleEndian>()?;
    Ok (Texture { name : texture_name, flags : flags, contents : contents })
}

fn read_plane(cursor : &mut Cursor<Vec<u8>>) -> Result<Plane> {
    let x = cursor.read_f32::<LittleEndian>()?;
    let y = cursor.read_f32::<LittleEndian>()?;
    let z = cursor.read_f32::<LittleEndian>()?;
    let dist = cursor.read_f32::<LittleEndian>()?;
    let plane = Plane { normal : [x,y,z], dist : dist };
    Ok (plane)
}

pub fn read_bsp(filename : &str) -> Result<BSP> {
    let path = Path::new(filename);
    let mut file = File::open(path).unwrap();
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).unwrap();
    let mut cursor = Cursor::new(bytes);
    let header = read_header(&mut cursor)?;
    let version = read_version(&mut cursor)?;
    assert_eq!(version, 0x2e);
    let dir_entries = read_directories(&mut cursor)?;
    let entities = read_entities(&mut cursor, &dir_entries[0])?;
    let textures = read_entry(&mut cursor, &dir_entries[1], read_texture)?; //read_textures(&mut cursor, &dir_entries[1])?;
    let planes = read_entry(&mut cursor, &dir_entries[2], read_plane)?;
    Ok ({ BSP { header : header, 
                dir_entries : dir_entries, 
                entities : entities, 
                textures : textures,
                planes : planes }})
}
