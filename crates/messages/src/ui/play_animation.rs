use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::packet::message::UIMessage;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayAnimation {
    pub player_id: Uuid,
    pub animation_path: PathBuf,
}

impl UIMessage {
    pub fn new_play_animation(data: PlayAnimation) -> Self {
        UIMessage::PlayAnimation(data)
    }
}
