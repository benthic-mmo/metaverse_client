use crate::errors::DownloadError;
use benthic_default_asset_converter::default_texture_path;
use benthic_protocol::{
    avatar::{
        skeleton::Skeleton,
        types::{AttachmentData, SkinData},
    },
    objects::types::{
        BlinnPhongMaterial, MaterialModel, RenderFace, RenderObject, TextureAnimation,
        TextureTransform,
    },
};
use log::warn;
use metaverse_avatar::skeleton::create_skeleton;
use metaverse_messages::{http::mesh::Mesh, utils::texture_entry::TextureEntries};
use std::{collections::HashMap, path::PathBuf};
use uuid::Uuid;

/// Creates a render object out of mesh data
///
/// # Parameters
/// - `mesh`: The mesh object to create the render object for
/// - `name`: The name of the object
/// - `texture_paths`: a hashmap associating the u32 face id with the path of its texture on disk
/// - `asset_id`: UUID of the mesh's asset. This is found in the object's sculpt texture value.
/// - `texture`: An optional list of texture entries, containing relevant texture data for building
///   a material of type [`MaterialModel`]
///- `attachment_data`: Optional attachment data, if this object is attached to an avatar
///- `texture_animation`: Optional texture animation data if this object has animated textures
///
///# Errors
///- returns [`DownloadError`] on failure
pub fn create_mesh_render_object(
    mesh: Mesh,
    name: String,
    texture_paths: HashMap<u32, PathBuf>,
    asset_id: Uuid,
    texture: &Option<TextureEntries>,
    attachment_data: Option<AttachmentData>,
    texture_animation: Option<TextureAnimation>,
) -> Result<RenderObject, DownloadError> {
    let skin = mesh.skin.as_ref();

    let faces = mesh
        .high_level_of_detail
        .iter()
        .enumerate()
        .filter(|(_, face)| !face.no_geometry)
        .map(|(face_index, face)| {
            let domain = &face.texture_coordinate_domain;

            let uv = face
                .texture_coordinate
                .iter()
                .map(|tc| {
                    let u = tc.u as f32 / 65535.0;
                    let v = tc.v as f32 / 65535.0;

                    [
                        domain.min[0] + u * (domain.max[0] - domain.min[0]),
                        1.0 - (domain.min[1] + v * (domain.max[1] - domain.min[1])),
                    ]
                })
                .collect();

            let vertices = face
                .vertices
                .iter()
                .map(|vertex| {
                    skin.map_or(*vertex, |skin| {
                        skin.bind_shape_matrix.transform_point3(*vertex)
                    })
                })
                .collect();

            let material = texture.as_ref().map(|entries| {
                let face = entries
                    .faces
                    .get(&(face_index as u32))
                    .unwrap_or(&entries.default);
                let tex_animation = texture_animation
                    .as_ref()
                    .and_then(|tex_anim| match tex_anim.face {
                        f if f == face_index as i8 || f == -1 => Some(tex_anim.clone()),
                        _ => None,
                    });

                MaterialModel::BlinnPhong(BlinnPhongMaterial {
                    texture_path: texture_paths
                        .get(&(face_index as u32))
                        .or_else(|| texture_paths.get(&u32::MAX))
                        .cloned()
                        .unwrap_or_else(default_texture_path),
                    texture_animation: tex_animation,
                    bump: face.bumpmap,
                    color: face.rgba,
                    fullbright: face.fullbright,
                    glow: face.glow,
                    shiny: face.shiny,
                    tex_gen: face.tex_gen,
                    texture_transform: TextureTransform {
                        repeat_u: face.repeat_u,
                        repeat_v: face.repeat_v,
                        offset_u: face.offset_u,
                        offset_v: face.offset_v,
                        rotation: face.rotation,
                    },
                })
            });

            RenderFace {
                face_index: face_index as u32,
                vertices,
                indices: face.indices.clone(),
                uv,
                normals: face.normals.clone(),
                weights: face.weights.clone(),
                material,
            }
        })
        .collect();

    let skin_data = if let Some(skin) = skin {
        let skeleton = create_skeleton(name.clone(), asset_id, skin).unwrap_or_else(|e| {
            warn!("Failed to create skeleton: {:?}", e);
            Skeleton::default()
        });

        Some(SkinData {
            skeleton,
            weights: mesh
                .high_level_of_detail
                .iter()
                .filter(|face| !face.no_geometry)
                .flat_map(|face| face.weights.clone().unwrap_or_default())
                .collect(),
            joint_names: skin.joints.clone(),
            inverse_bind_matrices: skin.inverse_bind_matrices.clone(),
        })
    } else {
        None
    };

    Ok(RenderObject {
        name,
        id: asset_id,
        faces,
        attachment_data,
        skin: skin_data,
    })
}
