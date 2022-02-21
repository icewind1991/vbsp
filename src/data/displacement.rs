use super::vector::Vector;
use binrw::{BinRead, BinResult, ReadOptions};
use bitflags::bitflags;
use num_enum::TryFromPrimitive;
use std::fmt::Debug;
use std::io::{Read, Seek};
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
        self.sub_neighbours.iter().filter_map(Option::as_ref)
    }
}

impl BinRead for DisplacementNeighbour {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        let raws = <[RawDisplacementSubNeighbour; 2]>::read_options(reader, options, args)?;
        Ok(DisplacementNeighbour {
            sub_neighbours: raws.map(|raw| raw.try_into().ok()),
        })
    }
}

static_assertions::const_assert_eq!(size_of::<DisplacementNeighbour>(), 12);

#[derive(Debug, Clone, BinRead)]
#[br(assert(neighbour_index == u16::MAX || (neighbour_orientation < 4 && span < 4 && neighbour_span < 4), "valid neighbour index with invalid enum fields"))]
struct RawDisplacementSubNeighbour {
    neighbour_index: u16,
    neighbour_orientation: u8,
    span: u8,
    #[br(align_after = align_of::< DisplacementSubNeighbour > ())]
    neighbour_span: u8,
}

#[test]
fn test_sub_neighbour_bytes() {
    super::test_read_bytes::<RawDisplacementSubNeighbour>();
}

#[derive(Debug, Clone)]
pub struct DisplacementSubNeighbour {
    pub neighbour_index: u16,
    /// Orientation of the neighbour relative to us
    pub neighbour_orientation: NeighbourOrientation,
    /// How the neighbour fits into us
    pub span: NeighbourSpan,
    /// How we fit into our neighbour
    pub neighbour_span: NeighbourSpan,
}

impl TryFrom<RawDisplacementSubNeighbour> for DisplacementSubNeighbour {
    type Error = ();

    fn try_from(value: RawDisplacementSubNeighbour) -> Result<Self, Self::Error> {
        match value.neighbour_index {
            u16::MAX => Err(()),
            neighbour_index => Ok(DisplacementSubNeighbour {
                neighbour_index,
                // note that we already checked if these enums are valid in the assert of the RawDisplacementSubNeighbour reader
                neighbour_orientation: NeighbourOrientation::try_from(value.neighbour_orientation)
                    .unwrap(),
                span: NeighbourSpan::try_from(value.span).unwrap(),
                neighbour_span: NeighbourSpan::try_from(value.neighbour_span).unwrap(),
            }),
        }
    }
}

static_assertions::const_assert_eq!(size_of::<DisplacementSubNeighbour>(), 6);

#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum NeighbourSpan {
    CornerToCorner,
    CornerToMidPoint,
    MidPointToCorner,
}

#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum NeighbourOrientation {
    Ccw0,
    Ccw90,
    Ccw180,
    Ccw270,
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
