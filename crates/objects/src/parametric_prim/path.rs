use glam::{Quat, Vec3};
use metaverse_messages::utils::path::{PathCurve, PrimPath};

use crate::errors::ObjectUpdateError;

use super::profile::MeshLayer;

fn linear_path_scale(scale: f32, t: f32) -> f32 {
    let begin = if scale > 1.0 { 2.0 - scale } else { 1.0 };
    let end = if scale < 1.0 { scale } else { 1.0 };

    begin + (end - begin) * t
}

fn taper_scale(taper: f32, t: f32) -> f32 {
    let mut begin = 1.0;
    let mut end = 1.0 - taper;

    if end > 1.0 {
        begin = 2.0 - end;
        end = 1.0;
    }

    begin + (end - begin) * t
}

pub(super) fn follow_path(
    path_data: PrimPath,
    profile: &MeshLayer,
) -> Result<Vec<MeshLayer>, ObjectUpdateError> {
    let length = path_data.end - path_data.begin;

    if length <= f32::EPSILON {
        return Err(ObjectUpdateError::MeshTooSmall);
    }

    let step_count = path_data.curve.high_resolution_steps().max(1.0).ceil() as usize;

    let mut layers = Vec::with_capacity(step_count + 1);

    match path_data.curve {
        PathCurve::Line | PathCurve::Flexible => {
            for i in 0..=step_count {
                let fraction = i as f32 / step_count as f32;
                let t = path_data.begin + length * fraction;

                let scale_x = linear_path_scale(path_data.scale_x, t);
                let scale_y = linear_path_scale(path_data.scale_y, t);

                let offset = Vec3::new(path_data.shear_x * t, path_data.shear_y * t, t - 0.5);

                let twist = Quat::from_rotation_z(
                    path_data.twist_begin
                        + (path_data.twist_end - path_data.twist_begin) * t * std::f32::consts::PI,
                );

                let vertices = profile
                    .0
                    .iter()
                    .map(|vertex| {
                        twist * Vec3::new(vertex.x * scale_x, vertex.y * scale_y, vertex.z) + offset
                    })
                    .collect();

                layers.push(MeshLayer(vertices));
            }
        }
        PathCurve::Circle | PathCurve::Circle2 | PathCurve::Test => {
            let skew = path_data.skew;
            let skew_mag = skew.abs();
            let hole_x = path_data.scale_x * (1.0 - skew_mag);
            let hole_y = path_data.scale_y;

            let mut radius_start = 0.5 * (1.0 - hole_y);
            let mut radius_end = radius_start;
            let radius_offset = path_data.radius_offset;

            if radius_offset < 0.0 {
                radius_start *= 1.0 + radius_offset;
            } else {
                radius_end *= 1.0 - radius_offset;
            }

            for i in 0..=step_count {
                let fraction = i as f32 / step_count as f32;
                let t = path_data.begin + length * fraction;

                let angle = std::f32::consts::TAU * path_data.revolutions * t;

                let radius = radius_start + (radius_end - radius_start) * t;

                let (s, c) = (angle.sin() * radius, angle.cos() * radius);

                let offset = Vec3::new(
                    path_data.shear_x * s + (-skew + (skew + skew) * t) * 0.5,
                    c + path_data.shear_y * s,
                    s,
                );

                let scale_x = hole_x * taper_scale(path_data.taper_x, t);
                let scale_y = hole_y * taper_scale(path_data.taper_y, t);

                let twist_angle = (path_data.twist_begin
                    + (path_data.twist_end - path_data.twist_begin) * t)
                    * std::f32::consts::TAU
                    - std::f32::consts::PI;

                let rotation = Quat::from_rotation_z(twist_angle) * Quat::from_rotation_x(angle);

                let vertices = profile
                    .0
                    .iter()
                    .map(|vertex| {
                        rotation * Vec3::new(vertex.x * scale_x, vertex.y * scale_y, vertex.z)
                            + offset
                    })
                    .collect();

                layers.push(MeshLayer(vertices));
            }
        }
    }

    Ok(layers)
}
