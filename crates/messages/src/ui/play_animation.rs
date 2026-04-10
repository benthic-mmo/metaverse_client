use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::packet::message::UIMessage;

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A message for informing the UI of a new animation to play
pub struct PlayAnimation {
    /// Player ID of the avatar playing the animation
    pub player_id: Uuid,
    /// Path on disk to animation file
    pub animation_path: PathBuf,
}

impl UIMessage {
    /// Create a new play_animation message
    pub fn new_play_animation(data: PlayAnimation) -> Self {
        UIMessage::PlayAnimation(data)
    }
}
