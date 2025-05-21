use flate2::bufread::ZlibDecoder;
use glam::{Vec3, Vec4};
use serde_llsd::LLSDValue;
use std::{
    collections::{BTreeMap, HashMap},
    io::{Error, ErrorKind, Read},
};

/// This is the Zlib magic number. In the binary, this is where the start of the zipped data
/// begins. This is followed by
/// 1   (No compression),
/// 94  (Fast compression),
/// 156 (Default compression),
/// 218 (Best compression ),
const ZLIB_MAGIC_NUMBER: u8 = 120;
const ZLIB_DECODING_TYPE: u8 = 218;

#[derive(Debug, Default)]
pub struct Mesh {
    high_level_of_detail: LevelOfDetail,
    medium_level_of_detail: Option<LevelOfDetail>,
    low_level_of_detail: Option<LevelOfDetail>,
    lowest_level_of_detail: Option<LevelOfDetail>,
    physics_convex: Vec<u8>,
    skin: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct LevelOfDetail {
    no_geometry: bool,
    position_domain: Option<PositionDomain>,
    weights: Vec<JointWeight>,
    texture_coordinate: Vec<TextureCoordinate>,
    texture_coordinate_domain: TextureCoordinateDomain,
    triangle_list: Vec<Vec3>,
}
impl LevelOfDetail {
    fn from_llsd(data: LLSDValue) -> std::io::Result<Self> {
        let array = data
            .as_array()
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Expected top-level array"))?;

        let map = array
            .get(0)
            .and_then(LLSDValue::as_map)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Expected map inside array"))?;

        let position_domain = map
            .get("PositionDomain")
            .and_then(LLSDValue::as_map)
            .ok_or_else(|| {
                Error::new(ErrorKind::InvalidData, "Missing or invalid PositionDomain")
            })?;

        let min = position_domain
            .get("Min")
            .and_then(LLSDValue::as_array)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Missing PositionDomain Min"))?;

        let max = position_domain
            .get("Max")
            .and_then(LLSDValue::as_array)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Missing PositionDomain Max"))?;

        fn extract_f32(value: &LLSDValue) -> Option<f32> {
            match value {
                LLSDValue::Real(n) => Some(*n as f32),
                LLSDValue::Integer(n) => Some(*n as f32),
                _ => None,
            }
        }
        let position_domain_min = Vec3::new(
            extract_f32(&min[0]).ok_or_else(|| Error::new(ErrorKind::InvalidData, "Invalid x"))?,
            extract_f32(&min[1]).ok_or_else(|| Error::new(ErrorKind::InvalidData, "Invalid y"))?,
            extract_f32(&min[2]).ok_or_else(|| Error::new(ErrorKind::InvalidData, "Invalid z"))?,
        );

        let position_domain_max = Vec3::new(
            extract_f32(&max[0]).ok_or_else(|| Error::new(ErrorKind::InvalidData, "Invalid x"))?,
            extract_f32(&max[1]).ok_or_else(|| Error::new(ErrorKind::InvalidData, "Invalid y"))?,
            extract_f32(&max[2]).ok_or_else(|| Error::new(ErrorKind::InvalidData, "Invalid z"))?,
        );

        fn get_binary(
            map: &HashMap<String, LLSDValue>,
            key: &str,
        ) -> Result<Vec<u8>, std::io::Error> {
            match map.get(key) {
                Some(LLSDValue::Binary(data)) => Ok(data.clone()),
                _ => Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Missing or invalid binary data for key: {}", key),
                )),
            }
        }

