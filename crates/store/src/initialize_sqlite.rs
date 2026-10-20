use crate::errors::InventoryError;
use sqlx::migrate::Migrator;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{path::PathBuf, str::FromStr};

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
    /// cache object for storing avatar cache functions
    pub avatar: AvatarCache,
    /// cache object for storing object cache functions
    pub object: ObjectCache,
}
impl Cache {
    /// create a new cache
    pub fn new(db: SqlitePool) -> Self {
        Cache {
            avatar: AvatarCache { db: db.clone() },
            object: ObjectCache { db: db.clone() },
        }
    }
}
/// Sqlite pool connection for avatar cache functions
#[derive(Clone, Debug)]
pub struct AvatarCache {
    /// the async pool of DB connections
    pub db: SqlitePool,
}

/// Sqlite pool connection for object cache functions
#[derive(Clone, Debug)]
pub struct ObjectCache {
    /// the async pool of db connections
    pub db: SqlitePool,
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
