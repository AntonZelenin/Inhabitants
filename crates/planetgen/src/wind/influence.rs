use glam::Vec3;

use crate::config::WindDeflectionConfig;
use crate::planet::PlanetData;

use super::velocity::{cube_face_point, direction_to_cube_uv};

#[derive(Clone)]
pub struct MountainInfluenceCubeFace {
    pub costs: Vec<Vec<f32>>,
    pub ridge_tangents: Vec<Vec<Vec3>>,
}

#[derive(Clone)]
pub struct MountainInfluenceMap {
    pub faces: [MountainInfluenceCubeFace; 6],
    pub resolution: usize,
}

/// Sample the planet heightmap at an arbitrary 3D direction using bilinear interpolation.
fn sample_heightmap(planet: &PlanetData, dir: Vec3) -> f32 {
    let dir = dir.normalize();
    let (face_idx, u, v) = direction_to_cube_uv(dir);

    let grid = planet.face_grid_size;
    let fx = ((u + 1.0) * 0.5) * (grid - 1) as f32;
    let fy = ((v + 1.0) * 0.5) * (grid - 1) as f32;

    let x0 = fx.floor() as usize;
    let y0 = fy.floor() as usize;
    let x1 = (x0 + 1).min(grid - 1);
    let y1 = (y0 + 1).min(grid - 1);

    let tx = fx - x0 as f32;
    let ty = fy - y0 as f32;

    let face = &planet.faces[face_idx];
    let h00 = face.heightmap[y0][x0];
    let h10 = face.heightmap[y0][x1];
    let h01 = face.heightmap[y1][x0];
    let h11 = face.heightmap[y1][x1];

    let h0 = h00 + (h10 - h00) * tx;
    let h1 = h01 + (h11 - h01) * tx;
    h0 + (h1 - h0) * ty
}

impl MountainInfluenceMap {
    pub fn build(planet: &PlanetData, resolution: usize, config: &WindDeflectionConfig) -> Self {
        let blank_face = MountainInfluenceCubeFace {
            costs: vec![vec![0.0; resolution]; resolution],
            ridge_tangents: vec![vec![Vec3::ZERO; resolution]; resolution],
        };

        let mut faces = [
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
        ];

        let eps = 2.0 / resolution as f32 * 0.5;

        for face_idx in 0..6 {
            for y in 0..resolution {
                let v = (y as f32 / (resolution - 1) as f32) * 2.0 - 1.0;
                for x in 0..resolution {
                    let u = (x as f32 / (resolution - 1) as f32) * 2.0 - 1.0;

                    let dir = cube_face_point(face_idx, u, v).normalize();
                    let height = sample_heightmap(planet, dir);

                    let cost = ((height - config.height_threshold) / config.height_scale)
                        .clamp(0.0, 1.0);
                    faces[face_idx].costs[y][x] = cost;

                    if cost > 0.0 {
                        // Compute height gradient via finite differences in tangent plane
                        let surface_normal = dir;
                        let east = get_tangent_east(surface_normal);
                        let north = surface_normal.cross(east).normalize();

                        let h_px = sample_heightmap(planet, (dir + east * eps).normalize());
                        let h_mx = sample_heightmap(planet, (dir - east * eps).normalize());
                        let h_py = sample_heightmap(planet, (dir + north * eps).normalize());
                        let h_my = sample_heightmap(planet, (dir - north * eps).normalize());

                        let grad_e = (h_px - h_mx) / (2.0 * eps);
                        let grad_n = (h_py - h_my) / (2.0 * eps);
                        let gradient = east * grad_e + north * grad_n;

                        // Ridge tangent = rotate gradient 90° in tangent plane
                        let tangent = surface_normal.cross(gradient);
                        let len = tangent.length();
                        faces[face_idx].ridge_tangents[y][x] = if len > 1e-6 {
                            tangent / len
                        } else {
                            Vec3::ZERO
                        };
                    }
                }
            }
        }

        // Spread/blur pass: propagate cost outward from mountain cells
        for _ in 0..config.spread_radius {
            let snapshot: Vec<Vec<Vec<f32>>> = faces
                .iter()
                .map(|f| f.costs.clone())
                .collect();
            let tangent_snapshot: Vec<Vec<Vec<Vec3>>> = faces
                .iter()
                .map(|f| f.ridge_tangents.clone())
                .collect();

            for face_idx in 0..6 {
                for y in 0..resolution {
                    for x in 0..resolution {
                        if snapshot[face_idx][y][x] > 0.0 {
                            for (dx, dy) in [(1i32, 0i32), (0, 1), (-1, 0), (0, -1)] {
                                let nx = x as i32 + dx;
                                let ny = y as i32 + dy;
                                if nx >= 0
                                    && ny >= 0
                                    && (nx as usize) < resolution
                                    && (ny as usize) < resolution
                                {
                                    let nx = nx as usize;
                                    let ny = ny as usize;
                                    let propagated =
                                        snapshot[face_idx][y][x] * config.spread_decay;
                                    if propagated > faces[face_idx].costs[ny][nx] {
                                        faces[face_idx].costs[ny][nx] = propagated;
                                        faces[face_idx].ridge_tangents[ny][nx] =
                                            tangent_snapshot[face_idx][y][x];
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Self { faces, resolution }
    }

    /// Sample cost and ridge tangent at a 3D direction using bilinear interpolation.
    pub fn sample(&self, position: Vec3) -> (f32, Vec3) {
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

        // Bilinear interpolation for cost
        let c00 = face.costs[y0][x0];
        let c10 = face.costs[y0][x1];
        let c01 = face.costs[y1][x0];
        let c11 = face.costs[y1][x1];
        let c0 = c00 + (c10 - c00) * tx;
        let c1 = c01 + (c11 - c01) * tx;
        let cost = c0 + (c1 - c0) * ty;

        // Bilinear interpolation for ridge tangent
        let t00 = face.ridge_tangents[y0][x0];
        let t10 = face.ridge_tangents[y0][x1];
        let t01 = face.ridge_tangents[y1][x0];
        let t11 = face.ridge_tangents[y1][x1];
        let t0 = t00.lerp(t10, tx);
        let t1 = t01.lerp(t11, tx);
        let tangent = t0.lerp(t1, ty);
        let len = tangent.length();
        let tangent = if len > 1e-6 { tangent / len } else { Vec3::ZERO };

        (cost, tangent)
    }
}

/// Get a tangent-plane east vector for a surface normal.
fn get_tangent_east(normal: Vec3) -> Vec3 {
    let up = Vec3::Y;
    let east_raw = up.cross(normal);
    if east_raw.length_squared() < 1e-12 {
        let fallback = Vec3::X;
        fallback.cross(normal).normalize()
    } else {
        east_raw.normalize()
    }
}
