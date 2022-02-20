use crate::data::*;
use crate::Bsp;
use arrayvec::ArrayVec;
use std::ops::Deref;

/// A handle represents a data structure in the bsp file and the bsp file containing it.
///
/// Keeping a reference of the bsp file with the data is required since a lot of data types
/// reference parts from other structures in the bsp file
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

impl<'a, T> AsRef<T> for Handle<'a, T> {
    fn as_ref(&self) -> &'a T {
        self.data
    }
}

impl<T> Deref for Handle<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, T> Handle<'a, T> {
    pub fn new(bsp: &'a Bsp, data: &'a T) -> Self {
        Handle { bsp, data }
    }
}

impl<'a> Handle<'a, Model> {
    /// Get all faces that make up the model
    pub fn faces(&self) -> impl Iterator<Item = Handle<'a, Face>> {
        let start = self.first_face as usize;
        let end = start + self.face_count as usize;
        let bsp = self.bsp;

        bsp.faces[start..end]
            .iter()
            .map(move |face| Handle::new(bsp, face))
    }
}

impl<'a> Handle<'a, TextureInfo> {
    /// Get the texture data references by the texture
    pub fn texture(&self) -> Option<&TextureData> {
        self.bsp
            .textures_data
            .get(self.data.texture_data_index as usize)
    }
}

impl<'a> Handle<'a, Face> {
    /// Get the texture of the face
    pub fn texture(&self) -> Option<Handle<TextureInfo>> {
        self.bsp
            .textures_info
            .get(self.texture_info as usize)
            .map(|texture_info| Handle {
                bsp: self.bsp,
                data: texture_info,
            })
    }

    /// Get all vertices making up the face
    pub fn vertices(&self) -> impl Iterator<Item = &'a Vertex> + 'a {
        let bsp = self.bsp;
        self.vertex_indexes()
            .flat_map(move |vert_index| bsp.vertices.get(vert_index as usize))
    }

    /// Get the vertex indexes of all vertices making up the face
    ///
    /// The indexes index into the `vertices` field of the bsp file
    pub fn vertex_indexes(&self) -> impl Iterator<Item = u16> + 'a {
        let bsp = self.bsp;
        (self.data.first_edge..(self.data.first_edge + self.data.num_edges as i32))
            .flat_map(move |surface_edge| bsp.surface_edges.get(surface_edge as usize))
            .flat_map(move |surface_edge| {
                bsp.edges
                    .get(surface_edge.edge_index())
                    .map(|edge| (edge, surface_edge.direction()))
            })
            .map(|(edge, direction)| match direction {
                EdgeDirection::FirstToLast => edge.start_index,
                EdgeDirection::LastToFirst => edge.end_index,
            })
    }

    pub fn edge_direction(&self) -> EdgeDirection {
        self.bsp.surface_edges[self.first_edge as usize].direction()
    }

    /// Check if the face is flagged as visible
    pub fn is_visible(&self) -> bool {
        self.texture()
            .map(|texture| {
                !texture.flags.intersects(
                    TextureFlags::LIGHT
                        | TextureFlags::SKY2D
                        | TextureFlags::SKY
                        | TextureFlags::WARP
                        | TextureFlags::TRIGGER
                        | TextureFlags::HINT
                        | TextureFlags::SKIP
                        | TextureFlags::NODRAW
                        | TextureFlags::HITBOX,
                )
            })
            .unwrap_or_default()
    }

    /// Triangulate the face
    ///
    /// Triangulation only works for faces that can be turned into a triangle fan trivially
    pub fn triangulate(&self) -> impl Iterator<Item = [Vector; 3]> + 'a {
        let mut vertices = self.vertices();

        let a = vertices.next().expect("face with <3 points");
        let mut b = vertices.next().expect("face with <3 points");

        vertices.map(move |c| {
            let points = [a.position, b.position, c.position];
            b = c;
            points
        })
    }

    pub fn displacement(&self) -> Option<Handle<'a, DisplacementInfo>> {
        self.bsp.displacement(self.displacement_info as usize)
    }
}

impl Handle<'_, Node> {
    /// Get the plane splitting this node
    pub fn plane(&self) -> Option<Handle<'_, Plane>> {
        self.bsp.plane(self.plane_index as _)
    }
}

impl<'a> Handle<'a, Leaf> {
    /// Get all other leaves visible from this one
    pub fn visible_set(&self) -> Option<impl Iterator<Item = Handle<'a, Leaf>>> {
        let cluster = self.cluster;
        let bsp = self.bsp;

