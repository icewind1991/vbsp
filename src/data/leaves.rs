use std::io::{Read, Seek};
use std::mem::{align_of, size_of};
use std::ops::Deref;

use binrw::{BinRead, BinResult, Endian};

use crate::bspfile::LumpType;
use crate::BspError;

use super::LumpArgs;

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

    pub fn clusters(&self) -> impl Iterator<Item = impl Iterator<Item = &Leaf>> {
        LeafClusters {
            leaves: &self.leaves,
            index: 0,
        }
    }
}

impl BinRead for Leaves {
    type Args<'a> = LumpArgs;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let item_size = match args.version {
            1 => size_of::<LeafV1>(),
            version => {
                return Err(binrw::Error::Custom {
                    err: Box::new(BspError::LumpVersion(
                        crate::error::UnsupportedLumpVersion {
                            lump_type: "leaves",
                            version: version as u16,
                        },
                    )),
                    pos: reader.stream_position().unwrap(),
                })
            }
        };
        if args.length % item_size != 0 {
            return Err(binrw::Error::Custom {
                err: Box::new(BspError::InvalidLumpSize {
                    lump: LumpType::Leaves,
                    element_size: item_size,
                    lump_size: args.length,
                }),
                pos: reader.stream_position().unwrap(),
            });
        }
        let num_entries = args.length / item_size;
        let mut entries = Vec::with_capacity(num_entries);

        for _ in 0..num_entries {
            entries.push(Leaf::read_options(
                reader,
                endian,
                LeafArgs {
                    version: args.version,
                },
            )?);
        }

        Ok(Self { leaves: entries })
    }
}

struct LeafClusters<'a> {
    leaves: &'a [Leaf],
    index: usize,
}

impl<'a> Iterator for LeafClusters<'a> {
    type Item = <&'a [Leaf] as IntoIterator>::IntoIter;

    fn next(&mut self) -> Option<Self::Item> {
        let cluster = self.leaves.get(self.index)?.cluster;
        let remaining_leaves = self.leaves.get(self.index..)?;
        let cluster_size = remaining_leaves
            .iter()
            .take_while(|leaf| leaf.cluster == cluster)
            .count();
        self.index += cluster_size;
        Some(remaining_leaves[0..cluster_size].iter())
    }
}

#[test]
fn test_leaf_clusters() {
    let leaves: Leaves = vec![
        Leaf {
            contents: 0,
            cluster: 0,
            ..Default::default()
        },
        Leaf {
            contents: 1,
            cluster: 0,
            ..Default::default()
        },
        Leaf {
            contents: 2,
            cluster: 1,
            ..Default::default()
        },
        Leaf {
            contents: 3,
            cluster: 2,
            ..Default::default()
        },
        Leaf {
            contents: 4,
            cluster: 2,
            ..Default::default()
        },
    ]
    .into();

    let clustered: Vec<Vec<i32>> = leaves
        .clusters()
        .map(|cluster| cluster.map(|leaf| leaf.contents).collect())
        .collect();
    assert_eq!(vec![vec![0, 1], vec![2], vec![3, 4]], clustered);
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
    type Item = Leaf;
    type IntoIter = <Vec<Leaf> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.leaves.into_iter()
    }
}

impl<'a> IntoIterator for &'a Leaves {
    type Item = &'a Leaf;
    type IntoIter = <&'a [Leaf] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.leaves[..].iter()
    }
}

impl<'a> IntoIterator for &'a mut Leaves {
    type Item = &'a mut Leaf;
    type IntoIter = <&'a mut [Leaf] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.leaves.iter_mut()
    }
}

#[derive(BinRead, Debug, Default, Clone, Copy)]
pub struct ColorRGBExp32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub exponent: i8,
}

#[derive(BinRead, Debug, Default, Clone, Copy)]
pub struct CompressedLightCube {
    pub color: [ColorRGBExp32; 6],
}

#[derive(Default, Debug, Clone, BinRead)]
pub struct LeafV1 {
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
    #[br(align_after = align_of::< LeafV1 > ())]
    pub leaf_watter_data_id: i16,
}

static_assertions::const_assert_eq!(size_of::<LeafV1>(), 32);

impl From<LeafV1> for Leaf {
    fn from(value: LeafV1) -> Self {
        Self {
            contents: value.contents,
            cluster: value.cluster,
            area_and_flags: value.area_and_flags,
            mins: value.mins,
            maxs: value.maxs,
            first_leaf_face: value.first_leaf_face,
            leaf_face_count: value.leaf_face_count,
            first_leaf_brush: value.first_leaf_brush,
            leaf_brush_count: value.leaf_brush_count,
            leaf_watter_data_id: value.leaf_watter_data_id,
            cube: Default::default(),
        }
    }
}

#[derive(Default, Debug, Clone)]
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
    pub leaf_watter_data_id: i16,
    pub cube: CompressedLightCube,
}

static_assertions::const_assert_eq!(size_of::<Leaf>(), 56);

#[test]
fn test_leaf_bytes() {
    super::test_read_bytes::<Leaf>();
}

#[derive(Default, Debug, Clone)]
pub struct LeafArgs {
    pub version: u32,
}

impl BinRead for Leaf {
    type Args<'a> = LeafArgs;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        match args.version {
            1 => LeafV1::read_options(reader, endian, ()).map(Leaf::from),
            version => Err(binrw::Error::Custom {
                err: Box::new(crate::error::UnsupportedLumpVersion {
                    lump_type: "leaves",
                    version: version as u16,
                }),
                pos: reader.stream_position().unwrap(),
            }),
        }
    }
}
