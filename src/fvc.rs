use crate::fields::{BcType, VolScalarField, VolVectorField};
use crate::geometry::Geometry;
use crate::mesh::Mesh;
use nalgebra::Vector3;

// Cell-centered ghost values put prescribed values at the boundary face,
// half a cell from the center. Solid faces impose stationary no-slip walls.
fn velocity_neighbors(
    u: &VolVectorField,
    geom: &Geometry,
    neighbors: [Option<usize>; 6],
    c: Vector3<f64>,
) -> [Vector3<f64>; 6] {
    let boundaries = u.boundary_field.faces();
    std::array::from_fn(|face| match neighbors[face] {
        Some(n) if !geom.is_solid[n] => u.internal_field[n],
        Some(_) => -c,
        None => match boundaries[face] {
            BcType::FixedValue(v) => 2.0 * v - c,
            BcType::ZeroGradient => c,
            BcType::Slip => {
                let mut v = c;
                v[face / 2] = -v[face / 2];
                v
            }
        },
    })
}

pub fn grad(p: &VolScalarField, mesh: &Mesh, geom: &Geometry) -> VolVectorField {
    let mut result = VolVectorField::new(mesh, Vector3::zeros());
    let boundaries = p.boundary_field.faces();
    let h = [mesh.dx, mesh.dy, mesh.dz];
    for k in 0..mesh.nz {
        for j in 0..mesh.ny {
            for i in 0..mesh.nx {
                let idx = mesh.cell_idx(i, j, k);
                if geom.is_solid[idx] {
                    continue;
                }
                let c = p.internal_field[idx];
                let neighbors = mesh.neighbors(i, j, k);
                let face: [f64; 6] = std::array::from_fn(|f| match neighbors[f] {
                    Some(n) if !geom.is_solid[n] => 0.5 * (p.internal_field[n] + c),
                    Some(_) => c,
                    None => match boundaries[f] {
                        BcType::FixedValue(v) => *v,
                        _ => c,
                    },
                });
                for axis in 0..3 {
                    result.internal_field[idx][axis] =
                        (face[2 * axis + 1] - face[2 * axis]) / h[axis];
                }
            }
        }
    }
    result
}

pub fn div(u: &VolVectorField, mesh: &Mesh, geom: &Geometry) -> VolScalarField {
    let mut result = VolScalarField::new(mesh, 0.0);
    let h = [mesh.dx, mesh.dy, mesh.dz];
    for k in 0..mesh.nz {
        for j in 0..mesh.ny {
            for i in 0..mesh.nx {
                let idx = mesh.cell_idx(i, j, k);
                if geom.is_solid[idx] {
                    continue;
                }
                let v = velocity_neighbors(u, geom, mesh.neighbors(i, j, k), u.internal_field[idx]);
                for axis in 0..3 {
                    result.internal_field[idx] +=
                        (v[2 * axis + 1][axis] - v[2 * axis][axis]) / (2.0 * h[axis]);
                }
            }
        }
    }
    result
}

pub fn laplacian_vector(u: &VolVectorField, mesh: &Mesh, geom: &Geometry) -> VolVectorField {
    let mut result = VolVectorField::new(mesh, Vector3::zeros());
    let h = [mesh.dx, mesh.dy, mesh.dz];
    for k in 0..mesh.nz {
        for j in 0..mesh.ny {
            for i in 0..mesh.nx {
                let idx = mesh.cell_idx(i, j, k);
                if geom.is_solid[idx] {
                    continue;
                }
                let c = u.internal_field[idx];
                let v = velocity_neighbors(u, geom, mesh.neighbors(i, j, k), c);
                for axis in 0..3 {
                    result.internal_field[idx] +=
                        (v[2 * axis + 1] - 2.0 * c + v[2 * axis]) / h[axis].powi(2);
                }
            }
        }
    }
    result
}

pub fn convect(u: &VolVectorField, mesh: &Mesh, geom: &Geometry) -> VolVectorField {
    let mut result = VolVectorField::new(mesh, Vector3::zeros());
    let h = [mesh.dx, mesh.dy, mesh.dz];
    for k in 0..mesh.nz {
        for j in 0..mesh.ny {
            for i in 0..mesh.nx {
                let idx = mesh.cell_idx(i, j, k);
                if geom.is_solid[idx] {
                    continue;
                }
                let c = u.internal_field[idx];
                let v = velocity_neighbors(u, geom, mesh.neighbors(i, j, k), c);
                for axis in 0..3 {
                    let derivative = if c[axis] >= 0.0 {
                        (c - v[2 * axis]) / h[axis]
                    } else {
                        (v[2 * axis + 1] - c) / h[axis]
                    };
                    result.internal_field[idx] += derivative * c[axis];
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_pressure_has_exact_gradient_including_boundary_cells() {
        let mesh = Mesh::new(6, 4, 3, 6.0, 4.0, 3.0);
        let geom = Geometry::new(&mesh);
        let mut p = VolScalarField::new(&mesh, 0.0);
        p.boundary_field.x_min = BcType::FixedValue(0.0);
        p.boundary_field.x_max = BcType::FixedValue(6.0);
        for k in 0..mesh.nz {
            for j in 0..mesh.ny {
                for i in 0..mesh.nx {
                    p.internal_field[mesh.cell_idx(i, j, k)] = i as f64 + 0.5;
                }
            }
        }
        for v in grad(&p, &mesh, &geom).internal_field {
            assert!((v - Vector3::x()).norm() < 1e-12);
        }
    }

    #[test]
    fn moving_wall_drives_diffusion_and_slip_blocks_normal_flux() {
        let mesh = Mesh::new(4, 4, 4, 4.0, 4.0, 4.0);
        let geom = Geometry::new(&mesh);
        let mut u = VolVectorField::new(&mesh, Vector3::zeros());
        u.boundary_field.y_max = BcType::FixedValue(Vector3::x());
        let diffusion = laplacian_vector(&u, &mesh, &geom);
        assert!((diffusion.internal_field[mesh.cell_idx(1, 3, 1)].x - 2.0).abs() < 1e-12);
        assert_eq!(
            diffusion.internal_field[mesh.cell_idx(1, 1, 1)],
            Vector3::zeros()
        );
        u.internal_field.fill(Vector3::z());
        u.boundary_field.z_min = BcType::Slip;
        u.boundary_field.z_max = BcType::Slip;
        let divergence = div(&u, &mesh, &geom);
        assert!((divergence.internal_field[mesh.cell_idx(1, 1, 0)] - 1.0).abs() < 1e-12);
        assert!((divergence.internal_field[mesh.cell_idx(1, 1, 3)] + 1.0).abs() < 1e-12);
    }

    #[test]
    fn solids_have_no_divergence_and_enforce_no_slip_diffusion() {
        let mesh = Mesh::new(4, 4, 4, 4.0, 4.0, 4.0);
        let mut geom = Geometry::new(&mesh);
        geom.is_solid[mesh.cell_idx(2, 1, 1)] = true;
        let u = VolVectorField::new(&mesh, Vector3::x());
        assert_eq!(
            div(&u, &mesh, &geom).internal_field[mesh.cell_idx(2, 1, 1)],
            0.0
        );
        assert!(
            (laplacian_vector(&u, &mesh, &geom).internal_field[mesh.cell_idx(1, 1, 1)].x + 2.0)
                .abs()
                < 1e-12
        );
    }
}
