use bevy::prelude::{EventReader, ResMut};

use crate::{login, utils};

pub fn send_chat_message(
    mut ev_client_event: EventReader<login::ClientEvent>,
    client_action_stream: ResMut<utils::ClientActionStream>,
) {
    for event in ev_client_event.read() {
        let mut client_action_stream = client_action_stream.0.lock().unwrap();
        client_action_stream.push_back(event.0.clone());
    }
}
