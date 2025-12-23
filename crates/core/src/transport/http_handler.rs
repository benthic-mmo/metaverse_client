use glam::{Quat, Vec3, Vec4};
use image::{DynamicImage, ImageBuffer, Luma, LumaA, Rgb, Rgba};
use jpeg2k::{Image, ImagePixelData};
use metaverse_agent::skeleton::create_skeleton;
use metaverse_messages::http::login::login_error::{LoginError, Reason};
use metaverse_messages::http::login::login_response::{LoginResponse, LoginStatus};
use metaverse_messages::http::login::simulator_login_protocol::SimulatorLoginProtocol;
use metaverse_messages::http::mesh::Mesh;
use metaverse_messages::http::{item::Item, scene::SceneGroup};
use metaverse_messages::ui::login_event::Login;
use metaverse_messages::utils::object_types::ObjectType;
use metaverse_messages::utils::render_data::{RenderObject, SkinData};
use metaverse_messages::utils::skeleton::Skeleton;
use std::io::Error;
use std::path::PathBuf;
use uuid::Uuid;

/// send the login to simulator xml-rpc request
pub async fn login_to_simulator(
    login: Login,
) -> Result<(LoginResponse, std::net::IpAddr), LoginError> {
    let url = login.url.clone();

    // determine the IP the login was sent on. This needs to be stored in order to properly send
    // the CircuitCode UDP packet. The server checks to ensure that the IP of the CircuitCode
    // packet is the same as the IP that send the login.
    // Determine local IP used for login
    let local_ip = {
        let parsed_url = url::Url::parse(&url).map_err(|e| LoginError {
            reason: Reason::Connection,
            message: format!("Invalid URL: {:?}", e),
        })?;

        let host = parsed_url.host_str().ok_or(LoginError {
            reason: Reason::Connection,
            message: "URL has no host".into(),
        })?;

        let port = parsed_url.port_or_known_default().ok_or(LoginError {
            reason: Reason::Connection,
            message: "URL has no port".into(),
        })?;

        // Open a temporary TCP connection to get local IP
        let stream = std::net::TcpStream::connect((host, port)).map_err(|e| LoginError {
            reason: Reason::Connection,
            message: format!("TCP connect failed: {:?}", e),
        })?;

        stream
            .local_addr()
            .map(|addr| addr.ip())
            .map_err(|e| LoginError {
                reason: Reason::Connection,
                message: format!("Failed to get local IP: {:?}", e),
            })?
    };

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
            LoginStatus::Success(success) => Ok((*success, local_ip)),
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
    path: &PathBuf,
) -> std::io::Result<DynamicImage> {
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
        _ => return Err(Error::other("Unknown pixel format".to_string())),
    };
    output.save(&path).unwrap();
    Ok(output)
}
fn io_error(msg: &str, err: impl std::fmt::Debug) -> std::io::Error {
    Error::other(format!("{}: {:?}", msg, err))
}

/// Download a scenegroup object. This is done by retrieving all of th meshes in the scenegroup and
/// calling download_renderable_mesh on them one by one, and then building a vector of created
/// meshes.
pub async fn download_scene_group(
    scene_group: &SceneGroup,
    url: &str,
    texture_path: &PathBuf,
) -> Result<Vec<RenderObject>, std::io::Error> {
    let mut meshes = Vec::new();
    for scene in &scene_group.parts {
        meshes.push(
            download_renderable_mesh(
                scene.sculpt.texture,
                scene.metadata.name.clone(),
                scene.scale,
                scene.rotation_offset,
                url,
                texture_path,
            )
            .await?,
        );
    }
    Ok(meshes)
}

fn apply_scale_rotation(v: Vec3, scale: Vec3, rotation: Quat) -> Vec3 {
    rotation * (v * scale)
}

/// retrieves mesh data and does operations on the received data to ready it for the metaverse_mesh
/// crate.
pub async fn download_renderable_mesh(
    asset_id: Uuid,
    name: String,
    scale: Vec3,
    rotation: Quat,
    url: &str,
    texture_path: &PathBuf,
) -> Result<RenderObject, std::io::Error> {
    let mesh = download_mesh(ObjectType::Mesh.to_string(), asset_id, url).await?;
    let domain = &mesh.high_level_of_detail.texture_coordinate_domain;
    let uvs: Vec<[f32; 2]> = mesh
        .high_level_of_detail
        .texture_coordinate
        .iter()
        .map(|tc| {
            // Normalize U and V from 0..65535 to 0..1
            let u_norm = tc.u as f32 / 65535.0;
            let v_norm = tc.v as f32 / 65535.0;

            // Flip V axis
            let v_flipped = 1.0 - v_norm;

            [
                domain.min[0] + u_norm * (domain.max[0] - domain.min[0]),
                domain.min[1] + v_flipped * (domain.max[1] - domain.min[1]),
            ]
        })
        .collect();

    let object = if let Some(skin) = &mesh.skin {
        // Apply bind shape matrix
        let vertices: Vec<Vec3> = mesh
            .high_level_of_detail
            .vertices
            .iter()
            .map(|v| {
                let v4 = skin.bind_shape_matrix * Vec4::new(v.x, v.y, v.z, 1.0);
                Vec3::new(v4.x, v4.y, v4.z)
            })
            .collect();

        let skeleton = create_skeleton(name.clone(), asset_id, skin).unwrap_or_else(|e| {
            println!("Failed to create skeleton: {:?}", e);
            Skeleton::default()
        });

        let skin_data = SkinData {
            skeleton,
            weights: mesh
                .high_level_of_detail
                .weights
                .clone()
                .unwrap_or_default(),
            joint_names: skin.joint_names.clone(),
            inverse_bind_matrices: skin.inverse_bind_matrices.clone(),
        };

        RenderObject {
            name: name,
            id: asset_id,
            indices: mesh.high_level_of_detail.indices,
            vertices,
            skin: Some(skin_data),
            texture: Some(texture_path.clone()),
            uv: Some(uvs),
        }
    } else {
        let vertices: Vec<Vec3> = mesh
            .high_level_of_detail
            .vertices
            .iter()
            .map(|v| apply_scale_rotation(*v, scale, rotation))
            .collect();

        RenderObject {
            name: name,
            id: asset_id,
            indices: mesh.high_level_of_detail.indices,
            vertices,
            skin: None,
            texture: Some(texture_path.clone()),
            uv: Some(uvs),
        }
    };

    Ok(object)
}
