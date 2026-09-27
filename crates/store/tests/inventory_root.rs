use httpmock::{Method::POST, MockServer};
use metaverse_messages::http::folder_request::FolderRequest;
use metaverse_store::{
    errors::InventoryError,
    initialize_sqlite::{Inventory, init_sqlite},
};
use sqlx::Row;
use std::{fs::File, io::Read};
use tempfile::TempDir;
use tokio::task::LocalSet;

fn folder_request() -> FolderRequest {
    FolderRequest {
        folder_id: Default::default(),
        owner_id: Default::default(),
        fetch_items: true,
        fetch_folders: true,
        sort_order: 0,
    }
}

async fn create_test_inventory() -> (TempDir, Inventory) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("inventory.db");

    let pool = init_sqlite(db_path).await.unwrap();

    (temp_dir, Inventory::new(pool))
}

async fn mock_inventory_server(file_path: &str) -> MockServer {
    let server = MockServer::start();

    let mut file = File::open(file_path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    server.mock(|when, then| {
        when.method(POST)
            .path("/inventory")
            .header("Content-Type", "application/llsd+xml");

        then.status(200).body(buffer);
    });

    server
}

#[tokio::test(flavor = "current_thread")]
async fn test_refresh_inventory_no_categories() {
    let local_set = LocalSet::new();

    local_set
        .run_until(async {
            let server = mock_inventory_server("tests/data/folder_data_3.txt").await;

            let (_temp_dir, inventory) = create_test_inventory().await;

            let result = inventory
                .refresh(folder_request(), server.url("/inventory"))
                .await;

            println!("refresh inventory result: {:?}", result);

            print_tables(&inventory.db).await;
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn test_refresh_inventory_categories() {
    let local_set = LocalSet::new();

    local_set
        .run_until(async {
            let server = mock_inventory_server("tests/data/folder_data_4.txt").await;

            let (_temp_dir, inventory) = create_test_inventory().await;

            let result = inventory
                .refresh(folder_request(), server.url("/inventory"))
                .await;

            println!("refresh inventory result: {:?}", result);

            let outfit = inventory.get_current_outfit().await;

            println!("outfit: {:?}", outfit);

            print_tables(&inventory.db).await;
        })
        .await;
}

async fn print_tables(pool: &sqlx::SqlitePool) {
    println!("____FOLDERS _______________");
    print_table(pool, "folders").await.unwrap();

    println!("____CATEGORIES ____________");
    print_table(pool, "categories").await.unwrap();

    println!("____ITEMS _________________");
    print_table(pool, "items").await.unwrap();
}

async fn print_table(pool: &sqlx::SqlitePool, table: &str) -> Result<(), InventoryError> {
    let rows = match table {
        "folders" => sqlx::query("SELECT * FROM folders").fetch_all(pool).await?,

        "categories" => {
            sqlx::query("SELECT * FROM categories")
                .fetch_all(pool)
                .await?
        }

        "items" => sqlx::query("SELECT * FROM items").fetch_all(pool).await?,

        _ => return Err(InventoryError::Error("Unknown table".into())),
    };

    for row in rows {
        let mut values = Vec::with_capacity(row.len());

        for i in 0..row.len() {
            let value: Result<String, _> = row.try_get(i);

            values.push(match value {
                Ok(value) => value,
                Err(_) => "<non-string>".to_string(),
            });
        }

        println!("{}", values.join(" | "));
    }

    Ok(())
}
