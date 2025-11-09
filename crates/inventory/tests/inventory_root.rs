use metaverse_inventory::agent::get_current_outfit;
use metaverse_inventory::initialize_sqlite::init_sqlite;
use metaverse_inventory::{errors::InventoryError, inventory_root::refresh_inventory_2};
use metaverse_messages::http::folder_request::FolderRequest;
use rusqlite::types::Value;
use rusqlite::Connection;
use std::{fs::File, io::Read};
use tempfile::TempDir;
use tokio::task::LocalSet;

use httpmock::{Method::POST, MockServer};

#[tokio::test(flavor = "current_thread")]
async fn test_refresh_inventory_no_categories() {
    let _ = MockServer::start();

    let server = MockServer::start();

    let mut file = File::open("tests/data/folder_data_3.txt").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    server.mock(|when, then| {
        when.method(POST)
            .path("/inventory")
            .header("Content-Type", "application/llsd+xml");
        then.status(200).body(buffer);
    });

    let folder_request = FolderRequest {
        folder_id: Default::default(),
        owner_id: Default::default(),
        fetch_items: true,
        fetch_folders: true,
        sort_order: 0,
    };

    let server_endpoint = format!("{}{}", server.url("/inventory"), "");
    let temp_file = TempDir::new().unwrap();
    let path = temp_file.path();
    let mut conn = init_sqlite(path.into()).unwrap();

    let local_set = LocalSet::new();
    local_set
        .run_until(async {
            let result = refresh_inventory_2(&mut conn, folder_request, server_endpoint).await;
            println!("refresh inventory result {:?}", result);
            print_tables(&conn);
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn test_refresh_inventory_categories() {
    let _ = MockServer::start();

    let server = MockServer::start();

    let mut file = File::open("tests/data/folder_data_4.txt").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    server.mock(|when, then| {
        when.method(POST)
            .path("/inventory")
            .header("Content-Type", "application/llsd+xml");
        then.status(200).body(buffer);
    });

    let folder_request = FolderRequest {
        folder_id: Default::default(),
        owner_id: Default::default(),
        fetch_items: true,
        fetch_folders: true,
        sort_order: 0,
    };

    let server_endpoint = format!("{}{}", server.url("/inventory"), "");
    let temp_file = TempDir::new().unwrap();
    let path = temp_file.path();
    let mut conn = init_sqlite(path.into()).unwrap();

    let local_set = LocalSet::new();
    local_set
        .run_until(async {
            let result = refresh_inventory_2(&mut conn, folder_request, server_endpoint).await;
            println!("refresh inventory result {:?}", result);

            let outfit = get_current_outfit(&conn);

            println!("outfit: {:?}", outfit);
            print_tables(&conn);
        })
        .await;
}

fn print_tables(conn: &Connection) {
    println!("____FOLDERS _______________");
    print_table(&conn, "folders").unwrap();

    println!("____CATEGORIES _______________");
    print_table(&conn, "categories").unwrap();

    println!("____ITEMS __________________");
    print_table(&conn, "items").unwrap();

    println!("____PERMISSIONS _____________");
    print_table(&conn, "permissions").unwrap();

    println!("____SALEINFO _____________");
    print_table(&conn, "sale_info").unwrap();
}

fn print_table(conn: &Connection, table: &str) -> Result<(), InventoryError> {
    let mut stmt = conn.prepare(&format!("SELECT * FROM {}", table))?;
    let column_count = stmt.column_count();

    let rows = stmt.query_map([], |row| {
        let mut values = Vec::with_capacity(column_count);
        for i in 0..column_count {
            let val: Value = row.get(i)?;
            values.push(format!("{val:?}"));
        }
        Ok(values.join(" | "))
    })?;

    for r in rows {
        println!("{}", r?);
    }

    Ok(())
}