        fn get_u16_vec(data: &[u8]) -> Result<Vec<u16>, std::io::Error> {
            if data.len() % 2 != 0 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "TriangleList data has odd length",
                ));
            }
            Ok(data
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect())
        }
        fn parse_packed_u16_positions(data: &[u8]) -> Result<Vec<Vec3>, std::io::Error> {
            if data.len() % 6 != 0 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Position data length must be a multiple of 6",
                ));
            }
            let mut positions = Vec::with_capacity(data.len() / 6);
            for chunk in data.chunks_exact(6) {
                let x = u16::from_le_bytes([chunk[0], chunk[1]]) as f32;
                let y = u16::from_le_bytes([chunk[2], chunk[3]]) as f32;
                let z = u16::from_le_bytes([chunk[4], chunk[5]]) as f32;

                positions.push(Vec3::new(x, y, z));
            }
            Ok(positions)
        }
        fn parse_weights(data: &[u8]) -> std::io::Result<Vec<JointWeight>> {
            let mut weights = Vec::new();
            let mut cursor = 0;
            while cursor < data.len() {
                let mut influences = Vec::with_capacity(4);
                for _ in 0..4 {
                    if cursor >= data.len() {
                        return Err(Error::new(
                            ErrorKind::UnexpectedEof,
                            "Unexpected end of weight data",
                        ));
                    }
                    let joint = data[cursor];
                    cursor += 1;
                    if joint == 0xFF {
                        break; // End of influences
                    }
                    if joint > 31 {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            format!("Invalid joint index: {}", joint),
                        ));
                    }
                    if cursor + 1 >= data.len() {
                        return Err(Error::new(
                            ErrorKind::UnexpectedEof,
                            "Unexpected end while reading weight value",
                        ));
                    }
                    let weight = u16::from_le_bytes([data[cursor], data[cursor + 1]]);
                    cursor += 2;
                    influences.push(JointWeight {
                        joint_index: joint,
                        weight,
                    });
                }
                weights.extend(influences);
            }
            Ok(weights)
        }
        fn parse_texcoords(data: &[u8]) -> std::io::Result<Vec<TextureCoordinate>> {
            if data.len() % 4 != 0 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "TexCoord0 data length is not a multiple of 4",
                ));
            }

            let mut coords = Vec::with_capacity(data.len() / 4);
            let mut cursor = 0;

            while cursor + 3 < data.len() {
                let u = u16::from_le_bytes([data[cursor], data[cursor + 1]]);
                let v = u16::from_le_bytes([data[cursor + 2], data[cursor + 3]]);
                cursor += 4;

                coords.push(TextureCoordinate { u, v });
            }

            Ok(coords)
        }
        fn parse_texture_coordinate_domain(
            map: &HashMap<String, LLSDValue>,
        ) -> std::io::Result<TextureCoordinateDomain> {
            // get the "TexCoord0Domain" key as a Map
            let domain_value = map.get("TexCoord0Domain").ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing TexCoord0Domain")
            })?;

            let domain_map = match domain_value {
                LLSDValue::Map(m) => m,
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "TexCoord0Domain is not a Map",
                    ));
                }
            };

            // parse "Min" array
            let min = match domain_map.get("Min") {
                Some(LLSDValue::Array(arr)) if arr.len() == 2 => {
                    let x = match &arr[0] {
                        LLSDValue::Real(f) => *f as f32,
                        _ => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid Min value",
                            ));
                        }
                    };
                    let y = match &arr[1] {
                        LLSDValue::Real(f) => *f as f32,
                        _ => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid Min value",
                            ));
                        }
                    };
                    [x, y]
                }
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Min is missing or invalid",
                    ));
                }
            };

            // parse "Max" array (same as Min)
            let max = match domain_map.get("Max") {
                Some(LLSDValue::Array(arr)) if arr.len() == 2 => {
                    let x = match &arr[0] {
                        LLSDValue::Real(f) => *f as f32,
                        _ => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid Max value",
                            ));
                        }
                    };
                    let y = match &arr[1] {
                        LLSDValue::Real(f) => *f as f32,
                        _ => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid Max value",
                            ));
                        }
                    };
                    [x, y]
                }
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Max is missing or invalid",
                    ));
                }
            };

            Ok(TextureCoordinateDomain { min, max })
        }
        let positions = parse_packed_u16_positions(&get_binary(map, "Position")?)?;
        fn expand_triangles_to_vec3(
            triangle_indices: Vec<u16>,
            positions: Vec<Vec3>,
        ) -> std::io::Result<Vec<Vec3>> {
            let mut triangles = Vec::with_capacity(triangle_indices.len());

            for index in &triangle_indices {
                let idx = *index as usize;
                if idx >= positions.len() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "Index {} out of bounds (positions length {})",
                            idx,
                            positions.len()
                        ),
                    ));
                }
                triangles.push(positions[idx]);
            }

            // Optional: check that triangles length is divisible by 3
            if triangles.len() % 3 != 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Expanded triangle vector length is not divisible by 3",
                ));
            }

            Ok(triangles)
        }
        Ok(LevelOfDetail {
            no_geometry: false,
            position_domain: Some(PositionDomain {
                min: position_domain_min,
                max: position_domain_max,
            }),
            texture_coordinate_domain: parse_texture_coordinate_domain(map)?,
            weights: parse_weights(&get_binary(map, "Weights")?)?,
            texture_coordinate: parse_texcoords(&get_binary(map, "TexCoord0")?)?,
            triangle_list: expand_triangles_to_vec3(
                get_u16_vec(&get_binary(map, "TriangleList")?)?,
                positions,
            )?,
        })
    }
}
#[derive(Debug, Default)]
pub struct PositionDomain {
    min: Vec3,
    max: Vec3,
}

