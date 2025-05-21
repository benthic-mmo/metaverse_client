use std::path::PathBuf;

use metaverse_messages::{
    capabilities::{item_data::ItemData, mesh_data::Mesh, scene_object::SceneObject},
    core::object_update::ObjectUpdate,
    utils::object_types::ObjectType,
};
use serde_llsd::de::xml;
use std::io::{Error, ErrorKind};
use uuid::Uuid;

/// Sends a call to the ViewerAsset endpoint to retrieve the object using the object's asset ID.
/// Creates a get request in the format of
/// http://<UUID OF VIEWERASSET ENDPOINT>?<OBJECT TYPE>_id=<ASSET ID>
/// for example
/// http://da4b15ea-1d97-4140-afe3-2dd1ce5560710000?bodypart_id=da4b15ea-1d97-4140-afe3-2dd1ce5560710000
/// If successful, this returns bytes that contain the object's information.
pub async fn download_asset(
    object_type: ObjectType,
    asset_id: Uuid,
    path: PathBuf,
    server_endpoint: &String,
) -> std::io::Result<ItemData> {
    let client = awc::Client::default();

    let url = format!(
        "{}?{}_id={}",
        server_endpoint,
        object_type.to_string(),
        asset_id
    );

    match client.get(url).send().await {
        Ok(mut response) => match response.body().await {
            Ok(body_bytes) => match object_type {
                ObjectType::Object => {
                    let scene_object = SceneObject::from_xml(&body_bytes);
                    if let Ok(scene) = scene_object {
                        let url = format!(
                            "{}?{}_id={}",
                            server_endpoint,
                            ObjectType::Mesh.to_string(),
                            scene.sculpt.texture
                        );
                        match client.get(url).send().await {
                            Ok(mut response) => match response.body().await {
                                Ok(body_bytes) => {
                                    Mesh::from_bytes(&body_bytes)?;
                                    ItemData::from_bytes(&body_bytes)
                                }
                                Err(e) => Err(Error::new(
                                    ErrorKind::Other,
                                    format!("Failed to retrieve mesh {:?}", e),
                                )),
                            },
                            Err(e) => Err(Error::new(
                                ErrorKind::Other,
                                format!("Failed to retrieve mesh {:?}", e),
                            )),
                        }
                    } else {
                        Err(Error::new(
                            ErrorKind::Other,
                            format!("Failed to parse scene object"),
                        ))
                    }
                }
                _ => {
                    if body_bytes != "" {
                        ItemData::from_bytes(&body_bytes)
                    } else {
                        Err(Error::new(ErrorKind::Other, "Failed to parse item data"))
                    }
                }
            },
            Err(e) => Err(Error::new(
                ErrorKind::Other,
                format!("Failed to read body {:?}", e),
            )),
        },
        Err(e) => Err(Error::new(
            ErrorKind::Other,
            format!("Failed to send http get request {:?}", e),
        )),
    }
}
