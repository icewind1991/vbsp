mod bspfile;
pub mod data;
pub mod error;
mod handle;
mod reader;

use crate::bspfile::LumpType;
pub use crate::data::TextureFlags;
pub use crate::data::Vector;
pub use crate::data::*;
use crate::error::ValidationError;
pub use crate::handle::Handle;
use binrw::io::Cursor;
use binrw::{BinRead, BinReaderExt};
use bspfile::BspFile;
pub use error::{BspError, StringError};
use lzma_rs::decompress::{Options, UnpackedSize};
use reader::LumpReader;
use std::cmp::min;
use std::{io::Read, ops::Deref};

pub type BspResult<T> = Result<T, BspError>;

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

// TODO: Store all the allocated objects inline to improve cache usage
/// A parsed bsp file
#[derive(Debug)]
#[non_exhaustive]
pub struct Bsp {
    pub header: Header,
    pub entities: Entities,
    pub textures_data: Vec<TextureData>,
    pub textures_info: Vec<TextureInfo>,
    pub texture_string_tables: Vec<i32>,
    pub texture_string_data: String,
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
    pub original_faces: Vec<Face>,
    pub vis_data: VisData,
    pub displacements: Vec<DisplacementInfo>,
    pub displacement_vertices: Vec<DisplacementVertex>,
    pub displacement_triangles: Vec<DisplacementTriangle>,
    vertex_normals: Vec<VertNormal>,
    vertex_normal_indices: Vec<VertNormalIndex>,
    pub static_props: PropStaticGameLump,
    pub pack: Packfile,
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
        let texture_string_tables = bsp_file
            .lump_reader(LumpType::TextureDataStringTable)?
            .read_vec(|r| r.read())?;
        let texture_string_data = String::from_utf8(
            bsp_file
                .get_lump(LumpType::TextureDataStringData)?
                .into_owned(),
        )
        .map_err(|e| BspError::String(StringError::NonUTF8(e.utf8_error())))?;
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
        let original_faces = bsp_file
            .lump_reader(LumpType::OriginalFaces)?
            .read_vec(|r| r.read())?;
        let vis_data = bsp_file.lump_reader(LumpType::Visibility)?.read_visdata()?;
        let displacements = bsp_file
            .lump_reader(LumpType::DisplacementInfo)?
            .read_vec(|r| r.read())?;
        let displacement_vertices = bsp_file
            .lump_reader(LumpType::DisplacementVertices)?
            .read_vec(|r| r.read())?;
        let displacement_triangles = bsp_file
            .lump_reader(LumpType::DisplacementTris)?
            .read_vec(|r| r.read())?;
        let vertex_normals = bsp_file
            .lump_reader(LumpType::VertNormals)?
            .read_vec(|r| r.read())?;
        let vertex_normal_indices = bsp_file
            .lump_reader(LumpType::VertNormalIndices)?
            .read_vec(|r| r.read())?;
        let game_lumps: GameLumpHeader = bsp_file.lump_reader(LumpType::GameLump)?.read()?;
        let pack = Packfile::read(bsp_file.lump_reader(LumpType::PakFile)?.into_data())?;

        let static_props = game_lumps
            .find(data)
            .ok_or(ValidationError::NoStaticPropLump)??;

