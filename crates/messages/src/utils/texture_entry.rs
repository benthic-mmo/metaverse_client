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
pub struct TextureEntry {
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

impl Default for TextureEntry {
    fn default() -> Self {
        TextureEntry {
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

impl TextureEntry {
    pub fn from_b64(b64: &[u8]) -> std::io::Result<Self> {
        let mut faces: HashMap<u32, TextureEntry> = HashMap::new();
        let mut texture = TextureEntry::default();
        if b64.len() < 16 {
            return Ok(texture);
        }
        let bytes = general_purpose::STANDARD.decode(b64).unwrap();
        let mut cursor = Cursor::new(&bytes[..]);

        {
            let mut id_bytes = [0u8; 16];
            cursor.read_exact(&mut id_bytes)?;
            texture.texture_id = Uuid::from_bytes(id_bytes);
            loop {
                let (bitfield, size) = read_b64_face_bitfield(&mut cursor)?;
                if bitfield == 0 {
                    break;
                }
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

        {
            let red = cursor.read_u8()?;
            let green = cursor.read_u8()?;
            let blue = cursor.read_u8()?;
            let alpha = cursor.read_u8()?;
            texture.rgba = Rgba::new(red, green, blue, alpha);
            loop {
                let (bitfield, size) = read_b64_face_bitfield(&mut cursor)?;
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

        texture.repeat_u = cursor.read_f32::<LittleEndian>()?;
        loop {
            let (bitfield, size) = read_b64_face_bitfield(&mut cursor)?;
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

        texture.repeat_v = cursor.read_f32::<LittleEndian>()?;
        loop {
            let (bitfield, size) = read_b64_face_bitfield(&mut cursor)?;
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

        texture.offset_u = (cursor.read_i16::<LittleEndian>()? as f32) / 32767.0;
        loop {
            let (bitfield, size) = read_b64_face_bitfield(&mut cursor)?;
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

        texture.offset_v = (cursor.read_i16::<LittleEndian>()? as f32) / 32767.0;
        loop {
            let (bitfield, size) = read_b64_face_bitfield(&mut cursor)?;
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

        {
            let rotation = cursor.read_i16::<LittleEndian>()? as f32;
            let wrapped = rotation.rem_euclid(TAU);
            let remainder = if wrapped > PI { wrapped - TAU } else { wrapped };
            let normalized = (remainder / TAU) * 32768.0 + 0.5;
            texture.rotation = normalized.round();
            loop {
                let (bitfield, size) = read_b64_face_bitfield(&mut cursor)?;
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

        texture.material = cursor.read_u8()?;
        loop {
            let (bitfield, size) = read_b64_face_bitfield(&mut cursor)?;
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

        texture.media = cursor.read_u8()?;
        loop {
            let (bitfield, size) = read_b64_face_bitfield(&mut cursor)?;
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

        texture.glow = cursor.read_u8()? as f32 * 255.0;
        loop {
            let (bitfield, size) = read_b64_face_bitfield(&mut cursor)?;
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

        {
            let mut id_bytes = [0u8; 16];
            cursor.read_exact(&mut id_bytes)?;
            texture.material_id = Uuid::from_bytes(id_bytes);
            loop {
                let (bitfield, size) = read_b64_face_bitfield(&mut cursor)?;
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

        // inherit missing fields to faces
        for face in faces.values_mut() {
            face.inherit_missing(&texture);
        }

        Ok(texture)
    }

    /// convert from bytes to a texture
    pub fn from_bytes(bytes: &[u8]) -> std::io::Result<Self> {
        let mut texture = TextureEntry::default();
        let mut faces: HashMap<u32, TextureEntry> = HashMap::new();
        if bytes.len() < 16 {
            return Ok(texture);
        }
        let mut cursor = Cursor::new(bytes);

        // two mysterious bytes of padding?
        cursor.read_u8()?;
        cursor.read_u8()?;

        fn remaining(cursor: &Cursor<&[u8]>) -> usize {
            cursor.get_ref().len() - cursor.position() as usize
        }

        let mut uuid = [0u8; 16];
        cursor.read_exact(&mut uuid)?;
        texture.texture_id = Uuid::from_bytes(uuid);

        // Texture IDs per face
        loop {
            if remaining(&cursor) < 1 {
                break;
            }
            let mask = read_face_bitfield(&mut cursor)?;
            if mask == 0 {
                break;
            }
            if remaining(&cursor) < 16 {
                break;
            }
            cursor.read_exact(&mut uuid)?;
            let id = Uuid::from_bytes(uuid);
            for_each_face(mask, |f| {
                faces.entry(f).or_default().texture_id = id;
            });
        }

        // RGBA
        if remaining(&cursor) >= 4 {
            let mut rgba = [0u8; 4];
            cursor.read_exact(&mut rgba)?;
            for c in &mut rgba {
                *c = !*c;
            }
            texture.rgba = Rgba::from(rgba);

            loop {
                if remaining(&cursor) < 1 {
                    break;
                }
                let mask = read_face_bitfield(&mut cursor)?;
                if mask == 0 {
                    break;
                }
                if remaining(&cursor) < 4 {
                    break;
                }
                cursor.read_exact(&mut rgba)?;
                for c in &mut rgba {
                    *c = !*c;
                }
                let c = Rgba::from(rgba);
                for_each_face(mask, |f| {
                    faces.entry(f).or_default().rgba = c;
                });
            }
        }

        // repeat_u
        if remaining(&cursor) >= 4 {
            texture.repeat_u = cursor.read_f32::<LittleEndian>()?;
            loop {
                if remaining(&cursor) < 1 {
                    break;
                }
                let mask = read_face_bitfield(&mut cursor)?;
                if mask == 0 {
                    break;
                }
                if remaining(&cursor) < 4 {
                    break;
                }
                let v = cursor.read_f32::<LittleEndian>()?;
                for_each_face(mask, |f| {
                    faces.entry(f).or_default().repeat_u = v;
                });
            }
        }

        // repeat_v
        if remaining(&cursor) >= 4 {
            texture.repeat_v = cursor.read_f32::<LittleEndian>()?;
            loop {
                if remaining(&cursor) < 1 {
                    break;
                }
                let mask = read_face_bitfield(&mut cursor)?;
                if mask == 0 {
                    break;
                }
                if remaining(&cursor) < 4 {
                    break;
                }
                let v = cursor.read_f32::<LittleEndian>()?;
                for_each_face(mask, |f| {
                    faces.entry(f).or_default().repeat_v = v;
                });
            }
        }

        // offset_u
        if remaining(&cursor) >= 2 {
            texture.offset_u = cursor.read_i16::<LittleEndian>()? as f32 / 32767.0;
            loop {
                if remaining(&cursor) < 1 {
                    break;
                }
                let mask = read_face_bitfield(&mut cursor)?;
                if mask == 0 {
                    break;
                }
                if remaining(&cursor) < 2 {
                    break;
                }
                let v = cursor.read_i16::<LittleEndian>()? as f32 / 32767.0;
                for_each_face(mask, |f| {
                    faces.entry(f).or_default().offset_u = v;
                });
            }
        }

        // offset_v
        if remaining(&cursor) >= 2 {
            texture.offset_v = cursor.read_i16::<LittleEndian>()? as f32 / 32767.0;
            loop {
                if remaining(&cursor) < 1 {
                    break;
                }
                let mask = read_face_bitfield(&mut cursor)?;
                if mask == 0 {
                    break;
                }
                if remaining(&cursor) < 2 {
                    break;
                }
                let v = cursor.read_i16::<LittleEndian>()? as f32 / 32767.0;
                for_each_face(mask, |f| {
                    faces.entry(f).or_default().offset_v = v;
                });
            }
        }

        // rotation
        if remaining(&cursor) >= 2 {
            texture.rotation = cursor.read_i16::<LittleEndian>()? as f32 * PI / 32767.0;
            loop {
                if remaining(&cursor) < 1 {
                    break;
                }
                let mask = read_face_bitfield(&mut cursor)?;
                if mask == 0 {
                    break;
                }
                if remaining(&cursor) < 2 {
                    break;
                }
                let r = cursor.read_i16::<LittleEndian>()? as f32 * PI / 32767.0;
                for_each_face(mask, |f| {
                    faces.entry(f).or_default().rotation = r;
                });
            }
        }

        // material
        if remaining(&cursor) >= 1 {
            texture.material = cursor.read_u8()?;
            loop {
                if remaining(&cursor) < 1 {
                    break;
                }
                let mask = read_face_bitfield(&mut cursor)?;
                if mask == 0 {
                    break;
                }
                if remaining(&cursor) < 1 {
                    break;
                }
                let v = cursor.read_u8()?;
                for_each_face(mask, |f| {
                    faces.entry(f).or_default().material = v;
                });
            }
        }

        // media
        if remaining(&cursor) >= 1 {
            texture.media = cursor.read_u8()?;
            loop {
                if remaining(&cursor) < 1 {
                    break;
                }
                let mask = read_face_bitfield(&mut cursor)?;
                if mask == 0 {
                    break;
                }
                if remaining(&cursor) < 1 {
                    break;
                }
                let v = cursor.read_u8()?;
                for_each_face(mask, |f| {
                    faces.entry(f).or_default().media = v;
                });
            }
        }

        // glow
        if remaining(&cursor) >= 1 {
            texture.glow = cursor.read_u8()? as f32 / 255.0;
            loop {
                if remaining(&cursor) < 1 {
                    break;
                }
                let mask = read_face_bitfield(&mut cursor)?;
                if mask == 0 {
                    break;
                }
                if remaining(&cursor) < 1 {
                    break;
                }
                let g = cursor.read_u8()? as f32 / 255.0;
                for_each_face(mask, |f| {
                    faces.entry(f).or_default().glow = g;
                });
            }
        }

        // material_id
        if remaining(&cursor) >= 16 {
            cursor.read_exact(&mut uuid)?;
            texture.material_id = Uuid::from_bytes(uuid);
            loop {
                if remaining(&cursor) < 1 {
                    break;
                }
                let mask = read_face_bitfield(&mut cursor)?;
                if mask == 0 {
                    break;
                }
                if remaining(&cursor) < 16 {
                    break;
                }
                cursor.read_exact(&mut uuid)?;
                let id = Uuid::from_bytes(uuid);
                for_each_face(mask, |f| {
                    faces.entry(f).or_default().material_id = id;
                });
            }
        }

        // inherit missing fields to faces
        for face in faces.values_mut() {
            face.inherit_missing(&texture);
        }

        Ok(texture)
    }

    fn inherit_missing(&mut self, base: &TextureEntry) {
        if self.texture_id.is_nil() {
            self.texture_id = base.texture_id;
        }
        if self.rgba == Rgba::default() {
            self.rgba = base.rgba;
        }
        if self.repeat_u == 0.0 {
            self.repeat_u = base.repeat_u;
        }
        if self.repeat_v == 0.0 {
            self.repeat_v = base.repeat_v;
        }
        if self.offset_u == 0.0 {
            self.offset_u = base.offset_u;
        }
        if self.offset_v == 0.0 {
            self.offset_v = base.offset_v;
        }
        if self.rotation == 0.0 {
            self.rotation = base.rotation;
        }
        if self.material == 0 {
            self.material = base.material;
        }
        if self.media == 0 {
            self.media = base.media;
        }
        if self.glow == 0.0 {
            self.glow = base.glow;
        }
        if self.material_id.is_nil() {
            self.material_id = base.material_id;
        }
    }
}

#[inline]
fn for_each_face(mask: u32, mut f: impl FnMut(u32)) {
    for face in 0..32 {
        if mask & (1 << face) != 0 {
            f(face);
        }
    }
}

fn read_face_bitfield<R: Read>(r: &mut R) -> std::io::Result<u32> {
    let first = r.read_u8()?;
    if first == 0 {
        return Ok(0);
    }
    if first & 0x80 == 0 {
        return Ok(first as u32);
    }
    let second = r.read_u8()?;
    let mut value = ((first as u32 & 0x7F) << 7) | (second as u32 & 0x7F);
    if second & 0x80 == 0 {
        return Ok(value);
    }
    let third = r.read_u8()?;
    let fourth = r.read_u8()?;
    value |= (third as u32) << 14;
    value |= (fourth as u32) << 22;
    Ok(value)
}

fn read_b64_face_bitfield(cursor: &mut Cursor<&[u8]>) -> std::io::Result<(u32, u32)> {
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

