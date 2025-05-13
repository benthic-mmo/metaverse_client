use metaverse_messages::core::object_update::ObjectUpdate;

pub fn handle_object_update(data: &ObjectUpdate) -> std::io::Result<String> {
    println!("{:?}",data);
    Ok("aaa".to_string())
}