#[derive(Debug, Default)]
pub struct JointWeight {
    joint_index: u8,
    weight: u16,
}

#[derive(Debug, Default)]
pub struct Position {
    position: Vec4,
}

#[derive(Debug, Default)]
pub struct TextureCoordinate {
    u: u16,
    v: u16,
}
#[derive(Debug, Default)]
pub struct TextureCoordinateDomain {
    min: [f32; 2],
    max: [f32; 2],
}
impl Mesh {
    pub fn from_bytes(bytes: &[u8]) -> std::io::Result<Self> {
        let mut mesh = Mesh {
            ..Default::default()
        };
        if let Ok(data) = serde_llsd::de::binary::from_bytes(bytes) {
            // Get the first ocurrence of the zlib magic number, which denotes the beginning of the
            // first data block.
            let sentinel_location = bytes
                .windows(2)
                .position(|w| w == [ZLIB_MAGIC_NUMBER, ZLIB_DECODING_TYPE])
                .expect("Zlib header not found");
            let compressed_data = &bytes[sentinel_location..];

            let map_data = data.into_map().unwrap();
            let (high_lod_offset, high_lod_size) =
                extract_offset_size(map_data.get("high_lod").unwrap())?;
            let (medium_lod_offset, medium_lod_size) =
                extract_offset_size(map_data.get("medium_lod").unwrap())?;
            let (low_lod_offset, low_lod_size) =
                extract_offset_size(map_data.get("low_lod").unwrap())?;
            let (lowest_lod_offset, lowest_lod_size) =
                extract_offset_size(map_data.get("lowest_lod").unwrap())?;
            let (physics_convex_offset, physics_convex_size) =
                extract_offset_size(map_data.get("physics_convex").unwrap())?;
            let (skin_offset, skin_offset_size) =
                extract_offset_size(map_data.get("skin").unwrap())?;

            let high_level_of_detail = decompress_slice(
                &compressed_data[high_lod_offset..high_lod_offset + high_lod_size],
            )?;
            let medium_level_of_detail = Some(decompress_slice(
                &compressed_data[medium_lod_offset..medium_lod_offset + medium_lod_size],
            )?);
            let low_level_of_detail = Some(decompress_slice(
                &compressed_data[low_lod_offset..low_lod_offset + low_lod_size],
            )?);
            let lowest_level_of_detail = Some(decompress_slice(
                &compressed_data[lowest_lod_offset..lowest_lod_offset + lowest_lod_size],
            )?);
            mesh.physics_convex = decompress_slice(
                &compressed_data
                    [physics_convex_offset..physics_convex_offset + physics_convex_size],
            )?;
            mesh.skin =
                decompress_slice(&compressed_data[skin_offset..skin_offset + skin_offset_size])?;

            if let Ok(decoded) = LevelOfDetail::from_llsd(
                serde_llsd::de::binary::from_bytes(&high_level_of_detail).unwrap(),
            ) {
                mesh.high_level_of_detail = decoded;
            }
            if let Some(data) = &medium_level_of_detail {
                if let Ok(decoded) =
                    LevelOfDetail::from_llsd(serde_llsd::de::binary::from_bytes(data).unwrap())
                {
                    mesh.medium_level_of_detail = Some(decoded)
                }
            }
            if let Some(data) = &low_level_of_detail {
                if let Ok(decoded) =
                    LevelOfDetail::from_llsd(serde_llsd::de::binary::from_bytes(data).unwrap())
                {
                    mesh.low_level_of_detail = Some(decoded)
                }
            }
            if let Some(data) = &lowest_level_of_detail {
                if let Ok(decoded) =
                    LevelOfDetail::from_llsd(serde_llsd::de::binary::from_bytes(data).unwrap())
                {
                    mesh.lowest_level_of_detail = Some(decoded)
                }
            }
        };
        Ok(mesh)
    }
}

fn decompress_slice(slice: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(slice);
    let mut decoded = Vec::new();
    decoder.read_to_end(&mut decoded)?;
    Ok(decoded)
}

fn extract_offset_size(map: &LLSDValue) -> std::io::Result<(usize, usize)> {
    if let LLSDValue::Map(inner) = map {
        let offset = match inner.get("offset") {
            Some(LLSDValue::Integer(val)) => *val as usize,
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Missing or invalid 'offset'",
                ));
            }
        };
        let size = match inner.get("size") {
            Some(LLSDValue::Integer(val)) => *val as usize,
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Missing or invalid 'size'",
                ));
            }
        };
        Ok((offset, size))
    } else {
        Err(Error::new(ErrorKind::InvalidData, "Expected a map"))
    }
}
