use std::{
    collections::HashMap,
    f32::consts::{PI, TAU},
    io::{Cursor, Read},
};

use base64::{Engine, engine::general_purpose};
use byteorder::{LittleEndian, ReadBytesExt};
use rgb::Rgba;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{errors::ParseError, parse};

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
    /// Convert from a b64 byte array to a TextureEntry object.
    /// Used by SceneObjects.
    pub fn from_b64(b64: &[u8]) -> Result<Self, ParseError> {
        let mut faces: HashMap<u32, TextureEntry> = HashMap::new();
        let mut texture = TextureEntry::default();

        if b64.len() < 16 {
            return Ok(texture);
        }

        let bytes = parse!(
            general_purpose::STANDARD
                .decode(b64)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        )?;

        let mut cursor = Cursor::new(&bytes[..]);

        let mut uuid = [0u8; 16];
        parse!(cursor.read_exact(&mut uuid))?;
        texture.texture_id = Uuid::from_bytes(uuid);

        loop {
            let (mask, _) = parse!(read_b64_face_bitfield(&mut cursor))?;

            if mask == 0 {
                break;
            }

            parse!(cursor.read_exact(&mut uuid))?;
            let id = Uuid::from_bytes(uuid);

            for_each_face(mask, |f| {
                faces.entry(f).or_default().texture_id = id;
            });
        }

        let mut c = [0u8; 4];
        parse!(cursor.read_exact(&mut c))?;

        for b in &mut c {
            *b = !*b;
        }

        texture.rgba = Rgba::from(c);

        loop {
            let (mask, _) = parse!(read_b64_face_bitfield(&mut cursor))?;

            if mask == 0 {
                break;
            }

            parse!(cursor.read_exact(&mut c))?;

            for b in &mut c {
                *b = !*b;
            }

            let col = Rgba::from(c);

            for_each_face(mask, |f| {
                faces.entry(f).or_default().rgba = col;
            });
        }

        texture.repeat_u = parse!(cursor.read_f32::<LittleEndian>())?;

        loop {
            let (mask, _) = parse!(read_b64_face_bitfield(&mut cursor))?;

            if mask == 0 {
                break;
            }

            let v = parse!(cursor.read_f32::<LittleEndian>())?;

            for_each_face(mask, |f| {
                faces.entry(f).or_default().repeat_u = v;
            });
        }

        texture.repeat_v = parse!(cursor.read_f32::<LittleEndian>())?;

        loop {
            let (mask, _) = parse!(read_b64_face_bitfield(&mut cursor))?;

            if mask == 0 {
                break;
            }

            let v = parse!(cursor.read_f32::<LittleEndian>())?;

            for_each_face(mask, |f| {
                faces.entry(f).or_default().repeat_v = v;
            });
        }

        texture.offset_u = parse!(cursor.read_i16::<LittleEndian>())? as f32 / 32767.0;

        loop {
            let (mask, _) = parse!(read_b64_face_bitfield(&mut cursor))?;

            if mask == 0 {
                break;
            }

            let v = parse!(cursor.read_i16::<LittleEndian>())? as f32 / 32767.0;

            for_each_face(mask, |f| {
                faces.entry(f).or_default().offset_u = v;
            });
        }

        texture.offset_v = parse!(cursor.read_i16::<LittleEndian>())? as f32 / 32767.0;

        loop {
            let (mask, _) = parse!(read_b64_face_bitfield(&mut cursor))?;

            if mask == 0 {
                break;
            }

            let v = parse!(cursor.read_i16::<LittleEndian>())? as f32 / 32767.0;

            for_each_face(mask, |f| {
                faces.entry(f).or_default().offset_v = v;
            });
        }

        texture.rotation = parse!(cursor.read_i16::<LittleEndian>())? as f32 / 32768.0 * TAU;

        loop {
            let (mask, _) = parse!(read_b64_face_bitfield(&mut cursor))?;

            if mask == 0 {
                break;
            }

            let r = parse!(cursor.read_i16::<LittleEndian>())? as f32 / 32768.0 * TAU;

            for_each_face(mask, |f| {
                faces.entry(f).or_default().rotation = r;
            });
        }

        texture.material = parse!(cursor.read_u8())?;

        loop {
            let (mask, _) = parse!(read_b64_face_bitfield(&mut cursor))?;

            if mask == 0 {
                break;
            }

            let v = parse!(cursor.read_u8())?;

            for_each_face(mask, |f| {
                faces.entry(f).or_default().material = v;
            });
        }

        texture.media = parse!(cursor.read_u8())?;

        loop {
            let (mask, _) = parse!(read_b64_face_bitfield(&mut cursor))?;

            if mask == 0 {
                break;
            }

            let v = parse!(cursor.read_u8())?;

            for_each_face(mask, |f| {
                faces.entry(f).or_default().media = v;
            });
        }

        texture.glow = parse!(cursor.read_u8())? as f32 / 255.0;

        loop {
            let (mask, _) = parse!(read_b64_face_bitfield(&mut cursor))?;

            if mask == 0 {
                break;
            }

            let g = parse!(cursor.read_u8())? as f32 / 255.0;

            for_each_face(mask, |f| {
                faces.entry(f).or_default().glow = g;
            });
        }

        let remaining = cursor.get_ref().len() - cursor.position() as usize;

        if remaining >= 16 {
            parse!(cursor.read_exact(&mut uuid))?;
            texture.material_id = Uuid::from_bytes(uuid);

            loop {
                if cursor.position() as usize >= cursor.get_ref().len() {
                    break;
                }

                let (mask, _) = parse!(read_b64_face_bitfield(&mut cursor))?;

                if mask == 0 {
                    break;
                }

                parse!(cursor.read_exact(&mut uuid))?;
                let id = Uuid::from_bytes(uuid);

                for_each_face(mask, |f| {
                    faces.entry(f).or_default().material_id = id;
                });
            }
        }

        for face in faces.values_mut() {
            face.inherit_missing(&texture);
        }

        Ok(texture)
    }

    /// Convert from bytes to a TextureEntry.
    /// Used by ObjectUpdate and ObjectUpdateCompressed packets.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut texture = TextureEntry::default();
        let mut faces: HashMap<u32, TextureEntry> = HashMap::new();

        if bytes.len() < 16 {
            return Ok(texture);
        }

        let mut cursor = Cursor::new(bytes);

        fn remaining(cursor: &Cursor<&[u8]>) -> usize {
            cursor.get_ref().len() - cursor.position() as usize
        }

        let mut uuid = [0u8; 16];
        // this needs to be here. There's two padding bytes for the texture entries.
        let _pad = parse!(cursor.read_u16::<LittleEndian>())?;
        parse!(cursor.read_exact(&mut uuid))?;

        texture.texture_id = Uuid::from_bytes(uuid);

        // Texture IDs per face
        loop {
            if remaining(&cursor) < 1 {
                break;
            }

            let mask = parse!(read_face_bitfield(&mut cursor))?;

            if mask == 0 {
                break;
            }

            if remaining(&cursor) < 16 {
                break;
            }

            parse!(cursor.read_exact(&mut uuid))?;

            let id = Uuid::from_bytes(uuid);

            for_each_face(mask, |f| {
                faces.entry(f).or_default().texture_id = id;
            });
        }

        // RGBA
        if remaining(&cursor) >= 4 {
            let mut rgba = [0u8; 4];

            parse!(cursor.read_exact(&mut rgba))?;

            for c in &mut rgba {
                *c = !*c;
            }

            texture.rgba = Rgba::from(rgba);

            loop {
                if remaining(&cursor) < 1 {
                    break;
                }

                let mask = parse!(read_face_bitfield(&mut cursor))?;

                if mask == 0 {
                    break;
                }

                if remaining(&cursor) < 4 {
                    break;
                }

                parse!(cursor.read_exact(&mut rgba))?;

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
            texture.repeat_u = parse!(cursor.read_f32::<LittleEndian>())?;

            loop {
                if remaining(&cursor) < 1 {
                    break;
                }
                let mask = parse!(read_face_bitfield(&mut cursor))?;

                if mask == 0 {
                    break;
                }

                if remaining(&cursor) < 4 {
                    break;
                }
                let v = parse!(cursor.read_f32::<LittleEndian>())?;

                for_each_face(mask, |f| {
                    faces.entry(f).or_default().repeat_u = v;
                });
            }
        }

        // repeat_v
        if remaining(&cursor) >= 4 {
            texture.repeat_v = parse!(cursor.read_f32::<LittleEndian>())?;

            loop {
                if remaining(&cursor) < 1 {
                    break;
                }
                let mask = parse!(read_face_bitfield(&mut cursor))?;

                if mask == 0 {
                    break;
                }

                if remaining(&cursor) < 4 {
                    break;
                }
                let v = parse!(cursor.read_f32::<LittleEndian>())?;

                for_each_face(mask, |f| {
                    faces.entry(f).or_default().repeat_v = v;
                });
            }
        }

        // offset_u
        if remaining(&cursor) >= 2 {
            let raw = parse!(cursor.read_i16::<LittleEndian>())?;
            texture.offset_u = raw as f32 / 32767.0;

            loop {
                if remaining(&cursor) < 1 {
                    break;
                }

                let mask = parse!(read_face_bitfield(&mut cursor))?;

                if mask == 0 {
                    break;
                }

                if remaining(&cursor) < 2 {
                    break;
                }

                let raw = parse!(cursor.read_i16::<LittleEndian>())?;
                let v = raw as f32 / 32767.0;

                for_each_face(mask, |f| {
                    faces.entry(f).or_default().offset_u = v;
                });
            }
        }

        // offset_v
        if remaining(&cursor) >= 2 {
            let raw = parse!(cursor.read_i16::<LittleEndian>())?;
            texture.offset_v = raw as f32 / 32767.0;

            loop {
                if remaining(&cursor) < 1 {
                    break;
                }

                let mask = parse!(read_face_bitfield(&mut cursor))?;

                if mask == 0 {
                    break;
                }

                if remaining(&cursor) < 2 {
                    break;
                }

                let raw = parse!(cursor.read_i16::<LittleEndian>())?;
                let v = raw as f32 / 32767.0;

                for_each_face(mask, |f| {
                    faces.entry(f).or_default().offset_v = v;
                });
            }
        }

        // rotation
        if remaining(&cursor) >= 2 {
            let raw = parse!(cursor.read_i16::<LittleEndian>())?;
            texture.rotation = raw as f32 * PI / 32767.0;

            loop {
                if remaining(&cursor) < 1 {
                    break;
                }

                let mask = parse!(read_face_bitfield(&mut cursor))?;

                if mask == 0 {
                    break;
                }

                if remaining(&cursor) < 2 {
                    break;
                }

                let raw = parse!(cursor.read_i16::<LittleEndian>())?;
                let r = raw as f32 * PI / 32767.0;

                for_each_face(mask, |f| {
                    faces.entry(f).or_default().rotation = r;
                });
            }
        }

        // material
        if remaining(&cursor) >= 1 {
            texture.material = parse!(cursor.read_u8())?;

            loop {
                if remaining(&cursor) < 1 {
                    break;
                }

                let mask = parse!(read_face_bitfield(&mut cursor))?;

                if mask == 0 {
                    break;
                }

                if remaining(&cursor) < 1 {
                    break;
                }

                let v = parse!(cursor.read_u8())?;

                for_each_face(mask, |f| {
                    faces.entry(f).or_default().material = v;
                });
            }
        }

        // media
        if remaining(&cursor) >= 1 {
            texture.media = parse!(cursor.read_u8())?;

            loop {
                if remaining(&cursor) < 1 {
                    break;
                }

                let mask = parse!(read_face_bitfield(&mut cursor))?;

                if mask == 0 {
                    break;
                }

                if remaining(&cursor) < 1 {
                    break;
                }

                let v = parse!(cursor.read_u8())?;

                for_each_face(mask, |f| {
                    faces.entry(f).or_default().media = v;
                });
            }
        }

        // glow
        if remaining(&cursor) >= 1 {
            let raw = parse!(cursor.read_u8())?;
            texture.glow = raw as f32 / 255.0;

            loop {
                if remaining(&cursor) < 1 {
                    break;
                }

                let mask = parse!(read_face_bitfield(&mut cursor))?;

                if mask == 0 {
                    break;
                }

                if remaining(&cursor) < 1 {
                    break;
                }

                let raw = parse!(cursor.read_u8())?;
                let g = raw as f32 / 255.0;

                for_each_face(mask, |f| {
                    faces.entry(f).or_default().glow = g;
                });
            }
        }

        // material_id
        if remaining(&cursor) >= 16 {
            parse!(cursor.read_exact(&mut uuid))?;

            texture.material_id = Uuid::from_bytes(uuid);

            loop {
                if remaining(&cursor) < 1 {
                    break;
                }

                let mask = parse!(read_face_bitfield(&mut cursor))?;

                if mask == 0 {
                    break;
                }

                if remaining(&cursor) < 16 {
                    break;
                }

                parse!(cursor.read_exact(&mut uuid))?;

                let id = Uuid::from_bytes(uuid);

                for_each_face(mask, |f| {
                    faces.entry(f).or_default().material_id = id;
                });
            }
        }

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

fn read_face_bitfield<R: Read>(r: &mut R) -> Result<u32, ParseError> {
    Ok(read_b64_face_bitfield(r)?.0)
}

fn read_b64_face_bitfield<R: Read>(r: &mut R) -> Result<(u32, u32), ParseError> {
    let mut face_bits = 0u32;
    let mut bitfield_size = 0u32;

    loop {
        let b = parse!(r.read_u8())?;

        face_bits = (face_bits << 7) | (b & 0x7F) as u32;
        bitfield_size += 7;

        if b & 0x80 == 0 {
            break;
        }
    }

    Ok((face_bits, bitfield_size))
}
