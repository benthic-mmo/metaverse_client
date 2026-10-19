use glam::Vec3;

use super::profile::MeshLayer;

/// Resample a closed polygonal ring evenly by perimeter distance.
pub(super) fn resample_ring(vertices: &[Vec3], count: usize) -> Vec<Vec3> {
    if vertices.is_empty() || count == 0 {
        return Vec::new();
    }

    if vertices.len() == 1 {
        return vec![vertices[0]; count];
    }

    let edge_lengths: Vec<f32> = (0..vertices.len())
        .map(|i| vertices[i].distance(vertices[(i + 1) % vertices.len()]))
        .collect();

    let perimeter: f32 = edge_lengths.iter().sum();

    if perimeter <= f32::EPSILON {
        return vec![vertices[0]; count];
    }

    let mut result = Vec::with_capacity(count);

    for sample in 0..count {
        let target = perimeter * sample as f32 / count as f32;
        let mut distance = 0.0;

        for (i, &edge_length) in edge_lengths.iter().enumerate() {
            let next_distance = distance + edge_length;

            if target <= next_distance || i == edge_lengths.len() - 1 {
                let t = if edge_length > f32::EPSILON {
                    ((target - distance) / edge_length).clamp(0.0, 1.0)
                } else {
                    0.0
                };

                let a = vertices[i];
                let b = vertices[(i + 1) % vertices.len()];
                result.push(a.lerp(b, t));
                break;
            }

            distance = next_distance;
        }
    }

    result
}

/// Create an annular cap between outer and inner profile rings.
pub(super) fn append_hollow_cap(
    layer: &MeshLayer,
    outer_count: usize,
    inner_count: usize,
    flip: bool,
    vertices: &mut Vec<Vec3>,
    indices: &mut Vec<u32>,
) {
    if outer_count < 3 || inner_count < 3 || layer.0.len() < outer_count + inner_count {
        return;
    }

    let outer = &layer.0[..outer_count];

    let inner: Vec<Vec3> = layer.0[outer_count..outer_count + inner_count]
        .iter()
        .rev()
        .copied()
        .collect();

    let count = outer_count.max(inner_count);
    let outer = resample_ring(outer, count);
    let inner = resample_ring(&inner, count);

    if outer.len() != count || inner.len() != count {
        return;
    }

    vertices.extend_from_slice(&outer);
    vertices.extend_from_slice(&inner);

    let inner_start = count as u32;

    for i in 0..count {
        let next = ((i + 1) % count) as u32;
        let outer_a = i as u32;
        let outer_b = next;
        let inner_a = inner_start + i as u32;
        let inner_b = inner_start + next;

        if flip {
            indices.extend_from_slice(&[outer_a, inner_a, outer_b, outer_b, inner_a, inner_b]);
        } else {
            indices.extend_from_slice(&[outer_a, outer_b, inner_a, outer_b, inner_b, inner_a]);
        }
    }
}
