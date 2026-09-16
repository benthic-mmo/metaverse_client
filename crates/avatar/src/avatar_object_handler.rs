use std::{collections::BTreeSet, fs, path::PathBuf};

use benthic_protocol::{
    render_data::{AvatarObject, RenderObject},
    session::{CacheDir, Session, create_sub_agent_dir, write_json},
    skeleton::{JointName, Skeleton},
};
use metaverse_mesh::mesh::generate::generate_skinned_mesh;
use uuid::Uuid;

use crate::{
    avatar::{Avatar, OutfitObject},
    avatar_object_handler::AvatarState::{FullyLoaded, Incomplete},
    errors::AvatarError,
    skeleton::update_global_avatar_skeleton,
};

pub enum AvatarState {
    FullyLoaded,
    Incomplete,
}

pub enum AvatarType {
    User,
    NonUser,
}

pub fn init_avatar<C, L, U>(
    session: &mut Session<C, Avatar, L, U>,
    avatar: &Avatar,
) -> Result<AvatarType, AvatarError> {
    if session.agent_id == avatar.agent_id {
        // if the session has initialized its inventory
        // insert the avatar and return the camera position
        if session.inventory_data.inventory_init {
            session.avatars.insert(avatar.agent_id, avatar.clone());
            Ok(AvatarType::User)
        } else {
            // if not, return an InventoryUninitialized error
            Err(AvatarError::InventoryUninitialized {})
        }
    } else {
        Err(AvatarError::Unimplemented {
            feature: "Non-Agent avatars".to_string(),
        })
    }
}

pub fn add_object_to_avatar<C, L, U>(
    session: &mut Session<C, Avatar, L, U>,
    agent_id: Uuid,
    object: OutfitObject,
) -> Result<AvatarState, AvatarError> {
    let avatar = session
        .avatars
        .get_mut(&agent_id)
        .ok_or(AvatarError::AgentNotFound { agent: agent_id })?;
    match &object {
        OutfitObject::MeshObject(path) => {
            let file = fs::File::open(path)?;
            let parts: Vec<RenderObject> = serde_json::from_reader(file)?;

            if let Some(skin) = &parts[0].skin {
                update_global_avatar_skeleton(avatar, &skin.skeleton);
            }
        }
        _ => {
            //TODO: unimplemented
            //warn!("Avatars wearing non-mesh objects are currently not supported.")
        }
    }
    avatar.items.push(object);
    if avatar.items.len() == avatar.outfit_size {
        Ok(FullyLoaded)
    } else {
        Ok(Incomplete)
    }
}

pub async fn finalize_avatar(
    agent_id: Uuid,
    skeleton: Skeleton,
    used_joints: BTreeSet<JointName>,
    items: Vec<OutfitObject>,
) -> Result<PathBuf, AvatarError> {
    let json_paths: Vec<PathBuf> = items
        .into_iter()
        .filter_map(|item| {
            if let OutfitObject::MeshObject(path) = item {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    let avatar_object = AvatarObject {
        objects: json_paths,
        global_skeleton: skeleton,
        used_joints: used_joints.clone(),
    };

    let json_path_str = agent_id.to_string();
    let json_path = PathBuf::from(&json_path_str);

    let json_path = if json_path.exists() {
        json_path.clone()
    } else {
        write_json(&avatar_object, &json_path_str, CacheDir::Agent(agent_id))?
    };

    let base_dir = create_sub_agent_dir(&agent_id.to_string())?;
    let glb_path = base_dir.join(format!("{:?}_high.glb", agent_id));
    if !glb_path.exists() {
        generate_skinned_mesh(json_path.clone(), glb_path.clone())?
    }

    Ok(glb_path)
}
