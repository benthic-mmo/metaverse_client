use std::path::PathBuf;

use glam::Vec3;
use metaverse_messages::capabilities::item::Item;
use metaverse_messages::capabilities::scene_object::SceneGroup;
use metaverse_messages::{
    capabilities::{item::ItemData, mesh_data::Mesh, scene_object::SceneObject},
    utils::{item_metadata::ItemMetadata, object_types::ObjectType},
};
use std::io::{Error, ErrorKind};

/// Sends a call to the ViewerAsset endpoint to retrieve the object using the object's asset ID.
/// Creates a get request in the format of
/// http://<UUID OF VIEWERASSET ENDPOINT>?<OBJECT TYPE>_id=<ASSET ID>
/// for example
/// http://da4b15ea-1d97-4140-afe3-2dd1ce5560710000?bodypart_id=da4b15ea-1d97-4140-afe3-2dd1ce5560710000
/// If successful, this returns bytes that contain the object's information.
pub async fn download_asset(
    metadata: ItemMetadata,
    path: PathBuf,
    server_endpoint: &String,
) -> std::io::Result<Item> {
    let client = awc::Client::default();

    let url = format!(
        "{}?{}_id={}",
        server_endpoint,
        metadata.item_type.to_string(),
        metadata.asset_id
    );

    match client.get(url).send().await {
        Ok(mut response) => match response.body().await {
            Ok(body_bytes) => match metadata.item_type {
                ObjectType::Object => {
                    let scene_group = SceneGroup::from_xml(&body_bytes).map_err(|e| {
                        Error::new(
                            ErrorKind::Other,
                            format!("Failed to parse scene object: {}", e),
                        )
                    })?;
                    println!("scene group is {:?}", scene_group);
                    let url = format!(
                        "{}?{}_id={}",
                        server_endpoint,
                        ObjectType::Mesh.to_string(),
                        scene_group.root.sculpt.texture
                    );
                    match client.get(url).send().await {
                        Ok(mut response) => match response.body().await {
                            Ok(body_bytes) => {
                                let mesh = Mesh::from_bytes(&body_bytes)?;
                                Ok(Item {
                                    metadata,
                                    data: Some(ItemData {
                                        mesh: Some(mesh),
                                        ..Default::default()
                                    }),
                                })
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
                }
                _ => {
                    if body_bytes != "" {
                        Ok(Item {
                            metadata,
                            data: Some(ItemData::from_bytes(&body_bytes)?),
                        })
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
