use std::path::PathBuf;

use crate::{
    errors::DownloadError,
    http_handler::{download_object, download_scene_group},
    object_handler::handle_texture,
};
use benthic_protocol::session::{CacheDir, cache_enabled, create_sub_agent_dir, write_json};
use log::warn;
use metaverse_messages::utils::object_types::ObjectType;
use metaverse_store::initialize_sqlite::Cache;
use uuid::Uuid;

pub async fn download_asset_objects(
    server_endpoint: &str,
    object_type: ObjectType,
    asset_id: Uuid,
    agent_id: Uuid,
    cache: Cache,
    out_dir: PathBuf,
) -> Result<PathBuf, DownloadError> {
    let scene_group = download_object(object_type.to_string(), asset_id, server_endpoint).await?;
    let base_dir = create_sub_agent_dir(&out_dir, &agent_id.to_string())?;

    let render_objects = download_scene_group(
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
        warn!("{:?}", e);
    }
    Ok(json)
}
