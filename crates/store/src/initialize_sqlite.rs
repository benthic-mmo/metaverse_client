use crate::errors::InventoryError;
use sqlx::migrate::Migrator;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{path::PathBuf, str::FromStr};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Debug, Clone)]
pub struct Inventory {
    pub db: SqlitePool,
}
impl Inventory {
    pub fn new(db: SqlitePool) -> Self {
        Inventory { db }
    }
}

#[derive(Clone, Debug)]
pub struct Cache {
    pub avatar: AvatarCache,
    pub object: ObjectCache,
}
impl Cache {
    pub fn new(db: SqlitePool) -> Self {
        Cache {
            avatar: AvatarCache { db: db.clone() },
            object: ObjectCache { db: db.clone() },
        }
    }
}

#[derive(Clone, Debug)]
pub struct AvatarCache {
    pub db: SqlitePool,
}

#[derive(Clone, Debug)]
pub struct ObjectCache {
    pub db: SqlitePool,
}

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
