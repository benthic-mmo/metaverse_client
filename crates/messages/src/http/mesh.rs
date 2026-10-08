use crate::{errors::ParseError, parse};
use benthic_protocol::{
    render_data::JointWeight,
    skeleton::{JointName, SkinJoint},
};
use flate2::bufread::ZlibDecoder;
use glam::{Mat4, Vec3};
use serde::{Deserialize, Serialize};
use serde_llsd_benthic::{LLSDValue, de::binary};
use std::{collections::HashMap, io::Read};

/// This is the Zlib magic number. In the binary, this is where the start of the zipped data
/// begins. This is followed by
/// 1   (No compression),
/// 94  (Fast compression),
/// 156 (Default compression),
/// 218 (Best compression ),
const ZLIB_MAGIC_NUMBER: u8 = 120;
const ZLIB_DECODING_TYPE: u8 = 218;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
/// A raw mesh object. Contains the position, levels of detail, physics data, and skin information.
pub struct Mesh {
    /// The position of the mesh in the world
    pub position: Option<Vec3>,
    /// Data for rendering the highest level of detail. This contains the most polygons.
    /// This is the default level of detail, and must be present.
    pub high_level_of_detail: MeshGeometry,
    /// Data for rendering a medium level of detail. This is a lower resolution version of the
    /// model.
    pub medium_level_of_detail: Option<MeshGeometry>,
    /// Data for rendering a low level of detail. This is an even lower resolution version of the
    /// model.
    pub low_level_of_detail: Option<MeshGeometry>,
    /// Data for rendering the lowest level of detail. This gives only a vague impression of the
    /// shape.
    pub lowest_level_of_detail: Option<MeshGeometry>,
    /// This is a physics representation taht uses convex hull approximation for collision and
    /// physics simulation.
    pub physics_convex: Option<Vec<u8>>,
    /// This is the skinning information, which tells the mesh how to deform based on the avatar's
    /// skeleton.
    pub skin: Option<Skin>,
}
impl Mesh {
    /// Converts mesh bytes to a mesh object.
    /// <https://wiki.secondlife.com/wiki/Mesh/Mesh_Asset_Format>
    ///
    /// The data structure starts out with a header in LL binary format.
    /// <https://wiki.secondlife.com/wiki/LLSD#binary_data>
    /// The header is uncompressed and contains the
    /// offset positions for each of the compressed values.
    /// Extracted from the binary format to a HashMap, it looks something like this.
    ///
    /// ```ignore
    /// Map({
    /// skin: Map({ size: Integer(598), offset: Integer(0) }),
    /// physics_convex: Map({ size: Integer(204), offset: Integer(598) }),
    /// lowest_lod: Map({ size: Integer(1305), offset: Integer(802) }),
    /// low_lod: Map({ size: Integer(2246), offset: Integer(2107) }),
    /// medium_lod: Map({ offset: Integer(4353), size: Integer(7672) }),
    /// high_lod: Map({ size: Integer(27225), offset: Integer(12025) }),
    /// });
    ///```
    /// The offset it points to is the exact position in the data of the next zlib magic
    /// number for decompressing each section
    /// Once decompressed, the data is encoded in the same binary llsd format that the header is.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut mesh = Mesh {
            ..Default::default()
        };
        let data = parse!(binary::from_bytes(bytes))?;
        // Get the first ocurrence of the zlib magic number, which denotes the beginning of the
        // first data block.
        let sentinel_location = bytes
            .windows(2)
            .position(|w| w == [ZLIB_MAGIC_NUMBER, ZLIB_DECODING_TYPE])
            .ok_or(ParseError::invalid_field(
                "Zlib magic number not found. Is this a mesh?",
            ))?;
        let compressed_data = &bytes[sentinel_location..];

        let map_data = data.into_map()?;

        let get_offset_size = |key: &str| -> Result<Option<(usize, usize)>, ParseError> {
            map_data.get(key).map(extract_offset_size).transpose()
        };

        // Helper to get optional offset/size
        let (high_lod_offset, high_lod_size) = extract_offset_size(
            map_data
                .get("high_lod")
                .ok_or(ParseError::missing_field("high_lod"))?,
        )?;

        let (medium_lod_offset, medium_lod_size) = get_offset_size("medium_lod")?.unwrap_or((0, 0));
        let (low_lod_offset, low_lod_size) = get_offset_size("low_lod")?.unwrap_or((0, 0));
        let (lowest_lod_offset, lowest_lod_size) = get_offset_size("lowest_lod")?.unwrap_or((0, 0));
        let (physics_convex_offset, physics_convex_size) =
            get_offset_size("physics_convex")?.unwrap_or((0, 0));
        let (skin_offset, skin_size) = get_offset_size("skin")?.unwrap_or((0, 0));

        let high_level_of_detail =
            decompress_slice(get_slice(compressed_data, high_lod_offset, high_lod_size)?)?;

        let medium_level_of_detail = if medium_lod_size > 0 {
            Some(decompress_slice(get_slice(
                compressed_data,
                medium_lod_offset,
                medium_lod_size,
            )?)?)
        } else {
            None
        };

        let low_level_of_detail = if low_lod_size > 0 {
            Some(decompress_slice(get_slice(
                compressed_data,
                low_lod_offset,
                low_lod_size,
            )?)?)
        } else {
            None
        };

        let lowest_level_of_detail = if lowest_lod_size > 0 {
            Some(decompress_slice(get_slice(
                compressed_data,
                lowest_lod_offset,
                lowest_lod_size,
            )?)?)
        } else {
            None
        };

        if physics_convex_size > 0 {
            let physics_convex = decompress_slice(get_slice(
                compressed_data,
                physics_convex_offset,
                physics_convex_size,
            )?)?;

            mesh.physics_convex = Some(physics_convex);
        }

        if skin_size > 0 {
            let skin = decompress_slice(get_slice(compressed_data, skin_offset, skin_size)?)?;

            mesh.skin = Some(parse!(Skin::from_llsd(parse!(binary::from_bytes(&skin))?))?);
        }

        if physics_convex_size > 0 {
            let physics_convex = decompress_slice(
                &compressed_data
                    [physics_convex_offset..physics_convex_offset + physics_convex_size],
            )?;
            mesh.physics_convex = Some(physics_convex);
        }

        if skin_size > 0 {
            let skin = decompress_slice(&compressed_data[skin_offset..skin_offset + skin_size])?;
            mesh.skin = Some(parse!(Skin::from_llsd(parse!(binary::from_bytes(&skin))?))?);
        }

        mesh.high_level_of_detail = parse!(MeshGeometry::from_llsd(
            parse!(binary::from_bytes(&high_level_of_detail))?,
            &mesh.skin
        ))?;
        mesh.medium_level_of_detail = medium_level_of_detail
            .as_ref()
            .map(|bytes| MeshGeometry::from_llsd(parse!(binary::from_bytes(bytes))?, &mesh.skin))
            .transpose()?;
        mesh.low_level_of_detail = low_level_of_detail
            .as_ref()
            .map(|bytes| MeshGeometry::from_llsd(parse!(binary::from_bytes(bytes))?, &mesh.skin))
            .transpose()?;
        mesh.lowest_level_of_detail = lowest_level_of_detail
            .as_ref()
            .map(|bytes| MeshGeometry::from_llsd(parse!(binary::from_bytes(bytes))?, &mesh.skin))
            .transpose()?;
        Ok(mesh)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
