use crate::errors::DownloadError;
use crate::utils::create_mesh_render_object;
use benthic_default_asset_converter::default_texture_path;
use benthic_protocol::avatar::types::AttachmentData;
use benthic_protocol::objects::types::RenderObject;
use benthic_protocol::session::session::CacheDir;
use benthic_protocol::session::session::cache_enabled;
use benthic_protocol::session::session::create_sub_agent_dir;
use benthic_protocol::session::session::write_json;
use image::{DynamicImage, ImageBuffer, Luma, LumaA, Rgb, Rgba};
use jpeg2k::{Image, ImagePixelData};
use log::info;
use metaverse_messages::http::mesh::Mesh;
use metaverse_messages::http::scene::SceneGroup;
use metaverse_messages::http::scene::SculptType;
use metaverse_messages::utils::object_types::ObjectType;
use metaverse_messages::utils::texture_entry::TextureEntries;
use metaverse_store::initialize_sqlite::Cache;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Sends a call to the ViewerAsset endpoint to retrieve the object
///
/// http://[UUID OF VIEWERASSET ENDPOINT]?[OBJECT TYPE]_id=[ASSET ID]
/// for example
/// http://da4b15ea-1d97-4140-afe3-2dd1ce5560710000?bodypart_id=da4b15ea-1d97-4140-afe3-2dd1ce5560710000
/// On success, returns the raw bytes downloaded from the endpoint.
///
/// #Parameters
/// - `asset_type`: Asset type used to construct the viewerasset request, such as "texture", or "mesh"
/// - `asset_id` : ID of the asset to retrieve
/// - `server_endpoint`: URL of the capability endpoint to make the request on
///
/// #Errors
/// - returns ['DownloadError'] if the request fails.
pub async fn download_asset(
    asset_type: String,
    asset_id: Uuid,
    server_endpoint: &str,
) -> Result<bytes::Bytes, DownloadError> {
    let client = awc::Client::default();
    let url = format!(
        "{}/?{}_id={}",
        server_endpoint.trim_end_matches('/'),
        asset_type,
        asset_id
    );
    let mut response = client.get(&url).send().await?;

    let status = response.status();

    if status == awc::http::StatusCode::NOT_FOUND {
        return Err(DownloadError::NotFound { url });
    }

    if !status.is_success() {
        return Err(DownloadError::HttpStatus {
            status: status.as_u16(),
            url,
        });
    }

    let body = response.body().await?;

    if body.is_empty() {
        return Err(DownloadError::EmptyBody {
            status: status.as_u16(),
            url,
            headers: vec![],
        });
    }

    Ok(body)
}

/// Retrieves and parses a texture from the capability endpoint.
///
/// This converts the retrieved JPEG2000 data into a regular PNG. This is necessary due to extremely
/// limited JPEG2K support pretty much everywhere.
///
/// #Parameters
/// - `asset_id`: The asset ID of the texture to retrieve
/// - `server_endpoint`: The server endpoint URL to send the request to
/// - `out_path`: The path to store the resulting image
///
/// #Errors
/// - returns [DownloadError] if the asset download fails, or the image cannot be decoded.
///
/// # Side Effects
/// [Storage: Filesystem]
pub async fn download_texture(
    asset_id: Uuid,
    server_endpoint: &str,
    out_path: &PathBuf,
) -> Result<(), DownloadError> {
    if out_path.exists() && cache_enabled() {
        return Ok(());
    }
    let item_type = ObjectType::Texture.to_string();
    let tex = &download_asset(item_type, asset_id, server_endpoint).await?;
    let img = Image::from_bytes(tex)?;
    let pixels = img.get_pixels(None)?;

    // Determine output format
    let output = match pixels.data {
        ImagePixelData::L8(data) => {
            ImageBuffer::<Luma<u8>, _>::from_raw(pixels.width, pixels.height, data)
                .map(DynamicImage::ImageLuma8)
                .unwrap()
        }
        ImagePixelData::La8(data) => {
            ImageBuffer::<LumaA<u8>, _>::from_raw(pixels.width, pixels.height, data)
                .map(DynamicImage::ImageLumaA8)
                .unwrap()
        }
        ImagePixelData::Rgb8(data) => {
            ImageBuffer::<Rgb<u8>, _>::from_raw(pixels.width, pixels.height, data)
                .map(DynamicImage::ImageRgb8)
                .unwrap()
        }
        ImagePixelData::Rgba8(data) => {
            ImageBuffer::<Rgba<u8>, _>::from_raw(pixels.width, pixels.height, data)
                .map(DynamicImage::ImageRgba8)
                .unwrap()
        }
        _ => return Err(DownloadError::UnknownPixelFormatError {}),
    };
    output.save(out_path)?;
    Ok(())
}

