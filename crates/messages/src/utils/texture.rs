use std::{
    collections::HashMap,
    f32::consts::{PI, TAU},
    io::{Cursor, Read},
};

use base64::{engine::general_purpose, Engine};
use byteorder::{LittleEndian, ReadBytesExt};
use rgb::Rgba;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// the texture data for an object
pub struct Texture {
    /// ID of the texture
    pub texture_id: Uuid,
    /// RGB alpha tint of the texture
    pub rgba: Rgba<u8>,
    /// U axis repeat of the texture
    pub repeat_u: f32,
    /// V axis repeat of the texture
    pub repeat_v: f32,
    /// U axis offset of the texture
    pub offset_u: f32,
    /// V axis offset of the texture
    pub offset_v: f32,
    /// rotation of the texture
    pub rotation: f32,
    /// Material of the texture, used for reflections
    pub material: u8,
    /// media of the texture
    pub media: u8,
    /// glow value of the texture
    pub glow: f32,
    /// UUID of the texture
    pub material_id: Uuid,
}

impl Default for Texture {
    fn default() -> Self {
        Texture {
            texture_id: Uuid::nil(),
            rgba: Rgba::new(0, 0, 0, 0),
            repeat_u: 0.0,
            repeat_v: 0.0,
            offset_u: 0.0,
            offset_v: 0.0,
            rotation: 0.0,
            material: 0,
            media: 0,
            glow: 0.0,
            material_id: Uuid::nil(),
        }
    }
}