/// Contains the geometry information of the mesh.
///
/// This includes all of the information required for creating and displaying the mesh.
pub struct MeshGeometry {
    /// Boolean flag to show that there is no mesh geometry.
    /// Unused and legacy code .
    pub no_geometry: bool,
    /// Used to decode compressed triangle positions
    pub position_domain: Option<PositionDomain>,
    /// Bone weights for skinning
    pub weights: Option<Vec<JointWeight>>,
    /// Stores UVs per vertex, used for texturing.
    /// TODO: this should be optional
    pub texture_coordinate: Vec<TextureCoordinate>,
    /// Used to decode compressed UVs
    pub texture_coordinate_domain: TextureCoordinateDomain,
    /// positions of vertices in 3d space.
    /// This should only be used for small meshes that don't have a lot of vertices. Storing these
    /// triangles duplicates vertices, which is inefficient.
    pub triangles: Option<Vec<Vec3>>,
    /// full list of vertices
    pub vertices: Vec<Vec3>,
    /// full list of indices
    /// This contains information on where in the triangle each of your vertices are. This saves
    /// space by not duplicating vertices and allows the renderer to handle building the triangles.
    pub indices: Vec<u16>,

    /// These normals are used by textures to determine which side is facing out, and which is
    /// facing in.
    pub normals: Option<Vec<Vec3>>,
}

