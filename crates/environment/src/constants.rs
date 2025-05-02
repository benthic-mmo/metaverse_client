use once_cell::sync::Lazy;

pub static COSINE_TABLE_16: Lazy<[f32; 256]> = Lazy::new(setup_cosines16);
const OO_SQRT2: f32 = 1.0 / std::f32::consts::SQRT_2;

/// The LayerData is encoded in a way that is similar to JPEG compression.
/// Instead of storing data in rows, it stores it in a zig-zag pattern.
/// build_copy_matrix stores where those values are in the zigzag.
///
/// for example:
/// I have unencoded data
/// a   b   c   d   e   f   g   h   i   j   k...
/// 0   1   2   3   4   5   6   7   8   9   10...
///
/// this would be encoded as
/// a   b   f   g   n   o   aa  bb  rr  ss ...
/// 0   1   5   6   14  15  27  28  44  45 ...
///
/// the copy matrix contains the unencoded data's location.
/// encoded[3] would be g
/// copy_matrix[3] would be 6
///
/// by reading through the encoded data and the copy matrix at the same time, the data's original
/// locations can be determined and reconstructed.
/// This matrix is identical for each xy value it is calculated for, and is created once at compile
/// time.
pub const fn build_copy_matrix16() -> [usize; 256] {
    let mut matrix = [0usize; 256];
    let mut diag = false;
    let mut right = true;
    let mut i = 0;
    let mut j = 0;
    let mut count = 0;

    while i < 16 && j < 16 {
        matrix[j * 16 + i] = count;
        count += 1;

        if !diag {
            if right {
                if i < 15 {
                    i += 1;
                } else {
                    j += 1;
                }
                right = false;
                diag = true;
            } else {
                if j < 15 {
                    j += 1;
                } else {
                    i += 1;
                }
                right = true;
                diag = true;
            }
        } else if right {
            if i < 15 {
                i += 1;
            } else {
                j += 1;
            }
            if j > 0 {
                j = j.saturating_sub(1);
            }
            if i == 15 || j == 0 {
                diag = false;
            }
        } else {
            if j < 15 {
                j += 1;
            } else {
                i += 1;
            }
            if i > 0 {
                i = i.saturating_sub(1);
            }
            if j == 15 || i == 0 {
                diag = false;
            }
        }
    }

    matrix
}

/// This is similar to how JPEGs are handled.
/// The LayerData packets are quantized and compressed. Before being sent from the server, they were
/// divided by a certain factorin order to make the bytes small enough to send with the packet.
/// sending floating point f32s would create an enormous packet, so the server divides each point by a
/// factor defined by the quantize table, which is the same on the client and server side.
/// by multiplying the point by its corresponding factor, you can return the compressed data back
/// to its f32 representation.
pub const fn build_dequantize_table16() -> [f32; 256] {
    let mut table = [0.0f32; 256];
    let mut j = 0;
    while j < 16 {
        let mut i = 0;
        while i < 16 {
            table[j * 16 + i] = 1.0 + 2.0 * ((i + j) as f32);
            i += 1;
        }
        j += 1;
    }
    table
}

/// this is used for the Inverse Discrete Cosine Transforms
/// DCT is a form of compression that uses cosines to encode larger data
pub fn setup_cosines16() -> [f32; 256] {
    let hposz: f32 = std::f32::consts::PI * 0.5 / 16.0;
    let mut cosine_table = [0.0f32; 256];
    for u in 0..16 {
        for n in 0..16 {
            cosine_table[u * 16 + n] = ((2.0 * n as f32 + 1.0) * u as f32 * hposz).cos();
        }
    }
    cosine_table
}

/// this runs the inverse discrete cosine transform on the columns of the data
pub fn idct_column16(linein: &[f32], lineout: &mut [f32], column: usize) {
    for n in 0..16 {
        let mut total = OO_SQRT2 * linein[column];
        for u in 1..16 {
            let usize = u * 16;
            total += linein[usize + column] * COSINE_TABLE_16[usize + n];
        }
        lineout[16 * n + column] = total;
    }
}

/// this runs the inverse discrete cosine transform on the rows of the data.
pub fn idct_line16(linein: &[f32], lineout: &mut [f32], line: usize) {
    const OOSOB: f32 = 2.0 / 16.0;
    let line_size = line * 16;
    for n in 0..16 {
        let mut total = OO_SQRT2 * linein[line_size];

        for u in 1..16 {
            total += linein[line_size + u] * COSINE_TABLE_16[u * 16 + n];
        }
        lineout[line_size + n] = total * OOSOB;
    }
}
