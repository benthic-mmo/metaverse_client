use byteorder::{LittleEndian, ReadBytesExt};
use glam::Vec3;

use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet_protocol::{Packet, PacketData},
    packet_types::PacketType,
};
use crate::parse;
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
/// The viewer uses this packet to determine the time of day in the region.
pub struct SimulatorViewerTimeMessage {
    /// Microseconds since the sim start, or the region time origin.
    pub seconds_since_start: u64,
    /// How many seconds are in one day
    pub seconds_per_day: u32,
    /// How many seconds are in one year. This is for determining live season changes.
    pub seconds_per_year: u32,
    /// Unit vector to determine which direction the sun is moving
    pub sun_direction: Vec3,
    /// A value representing the position of the sun in the sky  
    pub sun_phase: f32,
    /// angular velocity of the sun in 3d space
    pub sun_angle_velocity: Vec3,
}

impl PacketData for SimulatorViewerTimeMessage {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);

        let seconds_since_start = parse!(cursor.read_u64::<LittleEndian>())?;
        let seconds_per_day = parse!(cursor.read_u32::<LittleEndian>())?;
        let seconds_per_year = parse!(cursor.read_u32::<LittleEndian>())?;

        let sun_direction = Vec3::new(
            parse!(cursor.read_f32::<LittleEndian>())?,
            parse!(cursor.read_f32::<LittleEndian>())?,
            parse!(cursor.read_f32::<LittleEndian>())?,
        );

        let sun_phase = parse!(cursor.read_f32::<LittleEndian>())?;

        let sun_angle_velocity = Vec3::new(
            parse!(cursor.read_f32::<LittleEndian>())?,
            parse!(cursor.read_f32::<LittleEndian>())?,
            parse!(cursor.read_f32::<LittleEndian>())?,
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
