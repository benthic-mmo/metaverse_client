use metaverse_messages::region_handshake::RegionHandshake;
use log::LevelFilter;

#[test]
fn test_region_handshake() {
    init_logger();
    let test_packet: Vec<u8> = vec![
        148, 198, 128, 64, 16, 

        13, // Sim Access, set to 13 which corresponds to PG
        
        11, // length prefix 
        76, 98, 115, 97, 32, 80, 108, 97, 122, 97, 0,
    //  L   b   s    a   ""  P   l    a   z    a   \0 

        24, 7, 181, 149, 71, 53, 67, 235, 185, 212, 14, 193, 188, 133, 228, 137,
    //  UUID of the owner 

        0, 0, 0, 160, 65, 0, 0, 0, 0,
        111, 100, 95, 17, 133, 205, 75, 64, 174, 237, 200, 13, 181, 76, 50, 130, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 191,
        111, 6, 81, 195, 225, 70, 190, 134, 210, 86, 208, 140, 118, 169, 232, 191, 111, 6, 81, 195,
        225, 70, 190, 134, 210, 86, 208, 140, 118, 169, 232, 170, 26, 109, 180, 101, 184, 76, 187,
        175, 128, 210, 45, 217, 174, 203, 95, 170, 26, 109, 180, 101, 184, 76, 187, 175, 128, 210,
        45, 217, 174, 203, 95, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 65, 0, 0,
        32, 65, 0, 0, 32, 65, 0, 0, 32, 65, 145, 1, 180, 77, 146, 44, 76, 198, 148, 181, 109, 23,
        110, 57, 45, 196, 9, 0, 0, 0, 1, 0, 0, 0, 0, 0, 

        13, // length prefix
        79, 83, 103, 114, 105, 100, 32, 80, 108, 97, 122, 97, 0,
    //  O   S   g    r    i    d    ""  P   l    a   z    a   \0
        1, 198, 128, 64, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128,
    ];
    match RegionHandshake::from_bytes(&test_packet) {
        Ok(packet) => println!("Packet created successfully: {:?}", packet),
        Err(e) => eprintln!("Error creating packet: {}", e),
    }
}

fn init_logger() {
    let _ = env_logger::builder()
        .filter(None, LevelFilter::Info)
        .is_test(true)
        .try_init();
}


