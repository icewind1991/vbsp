use super::Handle;
use crate::data::*;

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
