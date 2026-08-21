use crate::mesh::Mesh;
use nalgebra::{Point3, Vector3};

pub struct Geometry {
    pub is_solid: Vec<bool>,   // true if the cell center is inside the STL
    pub is_surface: Vec<bool>, // true if this fluid cell is adjacent to a solid cell
}

impl Geometry {
    pub fn new(mesh: &Mesh) -> Self {
        Self {
            is_solid: vec![false; mesh.num_cells()],
            is_surface: vec![false; mesh.num_cells()],
        }
    }

    pub fn load_stl_from_bytes(&mut self, mesh: &Mesh, bytes: &[u8]) -> Result<(), std::io::Error> {
        let mut cursor = std::io::Cursor::new(bytes);
        let stl = stl_io::read_stl(&mut cursor)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Ray casting for voxelization (checking if cell center is inside the STL)
        for i in 0..mesh.nx {
            for j in 0..mesh.ny {
                for k in 0..mesh.nz {
                    let cx = (i as f64 + 0.5) * mesh.dx;
                    let cy = (j as f64 + 0.5) * mesh.dy;
                    let cz = (k as f64 + 0.5) * mesh.dz;

                    let ray_origin = Point3::new(cx as f32, cy as f32, cz as f32);
                    let ray_dir = Vector3::new(1.0, 0.0, 0.0); // Cast ray in +x direction

                    let mut intersect_count = 0;

                    for triangle in &stl.faces {
                        let v0 = stl.vertices[triangle.vertices[0]];
                        let v1 = stl.vertices[triangle.vertices[1]];
                        let v2 = stl.vertices[triangle.vertices[2]];

                        let p0 = Point3::new(v0[0], v0[1], v0[2]);
                        let p1 = Point3::new(v1[0], v1[1], v1[2]);
                        let p2 = Point3::new(v2[0], v2[1], v2[2]);

                        if ray_intersects_triangle(ray_origin, ray_dir, p0, p1, p2) {
                            intersect_count += 1;
                        }
                    }

                    // Odd number of intersections means the point is inside the solid
                    if intersect_count % 2 != 0 {
                        let c_idx = mesh.cell_idx(i, j, k);
                        self.is_solid[c_idx] = true;
                    }
                }
            }
        }

        // Build surface adjacency map: a fluid cell is "surface" if it touches a solid cell
        for i in 0..mesh.nx {
            for j in 0..mesh.ny {
                for k in 0..mesh.nz {
                    let c_idx = mesh.cell_idx(i, j, k);
                    if self.is_solid[c_idx] {
                        continue; // Only fluid cells can be surface cells
                    }
                    // Check 6 face-neighbors
                    let neighbors = [
                        if i > 0 { Some(mesh.cell_idx(i - 1, j, k)) } else { None },
                        if i < mesh.nx - 1 { Some(mesh.cell_idx(i + 1, j, k)) } else { None },
                        if j > 0 { Some(mesh.cell_idx(i, j - 1, k)) } else { None },
                        if j < mesh.ny - 1 { Some(mesh.cell_idx(i, j + 1, k)) } else { None },
                        if k > 0 { Some(mesh.cell_idx(i, j, k - 1)) } else { None },
                        if k < mesh.nz - 1 { Some(mesh.cell_idx(i, j, k + 1)) } else { None },
                    ];
                    for neighbor in &neighbors {
                        if let Some(n_idx) = neighbor {
                            if self.is_solid[*n_idx] {
                                self.is_surface[c_idx] = true;
                                break;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

// Möller–Trumbore intersection algorithm
fn ray_intersects_triangle(
    ray_origin: Point3<f32>,
    ray_dir: Vector3<f32>,
    v0: Point3<f32>,
    v1: Point3<f32>,
    v2: Point3<f32>,
) -> bool {
    let epsilon = 1e-6;
    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    let h = ray_dir.cross(&edge2);
    let a = edge1.dot(&h);

    if a > -epsilon && a < epsilon {
        return false; // Ray is parallel to triangle
    }

    let f = 1.0 / a;
    let s = ray_origin - v0;
    let u = f * s.dot(&h);

    if u < 0.0 || u > 1.0 {
        return false;
    }

    let q = s.cross(&edge1);
    let v = f * ray_dir.dot(&q);

    if v < 0.0 || u + v > 1.0 {
        return false;
    }

    let t = f * edge2.dot(&q);
    if t > epsilon {
        return true;
    }

    false
}
