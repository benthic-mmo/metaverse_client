use std::{collections::HashMap, sync::Arc, thread::sleep, time::Duration};

use benthic_protocol::{
    errors::ProtocolSessionError,
    messages::{
        login::{login_event::Login, login_response::LoginResponse},
        simulator::error_message::{
            CapabilityError, CircuitCodeError, CompleteAgentMovementError, FeatureError,
            MailboxSessionError,
        },
        ui_protocol::UIMessage,
    },
    session::{
        storage::initialize_share_dir,
        types::{EnvironmentCache, InventoryData, RegionData, Session, SessionContext},
    },
};
use glam::Vec2;
use metaverse_messages::{
    http::capabilities::{Capability, CapabilityRequest},
    packet::packet_protocol::Packet,
    udp::core::{
        agent_throttle::{AgentThrottle, ThrottleData},
        circuit_code::CircuitCode,
        complete_agent_movement::CompleteAgentMovementData,
    },
    utils::object_types::ObjectType,
};
use metaverse_store::initialize_sqlite::{Inventory, init_sqlite};
use uuid::Uuid;

#[cfg(feature = "inventory")]
use crate::inventory::{FetchCurrentOutfit, RefreshInventoryEvent};
use crate::{
    actors::{InteractionActor, SessionActor},
    capabilities::SendCapabilityRequest,
    environment::FetchEnvironmentEvent,
    session::{AttachSession, OutgoingPacket, StartSession},
    transport::http_handler::login_to_simulator,
    ui_interactions::SendUIMessage,
};

use log::{error, warn};

