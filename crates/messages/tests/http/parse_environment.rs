use std::{fs::File, io::Read};

use metaverse_messages::http::environment_data::DayCycle;

#[test]
fn handle_day_cycle_data() {
    let mut file = File::open("tests/data/Environment.txt").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let day_cycle = DayCycle::from_bytes(&buffer).unwrap();
    println!("{:?}", day_cycle);
}
