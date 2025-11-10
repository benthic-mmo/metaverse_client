use std::{fs, path::PathBuf, sync::LazyLock};

use include_dir::{include_dir, Dir};
use rusqlite::Connection;
use rusqlite_migration::Migrations;

use crate::errors::InventoryError;

static MIGRATIONS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/migrations");
static MIGRATIONS: LazyLock<Migrations<'static>> =
    LazyLock::new(|| Migrations::from_directory(&MIGRATIONS_DIR).unwrap());

pub fn init_sqlite(path: PathBuf) -> Result<Connection, InventoryError> {
    fs::remove_file(&path)?;
    let mut conn = Connection::open(path)?;
    MIGRATIONS.to_latest(&mut conn).unwrap();
    Ok(conn)
}
