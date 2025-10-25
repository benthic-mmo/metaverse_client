use std::path::PathBuf;

use glam::{Mat4, Vec3};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    http::mesh::JointWeight,
    utils::skeleton::{JointName, Skeleton},
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// This contains required data that will be used for rendering.
pub struct RenderObject {
    /// Name of the object to render
    pub name: String,
    /// ID of the object to render
    pub id: Uuid,
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
    /// Names of all of the joints. In order to correspond to weights and inverse bind matrices.
    pub joint_names: Vec<JointName>,
    /// Inverse bind matrix. Used to determine the shape of the skeleton in relation to the mesh.
    pub inverse_bind_matrices: Vec<Mat4>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// Object that contains the global skeleton for the agent object,  
pub struct AvatarObject {
    /// Path to the RenderObject json on file for the component parts of the outfit
    pub objects: Vec<PathBuf>,
    /// global skeleton tat is applied to all objects in the outfit
    pub global_skeleton: Skeleton,
}
