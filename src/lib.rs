mod buffer;
mod geometry;
mod greedy;
mod visible_faces;

use std::fmt::Debug;

pub use buffer::*;
pub use geometry::quad::*;
pub use geometry::shape::*;
pub use greedy::*;
pub use visible_faces::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoxelVisibility {
    Empty,
    Translucent,
    Opaque,
}

pub trait MeshVoxel {
    fn get_visibility(&self) -> VoxelVisibility;
}

pub trait MergeVoxel: MeshVoxel {
    type MergeValue: Eq;
    type MergeValueFacingNeighbour: Eq;

    fn merge_value(&self) -> Self::MergeValue;
    fn merge_value_facing_neighbour(&self) -> Self::MergeValueFacingNeighbour;
}
