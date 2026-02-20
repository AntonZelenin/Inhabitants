/// Shared utilities for cube map operations, including cross-face blurring.

use crate::wind::velocity::{cube_face_point, direction_to_cube_uv};

/// Read a pixel from a cubemap face, even if x/y are outside the face bounds.
///
/// If x/y are within the face, just return the value directly.
/// If x/y are outside (e.g. x = -1), figure out which neighboring face
/// that pixel belongs to and read from there instead.
/// This is needed for blurring: edge pixels need to average with their
/// neighbors, which may be on a different face of the cube.
fn sample_cross_face(
    faces: &[Vec<Vec<f32>>; 6],
    face_idx: usize,
    x: i32,
    y: i32,
    resolution: usize,
) -> f32 {
    let res = resolution as i32;
    if x >= 0 && x < res && y >= 0 && y < res {
        return faces[face_idx][y as usize][x as usize];
    }

    // Out of bounds: convert to UV, then to 3D, then back to the correct face
    let u = (x as f32 / (resolution - 1) as f32) * 2.0 - 1.0;
    let v = (y as f32 / (resolution - 1) as f32) * 2.0 - 1.0;
    let point = cube_face_point(face_idx, u, v);
    let dir = point.normalize();
    let (neighbor_face, nu, nv) = direction_to_cube_uv(dir);

    // Convert UV back to pixel coordinates and clamp
    let nx = (((nu + 1.0) * 0.5) * (resolution - 1) as f32).round() as usize;
    let ny = (((nv + 1.0) * 0.5) * (resolution - 1) as f32).round() as usize;
    let nx = nx.min(resolution - 1);
    let ny = ny.min(resolution - 1);

    faces[neighbor_face][ny][nx]
}

/// Apply a single box blur pass across all 6 cube faces with cross-face sampling.
/// Edge and corner pixels correctly sample from neighboring faces.
pub fn blur_cube_faces(faces: &[Vec<Vec<f32>>; 6], resolution: usize) -> [Vec<Vec<f32>>; 6] {
    let blank = vec![vec![0.0f32; resolution]; resolution];
    let mut out = [
        blank.clone(),
        blank.clone(),
        blank.clone(),
        blank.clone(),
        blank.clone(),
        blank,
    ];

    for face_idx in 0..6 {
        for y in 0..resolution {
            for x in 0..resolution {
                let mut sum = 0.0;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        sum += sample_cross_face(
                            faces,
                            face_idx,
                            x as i32 + dx,
                            y as i32 + dy,
                            resolution,
                        );
                    }
                }
                out[face_idx][y][x] = sum / 9.0;
            }
        }
    }

    out
}
