use super::vector::Vector;
use crate::data::try_read_enum;
use crate::error::InvalidNeighbourError;
use binrw::{BinRead, BinResult, ReadOptions};
use bitflags::bitflags;
use num_enum::TryFromPrimitive;
use std::fmt::Debug;
use std::io::{Read, Seek, SeekFrom};
use std::mem::{align_of, size_of};

#[derive(Debug, Clone, BinRead)]
pub struct DisplacementInfo {
    pub start_position: Vector,
    pub displacement_vertex_start: i32,
    pub displacement_triangle_tag_start: i32,

    pub power: i32,
    pub minimum_tesselation: i32,
    pub smoothing_angle: f32,
    pub contents: i32,

    pub map_face: u16,

    #[br(align_before = 4)]
    pub lightmap_alpha_start: i32,
    pub lightmap_sample_position_start: i32,

    pub edge_neighbours: [DisplacementNeighbour; 4],
    pub corner_neighbours: [DisplacementCornerNeighbour; 4],

    pub allowed_vertices: [u32; 10],
}

impl DisplacementInfo {
    pub fn vertex_count(&self) -> i32 {
        (2i32.pow(self.power as u32) + 1).pow(2)
    }

    pub fn triangle_count(&self) -> i32 {
        2 * 2i32.pow(self.power as u32).pow(2)
    }
}

#[test]
fn test_displacement_bytes() {
    super::test_read_bytes::<DisplacementInfo>();
}

static_assertions::const_assert_eq!(size_of::<DisplacementInfo>(), 176);

#[derive(Debug, Clone)]
pub struct DisplacementNeighbour {
    pub sub_neighbours: [Option<DisplacementSubNeighbour>; 2],
}

impl DisplacementNeighbour {
    pub fn iter(&self) -> impl Iterator<Item = &DisplacementSubNeighbour> {
        self.sub_neighbours.iter().filter_map(|sub| sub.as_ref())
    }
}

impl BinRead for DisplacementNeighbour {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        Ok(DisplacementNeighbour {
            sub_neighbours: [
                read_option_sub_neighbour(reader, options, args)?,
                read_option_sub_neighbour(reader, options, args)?,
            ],
        })
    }
}

fn read_option_sub_neighbour<R: Read + Seek>(
    reader: &mut R,
    options: &ReadOptions,
    args: (),
) -> BinResult<Option<DisplacementSubNeighbour>> {
    let neighbour_index = u16::read_options(reader, options, args)?;

    // for non-connected sub-neighbours, the orientations and spans sometimes contain garbage
    // so we just skip over it
    if neighbour_index == u16::MAX {
        reader.seek(SeekFrom::Current(
            size_of::<DisplacementSubNeighbour>() as i64 - 2,
        ))?;
        Ok(None)
    } else {
        reader.seek(SeekFrom::Current(-2))?;
        Ok(Some(DisplacementSubNeighbour::read_options(
            reader, options, args,
        )?))
    }
}

static_assertions::const_assert_eq!(size_of::<DisplacementNeighbour>(), 12);

#[test]
fn test_neighbour_bytes() {
    super::test_read_bytes::<DisplacementNeighbour>();
}

#[derive(Debug, Clone, BinRead)]
pub struct DisplacementSubNeighbour {
    pub neighbour_index: u16,
    /// Orientation of the neighbour relative to us
    pub neighbour_orientation: NeighbourOrientation,
    /// How the neighbour fits into us
    pub span: NeighbourSpan,
    /// How we fit into our neighbour
    #[br(align_after = align_of::<DisplacementSubNeighbour>())]
    pub neighbour_span: NeighbourSpan,
}

#[test]
fn test_sub_neighbour_bytes() {
    super::test_read_bytes::<DisplacementSubNeighbour>();
}

static_assertions::const_assert_eq!(size_of::<DisplacementSubNeighbour>(), 6);
static_assertions::const_assert_eq!(align_of::<DisplacementSubNeighbour>(), 2);

#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum NeighbourSpan {
    CornerToCorner = 0,
    CornerToMidPoint = 1,
    MidPointToCorner = 2,
}

impl BinRead for NeighbourSpan {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        try_read_enum(
            reader,
            options,
            args,
            InvalidNeighbourError::InvalidNeighbourSpan,
        )
    }
}

#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum NeighbourOrientation {
    Ccw0 = 0,
    Ccw90 = 1,
    Ccw180 = 2,
    Ccw270 = 3,
}

impl BinRead for NeighbourOrientation {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        try_read_enum(
            reader,
            options,
            args,
            InvalidNeighbourError::InvalidNeighbourOrientation,
        )
    }
}

#[derive(Debug, Clone, BinRead)]
pub struct DisplacementCornerNeighbour {
    neighbours: [u16; 4],
    #[br(align_after = align_of::< DisplacementCornerNeighbour > ())]
    neighbour_count: u8,
}

impl DisplacementCornerNeighbour {
    pub fn neighbours(&self) -> impl Iterator<Item = u16> + '_ {
        self.neighbours
            .iter()
            .copied()
            .take(self.neighbour_count as usize)
    }
}

static_assertions::const_assert_eq!(size_of::<DisplacementCornerNeighbour>(), 10);

#[test]
fn test_corner_neighbour_bytes() {
    super::test_read_bytes::<DisplacementCornerNeighbour>();
}

#[derive(Debug, Clone, BinRead)]
pub struct DisplacementVertex {
    pub vector: Vector,
    pub distance: f32,
    pub alpha: f32,
}

impl DisplacementVertex {
    pub fn displacement(&self) -> Vector {
        self.vector * self.distance
    }
}

#[derive(Debug, Clone, BinRead)]
pub struct DisplacementTriangle {
    pub tags: DisplacementTriangleFlags,
}

bitflags! {
    #[derive(BinRead)]
    pub struct DisplacementTriangleFlags: u8 {
        const SURFACE =       0x01;
        const WALKABLE =      0x02;
        const BULDABLE =      0x04;
        const SURFACE_PROP1 = 0x08;
        const SURFACE_PROP2 = 0x10;
    }
}