impl Texture {
    /// convert from base64 bytes to a texture
    pub fn from_bytes(b64: &[u8]) -> std::io::Result<Self> {
        let mut faces: HashMap<u32, Texture> = HashMap::new();
        let mut texture = Texture::default();
        if b64.len() < 16 {
            return Ok(texture);
        }

        let bytes = general_purpose::STANDARD.decode(b64).unwrap();
        let mut cursor = Cursor::new(&bytes[..]);

        // ---- Texture ID ----
        {
            let mut id_bytes = [0u8; 16];
            cursor.read_exact(&mut id_bytes)?;
            texture.texture_id = Uuid::from_bytes(id_bytes);

            loop {
                let (bitfield, size) = read_face_bitfield(&mut cursor)?;
                if bitfield == 0 {
                    break;
                }

                println!("aaaa {:?}", texture);
                for face_index in 0..size {
                    let bit = 1 << face_index;
                    if bitfield & bit != 0 {
                        let mut id_bytes = [0u8; 16];
                        cursor.read_exact(&mut id_bytes)?;
                        faces.entry(face_index).or_default().texture_id =
                            Uuid::from_bytes(id_bytes);
                    }
                }
            }
        }
        // ---- Color ----
        {
            let red = cursor.read_u8()?;
            let green = cursor.read_u8()?;
            let blue = cursor.read_u8()?;
            let alpha = cursor.read_u8()?;
            texture.rgba = Rgba::new(red, green, blue, alpha);

            loop {
                let (bitfield, size) = read_face_bitfield(&mut cursor)?;
                if bitfield == 0 {
                    break;
                }
                for face_index in 0..size {
                    let bit = 1 << face_index;
                    if bitfield & bit != 0 {
                        let r = cursor.read_u8()?;
                        let g = cursor.read_u8()?;
                        let b = cursor.read_u8()?;
                        let a = cursor.read_u8()?;
                        faces.entry(face_index).or_default().rgba = Rgba::new(r, g, b, a);
                    }
                }
            }
        }

        // ---- Repeat U ----
        texture.repeat_u = cursor.read_f32::<LittleEndian>()?;
        loop {
            let (bitfield, size) = read_face_bitfield(&mut cursor)?;
            if bitfield == 0 {
                break;
            }
            for face_index in 0..size {
                let bit = 1 << face_index;
                if bitfield & bit != 0 {
                    faces.entry(face_index).or_default().repeat_u =
                        cursor.read_f32::<LittleEndian>()?;
                }
            }
        }

        // ---- Repeat V ----
        texture.repeat_v = cursor.read_f32::<LittleEndian>()?;
        loop {
            let (bitfield, size) = read_face_bitfield(&mut cursor)?;
            if bitfield == 0 {
                break;
            }
            for face_index in 0..size {
                let bit = 1 << face_index;
                if bitfield & bit != 0 {
                    faces.entry(face_index).or_default().repeat_v =
                        cursor.read_f32::<LittleEndian>()?;
                }
            }
        }

        // ---- Offset U ----
        texture.offset_u = (cursor.read_i16::<LittleEndian>()? as f32) / 32767.0;
        loop {
            let (bitfield, size) = read_face_bitfield(&mut cursor)?;
            if bitfield == 0 {
                break;
            }
            for face_index in 0..size {
                let bit = 1 << face_index;
                if bitfield & bit != 0 {
                    let val = (cursor.read_i16::<LittleEndian>()? as f32) / 32767.0;
                    faces.entry(face_index).or_default().offset_u = val;
                }
            }
        }

        // ---- Offset V ----
        texture.offset_v = (cursor.read_i16::<LittleEndian>()? as f32) / 32767.0;
        loop {
            let (bitfield, size) = read_face_bitfield(&mut cursor)?;
            if bitfield == 0 {
                break;
            }
            for face_index in 0..size {
                let bit = 1 << face_index;
                if bitfield & bit != 0 {
                    let val = (cursor.read_i16::<LittleEndian>()? as f32) / 32767.0;
                    faces.entry(face_index).or_default().offset_v = val;
                }
            }
        }

        // ---- Rotation ----
        {
            let rotation = cursor.read_i16::<LittleEndian>()? as f32;
            let wrapped = rotation.rem_euclid(TAU);
            let remainder = if wrapped > PI { wrapped - TAU } else { wrapped };
            let normalized = (remainder / TAU) * 32768.0 + 0.5;
            texture.rotation = normalized.round();

            loop {
                let (bitfield, size) = read_face_bitfield(&mut cursor)?;
                if bitfield == 0 {
                    break;
                }
                for face_index in 0..size {
                    let bit = 1 << face_index;
                    if bitfield & bit != 0 {
                        let rotation = cursor.read_i16::<LittleEndian>()? as f32;
                        let wrapped = rotation.rem_euclid(TAU);
                        let remainder = if wrapped > PI { wrapped - TAU } else { wrapped };
                        let normalized = (remainder / TAU) * 32768.0 + 0.5;
                        faces.entry(face_index).or_default().rotation = normalized.round();
                    }
                }
            }
        }

        // ---- Material ----
        texture.material = cursor.read_u8()?;
        loop {
            let (bitfield, size) = read_face_bitfield(&mut cursor)?;
            if bitfield == 0 {
                break;
            }
            for face_index in 0..size {
                let bit = 1 << face_index;
                if bitfield & bit != 0 {
                    faces.entry(face_index).or_default().material = cursor.read_u8()?;
                }
            }
        }

        // ---- Media ----
        texture.media = cursor.read_u8()?;
        loop {
            let (bitfield, size) = read_face_bitfield(&mut cursor)?;
            if bitfield == 0 {
                break;
            }
            for face_index in 0..size {
                let bit = 1 << face_index;
                if bitfield & bit != 0 {
                    faces.entry(face_index).or_default().media = cursor.read_u8()?;
                }
            }
        }

        // ---- Glow ----
        texture.glow = cursor.read_u8()? as f32 * 255.0;
        loop {
            let (bitfield, size) = read_face_bitfield(&mut cursor)?;
            if bitfield == 0 {
                break;
            }
            for face_index in 0..size {
                let bit = 1 << face_index;
                if bitfield & bit != 0 {
                    faces.entry(face_index).or_default().glow = cursor.read_u8()? as f32 * 255.0;
                }
            }
        }

        // ---- Material ID ----
        {
            let mut id_bytes = [0u8; 16];
            cursor.read_exact(&mut id_bytes)?;
            texture.material_id = Uuid::from_bytes(id_bytes);

            loop {
                let (bitfield, size) = read_face_bitfield(&mut cursor)?;
                if bitfield == 0 {
                    break;
                }
                for face_index in 0..size {
                    let bit = 1 << face_index;
                    if bitfield & bit != 0 {
                        let mut id_bytes = [0u8; 16];
                        cursor.read_exact(&mut id_bytes)?;
                        faces.entry(face_index).or_default().material_id =
                            Uuid::from_bytes(id_bytes);
                    }
                }
            }
        }

        Ok(texture)
    }
}

// variable-length bitfield reader stays as helper since it's core logic
fn read_face_bitfield(cursor: &mut Cursor<&[u8]>) -> std::io::Result<(u32, u32)> {
    let mut face_bits = 0u32;
    let mut bitfield_size = 0u32;
    loop {
        let mut b = [0u8];
        cursor.read_exact(&mut b)?;
        let b = b[0];
        face_bits = (face_bits << 7) | (b & 0x7F) as u32;
        bitfield_size += 7;
        if b & 0x80 == 0 {
            break;
        }
    }
    Ok((face_bits, bitfield_size))
}
