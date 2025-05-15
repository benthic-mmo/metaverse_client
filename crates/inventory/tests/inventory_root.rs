use metaverse_inventory::inventory_root::FolderRequest;
use uuid::Uuid;

#[test]
fn inventory_root() {
    let folder = FolderRequest {
        folder_id: Uuid::nil(),
        owner_id: Uuid::nil(),
        fetch_folders: true,
        fetch_items: true,
        sort_order: 1,
    };
    println!("{:?}", folder.to_llsd())
}
