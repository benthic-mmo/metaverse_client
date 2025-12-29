use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use crate::packet::message::UIMessage;

/// this is the struct for sending mesh updates from the core to the UI.
/// the path is the path to the generated gltf file, and the position is where to place it in the
/// world.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MeshUpdate {
    /// path to the generated gtlf file that contains the mesh
    pub path: PathBuf,
    /// Scale of the mesh
    pub scale: Vec3,
    /// rotation of the mesh
    pub rotation: Quat,

    pub parent: Option<u32>,
    pub scene_id: Option<u32>,

    /// Where to render the mesh
    pub position: Vec3,
    /// The type of mesh getting rendered. Land, Object, Avatar, etc.
    pub mesh_type: MeshType,
    /// ID of the mesh. For agents, this will be the AgentID.
    pub id: Option<Uuid>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
/// Type of mesh the UI is rendering.
pub enum MeshType {
    /// Land type
    Land,
    /// Avatar type
    #[default]
    Avatar,
}

impl UIMessage {
    /// creates a new MeshUpdate message
    /// converts from y-up to z-up for ease of rendering.
    pub fn new_mesh_update(mut data: MeshUpdate) -> Self {
        data.rotation = yup_to_zup_rotation(data.rotation);
        data.position = yup_to_zup_vec3(data.position);
        data.scale = yup_to_zup_vec3(data.scale);
        UIMessage::MeshUpdate(data)
    }
}

fn yup_to_zup_rotation(q: Quat) -> Quat {
    let q_convert = Quat::from_axis_angle(Vec3::X, -std::f32::consts::FRAC_PI_2);

    q_convert * q * q_convert.inverse()
}
fn yup_to_zup_vec3(s: Vec3) -> Vec3 {
    Vec3::new(s.x, s.z, s.y)
}
