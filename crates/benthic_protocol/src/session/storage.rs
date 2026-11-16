use crate::errors::ProtocolSessionError;
use serde::Serialize;
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
use uuid::Uuid;

pub static CACHE_ENABLED: AtomicBool = AtomicBool::new(true);
pub fn cache_enabled() -> bool {
    CACHE_ENABLED.load(Ordering::Relaxed)
}
pub fn set_cache_enabled(enabled: bool) {
    CACHE_ENABLED.store(enabled, Ordering::Relaxed);
}

// prevents passing an agent id where an object id is expected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct AgentId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct ObjectId(pub Uuid);

impl AgentId {
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }
}
impl ObjectId {
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }
}
impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl std::fmt::Display for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone)]
pub struct BenthicDirs {
    root: PathBuf,
}

impl BenthicDirs {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn agents_root(&self) -> PathBuf {
        self.root.join("Avatars")
    }
    pub fn objects_root(&self) -> PathBuf {
        self.root.join("Objects")
    }
    pub fn object_updates_root(&self) -> PathBuf {
        self.root.join("ObjectUpdates")
    }
    pub fn textures_root(&self) -> PathBuf {
        self.root.join("Textures")
    }
    pub fn materials_root(&self) -> PathBuf {
        self.root.join("Materials")
    }
    pub fn animations_root(&self) -> PathBuf {
        self.root.join("Animations")
    }
    pub fn land_root(&self) -> PathBuf {
        self.root.join("Land")
    }
    pub fn agent_dir(&self, agent: AgentId) -> PathBuf {
        self.agents_root().join(agent.to_string())
    }

    pub fn ensure_agent_dir(&self, agent: AgentId) -> Result<PathBuf, ProtocolSessionError> {
        let dir = self.agent_dir(agent);
        ensure_dir(&dir)?;
        Ok(dir)
    }
    pub fn agent_object_file(&self, agent: AgentId, object: ObjectId, ext: &str) -> PathBuf {
        self.agent_dir(agent).join(format!("{}.{ext}", object))
    }
    pub fn agent_object_xml(&self, agent: AgentId, object: ObjectId) -> PathBuf {
        self.agent_object_file(agent, object, "xml")
    }
    pub fn agent_object_json(&self, agent: AgentId, object: ObjectId) -> PathBuf {
        self.agent_object_file(agent, object, "json")
    }
    pub fn agent_object_gltf(&self, agent: AgentId, object: ObjectId) -> PathBuf {
        self.agent_object_file(agent, object, "gltf")
    }
    pub fn agent_high_gltf(&self, agent: AgentId) -> PathBuf {
        self.agent_dir(agent).join(format!("{agent}_high.gltf"))
    }
    pub fn agent_finalized_json(&self, agent: AgentId) -> PathBuf {
        self.agent_dir(agent)
            .join(format!("{agent}_finalized.json"))
    }
    pub fn agent_avatar_object_json(&self, agent: AgentId) -> PathBuf {
        self.agent_dir(agent)
            .join(format!("{agent}_avatar_object.json"))
    }

    pub fn object_dir(&self, sculpt: ObjectId) -> PathBuf {
        self.objects_root().join(sculpt.to_string())
    }
    pub fn ensure_object_dir(&self, sculpt: ObjectId) -> Result<PathBuf, ProtocolSessionError> {
        let dir = self.object_dir(sculpt);
        ensure_dir(&dir)?;
        Ok(dir)
    }
    pub fn object_json(&self, sculpt: ObjectId) -> PathBuf {
        self.object_dir(sculpt).join(format!("{sculpt}.json"))
    }

    pub fn object_high_gltf(&self, sculpt: ObjectId) -> PathBuf {
        self.object_dir(sculpt).join(format!("{sculpt}_high.gltf"))
    }
    pub fn object_mesh_file(&self, sculpt: ObjectId) -> PathBuf {
        self.object_dir(sculpt).join(format!("{sculpt}.mesh"))
    }

    pub fn texture_file(&self, texture_id: Uuid) -> PathBuf {
        self.textures_root().join(format!("{texture_id}.png"))
    }
    pub fn ensure_textures_dir(&self) -> Result<PathBuf, ProtocolSessionError> {
        let dir = self.textures_root();
        ensure_dir(&dir)?;
        Ok(dir)
    }

    pub fn ensure_materials_dir(&self) -> Result<PathBuf, ProtocolSessionError> {
        let dir = self.materials_root();
        ensure_dir(&dir)?;
        Ok(dir)
    }
    pub fn material_file(&self, material_id: Uuid) -> PathBuf {
        self.materials_root().join(format!("{material_id}.json"))
    }

    pub fn filtered_animations_dir(&self) -> PathBuf {
        self.animations_root().join("filtered_animations")
    }

    pub fn ensure_filtered_animations_dir(&self) -> Result<PathBuf, ProtocolSessionError> {
        let dir = self.filtered_animations_dir();
        ensure_dir(&dir)?;
        Ok(dir)
    }

    pub fn filtered_animation_dir(&self, animation_id: Uuid) -> PathBuf {
        self.filtered_animations_dir()
            .join(animation_id.to_string())
    }

