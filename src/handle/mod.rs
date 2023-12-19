mod displacement;
mod face;
mod game;

use crate::data::*;
use crate::Bsp;
use ahash::RandomState;
use std::fmt::{Debug, Formatter};
use std::hash::BuildHasher;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

/// A handle represents a data structure in the bsp file and the bsp file containing it.
///
/// Keeping a reference of the bsp file with the data is required since a lot of data types
/// reference parts from other structures in the bsp file
pub struct Handle<'a, T> {
    bsp: &'a Bsp,
    data: &'a T,
}

impl<T: Debug> Debug for Handle<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle")
            .field("data", self.data)
            .finish_non_exhaustive()
    }
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

    pub fn textures(&self) -> impl Iterator<Item = Handle<'_, TextureInfo>> {
        self.bsp.textures()
    }
}

impl Handle<'_, Node> {
    /// Get the plane splitting this node
    pub fn plane(&self) -> Handle<'_, Plane> {
        self.bsp.plane(self.plane_index as _).unwrap()
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

impl<'a> Handle<'a, TextureInfo> {
    pub fn texture_data(&self) -> Handle<'a, TextureData> {
        Handle::new(
            self.bsp,
            &self.bsp.textures_data[self.data.texture_data_index as usize],
        )
    }
    pub fn name(&self) -> &'a str {
        self.texture_data().name()
    }

    /// Get a color that is unique but deterministic for this texture
    pub fn debug_color(&self) -> [u8; 3] {
        self.texture_data().debug_color()
    }

    pub fn u(&self, pos: Vector) -> f32 {
        (self.texture_transforms_u[0] * pos.x
            + self.texture_transforms_u[1] * pos.y
            + self.texture_transforms_u[2] * pos.z
            + self.texture_transforms_u[3])
            / self.texture_data().width as f32
    }

    pub fn v(&self, pos: Vector) -> f32 {
        (self.texture_transforms_v[0] * pos.x
            + self.texture_transforms_v[1] * pos.y
            + self.texture_transforms_v[2] * pos.z
            + self.texture_transforms_v[3])
            / self.texture_data().height as f32
    }

    pub fn uv(&self, pos: Vector) -> [f32; 2] {
        [self.u(pos), self.v(pos)]
    }
}

impl<'a> Handle<'a, TextureData> {
    pub fn name(&self) -> &'a str {
        let start = self.bsp.texture_string_tables[self.name_string_table_id as usize] as usize;
        let part = &self.bsp.texture_string_data[start..];
        if let Some((s, _)) = part.split_once('\0') {
            s
        } else {
            part
        }
    }

    /// Get a color that is unique but determistic for this texture
    pub fn debug_color(&self) -> [u8; 3] {
        let mut name_hasher = RandomState::with_seeds(0, 0, 0, 0).build_hasher();
        self.name().hash(&mut name_hasher);
        let name_hash = name_hasher.finish().to_be_bytes();
        [name_hash[0], name_hash[1], name_hash[2]]
    }
}
