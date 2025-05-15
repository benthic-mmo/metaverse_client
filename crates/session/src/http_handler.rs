use log::error;
use metaverse_messages::utils::object_types::ObjectType;
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
    server_endpoint: &String,
) -> std::io::Result<String> {
    let client = awc::Client::default();
    let url = format!(
        "{}?{}_id={}",
        server_endpoint,
        object_type.to_string(),
        asset_id
    );

    match client.get(url).send().await {
        Ok(mut response) => match response.body().await {
            Ok(body_bytes) => {
                println!("wearable: {:?}", body_bytes);
            }
            Err(e) => {
                error!("Failed to read body: {:?}", e);
            }
        },
        Err(e) => {
            error!("Failed to send with {:?}", e);
        }
    };
    Ok("aaa".to_string())
}
