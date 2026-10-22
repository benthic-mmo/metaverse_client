use crate::cache::avatar::{AvatarCache, QueuedAvatarCacheOperation};
use crate::cache::object::{ObjectCache, QueuedObjectCacheOperation};
use crate::errors::InventoryError;
use sqlx::migrate::Migrator;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{path::PathBuf, str::FromStr};
use tokio::sync::mpsc::{self, Sender};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Inventory object for handling user inventory
#[derive(Debug, Clone)]
pub struct Inventory {
    /// connection to sqlite db
    pub db: SqlitePool,
}
impl Inventory {
    /// create a new inventory object
    pub fn new(db: SqlitePool) -> Self {
        Inventory { db }
    }
}

/// Cache object containing the avatar and object cache
/// this is used both for runtime handling and storing state between sessions.
#[derive(Clone, Debug)]
pub struct Cache {
    /// Cache object for storing avatar cache functions.
    pub avatar: AvatarCache,
    /// Cache object for storing object cache functions.
    pub object: ObjectCache,
    /// Queue for cache operations.
    /// this is used to prevent overloading the sqlite cache
    pub queue: Sender<CacheOperation>,
}
impl Cache {
    /// Creates both caches and the shared operation queue.
    pub fn new(db: SqlitePool) -> (Self, mpsc::Receiver<CacheOperation>) {
        let (queue, receiver) = mpsc::channel(4096);

        let cache = Cache {
            avatar: AvatarCache {
                db: db.clone(),
                queue: queue.clone(),
            },
            object: ObjectCache {
                db,
                queue: queue.clone(),
            },
            queue,
        };

        (cache, receiver)
    }
}

/// A queued write operation for either cache.
#[derive(Debug)]
pub enum CacheOperation {
    /// An operation for the object cache.
    Object(QueuedObjectCacheOperation),
    /// An operation for the avatar cache.
    Avatar(QueuedAvatarCacheOperation),
}

/// Handler for initializing the sqlite db on disk
///
/// This creates the sqlite db on disck and runs the migrations to establish the correct db layout
///
/// # Parameters
/// - `path`: The requested path on disk for the sqlite db
///
/// # Errors
/// - Returns an [`InventoryError`] on failure
///
/// # Side Effects
/// [Storage: Filesystem]
pub async fn init_sqlite(path: PathBuf) -> Result<SqlitePool, InventoryError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let database_url = format!("sqlite:{}", path.display());

    let options = SqliteConnectOptions::from_str(&database_url)?.create_if_missing(true);

    let pool: SqlitePool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    MIGRATOR.run(&pool).await?;

    Ok(pool)
}
