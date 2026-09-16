use std::path::PathBuf;

use crate::{
    errors::DownloadError,
    http_handler::{download_object, download_scene_group},
    object_handler::handle_texture,
};
use benthic_protocol::session::{CacheDir, create_sub_agent_dir, write_json};
use log::warn;
use metaverse_cache::agent::sqlite_update_outfit_item_json_path;
use metaverse_messages::utils::object_types::ObjectType;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

pub async fn download_asset_objects(
    server_endpoint: &str,
    db_conn: Pool<Sqlite>,
    object_type: ObjectType,
    asset_id: Uuid,
    agent_id: Uuid,
) -> Result<PathBuf, DownloadError> {
    let scene_group = download_object(object_type.to_string(), asset_id, server_endpoint).await?;
    let base_dir = create_sub_agent_dir(&agent_id.to_string())?;

    let texture_id = scene_group.parts[0].shape.texture.texture_id;
    let texture_path = handle_texture(base_dir, texture_id, server_endpoint.to_string()).await;
    let render_objects = download_scene_group(&scene_group, server_endpoint, &texture_path).await?;

    let json_path = format!(
        "{:?}_{}",
        scene_group.parts[0].sculpt.texture, scene_group.parts[0].metadata.name
    );

    let json = if std::path::Path::new(&json_path).exists() {
        PathBuf::from(json_path.clone())
    } else {
        write_json(&render_objects, &json_path, CacheDir::Agent(agent_id))?
    };

    if let Err(e) = sqlite_update_outfit_item_json_path(&db_conn, asset_id, &json_path).await {
        warn!("{:?}", e);
    }
    Ok(json)
}
