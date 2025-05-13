use std::{
    collections::HashMap,
    f32::consts::{PI, TAU},
    io::{Cursor, Read},
};

use byteorder::{LittleEndian, ReadBytesExt};
use rgb::Rgba;
use uuid::Uuid;

pub fn parse_texture_data(bytes: &Vec<u8>) -> std::io::Result<String> {
    let mut faces: HashMap<u32, Texture> = HashMap::new();
    let texture_entry = Texture::from_bytes(&bytes, &mut faces)?;

    println!("{:?}", faces);
    println!("texture entry: {:?}", texture_entry);
    Ok("hello".to_string())
}

pub fn parse_visual_param_data(_: &Vec<u8>) -> std::io::Result<String> {
    Ok("visual param".to_string())
}

#[derive(Debug, Clone)]
pub struct Texture {
    pub texture_id: Uuid,
    pub rgba: Rgba<u8>,
    pub repeat_u: f32,
    pub repeat_v: f32,
    pub offset_u: f32,
    pub offset_v: f32,
    pub rotation: f32,
    pub material: u8,
    pub media: u8,
    pub glow: f32,
    pub material_id: Uuid,
}

impl Default for Texture {
    fn default() -> Self {
        Texture {
            texture_id: Uuid::nil(),
            rgba: Rgba {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
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
    fn from_bytes(bytes: &[u8], faces: &mut HashMap<u32, Texture>) -> std::io::Result<Self> {
        let mut default_texture = Texture::default();
        if bytes.len() < 16 {
            return Ok(default_texture);
        }
        let mut cursor = Cursor::new(bytes);
        read_texture_id(&mut cursor, &mut default_texture)?;

        // apply the texture_id field to the other faces in the packet
        create_faces(&mut cursor, faces, |cursor, texture| {
            read_texture_id(cursor, texture)
        })?;

        read_color(&mut cursor, &mut default_texture)?;
        // apply the color field to the other faces in the packet
        create_faces(&mut cursor, faces, |cursor, texture| {
            read_color(cursor, texture)
        })?;

        default_texture.repeat_u = cursor.read_f32::<LittleEndian>()?;
        // apply the repeat_u field to the other faces in the packet
        create_faces(&mut cursor, faces, |cursor, texture| {
            texture.repeat_u = cursor.read_f32::<LittleEndian>()?;
            Ok(())
        })?;

        default_texture.repeat_v = cursor.read_f32::<LittleEndian>()?;
        // apply the repeat_v field to the other faces in the packet
        create_faces(&mut cursor, faces, |cursor, texture| {
            texture.repeat_v = cursor.read_f32::<LittleEndian>()?;
            Ok(())
        })?;

        default_texture.offset_u = (cursor.read_i16::<LittleEndian>()? as f32) / 32767.0;
        // apply the offset_u field to the other faces in the packet
        create_faces(&mut cursor, faces, |cursor, texture| {
            texture.offset_u = (cursor.read_i16::<LittleEndian>()? as f32) / 32767.0;
            Ok(())
        })?;

        default_texture.offset_v = (cursor.read_i16::<LittleEndian>()? as f32) / 32767.0;
        // apply the offset_v field to the other faces in the packet
        create_faces(&mut cursor, faces, |cursor, texture| {
            texture.offset_v = (cursor.read_i16::<LittleEndian>()? as f32) / 32767.0;
            Ok(())
        })?;

        read_rotation(&mut cursor, &mut default_texture)?;
        // apply the rotation field to the other faces in the packet
        create_faces(&mut cursor, faces, |cursor, texture| {
            read_rotation(cursor, texture)
        })?;

        default_texture.material = cursor.read_u8()?;
        // apply the material field to the other faces in the packet
        create_faces(&mut cursor, faces, |cursor, texture| {
            texture.material = cursor.read_u8()?;
            Ok(())
        })?;

        default_texture.media = cursor.read_u8()?;
        // apply the media field to the other faces in the packet
        create_faces(&mut cursor, faces, |cursor, texture| {
            texture.media = cursor.read_u8()?;
            Ok(())
        })?;

        default_texture.glow = cursor.read_u8()? as f32 * 255.0;
        // apply the media field to the other faces in the packet
        create_faces(&mut cursor, faces, |cursor, texture| {
            texture.glow = cursor.read_u8()? as f32 * 255.0;
            Ok(())
        })?;

        read_material_id(&mut cursor, &mut default_texture)?;
        // apply the material ID field to the other faces in the packet create_faces(cursor, faces, read_material_id(&mut cursor, texture))
        create_faces(&mut cursor, faces, |cursor, texture| {
            read_material_id(cursor, texture)
        })?;

        println!("default texture: {:?}", default_texture);
        Ok(default_texture)
    }
}

#[derive(Debug, Clone)]
/// slider value of visual parameters
pub struct VisualParam {}

#[derive(Debug, Clone)]
pub struct AppearanceData {
    pub appearance_version: u8,
    pub current_outfit_folder_version: f32,
    pub flags: u32,
}

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
            break; // Exit loop when the high bit is not set (indicating end of variable-length field)
        }
    }

    Ok((face_bits, bitfield_size))
}

/// read_and_apply is a function that reads the correct data from the cursor, and applies it to the
/// face.
fn create_faces<F>(
    cursor: &mut Cursor<&[u8]>,
    faces: &mut HashMap<u32, Texture>,
    mut read_and_apply: F,
) -> std::io::Result<()>
where
    F: FnMut(&mut Cursor<&[u8]>, &mut Texture) -> std::io::Result<()>,
{
    loop {
        let (bitfield, size) = read_face_bitfield(cursor)?;
        if bitfield == 0 {
            break;
        }

        for face_index in 0..size {
            let bit = 1 << face_index;
            if bitfield & bit != 0 {
                let face = faces.entry(face_index).or_insert_with(Texture::default);
                read_and_apply(cursor, face)?;
            }
        }
    }
    Ok(())
}

fn read_rotation(cursor: &mut Cursor<&[u8]>, texture: &mut Texture) -> std::io::Result<()> {
    let rotation = cursor.read_i16::<LittleEndian>()? as f32;
    let wrapped = rotation.rem_euclid(TAU);
    let remainder = if wrapped > PI { wrapped - TAU } else { wrapped };

    let normalized = (remainder / TAU) * 32768.0 + 0.5;
    texture.rotation = normalized.round();
    Ok(())
}

fn read_texture_id(cursor: &mut Cursor<&[u8]>, texture: &mut Texture) -> std::io::Result<()> {
    let mut id_bytes = [0u8; 16];
    cursor.read_exact(&mut id_bytes)?;
    texture.texture_id = Uuid::from_bytes(id_bytes);
    Ok(())
}

fn read_material_id(cursor: &mut Cursor<&[u8]>, texture: &mut Texture) -> std::io::Result<()> {
    let mut id_bytes = [0u8; 16];
    cursor.read_exact(&mut id_bytes)?;
    texture.material_id = Uuid::from_bytes(id_bytes);
    Ok(())
}

fn read_color(cursor: &mut Cursor<&[u8]>, texture: &mut Texture) -> std::io::Result<()> {
    let red = cursor.read_u8()?;
    let green = cursor.read_u8()?;
    let blue = cursor.read_u8()?;
    let alpha = cursor.read_u8()?;
    texture.rgba = Rgba::new(red, green, blue, alpha);
    Ok(())
}
