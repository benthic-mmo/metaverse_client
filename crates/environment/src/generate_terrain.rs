use metaverse_messages::layer_data::{LayerData, LayerType};

pub fn generate_terrain(data: &LayerData){
    println!("GENERATING TERRAIN!!!!!");
    match data.layer_type {
        LayerType::Land | LayerType::LandExtended => println!("land"),
        LayerType::Wind | LayerType::WindExtended => println!("wind"),
        LayerType::Water | LayerType::WaterExtended => println!("water"),
        LayerType::Cloud | LayerType::CloudExtended => println!("cloud"),
        LayerType::Unknown => println!("unknown")
    }
}