        let bsp = Bsp {
            header: bsp_file.header().clone(),
            entities,
            textures_data,
            textures_info,
            texture_string_tables,
            texture_string_data,
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
            original_faces,
            vis_data,
            displacements,
            displacement_vertices,
            displacement_triangles,
            vertex_normals,
            vertex_normal_indices,
            static_props,
            pack,
        };
        bsp.validate()?;
        Ok(bsp)
    }

    pub fn leaf(&self, n: usize) -> Option<Handle<'_, Leaf>> {
        self.leaves.get(n).map(|leaf| Handle::new(self, leaf))
    }

    pub fn plane(&self, n: usize) -> Option<Handle<'_, Plane>> {
        self.planes.get(n).map(|plane| Handle::new(self, plane))
    }

    pub fn face(&self, n: usize) -> Option<Handle<'_, Face>> {
        self.faces.get(n).map(|face| Handle::new(self, face))
    }

    pub fn node(&self, n: usize) -> Option<Handle<'_, Node>> {
        self.nodes.get(n).map(|node| Handle::new(self, node))
    }

    pub fn displacement(&self, n: usize) -> Option<Handle<'_, DisplacementInfo>> {
        self.displacements
            .get(n)
            .map(|displacement| Handle::new(self, displacement))
    }

    fn displacement_vertex(&self, n: usize) -> Option<Handle<'_, DisplacementVertex>> {
        self.displacement_vertices
            .get(n)
            .map(|vert| Handle::new(self, vert))
    }

    /// Get the root node of the bsp
    pub fn root_node(&self) -> Handle<'_, Node> {
        self.node(0).unwrap()
    }

    /// Get all models stored in the bsp
    pub fn models(&self) -> impl Iterator<Item = Handle<'_, Model>> {
        self.models.iter().map(move |m| Handle::new(self, m))
    }

    /// Get all models stored in the bsp
    pub fn textures(&self) -> impl Iterator<Item = Handle<'_, TextureInfo>> {
        self.textures_info.iter().map(move |m| Handle::new(self, m))
    }

    /// Find a leaf for a specific position
    pub fn leaf_at(&self, point: Vector) -> Handle<'_, Leaf> {
        let mut current = self.root_node();

        loop {
            let plane = current.plane();
            let dot: f32 = point
                .iter()
                .zip(plane.normal.iter())
                .map(|(a, b)| a * b)
                .sum();

            let [front, back] = current.children;

            let next = if dot < plane.dist { back } else { front };

            if next < 0 {
                return self.leaf((!next) as usize).unwrap();
            } else {
                current = self.node(next as usize).unwrap();
            }
        }
    }

    pub fn static_props(&self) -> impl Iterator<Item = Handle<'_, StaticPropLump>> {
        self.static_props
            .props
            .props
            .iter()
            .map(|lump| Handle::new(self, lump))
    }

    /// Get all faces stored in the bsp
    pub fn original_faces(&self) -> impl Iterator<Item = Handle<Face>> {
        self.faces.iter().map(move |face| Handle::new(self, face))
    }

    fn validate(&self) -> BspResult<()> {
        self.validate_indexes(
            self.faces
                .iter()
                .filter_map(|face| face.displacement_index()),
            &self.displacements,
            "face",
            "displacement",
        )?;
        self.validate_indexes(
            self.displacements
                .iter()
                .map(|displacement| displacement.map_face),
            &self.faces,
            "displacement",
            "face",
        )?;
        self.validate_indexes(
            self.faces
                .iter()
                .map(|face| face.first_edge + face.num_edges as i32 - 1),
            &self.surface_edges,
            "face",
            "surface_edge",
        )?;
        self.validate_indexes(
            self.surface_edges.iter().map(|edge| edge.edge_index()),
            &self.edges,
            "surface_edge",
            "edge",
        )?;
        self.validate_indexes(
            self.edges
                .iter()
                .flat_map(|edge| [edge.start_index, edge.end_index]),
            &self.vertices,
            "edge",
            "vertex",
        )?;
        self.validate_indexes(
            self.displacements
                .iter()
                .flat_map(|displacement| &displacement.corner_neighbours)
                .flat_map(|corner| corner.neighbours()),
            &self.displacements,
            "displacement",
            "displacement",
        )?;
        self.validate_indexes(
            self.displacements
                .iter()
                .flat_map(|displacement| &displacement.edge_neighbours)
                .flat_map(|edge| edge.iter())
                .map(|sub| sub.neighbour_index),
            &self.displacements,
            "displacement",
            "displacement",
        )?;
        self.validate_indexes(
            self.faces.iter().map(|face| face.texture_info),
            &self.textures_info,
            "face",
            "texture_info",
        )?;
        self.validate_indexes(
            self.textures_info
                .iter()
                .map(|texture| texture.texture_data_index),
            &self.textures_data,
            "texture_info",
            "texture_data",
        )?;
        self.validate_indexes(
            self.textures_data
                .iter()
                .map(|texture| texture.name_string_table_id),
            &self.texture_string_tables,
            "textures_data",
            "texture_string_tables",
        )?;
        self.validate_indexes(
            self.texture_string_tables.iter().copied(),
            self.texture_string_data.as_bytes(),
            "texture_string_tables",
            "texture_string_data",
        )?;
        self.validate_indexes(
            self.nodes.iter().map(|node| node.plane_index),
            &self.planes,
            "node",
            "plane",
        )?;
        self.validate_indexes(
            self.nodes
                .iter()
                .flat_map(|node| node.children)
                .filter(|index| *index >= 0),
            &self.nodes,
            "node",
            "node",
        )?;
        self.validate_indexes(
            self.nodes
                .iter()
                .flat_map(|node| node.children)
                .filter_map(|index| (index < 0).then_some(!index)),
            &self.leaves,
            "node",
            "leaf",
        )?;
        self.validate_indexes(
            self.static_props().map(|prop| prop.prop_type),
            &self.static_props.dict.name,
            "static props",
            "static prop models",
        )?;
        self.validate_indexes(
            self.vertex_normal_indices.iter().map(|i| i.index),
            &self.vertex_normals,
            "vertex normal indices",
            "vertex normals",
        )?;

        if self.nodes.is_empty() {
            return Err(ValidationError::NoRootNode.into());
        }

        for face in &self.faces {
            if face.displacement_index().is_some() && face.num_edges != 4 {
                return Err(ValidationError::NonSquareDisplacement(face.num_edges).into());
            }
        }

        Ok(())
    }

    fn validate_indexes<
        'b,
        Index: TryInto<usize> + Into<i64> + Copy + Ord + Default,
        Indexes: Iterator<Item = Index>,
        T: 'b,
    >(
        &'b self,
        indexes: Indexes,
        list: &[T],
        source: &'static str,
        target: &'static str,
    ) -> BspResult<()> {
        let max = match indexes.max() {
            Some(max) => max,
            None => return Ok(()),
        };
        max.try_into()
            .ok()
            .and_then(|index| list.get(index))
            .ok_or_else(|| ValidationError::ReferenceOutOfRange {
                source_: source,
                target,
                index: max.into(),
                size: list.len(),
            })?;
        Ok(())
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

/// LZMA decompression with the header used by source
fn lzma_decompress_with_header(data: &[u8], expected_length: usize) -> Result<Vec<u8>, BspError> {
    // extra 8 byte because game lumps need some padding for reasons
    let mut output: Vec<u8> = Vec::with_capacity(min(expected_length + 8, 8 * 1024 * 1024));
    let mut cursor = Cursor::new(data);
    if b"LZMA" != &<[u8; 4]>::read(&mut cursor)? {
        return Err(BspError::LumpDecompressError(
            lzma_rs::error::Error::LzmaError("Invalid lzma header".into()),
        ));
    }
    let actual_size: u32 = cursor.read_le()?;
    let lzma_size: u32 = cursor.read_le()?;
    if data.len() < lzma_size as usize + 12 {
        return Err(BspError::UnexpectedCompressedLumpSize {
            got: data.len() as u32,
            expected: lzma_size,
        });
    }
    lzma_rs::lzma_decompress_with_options(
        &mut cursor,
        &mut output,
        &Options {
            unpacked_size: UnpackedSize::UseProvided(Some(actual_size as u64)),
            allow_incomplete: false,
            memlimit: None,
        },
    )
    .map_err(BspError::LumpDecompressError)?;
    if output.len() != expected_length {
        return Err(BspError::UnexpectedUncompressedLumpSize {
            got: output.len() as u32,
            expected: expected_length as u32,
        });
    }
    Ok(output)
}
