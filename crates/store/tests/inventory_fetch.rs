use std::path::PathBuf;

use metaverse_store::initialize_sqlite::{Inventory, init_sqlite};
use uuid::Uuid;

const DEFAULT_OUTFIT_ID: &str = "0a1e5fc2-4b8f-48eb-bf11-fea1dcdb46a4";

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
    let items = inventory
        .get_current_outfit(Uuid::parse_str(DEFAULT_OUTFIT_ID).unwrap())
        .await
        .unwrap();
    println!("{:?}", items);
}
