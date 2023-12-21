use super::Handle;
use crate::data::*;
use arrayvec::ArrayVec;

impl<'a> Handle<'a, DisplacementInfo> {
    pub fn edge_neighbours(&self) -> impl Iterator<Item = Handle<'a, DisplacementSubNeighbour>> {
        self.data
            .edge_neighbours
            .iter()
            .flat_map(|edge| edge.iter())
            .map(|sub| Handle::new(self.bsp, sub))
    }

    pub fn corner_neighbours(&self) -> impl Iterator<Item = Handle<'a, DisplacementInfo>> {
        self.data
            .corner_neighbours
            .iter()
            .flat_map(|corner| corner.neighbours())
            .filter_map(|id| self.bsp.displacement(id as usize))
    }

    pub fn displacement_vertices(&self) -> impl Iterator<Item = Handle<'a, DisplacementVertex>> {
        (self.displacement_vertex_start..(self.displacement_vertex_start + self.vertex_count()))
            .flat_map(|i| self.bsp.displacement_vertex(i as usize))
    }

    pub fn face(&self) -> Option<Handle<'a, Face>> {
        self.bsp.face(self.map_face as usize)
    }

    /// Get the positions of the corners of the displaced face
    fn corner_positions(&self) -> [Vector; 4] {
        let face = self.face().unwrap();
        let vertices: [_; 4] = face
            .vertices()
            .collect::<ArrayVec<_, 4>>()
            .as_ref()
            .try_into()
            .unwrap();
        let mut corner_positions: [Vector; 4] = vertices.map(|v| v.position);

        // find the corner closest to the start position of the displacement
        let start_index = corner_positions
            .iter()
            .copied()
            .map(|point| point - self.start_position)
            .enumerate()
            .min_by(|(_a, a_pos), (_b, b_pos)| (a_pos).partial_cmp(b_pos).unwrap())
            .map(|(i, _pos)| i)
            .unwrap();

        corner_positions.rotate_left(start_index);
        corner_positions
    }

    fn subdivided_face(&self) -> impl Iterator<Item = Vector> + 'a {
        let steps = 2usize.pow(self.power as u32) + 1;
        let corner_positions = self.corner_positions();

        let step_scale = 1.0 / (steps as f32 - 1.0);
        let edge_intervals = [
            (corner_positions[1] - corner_positions[0]) * step_scale,
            (corner_positions[2] - corner_positions[3]) * step_scale,
        ];

        (0..steps)
            .flat_map(move |x| (0..steps).map(move |y| (x, y)))
            .map(move |(x, y)| {
                let edge_positions = [
                    corner_positions[0] + edge_intervals[0] * x as f32,
                    corner_positions[3] + edge_intervals[1] * x as f32,
                ];
                let segment_interval = (edge_positions[1] - edge_positions[0]) * step_scale;
                edge_positions[0] + (segment_interval * y as f32)
            })
    }

    pub fn displaced_vertices(&self) -> impl Iterator<Item = Vector> + 'a {
        self.displacement_vertices()
            .zip(self.subdivided_face())
            .map(move |(displacement, base_pos)| base_pos + displacement.displacement())
    }

    pub fn triangulated_displaced_vertices(&self) -> impl Iterator<Item = Vector> + 'a {
        let vertices: Vec<_> = self.displaced_vertices().collect();
        let steps = 2usize.pow(self.power as u32);

        let index = move |x: usize, y: usize| y * (steps + 1) + x;

        (0..steps)
            .flat_map(move |x| (0..steps).map(move |y| (x, y)))
            .flat_map(move |(x, y)| {
                [
                    vertices[index(x, y)],
                    vertices[index(x + 1, y)],
                    vertices[index(x, y + 1)],
                    vertices[index(x + 1, y)],
                    vertices[index(x + 1, y + 1)],
                    vertices[index(x, y + 1)],
                ]
            })
    }
}

impl<'a> Handle<'a, DisplacementSubNeighbour> {
    pub fn displacement(&self) -> Option<Handle<'a, DisplacementInfo>> {
        self.bsp.displacement(self.data.neighbour_index as usize)
    }
}
