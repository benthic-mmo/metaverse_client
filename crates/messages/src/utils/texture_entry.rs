use crate::{errors::ParseError, parse};
use base64::{Engine, engine::general_purpose};
use benthic_protocol::objects::types::{BumpMap, ShinyLevel, TexGen};
use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt};
use rgb::Rgba;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    f32::consts::TAU,
    io::{Cursor, Read},
};
use uuid::Uuid;

bitflags! {
    /// Face bitmask identifying which faces a texture targets.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FaceIndices: u32 {
        /// Targets face index 0
        const FACE_0 = 1 << 0;
        /// Targets face index 1
        const FACE_1 = 1 << 1;
        /// Targets face index 2
        const FACE_2 = 1 << 2;
        /// Targets face index 3
        const FACE_3 = 1 << 3;
        /// Targets face index 4
        const FACE_4 = 1 << 4;
        /// Targets face index 5
        const FACE_5 = 1 << 5;
        /// Targets face index 6
        const FACE_6 = 1 << 6;
        /// Targets face index 7
        const FACE_7 = 1 << 7;
        /// Targets face index 8
        const FACE_8 = 1 << 8;
        /// Targets face index 9
        const FACE_9 = 1 << 9;
        /// Targets face index 10
        const FACE_10 = 1 << 10;
        /// Targets face index 11
        const FACE_11 = 1 << 11;
        /// Targets face index 12
        const FACE_12 = 1 << 12;
        /// Targets face index 13
        const FACE_13 = 1 << 13;
        /// Targets face index 14
        const FACE_14 = 1 << 14;
        /// Targets face index 15
        const FACE_15 = 1 << 15;
        /// Targets face index 16
        const FACE_16 = 1 << 16;
        /// Targets face index 17
        const FACE_17 = 1 << 17;
        /// Targets face index 18
        const FACE_18 = 1 << 18;
        /// Targets face index 19
        const FACE_19 = 1 << 19;
        /// Targets face index 20
        const FACE_20 = 1 << 20;
        /// Targets face index 21
        const FACE_21 = 1 << 21;
        /// Targets face index 22
        const FACE_22 = 1 << 22;
        /// Targets face index 23
        const FACE_23 = 1 << 23;
        /// Targets face index 24
        const FACE_24 = 1 << 24;
        /// Targets face index 25
        const FACE_25 = 1 << 25;
        /// Targets face index 26
        const FACE_26 = 1 << 26;
        /// Targets face index 27
        const FACE_27 = 1 << 27;
        /// Targets face index 28
        const FACE_28 = 1 << 28;
        /// Targets face index 29
        const FACE_29 = 1 << 29;
        /// Targets face index 30
        const FACE_30 = 1 << 30;
        /// Targets face index 31
        const FACE_31 = 1 << 31;
    }
}

// bitmask for a fullbright texture
const FULLBRIGHT_MASK: u8 = 0x01;
/// fullbright is stored at bit position 5
const FULLBRIGHT_SHIFT: u8 = 5;
/// SHininess is stored at bit position 6
const SHINY_SHIFT: u8 = 6;
/// bit 0 of a media related flag field to determine if it is set or not
const MEDIA_MASK: u8 = 0x01;

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
    /// type of bump texture applied to this texture. Allows for creating visually rough surfaces without
    /// having to create rough geometry.
    pub bumpmap: BumpMap,
    /// If this texture is affected by lighting or is at full brightness
    pub fullbright: bool,
    /// shininess of the texture
    pub shiny: ShinyLevel,
    /// media flag to allow playing media on the face.
    /// The media URL rquired for this is stored in the objectupdate or scene object.
    pub has_media: bool,
    /// Tells the core how to calculate texture UVs for a face.
    /// determines how a texture is projected across a face
    pub tex_gen: TexGen,
    /// brightness and color of a glow
    pub glow: f32,
    /// Asset UUID of the glTF PBR material for this face (null = legacy Blinn-Phong).  
    pub material_id: Uuid,
}

impl Default for TextureEntry {
    fn default() -> Self {
        Self {
            texture_id: Uuid::nil(),
            rgba: Rgba::new(255, 255, 255, 255),
            repeat_u: 1.0,
            repeat_v: 1.0,
            offset_u: 0.0,
            offset_v: 0.0,
            rotation: 0.0,
            bumpmap: BumpMap::None,
            fullbright: false,
            shiny: ShinyLevel::Low,
            has_media: false,
            tex_gen: TexGen::Default,
            glow: 0.0,
            material_id: Uuid::nil(),
        }
    }
}
impl TextureEntry {
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
        if self.bumpmap == BumpMap::None {
            self.bumpmap = base.bumpmap;
        }
        if !self.fullbright {
            self.fullbright = base.fullbright;
        }
        if self.shiny == ShinyLevel::Low {
            self.shiny = base.shiny;
        }
        if !self.has_media {
            self.has_media = base.has_media;
        }
        if self.tex_gen == TexGen::Default {
            self.tex_gen = base.tex_gen;
        }
        if self.glow == 0.0 {
            self.glow = base.glow;
        }
        if self.material_id.is_nil() {
            self.material_id = base.material_id;
        }
    }
}

