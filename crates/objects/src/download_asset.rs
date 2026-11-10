use crate::errors::DownloadError;
use crate::utils::create_mesh_render_object;
use awc::Client;
use benthic_default_asset_converter::default_texture_path;
use benthic_protocol::avatar::types::AttachmentData;
use benthic_protocol::objects::types::RenderObject;
use benthic_protocol::session::rate_limiter::RateLimiter;
use benthic_protocol::session::storage::CacheDir;
use benthic_protocol::session::storage::cache_enabled;
use benthic_protocol::session::storage::create_sub_agent_dir;
use benthic_protocol::session::storage::write_json;
use image::{DynamicImage, ImageBuffer, Luma, LumaA, Rgb, Rgba};
use jpeg2k::{Image, ImagePixelData};
use ktx::{
    encode_params::{BasisParams, KTX_BASIS_CODEC_UASTC_LDR_4X4, KTX_BASIS_PARAMS_SIZE},
    texture::{TextureCreateInfo, TextureCreateStorage},
    texture2::Texture2,
    vkformat::VkFormat,
};
use log::info;
use metaverse_messages::http::mesh::Mesh;
use metaverse_messages::http::scene::SceneGroup;
use metaverse_messages::http::scene::SculptType;
use metaverse_messages::udp::object::util::PBRMaterial;
use metaverse_messages::utils::object_types::ObjectType;
use metaverse_messages::utils::texture_entry::TextureEntries;
use metaverse_store::initialize_sqlite::Cache;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
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
    client: Client,
    rate_limiter: Arc<RateLimiter>,
) -> Result<bytes::Bytes, DownloadError> {
    rate_limiter.acquire().await;

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

/// Downloads a mesh and writes the raw bytes to disk
///
/// Parameters
/// - `asset_id`: The asset ID of the mesh to receive
/// - `server_endpoint`: the server endpoint URL to send the request to
/// - `out_path`: The path to store the resulting bytes
pub async fn download_mesh(
    asset_id: Uuid,
    server_endpoint: &str,
    out_path: &PathBuf,
    client: Client,
    rate_limiter: Arc<RateLimiter>,
) -> Result<(), DownloadError> {
    let bytes = download_asset(
        ObjectType::Mesh.to_string(),
        asset_id,
        server_endpoint,
        client,
        rate_limiter,
    )
    .await?;

    tokio::fs::write(&out_path, bytes).await?;
    Ok(())
}

/// Downloads a material and writes the raw bytes to disk
///
/// Parameters
/// - `asset_id`: The asset ID of the mesh to receive
/// - `server_endpoint`: the server endpoint URL to send the request to
/// - `out_path`: The path to store the resulting bytes
pub async fn download_material(
    asset_id: Uuid,
    server_endpoint: &str,
    out_path: &PathBuf,
    client: Client,
    rate_limiter: Arc<RateLimiter>,
) -> Result<(), DownloadError> {
    let bytes = download_asset(
        ObjectType::Material.to_string(),
        asset_id,
        server_endpoint,
        client,
        rate_limiter,
    )
    .await?;

    tokio::fs::write(&out_path, bytes).await?;
    Ok(())
}

pub async fn download_object(
    asset_id: Uuid,
    server_endpoint: &str,
    out_path: &PathBuf,
    client: Client,
    rate_limiter: Arc<RateLimiter>,
) -> Result<(), DownloadError> {
    let bytes = download_asset(
        ObjectType::Object.to_string(),
        asset_id,
        server_endpoint,
        client,
        rate_limiter,
    )
    .await?;

    tokio::fs::write(&out_path, bytes).await?;
    Ok(())
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
    client: Client,
    rate_limiter: Arc<RateLimiter>,
) -> Result<(), DownloadError> {
    if out_path.exists() && cache_enabled() {
        return Ok(());
    }
    let item_type = ObjectType::Texture.to_string();
    let tex = &download_asset(item_type, asset_id, server_endpoint, client, rate_limiter).await?;
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

pub async fn download_ktx2_texture(
    asset_id: Uuid,
    server_endpoint: &str,
    out_path: &PathBuf,
    client: Client,
    rate_limiter: Arc<RateLimiter>,
) -> Result<(), DownloadError> {
    if out_path.exists() && cache_enabled() {
        return Ok(());
    }

    let item_type = ObjectType::Texture.to_string();
    let tex = download_asset(item_type, asset_id, server_endpoint, client, rate_limiter).await?;

    let img = Image::from_bytes(&tex)?;
    let pixels = img.get_pixels(None)?;

    // Normalize source pixels to RGBA8 in memory.
    let output = match pixels.data {
        ImagePixelData::L8(data) => {
            ImageBuffer::<Luma<u8>, _>::from_raw(pixels.width, pixels.height, data)
                .map(DynamicImage::ImageLuma8)
                .ok_or(DownloadError::UnknownPixelFormatError {})?
        }
        ImagePixelData::La8(data) => {
            ImageBuffer::<LumaA<u8>, _>::from_raw(pixels.width, pixels.height, data)
                .map(DynamicImage::ImageLumaA8)
                .ok_or(DownloadError::UnknownPixelFormatError {})?
        }
        ImagePixelData::Rgb8(data) => {
            ImageBuffer::<Rgb<u8>, _>::from_raw(pixels.width, pixels.height, data)
                .map(DynamicImage::ImageRgb8)
                .ok_or(DownloadError::UnknownPixelFormatError {})?
        }
        ImagePixelData::Rgba8(data) => {
            ImageBuffer::<Rgba<u8>, _>::from_raw(pixels.width, pixels.height, data)
                .map(DynamicImage::ImageRgba8)
                .ok_or(DownloadError::UnknownPixelFormatError {})?
        }
        _ => return Err(DownloadError::UnknownPixelFormatError {}),
    };

    let rgba = output.to_rgba8();

    // Create an uncompressed RGBA8 KTX2 texture.
    let create_info = TextureCreateInfo {
        gl_internalformat: 0,
        vk_format: VkFormat::R8G8B8A8_UNORM,
        dfd: None,
        base_width: pixels.width,
        base_height: pixels.height,
        base_depth: 1,
        num_dimensions: 2,
        num_levels: 1,
        num_layers: 1,
        num_faces: 1,
        is_array: false,
        generate_mipmaps: false,
    };

    let mut texture = Texture2::create(&create_info, TextureCreateStorage::AllocStorage)?;

    // Copy the decoded pixels into the KTX2 texture.
    texture.set_image_from_memory(0, 0, 0, rgba.as_raw())?;

    // Configure Basis Universal UASTC encoding.
    let params = BasisParams {
        struct_size: KTX_BASIS_PARAMS_SIZE,
        codec: KTX_BASIS_CODEC_UASTC_LDR_4X4,
        ..Default::default()
    };

    texture.compress_basis_ex(&params)?;

    // Write the compressed KTX2 file.
    texture.write_to_named_file(out_path)?;

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
    client: Client,
    rate_limiter: Arc<RateLimiter>,
) -> Result<HashMap<u32, PathBuf>, DownloadError> {
    let mut paths = HashMap::new();

    let Some(texture) = texture else {
        paths.insert(u32::MAX, default_texture_path());
        return Ok(paths);
    };

    let download = |texture_id: Uuid| {
        let texture_path = out_dir.join(format!("{:?}.ktx2", texture_id));
        let server_endpoint = server_endpoint.to_owned();
        let client = client.clone();
        let rate_limiter = rate_limiter.clone();

        async move {
            match download_texture(
                texture_id,
                &server_endpoint,
                &texture_path,
                client,
                rate_limiter,
            )
            .await
            {
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
