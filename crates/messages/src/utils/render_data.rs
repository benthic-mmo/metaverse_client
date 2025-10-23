use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::{http::mesh::JointWeight, utils::skeleton::Skeleton};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// This contains required data that will be used for rendering.
pub struct RenderObject {
    /// Name of the object to render
    pub name: String,
    /// full list of vertices
    pub vertices: Vec<Vec3>,
    /// full list of indices
    /// This contains information on where in the triangle each of your vertices are. This saves
    /// space by not duplicating vertices and allows the renderer to handle building the triangles.
    pub indices: Vec<u16>,
    /// The skeleton of the object.
    pub skin: Option<SkinData>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// Contains skinning information for RenderObjects
pub struct SkinData {
    /// Joint information for the mesh
    pub skeleton: Skeleton,
    /// Weight information for the mesh's joints
    pub weights: Vec<JointWeight>,
}
