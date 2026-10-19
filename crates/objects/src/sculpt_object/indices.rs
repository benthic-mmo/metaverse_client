use glam::Vec3;

pub(super) fn create_sculpt_indices(rows: &[Vec<Vec3>], invert: bool) -> Vec<u32> {
    if rows.len() < 2 || rows[0].len() < 2 {
        return Vec::new();
    }

    let width = rows[0].len();

    assert!(
        rows.iter().all(|row| row.len() == width),
        "Sculpt rows must all have the same width"
    );

    let mut indices = Vec::with_capacity((rows.len() - 1) * (width - 1) * 6);

    for y in 1..rows.len() {
        for x in 1..width {
            let p4 = (y * width + x) as u32;
            let p3 = p4 - 1;
            let p2 = p4 - width as u32;
            let p1 = p3 - width as u32;

            if invert {
                indices.extend_from_slice(&[p1, p4, p3, p1, p2, p4]);
            } else {
                indices.extend_from_slice(&[p1, p3, p4, p1, p4, p2]);
            }
        }
    }

    indices
}
