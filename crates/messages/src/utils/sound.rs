use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read};
use uuid::Uuid;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Handles sounds attached to the object
pub struct AttachedSound {
    /// Asset UUID of any attached looped sounds
    pub sound_id: Uuid,
    /// UUID of the owner of the object. Null if there is no looped sound or particle system
    /// attached to the object.
    pub owner_id: Uuid,
    /// Gain of the attached sound
    pub gain: f32,
    /// Stores flags related to attached sounds
    pub flags: u8,
    /// Radius from the center of the object that the sound should be audible from
    pub radius: f32,
}
impl AttachedSound {
    /// Convert bytes into a Sound object
    pub fn from_bytes(bytes: &[u8]) -> std::io::Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let mut sound_id_bytes = [0u8; 16];
        cursor.read_exact(&mut sound_id_bytes)?;

        let mut owner_id_bytes = [0u8; 16];
        cursor.read_exact(&mut owner_id_bytes)?;

        Ok(Self {
            sound_id: Uuid::from_bytes(sound_id_bytes),
            owner_id: Uuid::from_bytes(owner_id_bytes),
            gain: cursor.read_f32::<LittleEndian>()?,
            flags: cursor.read_u8()?,
            radius: cursor.read_f32::<LittleEndian>()?,
        })
    }
}
