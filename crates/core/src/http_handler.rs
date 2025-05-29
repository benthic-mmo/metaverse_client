use metaverse_messages::capabilities::mesh::Mesh;
use metaverse_messages::{
    capabilities::{item::Item, scene::SceneGroup},
    utils::item_metadata::ItemMetadata,
};
use std::io::{Error, ErrorKind};

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
        return Err(Error::new(ErrorKind::Other, "Empty response body"));
    }
    Ok(body_bytes)
}

/// retrieve an Object from the ViewerAsset endpoint.
/// this needs to be parsed as a SceneGroup.
pub async fn download_object(
    metadata: ItemMetadata,
    server_endpoint: &str,
) -> std::io::Result<SceneGroup> {
    SceneGroup::from_xml(&download_asset(metadata, server_endpoint).await?).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to parse SceneGroup XML: {}", e),
        )
    })
}

/// Retrieve an inventory item from the ViewerAsset endpoint.
/// this needs to be parsed as an Item object
pub async fn download_item(metadata: ItemMetadata, server_endpoint: &str) -> std::io::Result<Item> {
    Item::from_bytes(&download_asset(metadata, server_endpoint).await?).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to parse SceneGroup XML: {}", e),
        )
    })
}

/// Retrieve a mesh from the ViewerAsset endpoint.
/// This needs to be parsed as a Mesh object.
pub async fn download_mesh(metadata: ItemMetadata, server_endpoint: &str) -> std::io::Result<Mesh> {
    Mesh::from_bytes(&download_asset(metadata, server_endpoint).await?).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Failed to parse SceneGroup XML: {}", e),
        )
    })
}

fn io_error(msg: &str, err: impl std::fmt::Debug) -> std::io::Error {
    Error::new(ErrorKind::Other, format!("{}: {:?}", msg, err))
}
