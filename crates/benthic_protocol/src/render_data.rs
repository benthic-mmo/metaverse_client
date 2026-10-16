use std::{collections::BTreeSet, path::PathBuf};

use glam::{Mat4, Quat, Vec3};
use rgb::Rgba;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    objects::AttachPoint,
    skeleton::{Skeleton, SkinJoint},
};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
/// Information about the weights of each joint.
/// This corresponds to each vertex in the mesh
pub struct JointWeight {
    /// The index of the joint the vertex corresponds to
    pub indices: [u8; 4],
    /// How strongly the joint influences the vertex
    pub weights: [f32; 4],
    /// The name of the joint that the weight corresponds to.
    pub joint_name: [SkinJoint; 4],
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RenderFace {
    pub face_index: u32,
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub uv: Vec<[f32; 2]>,
    pub normals: Option<Vec<Vec3>>,
    pub texture: Option<PathBuf>,
    pub weights: Option<Vec<JointWeight>>,

    pub color: Rgba<u8>,
    // TODO: Implement these soon
    //
    //pub shiny: Option<ShinyLevel>,
    //pub fullbright: bool,
    //pub glow: f32,
    //pub bump: Option<BumpMapping>,
    //pub texture_transform: Option<TextureTransform>,
    //pub alpha_mode: Option<AlphaMode>,
}
/// This contains required data that will be used for rendering.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RenderObject {
    pub name: String,
    pub id: Uuid,
    pub faces: Vec<RenderFace>,
    pub attachment_data: Option<AttachmentData>,
    pub skin: Option<SkinData>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AttachmentData {
    pub attach_point: AttachPoint,
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// Contains skinning information for RenderObjects
pub struct SkinData {
    /// Joint information for the mesh
    pub skeleton: Skeleton,
    /// Weight information for the mesh's joints
    pub weights: Vec<JointWeight>,
    /// Names of all of the joints. In order to correspond to weights and inverse bind matrices.
    pub joint_names: Vec<SkinJoint>,
    /// Inverse bind matrix. Used to determine the shape of the skeleton in relation to the mesh.
    pub inverse_bind_matrices: Vec<Mat4>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// Object that contains the global skeleton for the agent object,  
pub struct AvatarObject {
    /// Path to the RenderObject json on file for the component parts of the outfit
    pub objects: Vec<PathBuf>,
    /// global skeleton that is applied to all objects in the outfit
    pub global_skeleton: Skeleton,
    /// List of joints used by the avatar
    pub used_joints: BTreeSet<SkinJoint>,
}
