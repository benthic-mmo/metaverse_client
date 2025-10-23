use metaverse_messages::http::login::login_error::{LoginError, Reason};
use metaverse_messages::http::login::login_response::{LoginResponse, LoginStatus};
use metaverse_messages::http::login::simulator_login_protocol::SimulatorLoginProtocol;
use metaverse_messages::http::mesh::Mesh;
use metaverse_messages::ui::login_event::Login;
use metaverse_messages::{
    http::{item::Item, scene::SceneGroup},
    utils::item_metadata::ItemMetadata,
};
use std::io::Error;

pub async fn login_to_simulator(login: Login) -> Result<LoginResponse, LoginError> {
    let url = login.url.clone();
    let client = awc::Client::default();
    // Serialize login data to XML-RPC
    let xml = SimulatorLoginProtocol::new(login).to_xml();
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

    match LoginResponse::from_xml(&xml_string) {
        Ok(login) => match login {
            LoginStatus::Success(success) => Ok(success),
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
    metadata: ItemMetadata,
    server_endpoint: &str,
) -> std::io::Result<bytes::Bytes> {
    let client = awc::Client::default();
    let item_type = metadata.item_type.to_string();
    let asset_id = metadata.asset_id;
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
    metadata: ItemMetadata,
    server_endpoint: &str,
) -> std::io::Result<SceneGroup> {
    SceneGroup::from_xml(&download_asset(metadata, server_endpoint).await?)
        .map_err(|e| Error::other(format!("Failed to parse object: {}", e)))
}

/// Retrieve an inventory item from the ViewerAsset endpoint.
/// this needs to be parsed as an Item object
pub async fn download_item(metadata: ItemMetadata, server_endpoint: &str) -> std::io::Result<Item> {
    Item::from_bytes(&download_asset(metadata, server_endpoint).await?)
        .map_err(|e| Error::other(format!("Failed to parse item: {}", e)))
}

/// Retrieve a mesh from the ViewerAsset endpoint.
/// This needs to be parsed as a Mesh object.
pub async fn download_mesh(metadata: ItemMetadata, server_endpoint: &str) -> std::io::Result<Mesh> {
    Mesh::from_bytes(&download_asset(metadata, server_endpoint).await?)
        .map_err(|e| Error::other(format!("Failed to parse SceneGroup XML: {}", e)))
}

fn io_error(msg: &str, err: impl std::fmt::Debug) -> std::io::Error {
    Error::other(format!("{}: {:?}", msg, err))
}
