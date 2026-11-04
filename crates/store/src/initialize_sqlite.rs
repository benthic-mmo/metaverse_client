use crate::cache::avatar::AVATAR_CACHE_SCHEMA;
use crate::cache::object::ObjectCache;
use crate::cache::{avatar::AvatarCache, object::OBJECT_CACHE_SCHEMA};
use crate::errors::InventoryError;
use log::info;
use sqlx::{
    SqlitePool,
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};
use std::{path::PathBuf, str::FromStr};
use surrealdb::{
    Surreal,
    engine::local::{Db, Mem},
};
use tokio::sync::mpsc::{self, Sender};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Inventory database.
///
/// Inventory data remains in SQLite.
#[derive(Debug, Clone)]
pub struct Inventory {
    pub db: SqlitePool,
}

impl Inventory {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

/// Cache database.
///
/// Avatar and object cache data are stored in SurrealDB.
#[derive(Debug, Clone)]
pub struct Cache {
    pub avatar: AvatarCache,
    pub object: ObjectCache,
}

impl Cache {
    /// Creates both caches and the shared operation queue.
    pub fn new(db: Surreal<Db>) -> Self {
        let cache = Self {
            avatar: AvatarCache { db: db.clone() },
            object: ObjectCache { db: db.clone() },
        };
        cache
    }
}

pub async fn init_sqlite(path: PathBuf) -> Result<SqlitePool, InventoryError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let database_url = format!("sqlite:{}", path.display());

    let options = SqliteConnectOptions::from_str(&database_url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    MIGRATOR.run(&pool).await?;

    Ok(pool)
}

pub async fn init_surreal() -> Result<Surreal<Db>, InventoryError> {
    info!("SurrealDB: creating in-memory database");

    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("benthic").use_db("cache").await?;
    let response = db.query(OBJECT_CACHE_SCHEMA).await?;
    response.check()?;
    let response = db.query(AVATAR_CACHE_SCHEMA).await?;
    response.check()?;

    info!("SurrealDB: schema initialized");

    Ok(db)
}
/// Initialize both databases and construct the cache.
///
/// Returns:
/// - Inventory: SQLite inventory database
/// - Cache: SurrealDB avatar/object cache
/// - Receiver: shared cache operation queue
pub async fn init_databases(inventory_path: PathBuf) -> Result<(Inventory, Cache), InventoryError> {
    let sqlite = init_sqlite(inventory_path).await?;
    let surreal = init_surreal().await?;

    let inventory = Inventory::new(sqlite);
    let cache = Cache::new(surreal);

    Ok((inventory, cache))
}
