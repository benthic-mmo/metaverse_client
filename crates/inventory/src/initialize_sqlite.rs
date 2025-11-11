use crate::errors::InventoryError;
use sqlx::migrate::Migrator;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::{fs, io::ErrorKind, path::PathBuf};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn init_sqlite(path: PathBuf) -> Result<SqlitePool, InventoryError> {
    // Remove existing DB if it exists
    match fs::remove_file(&path) {
        Ok(_) => (),
        Err(ref e) if e.kind() == ErrorKind::NotFound => (),
        Err(e) => return Err(e.into()),
    }
    fs::File::create(&path)?;
    let database_url = format!("sqlite:{}", path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    MIGRATOR.run(&pool).await?;

    Ok(pool)
}
