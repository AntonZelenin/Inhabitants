// Vertical air movement computed from wind field divergence

use super::velocity::{WindCubeMap, cube_face_point, direction_to_cube_uv};
use glam::Vec3;

/// A single cube face storing pre-computed vertical air movement values
#[derive(Clone)]
pub struct VerticalAirCubeFace {
    /// Grid of divergence values [y][x], negative = rising, positive = sinking
    pub values: Vec<Vec<f32>>,
}

/// Pre-computed vertical air movement cube map for the entire planet.
/// Computed from the surface divergence of the horizontal wind field.
#[derive(Clone)]
pub struct VerticalAirCubeMap {
    pub faces: [VerticalAirCubeFace; 6],
    pub resolution: usize,
}

impl VerticalAirCubeMap {
    /// Build from an existing wind cube map by computing surface divergence.
    ///
    /// Uses finite differences on each cube face to approximate
    /// div(v) = d(vx)/du + d(vy)/dv in cube-face coordinates.
    /// The result is normalized to roughly [-1, 1].
    pub fn build_from_wind(wind: &WindCubeMap) -> Self {
        let resolution = wind.resolution;
        let blank_face = VerticalAirCubeFace {
            values: vec![vec![0.0; resolution]; resolution],
        };

        let mut faces = [
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
        ];

        let mut max_abs: f32 = 0.0;

        for face_idx in 0..6 {
            for y in 0..resolution {
                for x in 0..resolution {
                    let div = compute_divergence(wind, face_idx, x, y);
                    faces[face_idx].values[y][x] = div;
                    max_abs = max_abs.max(div.abs());
                }
            }
        }

        // Normalize to [-1, 1]
        if max_abs > 1e-6 {
            for face in &mut faces {
                for row in &mut face.values {
                    for val in row.iter_mut() {
                        *val /= max_abs;
                    }
                }
            }
        }

        Self { faces, resolution }
    }

    /// Sample vertical air movement at a given position using bilinear interpolation.
    ///
    /// Returns a value in [-1, 1]: negative = rising air, positive = sinking air.
    pub fn sample(&self, position: Vec3) -> f32 {
        let dir = position.normalize();
        let (face_idx, u, v) = direction_to_cube_uv(dir);

        let fx = ((u + 1.0) * 0.5) * (self.resolution - 1) as f32;
        let fy = ((v + 1.0) * 0.5) * (self.resolution - 1) as f32;

        let x0 = fx.floor() as usize;
        let y0 = fy.floor() as usize;
        let x1 = (x0 + 1).min(self.resolution - 1);
        let y1 = (y0 + 1).min(self.resolution - 1);

        let tx = fx - x0 as f32;
        let ty = fy - y0 as f32;

        let face = &self.faces[face_idx];
        let v00 = face.values[y0][x0];
        let v10 = face.values[y0][x1];
        let v01 = face.values[y1][x0];
        let v11 = face.values[y1][x1];

        let v0 = v00 + (v10 - v00) * tx;
        let v1 = v01 + (v11 - v01) * tx;
        v0 + (v1 - v0) * ty
    }
}

/// Compute surface divergence at a grid cell using central finite differences.
///
/// Projects wind vectors onto the local tangent basis (du, dv) of the cube face,
/// then computes d(wind_u)/du + d(wind_v)/dv.
fn compute_divergence(wind: &WindCubeMap, face_idx: usize, x: usize, y: usize) -> f32 {
    let res = wind.resolution;

    // Get the local tangent basis at this cell
    let u = (x as f32 / (res - 1) as f32) * 2.0 - 1.0;
    let v = (y as f32 / (res - 1) as f32) * 2.0 - 1.0;

    // Compute local tangent vectors by finite differencing the cube face mapping
    let du = 2.0 / (res - 1) as f32;

    let dir_u_plus = cube_face_point(face_idx, u + du, v).normalize();
    let dir_u_minus = cube_face_point(face_idx, u - du, v).normalize();
    let dir_v_plus = cube_face_point(face_idx, u, v + du).normalize();
    let dir_v_minus = cube_face_point(face_idx, u, v - du).normalize();

    // Tangent vectors on the sphere surface
    let tangent_u = (dir_u_plus - dir_u_minus).normalize();
    let tangent_v = (dir_v_plus - dir_v_minus).normalize();

    // Use clamped indices for boundary cells
    let x_plus = (x + 1).min(res - 1);
    let x_minus = x.saturating_sub(1);
    let y_plus = (y + 1).min(res - 1);
    let y_minus = y.saturating_sub(1);

    // Wind vectors at neighboring cells
    let wind_xp = wind.faces[face_idx].velocities[y][x_plus];
    let wind_xm = wind.faces[face_idx].velocities[y][x_minus];
    let wind_yp = wind.faces[face_idx].velocities[y_plus][x];
    let wind_ym = wind.faces[face_idx].velocities[y_minus][x];

    // Project onto tangent directions
    let wu_xp = wind_xp.dot(tangent_u);
    let wu_xm = wind_xm.dot(tangent_u);
    let wv_yp = wind_yp.dot(tangent_v);
    let wv_ym = wind_ym.dot(tangent_v);

    // Effective grid spacing (number of cells between samples)
    let dx = (x_plus - x_minus) as f32;
    let dy = (y_plus - y_minus) as f32;

    // Central differences
    let du_dx = if dx > 0.0 { (wu_xp - wu_xm) / dx } else { 0.0 };
    let dv_dy = if dy > 0.0 { (wv_yp - wv_ym) / dy } else { 0.0 };

    // Surface divergence: positive = diverging = sinking, negative = converging = rising
    du_dx + dv_dy
}

/// Convert vertical air movement value to RGB color.
///
/// * Negative (rising air / convergence): blue
/// * Zero (neutral): white
/// * Positive (sinking air / divergence): red
pub fn divergence_to_color(value: f32) -> Vec3 {
    let clamped = value.clamp(-1.0, 1.0);
    if clamped < 0.0 {
        // Rising: white → blue
        let t = -clamped; // 0..1
        Vec3::new(1.0 - t, 1.0 - t, 1.0)
    } else {
        // Sinking: white → red
        let t = clamped; // 0..1
        Vec3::new(1.0, 1.0 - t, 1.0 - t)
    }
}
