use actix::Actor;
use hex::FromHex;
use log::{info, LevelFilter};

use metaverse_instantiator::config_generator::{
    create_default_config, create_default_region_config, create_default_standalone_config,
};
use metaverse_instantiator::models::server::{CommandMessage, ExecData, ServerState, SimServer};
use metaverse_instantiator::server::{download_sim, read_config};
use metaverse_login::models::simulator_login_protocol::Login;
use metaverse_messages::models::circuit_code::CircuitCodeData;
use metaverse_messages::models::header::*;
use metaverse_messages::models::packet::Packet;
use metaverse_session::session::new_session;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

fn init_logger() {
    let _ = env_logger::builder()
        .filter(None, LevelFilter::Info)
        .is_test(true)
        .try_init();
}

// this should be in messages
#[test]
fn circuit_code_from_bytes() {
    let bytes = match Vec::from_hex("000003040006000000000000000008004500004a07d94000401134c87f0000017f000001a22e23280036fe49000000000000ffff0003a78a4d2f983bd7bd87d9447e87074205ee2a74869dc18bb1044f4c68906b2cb608b2e197") {
        Ok(bytes) => {
            bytes
        }
        Err(_) => {
            panic!("didn't work");
        }
    };
    match Packet::from_bytes(&bytes) {
        Ok(packet) => {
            println!("packet's id, header {:?}, {:?}", packet.header, packet.body);
            let correct_packet = Packet {
                header: Header {
                    id: 3,
                    frequency: PacketFrequency::Low,
                    reliable: false,
                    sequence_number: 0,
                    appended_acks: false,
                    zerocoded: false,
                    resent: false,
                    ack_list: None,
                    size: None,
                },
                body: Arc::new(CircuitCodeData {
                    code: 808464436,
                    id: Uuid::new_v4(),
                    session_id: Uuid::new_v4(),
                }),
            };

            let serialized = packet.to_bytes();
            let serialized_packet_2 = correct_packet.to_bytes();
            println!("original packet _______________: {:?}", bytes);
            println!("Serialized UseCircuitCodePacket: {:?}", serialized);
            println!("unserialized correct packet____: {:?}", serialized_packet_2);
            assert!(bytes == serialized);
        }
        Err(e) => {
            eprintln!("Error deserializing UseCircuitCodePacket: {}", e);
        }
    }
}

#[actix_rt::test]
async fn test_local() {
    init_logger();

    let notify = Arc::new(Notify::new());
    let state = Arc::new(Mutex::new(ServerState::Starting));

    // start the sim server, and initialize logging
    let sim_server = setup_server(Arc::clone(&notify), Arc::clone(&state)).await;
    send_setup_commands(&sim_server);

    notify.notified().await;
    if *state.lock().unwrap() == ServerState::Running {
        info!("Server started. Running test commands");
        sim_server.do_send(CommandMessage{
            command: "create user default user password email@email.com 9dc18bb1-044f-4c68-906b-2cb608b2e197 default".to_string()
        });

        let session = new_session(
            Login {
                first: "default".to_string(),
                last: "user".to_string(),
                passwd: "password".to_string(),
                start: "home".to_string(),
                channel: "benthic".to_string(),
                agree_to_tos: true,
                read_critical: true,
            },
            build_test_url("http://127.0.0.1", 9000),
        )
        .await;
        match session {
            Ok(_) => sleep(Duration::from_secs(3)).await,
            Err(e) => info!("sesion failed to start: {}", e),
        }
        sleep(Duration::from_secs(10)).await;
        sim_server.do_send(CommandMessage {
            command: "quit".to_string(),
        });
    } else {
        panic!("server failed to start")
    }
    notify.notified().await;
}

fn send_setup_commands(sim_server: &actix::Addr<SimServer>) {
    // This is required for first time startup. This assigns the default user as the region owner.
    // TODO: make this into a sql query that automatically adds this to the default region on
    // startup
    sim_server.do_send(CommandMessage {
        command: "default".to_string(),
    });
    sim_server.do_send(CommandMessage {
        command: "user".to_string(),
    });
    sim_server.do_send(CommandMessage {
        command: "password".to_string(),
    });
    sim_server.do_send(CommandMessage {
        command: "email@email.com".to_string(),
    });
    sim_server.do_send(CommandMessage {
        command: "9dc18bb1-044f-4c68-906b-2cb608b2e197".to_string(),
    });
}

async fn setup_server(
    notify: Arc<Notify>,
    state: Arc<Mutex<ServerState>>,
) -> actix::Addr<SimServer> {
    let (stdin_sender, stdin_receiver) = mpsc::channel::<CommandMessage>(100);

    let (url, archive, base_dir, executable) = match read_config() {
        Ok((url, archive, base_dir, executable)) => (url, archive, base_dir, executable),
        Err(e) => panic!("Error: {}", e),
    };

    info!("downloading server. On first run, this may take a while");
    match download_sim(&url, &archive, &base_dir).await {
        Ok(_) => info!("downloaded sim successfully"),
        Err(e) => info!("failed to download sim {}", e),
    };

    let sim_server = SimServer {
        state: Arc::clone(&state),
        sim_config: create_default_config(),
        standalone_config: create_default_standalone_config(),
        regions_config: create_default_region_config(),
        process: None,
        process_stdout_sender: None,
        process_stdin_receiver: Some(stdin_receiver),
        process_stdin_sender: Some(stdin_sender),
        notify: Arc::clone(&notify),
        exec_data: ExecData {
            base_dir,
            executable,
            init_command: "mono".to_string(),
        },
    }
    .start();
    info!("Waiting for the server to start...");

    sim_server
}

/// helper function for building URL. May be unnescecary
fn build_test_url(url: &str, port: u16) -> String {
    let mut url_string = "".to_owned();
    url_string.push_str(url);
    url_string.push(':');
    url_string.push_str(&port.to_string());
    println!("url string {}", url_string);
    url_string
}