/// Texture information per-face
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct TextureEntries {
    /// Default texture. This is the texture that is applied if the face does not have a texture
    /// override
    pub default: TextureEntry,
    /// per-face texture overrides.
    /// u32 is the id of the face, the TextureEntry is the texture data.
    pub faces: HashMap<u32, TextureEntry>,
}
impl TextureEntries {
    /// Convert from a b64 byte array to a TextureEntry object.
    /// Used by SceneObjects.
    pub fn from_b64(b64: &[u8]) -> Result<Self, ParseError> {
        let mut faces: HashMap<u32, TextureEntry> = HashMap::new();
        let mut texture = TextureEntry::default();

        if b64.len() < 16 {
            return Ok(TextureEntries {
                default: texture,
                faces,
            });
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

        let bump = parse!(cursor.read_u8())?;
        texture.bumpmap = BumpMap::from_bits(bump);
        texture.fullbright = (bump >> FULLBRIGHT_SHIFT) & FULLBRIGHT_MASK != 0;
        texture.shiny = ShinyLevel::from_bits(bump >> SHINY_SHIFT);

        loop {
            let (mask, _) = parse!(read_b64_face_bitfield(&mut cursor))?;
            if mask == 0 {
                break;
            }
            let v = parse!(cursor.read_u8())?;

            for_each_face(mask, |f| {
                let face = faces.entry(f).or_default();
                face.bumpmap = BumpMap::from_bits(v);
                face.fullbright = (v >> FULLBRIGHT_SHIFT) & FULLBRIGHT_MASK != 0;
                face.shiny = ShinyLevel::from_bits(v >> SHINY_SHIFT);
            });
        }

        // media/texgen byte (.....TTM)
        let media = parse!(cursor.read_u8())?;
        texture.has_media = media & MEDIA_MASK != 0;
        texture.tex_gen = TexGen::from_bits(media);

        loop {
            let (mask, _) = parse!(read_b64_face_bitfield(&mut cursor))?;
            if mask == 0 {
                break;
            }
            let v = parse!(cursor.read_u8())?;

            for_each_face(mask, |f| {
                let face = faces.entry(f).or_default();
                face.has_media = v & MEDIA_MASK != 0;
                face.tex_gen = TexGen::from_bits(v);
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

        Ok(TextureEntries {
            default: texture,
            faces,
        })
    }

    /// Convert from bytes to a TextureEntry.
    /// Used by ObjectUpdate and ObjectUpdateCompressed packets.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut texture = TextureEntry::default();
        let mut faces: HashMap<u32, TextureEntry> = HashMap::new();

        if bytes.len() < 16 {
            return Ok(TextureEntries {
                default: texture,
                faces,
            });
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
            texture.rotation = raw as f32 / 32768.0 * TAU;

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
                let r = raw as f32 / 32768.0 * TAU;

                for_each_face(mask, |f| {
                    faces.entry(f).or_default().rotation = r;
                });
            }
        }

        if remaining(&cursor) >= 1 {
            let bump = parse!(cursor.read_u8())?;
            texture.bumpmap = BumpMap::from_bits(bump);
            texture.fullbright = (bump >> FULLBRIGHT_SHIFT) & 1 != 0;
            texture.shiny = ShinyLevel::from_bits(bump >> SHINY_SHIFT);

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
                    let face = faces.entry(f).or_default();
                    face.bumpmap = BumpMap::from_bits(v);
                    face.fullbright = (v >> FULLBRIGHT_SHIFT) & 1 != 0;
                    face.shiny = ShinyLevel::from_bits(v >> SHINY_SHIFT);
                });
            }
        }

        if remaining(&cursor) >= 1 {
            let media = parse!(cursor.read_u8())?;
            texture.has_media = media & MEDIA_MASK != 0;
            texture.tex_gen = TexGen::from_bits(media);

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
                    let face = faces.entry(f).or_default();
                    face.has_media = v & MEDIA_MASK != 0;
                    face.tex_gen = TexGen::from_bits(v);
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

        Ok(TextureEntries {
            default: texture,
            faces,
        })
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
