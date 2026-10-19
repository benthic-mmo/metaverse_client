use glam::Vec3;
use metaverse_messages::utils::path::{HollowShape, ProfileShape};

use crate::errors::ObjectUpdateError;

/// One layer of vertices along a primitive's path.
pub(super) struct MeshLayer(pub(super) Vec<Vec3>);

pub(super) fn hollow_profile_shape(shape: HollowShape, profile: ProfileShape) -> ProfileShape {
    match shape {
        HollowShape::Same => profile,
        HollowShape::Circle => ProfileShape::Circle,
        HollowShape::Square => ProfileShape::Square,
        HollowShape::Triangle => ProfileShape::EqualTriangle,
        HollowShape::Unknown => ProfileShape::Square,
    }
}

pub(super) fn build_profile(
    hollow_shape: HollowShape,
    hollow_size: f32,
    profile: ProfileShape,
) -> Result<MeshLayer, ObjectUpdateError> {
    let mut vertices = build_profile_vertices(profile)?;

    if hollow_size <= 0.0 {
        return Ok(MeshLayer(vertices));
    }

    let inner_profile = hollow_profile_shape(hollow_shape, profile);
    let mut inner = build_profile_vertices(inner_profile)?;

    // The inner perimeter runs in the opposite direction to the outer one.
    inner.reverse();

    let hollow_size = hollow_size.clamp(0.0, 1.0);
    vertices.extend(inner.into_iter().map(|vertex| vertex * hollow_size));

    Ok(MeshLayer(vertices))
}

pub(super) fn build_profile_vertices(
    profile: ProfileShape,
) -> Result<Vec<Vec3>, ObjectUpdateError> {
    let sides = profile.default_sides_high();
    let mut vertices = Vec::with_capacity(sides);

    match profile {
        ProfileShape::Circle => {
            for i in 0..sides {
                let angle = std::f32::consts::TAU * i as f32 / sides as f32;
                vertices.push(Vec3::new(angle.cos() * 0.5, angle.sin() * 0.5, 0.0));
            }
        }
        ProfileShape::HalfCircle => {
            for i in 0..sides {
                let angle = std::f32::consts::PI * i as f32 / (sides - 1) as f32;

                vertices.push(Vec3::new(angle.cos() * 0.5, angle.sin() * 0.5, 0.0));
            }
        }
        ProfileShape::IsoTriangle => {
            for (x, y) in [(0.0, 0.5), (-0.4330127, -0.25), (0.4330127, -0.25)] {
                vertices.push(Vec3::new(x, y, 0.0));
            }
        }
        ProfileShape::RightTriangle => {
            vertices.extend_from_slice(&[
                Vec3::new(-0.5, -0.5, 0.0),
                Vec3::new(0.5, -0.5, 0.0),
                Vec3::new(-0.5, 0.5, 0.0),
            ]);
        }
        ProfileShape::Square => {
            vertices.extend_from_slice(&[
                Vec3::new(-0.5, -0.5, 0.0),
                Vec3::new(0.5, -0.5, 0.0),
                Vec3::new(0.5, 0.5, 0.0),
                Vec3::new(-0.5, 0.5, 0.0),
            ]);
        }
        ProfileShape::EqualTriangle => {
            for (x, y) in [(1.0, 0.0), (-0.5, 0.8660254), (-0.5, -0.8660254)] {
                vertices.push(Vec3::new(x * 0.53, y * 0.53, 0.0));
            }
        }
        ProfileShape::Unknown => {
            return Err(ObjectUpdateError::Unimplemented {
                feature: format!("Profile shape {profile:?}"),
            });
        }
    }

    Ok(vertices)
}

pub(super) fn profile_vertex_count(profile: ProfileShape) -> usize {
    match profile {
        ProfileShape::Square => 4,
        ProfileShape::IsoTriangle | ProfileShape::RightTriangle | ProfileShape::EqualTriangle => 3,
        ProfileShape::Circle | ProfileShape::HalfCircle => profile.default_sides_high(),
        ProfileShape::Unknown => 0,
    }
}

/// Returns the start vertex and segment count for each logical face.
pub(super) fn profile_face_ranges(profile: ProfileShape, ring_size: usize) -> Vec<(usize, usize)> {
    if ring_size < 2 {
        return Vec::new();
    }

    match profile {
        ProfileShape::Circle => vec![(0, ring_size)],
        ProfileShape::HalfCircle
        | ProfileShape::Square
        | ProfileShape::IsoTriangle
        | ProfileShape::RightTriangle
        | ProfileShape::EqualTriangle => (0..ring_size).map(|i| (i, 1)).collect(),
        ProfileShape::Unknown => Vec::new(),
    }
}
