use benthic_protocol::render_data::RenderObject;
use image::{DynamicImage, ImageBuffer, Luma, LumaA, Rgb, Rgba};
use jpeg2k::{Image, ImagePixelData};
use metaverse_messages::http::mesh::Mesh;
use metaverse_messages::http::{item::Item, scene::SceneGroup};
use metaverse_messages::utils::object_types::ObjectType;
use std::io::Error;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::errors::DownloadError;
use crate::object_handler::create_render_object;

/// Sends a call to the ViewerAsset endpoint to retrieve the object using the object's asset ID.
/// Creates a get request in the format of
/// http://[UUID OF VIEWERASSET ENDPOINT]?[OBJECT TYPE]_id=[ASSET ID]
/// for example
/// http://da4b15ea-1d97-4140-afe3-2dd1ce5560710000?bodypart_id=da4b15ea-1d97-4140-afe3-2dd1ce5560710000
/// If successful, this returns bytes that contain the object's information.
pub async fn download_asset(
    item_type: String,
    asset_id: Uuid,
    server_endpoint: &str,
) -> Result<bytes::Bytes, DownloadError> {
    let client = awc::Client::default();
    let url = format!("{}/?{}_id={}", server_endpoint, item_type, asset_id);
    let mut response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| io_error("Failed to send HTTP GET request", e))?;

    let body_bytes = response
        .body()
        .await
        .map_err(|e| io_error("Failed to read response body", e))?;
    if body_bytes.is_empty() {
        return Err(DownloadError::EmptyBody {});
    }
    Ok(body_bytes)
}

/// retrieve an Object from the ViewerAsset endpoint.
/// this needs to be parsed as a SceneGroup.
pub async fn download_object(
    item_type: String,
    asset_id: Uuid,
    server_endpoint: &str,
) -> Result<SceneGroup, DownloadError> {
    Ok(SceneGroup::from_xml(
        &download_asset(item_type, asset_id, server_endpoint).await?,
    )?)
}

/// Retrieve an inventory item from the ViewerAsset endpoint.
/// this needs to be parsed as an Item object
pub async fn download_item(
    item_type: String,
    asset_id: Uuid,
    server_endpoint: &str,
) -> Result<Item, DownloadError> {
    Ok(Item::from_bytes(
        &download_asset(item_type, asset_id, server_endpoint).await?,
    )?)
}

/// Retrieve a mesh from the ViewerAsset endpoint.
/// This needs to be parsed as a Mesh object.
pub async fn download_mesh(
    item_type: String,
    asset_id: Uuid,
    server_endpoint: &str,
) -> Result<Mesh, DownloadError> {
    Ok(Mesh::from_bytes(
        &download_asset(item_type, asset_id, server_endpoint).await?,
    )?)
}

/// Retrieve a texture from the ViewerAsset endpoint.
pub async fn download_texture(
    item_type: String,
    asset_id: Uuid,
    server_endpoint: &str,
    path: &PathBuf,
) -> Result<(), DownloadError> {
    let tex = &download_asset(item_type, asset_id, server_endpoint).await?;
    let img = Image::from_bytes(tex).unwrap();
    let pixels = img.get_pixels(None).unwrap();

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
    output.save(path).unwrap();
    Ok(())
}
fn io_error(msg: &str, err: impl std::fmt::Debug) -> std::io::Error {
    Error::other(format!("{}: {:?}", msg, err))
}

/// Download a scenegroup object. This is done by retrieving all of the meshes in the scenegroup and
/// calling download_renderable_mesh on them one by one, and then building a vector of created
/// meshes.
pub async fn download_scene_group(
    scene_group: &SceneGroup,
    url: &str,
    texture_path: &Path,
) -> Result<Vec<RenderObject>, DownloadError> {
    let mut render_objects = Vec::new();
    for scene in &scene_group.parts {
        let mesh = download_mesh(ObjectType::Mesh.to_string(), scene.sculpt.texture, url).await?;
        render_objects.push(create_render_object(
            mesh,
            scene.metadata.name.clone(),
            texture_path,
            scene.sculpt.texture,
        )?);
    }
    Ok(render_objects)
}