/// Retrieves per-face textures from the capabiilty endpoint.
///
/// Returns a hashmap linking the u32 face id to its texture's path on disk.
///
/// #Parameters
/// - `out_dir`: Path of the directory on disk to store the downloaded textures
/// - `texture`: TextureEntries object that contains the per-face texture information. If this is
///   None, the only value added will be the default texture to the default path.
/// - `server_endpoint`: URL of the capability endpoint to request
///
/// #Errors
/// - Returns [DownloadError] if downloads fail
///
/// # Side Effects
/// [Storage: Filesystem]
pub async fn download_face_textures(
    out_dir: &Path,
    texture: &Option<TextureEntries>,
    server_endpoint: &str,
) -> Result<HashMap<u32, PathBuf>, DownloadError> {
    let mut paths = HashMap::new();

    let Some(texture) = texture else {
        paths.insert(u32::MAX, default_texture_path());
        return Ok(paths);
    };

    let download = |texture_id: Uuid| {
        let texture_path = out_dir.join(format!("{:?}.png", texture_id));
        let server_endpoint = server_endpoint.to_owned();

        async move {
            match download_texture(texture_id, &server_endpoint, &texture_path).await {
                Ok(()) => Ok(texture_path),
                Err(DownloadError::NotFound { .. }) => Ok(default_texture_path()),
                Err(e) => Err(e),
            }
        }
    };

    let default_path = download(texture.default.texture_id).await?;
    paths.insert(u32::MAX, default_path);

    for (&face_index, face) in &texture.faces {
        let path = download(face.texture_id).await?;
        paths.insert(face_index, path);
    }

    Ok(paths)
}

/// Retrieves and parses a pbr material from the capability endpoint
///
/// TODO: this is a stub
///
///# Parameters
///- `material_id`: The ID to retrieve from the server
///- `server_endpoint`: the capability endpoint to retrieve from
///
/// #Errors
/// - returns [`DownloadError`] if downloads fail
pub async fn download_pbr_material(
    material_id: Uuid,
    server_endpoint: &str,
) -> Result<(), DownloadError> {
    let _material_bytes = download_asset(
        ObjectType::Material.to_string(),
        material_id,
        server_endpoint,
    )
    .await?;

    Ok(())
}

/// Retrieves and parses an asset in an avatar's outfit
///
/// TODO: currently only supports mesh objects. This needs to be generalized to handle other types of
/// prims.
///
/// - Writes render objects generated from the scene group to disk cache
/// - Updates sqlite cache with asset's JSON path.
///
/// # Parameters
/// - `server_endpoint`: URL of the capability endpoint to retrieve from
/// - `object_type`: Type of the object being retrieved from the server
/// - `asset_id`: ID of asset being retrieved from the server
/// - `cache`: session-wide cache object, containing the sqlite db connection and useful functions
/// - `out_dir`: root dir where output JSON will be written
///
/// # Errors
/// - returns [`DownloadError`] if downloads fail
///
/// # Side Effects
/// [Storage: Filesystem + SQLite]
pub async fn download_avatar_assets(
    server_endpoint: &str,
    object_type: ObjectType,
    asset_id: Uuid,
    agent_id: Uuid,
    cache: Cache,
    out_dir: PathBuf,
) -> Result<PathBuf, DownloadError> {
    let scene_group = SceneGroup::from_xml(
        &download_asset(object_type.to_string(), asset_id, server_endpoint).await?,
    )?;
    let base_dir = create_sub_agent_dir(&out_dir, &agent_id.to_string())?;

    let render_objects = download_scene_group_assets(
        &scene_group,
        server_endpoint,
        &base_dir,
        server_endpoint.to_string(),
    )
    .await?;

    let json_path = format!(
        "{:?}_{}",
        scene_group.parts[0].sculpt.texture, scene_group.parts[0].metadata.name
    );

    let json = if std::path::Path::new(&json_path).exists() || !cache_enabled() {
        PathBuf::from(json_path.clone())
    } else {
        write_json(
            &render_objects,
            &json_path,
            CacheDir::Agent(agent_id),
            &out_dir,
        )?
    };
    if cache_enabled()
        && let Err(e) = cache
            .avatar
            .update_outfit_item_json_path(asset_id, &json_path)
            .await
    {
        Err(e)?;
    }
    Ok(json)
}

/// Handles downloading scene group assets.
/// TODO: Needs to handle non-mesh assets.
async fn download_scene_group_assets(
    scene_group: &SceneGroup,
    url: &str,
    out_dir: &Path,
    server_endpoint: String,
) -> Result<Vec<RenderObject>, DownloadError> {
    let mut render_objects = Vec::new();

    for scene in &scene_group.parts {
        let texture_paths = download_face_textures(
            out_dir,
            &Some(scene.shape.texture.clone()),
            &server_endpoint,
        )
        .await?;

        if scene.sculpt.sculpt_type == SculptType::Mesh {
            let mesh = Mesh::from_bytes(
                &download_asset(ObjectType::Mesh.to_string(), scene.sculpt.texture, url).await?,
            )?;

            render_objects.push(create_mesh_render_object(
                mesh,
                scene.metadata.name.clone(),
                texture_paths,
                scene.sculpt.texture,
                &Some(scene.shape.texture.clone()),
                Some(AttachmentData {
                    attach_point: scene.shape.last_attach_point,
                    position: scene.attached_pos,
                    rotation: scene.rotation_offset,
                    scale: scene.scale,
                }),
                scene.shape.texture_animation.clone(),
            )?)
        } else {
            // TODO: handle non-mesh assets here
            info!("Non-mesh scene group")
        };
    }
    Ok(render_objects)
}
