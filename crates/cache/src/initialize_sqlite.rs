use crate::errors::InventoryError;
use sqlx::migrate::Migrator;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::{path::PathBuf, str::FromStr};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

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