        if cluster < 0 {
            None
        } else {
            let visible_clusters = bsp.vis_data.visible_clusters(cluster);
            Some(
                bsp.leaves
                    .iter()
                    .filter(move |leaf| {
                        if leaf.cluster == cluster {
                            true
                        } else if leaf.cluster > 0 {
                            visible_clusters[leaf.cluster as u64]
                        } else {
                            false
                        }
                    })
                    .map(move |leaf| Handle { bsp, data: leaf }),
            )
        }
    }

    /// Get all faces in this leaf
    pub fn faces(&self) -> impl Iterator<Item = Handle<'a, Face>> {
        let start = self.first_leaf_face as usize;
        let end = start + self.leaf_face_count as usize;
        let bsp = self.bsp;
        bsp.leaf_faces[start..end]
            .iter()
            .filter_map(move |leaf_face| bsp.face(leaf_face.face as usize))
    }
}

impl<'a> Handle<'a, DisplacementInfo> {
    pub fn edge_neighbours(&self) -> impl Iterator<Item = Handle<'a, DisplacementSubNeighbour>> {
        self.data
            .edge_neighbours
            .iter()
            .flat_map(|edge| &edge.sub_neighbours)
            .filter_map(|sub| sub.as_ref())
            .map(|sub| Handle::new(self.bsp, sub))
    }

    pub fn corner_neighbours(&self) -> impl Iterator<Item = Handle<'a, DisplacementInfo>> {
        self.data
            .corner_neighbours
            .iter()
            .flat_map(|corner| &corner.neighbours[0..corner.neighbour_count.min(4) as usize])
            .copied()
            .filter_map(|id| self.bsp.displacement(id as usize))
    }

    pub fn displacement_vertices(&self) -> impl Iterator<Item = Handle<'a, DisplacementVertex>> {
        (self.displacement_vertex_start..(self.displacement_vertex_start + self.vertex_count()))
            .flat_map(|i| self.bsp.displacement_vertex(i as usize))
    }

    pub fn face(&self) -> Option<Handle<'a, Face>> {
        self.bsp.face(self.map_face as usize)
    }

    pub fn displaced_vertices(&self) -> Option<impl Iterator<Item = Vector> + 'a> {
        let face = self.face()?;
        let steps = 2usize.pow(self.power as u32) + 1;

        let mut corner_positions: ArrayVec<_, 4> = face.vertices().map(|v| v.position).collect();

        let start_index = corner_positions
            .iter()
            .copied()
            .map(|point| point - self.start_position)
            .enumerate()
            .min_by(|(_a, a_pos), (_b, b_pos)| (a_pos).partial_cmp(b_pos).unwrap())
            .map(|(i, _pos)| i)
            .unwrap();

        corner_positions.rotate_left(start_index);

        let step_scale = 1.0 / (steps as f32 - 1.0);
        let edge_intervals = [
            (corner_positions[1] - corner_positions[0]) * step_scale,
            (corner_positions[2] - corner_positions[3]) * step_scale,
        ];

        Some(
            self.displacement_vertices()
                .enumerate()
                .map(move |(i, displacement)| {
                    let y = (i % steps) as f32;
                    let x = (i / steps) as f32;
                    let edge_positions = [
                        corner_positions[0] + edge_intervals[0] * x,
                        corner_positions[3] + edge_intervals[1] * x,
                    ];
                    let segment_interval = (edge_positions[1] - edge_positions[0]) * step_scale;
                    let base_pos = edge_positions[0] + (segment_interval * y);
                    base_pos + displacement.displacement()
                }),
        )
    }

    pub fn triangulated_displaced_vertices(&self) -> Option<impl Iterator<Item = Vector> + 'a> {
        let vertices: Vec<_> = self.displaced_vertices()?.collect();
        let steps = 2usize.pow(self.power as u32);

        let index = move |x: usize, y: usize| y * (steps + 1) + x;

        Some(
            (0..steps)
                .flat_map(move |x| (0..steps).map(move |y| (x, y)))
                .flat_map(move |(x, y)| {
                    [
                        vertices[index(x, y)],
                        vertices[index(x + 1, y)],
                        vertices[index(x, y + 1)],
                        vertices[index(x, y + 1)],
                        vertices[index(x + 1, y + 1)],
                        vertices[index(x + 1, y)],
                    ]
                }),
        )
    }
}

impl<'a> Handle<'a, DisplacementSubNeighbour> {
    pub fn displacement(&self) -> Option<Handle<'a, DisplacementInfo>> {
        self.bsp.displacement(self.data.neighbour_index as usize)
    }
}