    pub fn ensure_filtered_animation_dir(
        &self,
        animation_id: Uuid,
    ) -> Result<PathBuf, ProtocolSessionError> {
        let dir = self.filtered_animation_dir(animation_id);
        ensure_dir(&dir)?;
        Ok(dir)
    }

    // Per-agent animation directory: Avatars/<agent_id>/animations/
    pub fn agent_animation_dir(&self, agent: AgentId) -> PathBuf {
        self.agent_dir(agent).join("animations")
    }

    pub fn ensure_agent_animation_dir(
        &self,
        agent: AgentId,
    ) -> Result<PathBuf, ProtocolSessionError> {
        let dir = self.agent_animation_dir(agent);
        ensure_dir(&dir)?;
        Ok(dir)
    }

    /// Per-prim cache dir, keyed by a 64-bit hash: Objects/prim_<hash>/
    pub fn prim_dir(&self, cache_key: u64) -> PathBuf {
        self.objects_root().join(format!("prim_{cache_key:016x}"))
    }

    pub fn ensure_prim_dir(&self, cache_key: u64) -> Result<PathBuf, ProtocolSessionError> {
        let dir = self.prim_dir(cache_key);
        ensure_dir(&dir)?;
        Ok(dir)
    }

    pub fn prim_json(&self, cache_key: u64) -> PathBuf {
        self.prim_dir(cache_key).join(format!("{cache_key}.json"))
    }

    pub fn prim_gltf(&self, cache_key: u64) -> PathBuf {
        self.prim_dir(cache_key).join(format!("{cache_key}.gltf"))
    }

    pub fn empty_object_dir(&self) -> PathBuf {
        self.objects_root().join("empty")
    }
    pub fn ensure_empty_object_dir(&self) -> Result<PathBuf, ProtocolSessionError> {
        let dir = self.empty_object_dir();
        ensure_dir(&dir)?;
        Ok(dir)
    }
    pub fn object_update_json(&self, object: ObjectId) -> PathBuf {
        self.object_updates_root()
            .join(format!("{object}_ObjectUpdate.json"))
    }

    pub fn ensure_object_updates_dir(&self) -> Result<PathBuf, ProtocolSessionError> {
        let dir = self.object_updates_root();
        ensure_dir(&dir)?;
        Ok(dir)
    }

    pub fn inventory_db(&self) -> PathBuf {
        self.root.join("inventory.db")
    }

    pub fn login_conf_json(&self) -> PathBuf {
        self.root.join("login_conf.json")
    }

    pub fn ensure_layout(&self) -> Result<(), ProtocolSessionError> {
        for dir in [
            self.agents_root(),
            self.objects_root(),
            self.object_updates_root(),
            self.textures_root(),
            self.materials_root(),
            self.animations_root(),
            self.land_root(),
            self.filtered_animations_dir(),
        ] {
            ensure_dir(&dir)?;
        }
        Ok(())
    }
}

fn ensure_dir(dir: &Path) -> Result<(), ProtocolSessionError> {
    std::fs::create_dir_all(dir).map_err(|e| ProtocolSessionError::DirCreation {
        dir: dir.to_path_buf(),
        error: e,
    })
}

/// Initialize the viewer's cache in the share dir on disk.
pub fn initialize_share_dir() -> Result<BenthicDirs, ProtocolSessionError> {
    let data_dir = dirs::data_dir().ok_or(ProtocolSessionError::NotFound {})?;
    let share_dir = data_dir.join("benthic");
    ensure_dir(&share_dir)?;
    Ok(BenthicDirs::new(share_dir))
}

/// Where a JSON blob belongs.
pub enum CacheDir {
    /// Per-agent object: agents/<agent_id>/<object_id>.json
    AgentObject { agent: AgentId, object: ObjectId },
    /// Per-object (non-agent): objects/<sculpt_id>/<filename>.json
    Object(ObjectId),
    /// Land: land/<filename>.json
    Land,
}

/// Serialize a Rust object and write its JSON to the correct directory.
///
/// `filename` is the stem; `.json` is appended. For `CacheDir::AgentObject`
/// the filename is *forced* to the object id so the layout invariant
/// `agents/<agent_id>/<object_id>.json` holds.
pub fn write_json<T: Serialize>(
    dirs: &BenthicDirs,
    data: &T,
    filename: &str,
    cache_dir: CacheDir,
) -> Result<PathBuf, ProtocolSessionError> {
    let path = match cache_dir {
        CacheDir::AgentObject { agent, object } => {
            // Ignore `filename` here: the invariant is the object id.
            dirs.ensure_agent_dir(agent)?;
            dirs.agent_object_json(agent, object)
        }
        CacheDir::Object(sculpt) => {
            let dir = dirs.ensure_object_dir(sculpt)?;
            dir.join(format!("{filename}.json"))
        }
        CacheDir::Land => {
            let dir = dirs.land_root();
            ensure_dir(&dir)?;
            dir.join(format!("{filename}.json"))
        }
    };

    let json =
        serde_json::to_vec(data).map_err(|error| ProtocolSessionError::JsonWriteError { error })?;

    // Atomic-ish write: temp then rename, so a crash doesn't leave a
    // half-written file that a loader later picks up.
    let tmp = path.with_extension("json.tmp");
    File::create(&tmp)?.write_all(&json)?;
    std::fs::rename(&tmp, &path)?;

    Ok(path)
}