impl MeshGeometry {
    fn from_llsd(data: LLSDValue, skin: &Option<Skin>) -> Result<Self, ParseError> {
        let array = data
            .as_array()
            .ok_or_else(|| ParseError::missing_field("Expected top level array"))?;

        // this length is always one.
        let map = array
            .first()
            .and_then(LLSDValue::as_map)
            .ok_or_else(|| ParseError::missing_field("Expected map inside array"))?;

        // Some faces have no geometry.
        if matches!(map.get("NoGeometry"), Some(LLSDValue::Boolean(true))) {
            return Ok(Self {
                no_geometry: true,
                ..Default::default()
            });
        }

        let position_domain = map
            .get("PositionDomain")
            .and_then(LLSDValue::as_map)
            .ok_or_else(|| ParseError::missing_field("PositionDomain"))?;

        let min = position_domain
            .get("Min")
            .and_then(LLSDValue::as_array)
            .ok_or_else(|| ParseError::missing_field("PositionDomain Min"))?;

        let max = position_domain
            .get("Max")
            .and_then(LLSDValue::as_array)
            .ok_or_else(|| ParseError::missing_field("PositionDomain Max"))?;

        let position_domain_min = Vec3::new(
            parse_f32(&min[0]).ok_or_else(|| ParseError::invalid_field("position domain min x"))?,
            parse_f32(&min[1]).ok_or_else(|| ParseError::invalid_field("position domain min y"))?,
            parse_f32(&min[2]).ok_or_else(|| ParseError::invalid_field("position domain min z"))?,
        );

        let position_domain_max = Vec3::new(
            parse_f32(&max[0]).ok_or_else(|| ParseError::invalid_field("position domain max x"))?,
            parse_f32(&max[1]).ok_or_else(|| ParseError::invalid_field("position domain max y"))?,
            parse_f32(&max[2]).ok_or_else(|| ParseError::invalid_field("position domain max z"))?,
        );

        // Decode positions.
        let position_bytes = parse_binary(map, "Position")?;

        if position_bytes.len() % 6 != 0 {
            return Err(ParseError::mesh_error(
                "Position data length is not a multiple of 6",
            ));
        }

        let mut positions = Vec::with_capacity(position_bytes.len() / 6);

        let (chunks, _) = position_bytes.as_chunks::<6>();
        for chunk in chunks {
            let x = u16::from_le_bytes([chunk[0], chunk[1]]);
            let y = u16::from_le_bytes([chunk[2], chunk[3]]);
            let z = u16::from_le_bytes([chunk[4], chunk[5]]);

            let xf = position_domain_min.x
                + (x as f32 / 65535.0) * (position_domain_max.x - position_domain_min.x);

            let yf = position_domain_min.y
                + (y as f32 / 65535.0) * (position_domain_max.y - position_domain_min.y);

            let zf = position_domain_min.z
                + (z as f32 / 65535.0) * (position_domain_max.z - position_domain_min.z);

            positions.push(Vec3::new(xf, yf, zf));
        }

        // Parse triangle indices.
        let triangle_bytes = parse_binary(map, "TriangleList")?;

        if triangle_bytes.len() % 2 != 0 {
            return Err(ParseError::mesh_error(
                "TriangleList data has odd length (should be even)",
            ));
        }

        let mut triangle_indices = Vec::with_capacity(triangle_bytes.len() / 2);

        let (chunks, _) = triangle_bytes.as_chunks::<2>();
        for chunk in chunks {
            triangle_indices.push(u16::from_le_bytes([chunk[0], chunk[1]]));
        }

        // Viewer drops trailing indices that don't form a complete triangle.
        triangle_indices.truncate(triangle_indices.len() / 3 * 3);

        let weights = map
            .get("Weights")
            .zip(skin.as_ref())
            .map(|(weights_llsd, skin)| handle_skin(weights_llsd, &skin.joints))
            .transpose()?;

        let data = parse_binary(map, "TexCoord0")?;

        if data.len() % 4 != 0 {
            return Err(ParseError::invalid_field(
                "TexCoord0 is not a multiple of 4",
            ));
        }

        let (chunks, _) = data.as_chunks::<4>();
        let texture_coordinate = chunks
            .iter()
            .map(|chunk| {
                let u = u16::from_le_bytes([chunk[0], chunk[1]]);
                let v = u16::from_le_bytes([chunk[2], chunk[3]]);

                TextureCoordinate { u, v }
            })
            .collect::<Vec<_>>();

        let domain_value = map
            .get("TexCoord0Domain")
            .ok_or_else(|| ParseError::missing_field("TexCoord0Domain"))?;

        let domain_map = match domain_value {
            LLSDValue::Map(m) => m,
            _ => {
                return Err(ParseError::invalid_field("TexCoord0Domain is not a Map"));
            }
        };

        let min = match domain_map.get("Min") {
            Some(LLSDValue::Array(arr)) if arr.len() == 2 => {
                let u = parse_f32(&arr[0])
                    .ok_or_else(|| ParseError::invalid_field("Invalid TexCoord0Domain Min U"))?;

                let v = parse_f32(&arr[1])
                    .ok_or_else(|| ParseError::invalid_field("Invalid TexCoord0Domain Min V"))?;

                [u, v]
            }

            _ => {
                return Err(ParseError::invalid_field("Min is missing or invalid"));
            }
        };

        let max = match domain_map.get("Max") {
            Some(LLSDValue::Array(arr)) if arr.len() == 2 => {
                let u = parse_f32(&arr[0])
                    .ok_or_else(|| ParseError::invalid_field("Invalid TexCoord0Domain Max U"))?;

                let v = parse_f32(&arr[1])
                    .ok_or_else(|| ParseError::invalid_field("Invalid TexCoord0Domain Max V"))?;

                [u, v]
            }

            _ => {
                return Err(ParseError::invalid_field("Max is missing or invalid"));
            }
        };

        let texture_coordinate_domain = TextureCoordinateDomain { min, max };

        let normals = map
            .get("Normal")
            .map(|_| {
                let normal_bytes = parse_binary(map, "Normal")?;

                if normal_bytes.len() % 6 != 0 {
                    return Err(ParseError::mesh_error(
                        "Normal data length is not a multiple of 6",
                    ));
                }

                let (chunks, _) = normal_bytes.as_chunks::<6>();

                let normals = chunks
                    .iter()
                    .map(|chunk| {
                        let x = u16::from_le_bytes([chunk[0], chunk[1]]);
                        let y = u16::from_le_bytes([chunk[2], chunk[3]]);
                        let z = u16::from_le_bytes([chunk[4], chunk[5]]);

                        Vec3::new(
                            x as f32 / 65535.0 * 2.0 - 1.0,
                            y as f32 / 65535.0 * 2.0 - 1.0,
                            z as f32 / 65535.0 * 2.0 - 1.0,
                        )
                    })
                    .collect::<Vec<_>>();

                Ok(normals)
            })
            .transpose()?;

        Ok(MeshGeometry {
            no_geometry: false,
            position_domain: Some(PositionDomain {
                min: position_domain_min,
                max: position_domain_max,
            }),
            texture_coordinate_domain,
            weights,
            texture_coordinate,
            triangles: None,
            vertices: positions,
            indices: triangle_indices,
            normals,
        })
    }
}
fn handle_skin(data: &LLSDValue, joints: &[SkinJoint]) -> Result<Vec<JointWeight>, ParseError> {
    let map = match data {
        LLSDValue::Binary(map) => map,
        _ => return Err(ParseError::invalid_field("Weights")),
    };
    let mut weights = Vec::new();
    let mut iter = map.iter().cloned(); // iterator over bytes, cloning to get u8 values
    while let Some(_) = iter.clone().next() {
        let mut joint_indices = [0u8; 4];
        let mut joint_weights = [0u16; 4];

        for i in 0..4 {
            let joint = match iter.next() {
                Some(j) => j,
                None => {
                    return Err(ParseError::invalid_field("Unexpected end of weight data"));
                }
            };
            if joint == 0xFF {
                break; // end of influences for this vertex
            }

            // Next 2 bytes for weight
            let w1 = match iter.next() {
                Some(b) => b,
                None => {
                    return Err(ParseError::invalid_field(
                        "Unexpected end while reading weight value",
                    ));
                }
            };
            let w2 = match iter.next() {
                Some(b) => b,
                None => {
                    return Err(ParseError::invalid_field(
                        "Unexpected end while reading weight value",
                    ));
                }
            };
            let weight = u16::from_le_bytes([w1, w2]);

            joint_indices[i] = joint;
            joint_weights[i] = weight;
        }
        let raw_f32: [f32; 4] = joint_weights.map(|w| w as f32);
        let total: f32 = raw_f32.iter().sum();

        let normalized_weights: [f32; 4] = if total > 0.0 {
            raw_f32.map(|w| w / total)
        } else {
            [0.25, 0.25, 0.25, 0.25]
        };
        weights.push(JointWeight {
            indices: joint_indices,
            weights: normalized_weights,
            joint_name: [
                joints[joint_indices[0] as usize],
                joints[joint_indices[1] as usize],
                joints[joint_indices[2] as usize],
                joints[joint_indices[3] as usize],
            ],
        });
    }
    Ok(weights)
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
/// The position domain of the mesh. Used for decompressing the triangle data.
/// this provides the minimum corner and the maximum corner of the 3d bounding box.
/// this is the range into which the u16 values of the triangles are unpacked.
pub struct PositionDomain {
    /// Minimum corner of the 3d bounding box
    pub min: Vec3,
    /// Maximum corner of the 3d bounding box
    pub max: Vec3,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
/// UV data for texturing the mesh.
/// This corresponds to each vertex in the mesh.
/// This is used to wrap the image around the mesh, based on vertex points and xy (or uv) points on
/// the texture.
pub struct TextureCoordinate {
    /// horizontal coordinate where the vertex draws its color from
    pub u: u16,
    /// vertical coordinate where the vertex draws its color from
    pub v: u16,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
/// The position domain of the texture. Used for decompressing the UV location data.
pub struct TextureCoordinateDomain {
    /// The minimum values found in the mesh UVs
    pub min: [f32; 2],
    /// The maximum values found in the mesh UVs
    pub max: [f32; 2],
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// This contains skin information that is used by avatars. This can alter all or part of the
/// skeleton
pub struct Skin {
    /// The names of the joints that are going to be altered. A full avatar replacement will
    /// replace all of the joints, and a partial skeleton will only replace some.
    pub joints: Vec<SkinJoint>,
    /// The inverse bind matrices used to determine the joint's transform, scale and rotation. This
    /// matrix aligns with the joint names. inverse_bind_matrices[0] corresponds to joint_names[0],
    /// describing the scale, rotation and transform of each joint, and where the joint should be
    /// applied to the mesh.
    pub inverse_bind_matrices: Vec<Mat4>,
    /// The bind shape matrix is used to determine each coordinate's location and offset. Apply
    /// this to each coordinate in the mesh's vertices to apply the scale and roatation of
    /// the sub-object in global space.
    pub bind_shape_matrix: Mat4,
}
impl Skin {
    fn from_llsd(data: LLSDValue) -> Result<Self, ParseError> {
        let map = data
            .as_map()
            .ok_or_else(|| ParseError::missing_field("Expected top level map"))?;

        // Parse joint_names (which includes collision joints)
        let joints: Vec<SkinJoint> = map
            .get("joint_names")
            .and_then(LLSDValue::as_array)
            .ok_or_else(|| ParseError::missing_field("joint_names"))?
            .iter()
            .map(|v| {
                v.as_string()
                    .ok_or_else(|| ParseError::invalid_field("joint name (not a string)"))
                    .and_then(|s| {
                        JointName::resolve_joint_and_collision(s).ok_or_else(|| {
                            ParseError::invalid_field(format!("Unknown joint name: {s}"))
                        })
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Parse inverse_bind_matrix
        let inverse_bind_matrices = map
            .get("inverse_bind_matrix")
            .and_then(LLSDValue::as_array)
            .ok_or_else(|| ParseError::missing_field("inverse_bind_matrix"))?
            .iter()
            .map(|matrix_val| {
                let flat = matrix_val
                    .as_array()
                    .ok_or_else(|| ParseError::invalid_field("Expected matrix array"))?;

                if flat.len() != 16 {
                    return Err(ParseError::invalid_field("Matrix must have 16 elements"));
                }

                let mut floats = [0.0f32; 16];
                for (i, val) in flat.iter().enumerate() {
                    floats[i] = *val
                        .as_real()
                        .ok_or_else(|| ParseError::invalid_field("Matrix element not real"))?
                        as f32;
                }
                Ok(Mat4::from_cols_array(&floats))
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Parse bind_shape_matrix
        let bind_shape_vals = map
            .get("bind_shape_matrix")
            .and_then(LLSDValue::as_array)
            .ok_or_else(|| ParseError::missing_field("bind_shape_matrix"))?;

        if bind_shape_vals.len() != 16 {
            return Err(ParseError::invalid_field(
                "bind_shape_matrix must have 16 elements",
            ));
        }

        let mut bind_shape_array = [0.0f32; 16];
        for (i, val) in bind_shape_vals.iter().enumerate() {
            bind_shape_array[i] = *val
                .as_real()
                .ok_or_else(|| ParseError::invalid_field("Invalid bind shape matrix element"))?
                as f32;
        }

        let bind_shape_matrix = Mat4::from_cols_array(&bind_shape_array);

        Ok(Self {
            joints,
            inverse_bind_matrices,
            bind_shape_matrix,
        })
    }
}

fn decompress_slice(slice: &[u8]) -> Result<Vec<u8>, ParseError> {
    let mut decoder = ZlibDecoder::new(slice);
    let mut decoded = Vec::new();
    parse!(decoder.read_to_end(&mut decoded))?;
    Ok(decoded)
}

fn extract_offset_size(map: &LLSDValue) -> Result<(usize, usize), ParseError> {
    if let LLSDValue::Map(inner) = map {
        let offset = match inner.get("offset") {
            Some(LLSDValue::Integer(val)) => *val as usize,
            _ => return Err(ParseError::missing_field("offset")),
        };
        let size = match inner.get("size") {
            Some(LLSDValue::Integer(val)) => *val as usize,
            _ => return Err(ParseError::missing_field("size")),
        };
        Ok((offset, size))
    } else {
        Err(ParseError::missing_field("Expected a map"))
    }
}

/// helper function for parsing f32s
fn parse_f32(value: &LLSDValue) -> Option<f32> {
    match value {
        LLSDValue::Real(n) => Some(*n as f32),
        LLSDValue::Integer(n) => Some(*n as f32),
        _ => None,
    }
}

/// helper function for parsing binary data
fn parse_binary<'a>(
    map: &'a HashMap<String, LLSDValue>,
    key: &str,
) -> Result<&'a [u8], ParseError> {
    match map.get(key) {
        Some(LLSDValue::Binary(data)) => Ok(data),
        _ => Err(ParseError::invalid_field(format!(
            "Missing or invalid binary data for key: {}",
            key
        ))),
    }
}

fn get_slice<'a>(data: &'a [u8], offset: usize, size: usize) -> Result<&'a [u8], ParseError> {
    let end = offset
        .checked_add(size)
        .ok_or(ParseError::invalid_field("slice range overflow"))?;

    data.get(offset..end)
        .ok_or(ParseError::invalid_field("slice range exceeds data"))
}
