pub struct Mesh{
    
}
impl Mesh{
    pub fn from_bytes(bytes: &[u8]) -> std::io::Result<Self>{
        println!("bytes {:?}", bytes);
        Ok(Mesh{})
    }
}
