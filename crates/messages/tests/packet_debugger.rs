// this is not a test file!
// this is a command line utility for decoding UDP packets!

use hex::FromHex;
use std::io::{self, Write};

use metaverse_messages::packet::Packet;

#[test]
fn test_packet_decoder() {
    println!("Metaverse Packet Analyzer");
    println!("get the packet body as a hex stream from wireshark");
    println!("Type 'exit' to quit.");

    loop {
        let mut input = String::new();
        print!("Enter packet data as hex stream: ");
        io::stdout().flush().expect("Failed to flush stdout");

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        // Remove any trailing newline characters
        let input = input.trim();

        // Convert the input string to bytes
        let bytes = match Vec::from_hex(input) {
            Ok(bytes) => bytes,
            Err(_) => {
                eprintln!("could not decode hex");
                continue;
            }
        };

        // Create a Packet instance from the byte slice
        match Packet::from_bytes(&bytes) {
            Ok(packet) => println!("Packet created successfully: {:?}", packet),
            Err(e) => eprintln!("Error creating packet: {}", e),
        }
    }
}