pub(crate) async fn handle_login(
    login_data: Login,
    interaction_addr: &actix::Addr<InteractionActor>,
    mailbox_addr: &actix::Addr<SessionActor>,
    session_ctx: Arc<SessionContext>,
) -> Result<(), ProtocolSessionError> {
    let (login_response, local_ip) = match login_to_simulator(login_data).await {
        Ok((login_response, local_ip)) => {
            if let Err(e) = interaction_addr
                .send(SendUIMessage {
                    ui_message: UIMessage::new_login_response_event(LoginResponse {
                        firstname: login_response.first_name.clone(),
                        lastname: login_response.last_name.clone(),
                        agent_id: login_response.agent_id,
                    }),
                })
                .await
            {
                error!("Failed to send login response to UI {:?}", e)
            };
            (login_response, local_ip)
        }
        Err(e) => {
            if let Err(e) = interaction_addr
                .send(SendUIMessage {
                    ui_message: UIMessage::new_session_error(e.clone().into()),
                })
                .await
            {
                error!("Failed to send session error: {:?}", e)
            };
            Err(e)?
        }
    };

    // get the current outfit root from the login response's inventory skeleton
    let current_outfit_root = login_response
        .inventory_skeleton
        .as_ref()
        .and_then(|folders| {
            folders
                .iter()
                .find(|folder| folder.type_default == ObjectType::CurrentOutfit)
                .map(|folder| folder.folder_id)
        })
        .unwrap_or(Uuid::nil());

    let dirs = initialize_share_dir()?;
    let db_path = dirs.inventory_db();
    let inventory_db = init_sqlite(db_path.clone())
        .await
        .map_err(|err| FeatureError::Inventory(format!("Failed to initialize SQLite: {}", err)))?;

    let inventory = Inventory::new(inventory_db);
    if session_ctx
        .session_id
        .set(login_response.session_id)
        .is_err()
    {
        warn!("session_id already set; ignoring duplicate login");
    }
    if session_ctx.agent_id.set(login_response.agent_id).is_err() {
        warn!("agent_id already set; ignoring duplicate login");
    }
    if session_ctx
        .address
        .set(format!(
            "{}:{}",
            login_response.sim_ip, login_response.sim_port
        ))
        .is_err()
    {
        warn!("address already set; ignoring duplicate login");
    }
    if session_ctx
        .seed_capability_url
        .set(login_response.seed_capability.unwrap())
        .is_err()
    {
        warn!("seed capability url is already set. Ignoring duplicate capability")
    }

    let session = Session {
        inventory,
        local_ip,
        capability_urls: HashMap::new(),
        region_data: RegionData {
            region_coordinates: Vec2 {
                x: (login_response.region_x.unwrap() as f32),
                y: (login_response.region_y.unwrap() as f32),
            },
            region_id: format!("{}:{}", login_response.sim_ip, login_response.sim_port),
            ..Default::default()
        },

        environment_cache: EnvironmentCache {
            patch_queue: HashMap::new(),
            patch_cache: HashMap::new(),
        },

        inventory_data: InventoryData {
            inventory_root: login_response.inventory_root.ok_or_else(|| {
                FeatureError::Inventory("Login response contained no inventory_root".to_string())
            })?,
            inventory_lib_owner: login_response.inventory_lib_owner.ok_or_else(|| {
                FeatureError::Inventory(
                    "Login response contained no inventory_lib_owner".to_string(),
                )
            })?,
            inventory_init: false,
            current_outfit_root,
            current_outfit_init: false,
        },

        #[cfg(feature = "avatar")]
        avatars: HashMap::new(),
    };

    if let Err(e) = interaction_addr
        .send(StartSession {
            local_ip: session.local_ip,
        })
        .await
    {
        Err(MailboxSessionError {
            message: e.to_string(),
        })?;
    };
    if let Err(e) = mailbox_addr.send(AttachSession { session }).await {
        Err(MailboxSessionError {
            message: e.to_string(),
        })?;
    };

    match CapabilityRequest::new_capability_request(vec![
        Capability::ViewerAsset,
        Capability::FetchInventoryDescendents2,
        Capability::ExtEnvironment,
    ]) {
        Ok(caps) => {
            if let Err(e) = interaction_addr
                .send(SendCapabilityRequest {
                    capability_request: caps,
                })
                .await
            {
                Err(CapabilityError {
                    message: e.to_string(),
                })?
            }
        }
        Err(e) => {
            error!("{:?}", e)
        }
    };

    if let Err(e) = interaction_addr
        .send(OutgoingPacket {
            packet: Packet::new_circuit_code(CircuitCode {
                code: login_response.circuit_code,
                session_id: login_response.session_id,
                id: login_response.agent_id,
            }),
        })
        .await
    {
        Err(CircuitCodeError {
            message: e.to_string(),
        })?;
    };

    sleep(Duration::from_secs(1));
    if let Err(e) = interaction_addr
        .send(OutgoingPacket {
            packet: Packet::new_complete_agent_movement(CompleteAgentMovementData {
                circuit_code: login_response.circuit_code,
                session_id: login_response.session_id,
                agent_id: login_response.agent_id,
            }),
        })
        .await
    {
        Err(CompleteAgentMovementError {
            message: e.to_string(),
        })?;
    };
    if let Err(e) = interaction_addr
        .send(OutgoingPacket {
            packet: Packet::new_agent_throttle(AgentThrottle {
                agent_id: login_response.agent_id,
                session_id: login_response.session_id,
                circuit_code: login_response.circuit_code,
                gen_counter: 0,
                throttles: ThrottleData {
                    ..Default::default()
                },
            }),
        })
        .await
    {
        Err(CompleteAgentMovementError {
            message: e.to_string(),
        })?;
    };

    #[cfg(feature = "inventory")]
    if let Err(e) = mailbox_addr.send(FetchCurrentOutfit {}).await {
        Err(CapabilityError {
            message: e.to_string(),
        })?
    }
    #[cfg(feature = "inventory")]
    if let Err(e) = mailbox_addr.send(RefreshInventoryEvent {}).await {
        Err(CapabilityError {
            message: e.to_string(),
        })?
    }

    if let Err(e) = mailbox_addr.send(FetchEnvironmentEvent {}).await {
        Err(CapabilityError {
            message: e.to_string(),
        })?
    }

    Ok(())
}
