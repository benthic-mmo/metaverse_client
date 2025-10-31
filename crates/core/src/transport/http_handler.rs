use image::{DynamicImage, ImageBuffer, Luma, LumaA, Rgb, Rgba};
use jpeg2k::{Image, ImagePixelData};
use metaverse_messages::http::login::login_error::{LoginError, Reason};
use metaverse_messages::http::login::login_response::{LoginResponse, LoginStatus};
use metaverse_messages::http::login::simulator_login_protocol::SimulatorLoginProtocol;
use metaverse_messages::http::mesh::Mesh;
use metaverse_messages::http::{item::Item, scene::SceneGroup};
use metaverse_messages::ui::login_event::Login;
use std::io::Error;
use uuid::Uuid;

/// send the login to simulator xml-rpc request
pub async fn login_to_simulator(login: Login) -> Result<LoginResponse, LoginError> {
    let url = login.url.clone();
    let client = awc::Client::default();
    // Serialize login data to XML-RPC
    let xml = SimulatorLoginProtocol::new(login).to_xmlrpc();
    // Send POST request
    let mut response = client
        .post(&url)
        .insert_header(("Content-Type", "text/xml; charset=utf-8"))
        .send_body(xml)
        .await
        .map_err(|e| LoginError {
            reason: Reason::Connection,
            message: format!("{:?}", e),
        })?;

    let body_bytes = response.body().await.map_err(|e| LoginError {
        reason: Reason::Connection,
        message: format!("{:?}", e),
    })?;

    let xml_string = String::from_utf8(body_bytes.to_vec()).unwrap();
    // Parse XML-RPC response

    match LoginResponse::from_xmlrpc(&xml_string) {
        Ok(login) => match login {
            LoginStatus::Success(success) => Ok(*success),
            LoginStatus::Failure(failure) => Err(failure),
        },
        Err(e) => Err(e)?,
    }
}

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
) -> std::io::Result<bytes::Bytes> {
    let client = awc::Client::default();
    let url = format!("{}?{}_id={}", server_endpoint, item_type, asset_id);
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
        return Err(Error::other("Empty response body"));
    }
    Ok(body_bytes)
}

/// retrieve an Object from the ViewerAsset endpoint.
/// this needs to be parsed as a SceneGroup.
pub async fn download_object(
    item_type: String,
    asset_id: Uuid,
    server_endpoint: &str,
) -> std::io::Result<SceneGroup> {
    SceneGroup::from_xml(&download_asset(item_type, asset_id, server_endpoint).await?)
        .map_err(|e| Error::other(format!("Failed to parse object: {}", e)))
}

/// Retrieve an inventory item from the ViewerAsset endpoint.
/// this needs to be parsed as an Item object
pub async fn download_item(
    item_type: String,
    asset_id: Uuid,
    server_endpoint: &str,
) -> std::io::Result<Item> {
    Item::from_bytes(&download_asset(item_type, asset_id, server_endpoint).await?)
        .map_err(|e| Error::other(format!("Failed to parse item: {}", e)))
}

/// Retrieve a mesh from the ViewerAsset endpoint.
/// This needs to be parsed as a Mesh object.
pub async fn download_mesh(
    item_type: String,
    asset_id: Uuid,
    server_endpoint: &str,
) -> std::io::Result<Mesh> {
    Mesh::from_bytes(&download_asset(item_type, asset_id, server_endpoint).await?)
        .map_err(|e| Error::other(format!("Failed to parse SceneGroup XML: {}", e)))
}

/// Retrieve a texture from the ViewerAsset endpoint.
pub async fn download_texture(
    item_type: String,
    asset_id: Uuid,
    server_endpoint: &str,
) -> std::io::Result<DynamicImage> {
    let tex = &download_asset(item_type, asset_id, server_endpoint).await?;
    let img = Image::from_bytes(tex).unwrap();
    let pixels = img.get_pixels(None).unwrap();
    // Determine output format

    match pixels.data {
        ImagePixelData::L8(data) => {
            Ok(
                ImageBuffer::<Luma<u8>, _>::from_raw(pixels.width, pixels.height, data)
                    .map(DynamicImage::ImageLuma8)
                    .unwrap(),
            )
        }
        ImagePixelData::La8(data) => {
            Ok(
                ImageBuffer::<LumaA<u8>, _>::from_raw(pixels.width, pixels.height, data)
                    .map(DynamicImage::ImageLumaA8)
                    .unwrap(),
            )
        }
        ImagePixelData::Rgb8(data) => {
            Ok(
                ImageBuffer::<Rgb<u8>, _>::from_raw(pixels.width, pixels.height, data)
                    .map(DynamicImage::ImageRgb8)
                    .unwrap(),
            )
        }
        ImagePixelData::Rgba8(data) => {
            Ok(
                ImageBuffer::<Rgba<u8>, _>::from_raw(pixels.width, pixels.height, data)
                    .map(DynamicImage::ImageRgba8)
                    .unwrap(),
            )
        }
        _ => Err(Error::other(format!("Unknown pixel format"))),
    }
}

fn io_error(msg: &str, err: impl std::fmt::Debug) -> std::io::Error {
    Error::other(format!("{}: {:?}", msg, err))
}
