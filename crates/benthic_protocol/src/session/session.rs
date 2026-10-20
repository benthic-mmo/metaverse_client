use std::{
    fs::{File, create_dir_all},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

use serde::Serialize;
use uuid::Uuid;

use crate::errors::ProtocolSessionError;

/// flag for setting if caching is allowed.
pub static CACHE_ENABLED: AtomicBool = AtomicBool::new(true);
/// return the cache status, on or off.
pub fn cache_enabled() -> bool {
    CACHE_ENABLED.load(Ordering::Relaxed)
}
/// set the cache status
pub fn set_cache_enabled(enabled: bool) {
    CACHE_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Ensure a directory exists
fn create_sub_dir(base: &Path, name: &str) -> Result<PathBuf, ProtocolSessionError> {
    let dir = base.join(name);
    create_dir_all(&dir).map_err(|e| ProtocolSessionError::DirCreation {
        dir: dir.clone(),
        error: e,
    })?;
    Ok(dir)
}

/// Initialize the viewer's cache in the share dir on disk
pub fn initialize_share_dir() -> Result<PathBuf, ProtocolSessionError> {
    let data_dir = dirs::data_dir().ok_or(ProtocolSessionError::NotFound {})?;

    let share_dir = data_dir.join("benthic");

    create_dir_all(&share_dir).map_err(|e| ProtocolSessionError::DirCreation {
        dir: share_dir.clone(),
        error: e,
    })?;
    Ok(share_dir)
}

/// Create a subdirectory in the benthic share dir
pub fn create_sub_share_dir(root: &Path, name: &str) -> Result<PathBuf, ProtocolSessionError> {
    create_sub_dir(root, name)
}

/// Create a subdirectory for user agents
pub fn create_sub_agent_dir(root: &Path, name: &str) -> Result<PathBuf, ProtocolSessionError> {
    let agent_dir = create_sub_share_dir(root, "agent")?;
    create_sub_dir(&agent_dir, name)
}

/// Create a subdirectory for global objects
pub fn create_sub_object_dir(root: &Path, name: &str) -> Result<PathBuf, ProtocolSessionError> {
    let object_dir = create_sub_share_dir(root, "object")?;
    create_sub_dir(&object_dir, name)
}

/// Create a subdirectory for user agents
pub fn create_sub_land_dir(root: &Path) -> Result<PathBuf, ProtocolSessionError> {
    create_sub_share_dir(root, "land")
}

/// Create the global animations directory.
pub fn create_animation_dir(root: &Path) -> Result<PathBuf, ProtocolSessionError> {
    create_sub_share_dir(root, "animations")
}

/// Create the global animations directory.
pub fn create_textures_dir(root: &Path) -> Result<PathBuf, ProtocolSessionError> {
    create_sub_share_dir(root, "textures")
}

/// Create the global materials directory.
pub fn create_materials_dir(root: &Path) -> Result<PathBuf, ProtocolSessionError> {
    create_sub_share_dir(root, "materials")
}

/// Create the directory containing shared filtered animations.
pub fn create_filtered_animations_dir(root: &Path) -> Result<PathBuf, ProtocolSessionError> {
    let animations_dir = create_animation_dir(root)?;
    create_sub_dir(&animations_dir, "filtered_animations")
}

/// Create the directory for a specific filtered animation.
pub fn create_filtered_animation_dir(
    root: &Path,
    animation_id: &Uuid,
) -> Result<PathBuf, ProtocolSessionError> {
    let filtered_dir = create_filtered_animations_dir(root)?;
    create_sub_dir(&filtered_dir, &animation_id.to_string())
}

/// Create the directory containing agent-specific animations.
pub fn create_animation_agents_dir(root: &Path) -> Result<PathBuf, ProtocolSessionError> {
    let animations_dir = create_animation_dir(root)?;
    create_sub_dir(&animations_dir, "agents")
}

/// Create the directory for a specific agent's animations.
pub fn create_agent_animation_dir(
    root: &Path,
    agent_id: &Uuid,
) -> Result<PathBuf, ProtocolSessionError> {
    let agent_dir = create_sub_agent_dir(root, &agent_id.to_string())?;
    create_sub_dir(&agent_dir, "animations")
}

pub enum CacheDir {
    Agent(Uuid),
    Object(Uuid),
    Land,
}

pub fn write_json<T: Serialize>(
    data: &T,
    filename: &str,
    cache_dir: CacheDir,
    root: &Path,
) -> Result<PathBuf, ProtocolSessionError> {
    let dir = match cache_dir {
        CacheDir::Agent(id) => create_sub_agent_dir(root, &id.to_string())?,
        CacheDir::Object(id) => create_sub_object_dir(root, &id.to_string())?,
        CacheDir::Land => create_sub_land_dir(root)?,
    };

    let path = dir.join(format!("{filename}.json"));

    let json =
        serde_json::to_vec(data).map_err(|error| ProtocolSessionError::JsonWriteError { error })?;

    File::create(&path)?.write_all(&json)?;

    Ok(path)
}
