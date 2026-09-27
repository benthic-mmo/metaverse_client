use std::path::PathBuf;

use metaverse_store::initialize_sqlite::{Inventory, init_sqlite};

#[tokio::test]
async fn fetch_inventory() {
    let sqlite_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/inventory.db");

    let pool = init_sqlite(sqlite_path).await.unwrap();
    let inventory = Inventory::new(pool);
    let folders = inventory.fetch_inventory().await.unwrap();
    println!("{:?}", folders);
}

#[tokio::test]
async fn get_current_outfit() {
    let sqlite_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/inventory.db");

    let pool = init_sqlite(sqlite_path).await.unwrap();
    let inventory = Inventory::new(pool);
    let items = inventory.get_current_outfit().await.unwrap();
    println!("{:?}", items);
}
