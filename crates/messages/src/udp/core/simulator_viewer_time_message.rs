use byteorder::{LittleEndian, ReadBytesExt};
use glam::Vec3;

use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use std::io::Cursor;

impl Packet {
    /// create a new simulator viewer time packet
    pub fn new_simulator_viewer_time_message(
        simulator_viewer_time_message: SimulatorViewerTimeMessage,
    ) -> Self {
        Packet {
            header: Header {
                id: 150,
                reliable: true,
                zerocoded: false,
                frequency: PacketFrequency::Low,
                ..Default::default()
            },
            body: PacketType::SimulatorViewerTimeMessage(Box::new(simulator_viewer_time_message)),
        }
    }
}

/// TODO: unimplemented
#[derive(Debug, Clone)]
pub struct SimulatorViewerTimeMessage {
    pub seconds_since_start: u64,
    pub seconds_per_day: u32,
    pub seconds_per_year: u32,
    pub sun_direction: Vec3,
    pub sun_phase: f32,
    pub sun_angle_velocity: Vec3,
}

impl PacketData for SimulatorViewerTimeMessage {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);
        let seconds_since_start = cursor.read_u64::<LittleEndian>().unwrap();
        let seconds_per_day = cursor.read_u32::<LittleEndian>().unwrap();
        let seconds_per_year = cursor.read_u32::<LittleEndian>().unwrap();
        let sun_direction = Vec3::new(
            cursor.read_f32::<LittleEndian>().unwrap(),
            cursor.read_f32::<LittleEndian>().unwrap(),
            cursor.read_f32::<LittleEndian>().unwrap(),
        );
        let sun_phase = cursor.read_f32::<LittleEndian>().unwrap();
        let sun_angle_velocity = Vec3::new(
            cursor.read_f32::<LittleEndian>().unwrap(),
            cursor.read_f32::<LittleEndian>().unwrap(),
            cursor.read_f32::<LittleEndian>().unwrap(),
        );
        Ok(SimulatorViewerTimeMessage {
            seconds_since_start,
            seconds_per_day,
            seconds_per_year,
            sun_direction,
            sun_phase,
            sun_angle_velocity,
        })
    }
    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}
