use std::vec;

use crate::fields::{BcType, VolScalarField};
use crate::geometry::Geometry;
use crate::mesh::Mesh;

pub fn advance(
    u: &mut crate::fields::VolVectorField,
    p: &mut VolScalarField,
    mesh: &Mesh,
    geom: &Geometry,
    nu: f64,
    density: f64,
    dt: f64,
) -> f64 {
    let diffusion = crate::fvc::laplacian_vector(u, mesh, geom);
    let convection = crate::fvc::convect(u, mesh, geom);
    for i in 0..mesh.num_cells() {
        if geom.is_solid[i] {
            u.internal_field[i].fill(0.0);
        } else {
            u.internal_field[i] +=
                (diffusion.internal_field[i] * nu - convection.internal_field[i]) * dt;
        }
    }
    let mut source = crate::fvc::div(u, mesh, geom);
    for value in &mut source.internal_field {
        *value *= density / dt;
    }
    solve_pressure_poisson(p, mesh, &source, geom, 100);
    let gradient = crate::fvc::grad(p, mesh, geom);
    for i in 0..mesh.num_cells() {
        if !geom.is_solid[i] {
            u.internal_field[i] -= gradient.internal_field[i] * (dt / density);
        }
    }
    crate::fvc::div(u, mesh, geom)
        .internal_field
        .iter()
        .fold(0.0_f64, |m, v| m.max(v.abs()))
}

// Apply exactly the operator used by the velocity projection: D(G(p)).
// The velocity correction has homogeneous boundary data: prescribed velocity
// faces cannot change their flux, while pressure outlets extrapolate velocity.
// With the paired velocity/pressure boundary conditions this is symmetric
// negative semidefinite, including solid faces and the all-Neumann nullspace.
fn apply_laplacian(
    p: &VolScalarField,
    mesh: &Mesh,
    geom: &Geometry,
    input: &[f64],
    is_delta: bool,
) -> Vec<f64> {
    let scalar_bc = |bc: &BcType<f64>| match bc {
        BcType::FixedValue(v) => BcType::FixedValue(if is_delta { 0.0 } else { *v }),
        _ => BcType::ZeroGradient,
    };
    let pressure = VolScalarField {
        internal_field: input.to_vec(),
        boundary_field: crate::fields::BoundaryField {
            x_min: scalar_bc(&p.boundary_field.x_min),
            x_max: scalar_bc(&p.boundary_field.x_max),
            y_min: scalar_bc(&p.boundary_field.y_min),
            y_max: scalar_bc(&p.boundary_field.y_max),
            z_min: scalar_bc(&p.boundary_field.z_min),
            z_max: scalar_bc(&p.boundary_field.z_max),
        },
    };
    let mut correction = crate::fvc::grad(&pressure, mesh, geom);
    let velocity_bc = |bc: &BcType<f64>| match bc {
        BcType::FixedValue(_) => BcType::ZeroGradient,
        _ => BcType::FixedValue(nalgebra::Vector3::zeros()),
    };
    correction.boundary_field = crate::fields::BoundaryField {
        x_min: velocity_bc(&p.boundary_field.x_min),
        x_max: velocity_bc(&p.boundary_field.x_max),
        y_min: velocity_bc(&p.boundary_field.y_min),
        y_max: velocity_bc(&p.boundary_field.y_max),
        z_min: velocity_bc(&p.boundary_field.z_min),
        z_max: velocity_bc(&p.boundary_field.z_max),
    };
    crate::fvc::div(&correction, mesh, geom).internal_field
}

pub fn solve_pressure_poisson(
    p: &mut VolScalarField,
    mesh: &Mesh,
    source: &VolScalarField,
    geom: &Geometry,
    max_iter: usize,
) {
    let n_cells = mesh.nx * mesh.ny * mesh.nz;
    let tolerance = 1e-6;
    let ap0 = apply_laplacian(p, mesh, geom, &p.internal_field, false);

    let mut r = vec![0.0; n_cells];
    for i in 0..n_cells {
        r[i] = if geom.is_solid[i] {
            0.0
        } else {
            source.internal_field[i] - ap0[i]
        };
    }

    let mut d = r.clone();

    let mut r_dot_r: f64 = r.iter().map(|&x| x * x).sum();

    for _ in 0..max_iter {
        if r_dot_r < tolerance * tolerance {
            break;
        }
        let q = apply_laplacian(p, mesh, geom, &d, true);

        let mut d_dot_q = 0.0;
        for i in 0..n_cells {
            d_dot_q += d[i] * q[i];
        }
        if d_dot_q.abs() < 1e-15 {
            break;
        }
        let alpha = r_dot_r / d_dot_q;
        let mut next_r_dot_r = 0.0;
        for i in 0..n_cells {
            p.internal_field[i] += alpha * d[i];
            r[i] -= alpha * q[i];
            next_r_dot_r += r[i] * r[i];
        }
        let beta = next_r_dot_r / r_dot_r;
        for i in 0..n_cells {
            d[i] = r[i] + beta * d[i];
        }
        r_dot_r = next_r_dot_r;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fields::VolVectorField;
    use nalgebra::Vector3;

    #[test]
    fn poisson_recovers_linear_gradient_without_coupling_solids() {
        let mesh = Mesh::new(8, 4, 3, 8.0, 4.0, 3.0);
        let mut geom = Geometry::new(&mesh);
        for k in 0..mesh.nz {
            for i in 0..mesh.nx {
                geom.is_solid[mesh.cell_idx(i, 0, k)] = true;
            }
        }
        let mut p = VolScalarField::new(&mesh, 0.0);
        p.boundary_field.x_min = BcType::FixedValue(0.0);
        p.boundary_field.x_max = BcType::FixedValue(8.0);
        for (i, solid) in geom.is_solid.iter().enumerate() {
            if *solid {
                p.internal_field[i] = 1234.0;
            }
        }
        let source = VolScalarField::new(&mesh, 0.0);
        solve_pressure_poisson(&mut p, &mesh, &source, &geom, 1000);
        // Collocated pressure can have a checkerboard null mode with two
        // pressure outlets. Check its physical gradient rather than its gauge.
        let gradient = crate::fvc::grad(&p, &mesh, &geom);
        for (idx, value) in gradient.internal_field.iter().enumerate() {
            if !geom.is_solid[idx] {
                assert!((*value - Vector3::x()).norm() < 1e-6);
            }
        }
        let residual = apply_laplacian(&p, &mesh, &geom, &p.internal_field, false);
        assert!(residual.iter().map(|v| v * v).sum::<f64>().sqrt() < 1e-6);
        assert_eq!(p.internal_field[mesh.cell_idx(0, 0, 0)], 1234.0);
    }

    #[test]
    fn projection_conserves_flux_on_anisotropic_grids_with_solids_and_outlets() {
        for outlet in [false, true] {
            let mesh = Mesh::new(12, 9, 4, 2.0, 1.0, 0.75);
            let mut geom = Geometry::new(&mesh);
            for k in 0..mesh.nz {
                geom.is_solid[mesh.cell_idx(5, 4, k)] = true;
            }
            let mut u = VolVectorField::new(&mesh, Vector3::zeros());
            u.boundary_field.x_min = BcType::FixedValue(Vector3::zeros());
            u.boundary_field.x_max = BcType::FixedValue(Vector3::zeros());
            u.boundary_field.y_min = BcType::FixedValue(Vector3::zeros());
            u.boundary_field.y_max = BcType::FixedValue(Vector3::x());
            u.boundary_field.z_min = BcType::Slip;
            u.boundary_field.z_max = BcType::Slip;
            let mut p = VolScalarField::new(&mesh, 0.0);
            if outlet {
                u.boundary_field.x_min = BcType::FixedValue(Vector3::x());
                u.boundary_field.x_max = BcType::ZeroGradient;
                p.boundary_field.x_max = BcType::FixedValue(2.0);
            }
            // Nontrivial 3D initial field exercises more than the lid forcing.
            for (idx, velocity) in u.internal_field.iter_mut().enumerate() {
                if !geom.is_solid[idx] {
                    let phase = idx as f64;
                    *velocity = Vector3::new(phase.sin(), (phase * 0.7).cos(), (phase * 0.3).sin());
                }
            }
            for _ in 0..3 {
                let divergence = advance(&mut u, &mut p, &mesh, &geom, 0.01, 1.7, 0.001);
                assert!(
                    divergence < 1e-8,
                    "outlet={outlet}, divergence={divergence}"
                );
                for (idx, velocity) in u.internal_field.iter().enumerate() {
                    if geom.is_solid[idx] {
                        assert_eq!(*velocity, Vector3::zeros());
                    }
                }
            }
        }
    }

    #[test]
    fn cavity_lid_drives_flow_and_density_scales_pressure() {
        let mesh = Mesh::new(8, 8, 2, 1.0, 1.0, 0.25);
        let geom = Geometry::new(&mesh);
        let run = |density| {
            let mut u = VolVectorField::new(&mesh, Vector3::zeros());
            u.boundary_field.x_min = BcType::FixedValue(Vector3::zeros());
            u.boundary_field.x_max = BcType::FixedValue(Vector3::zeros());
            u.boundary_field.y_min = BcType::FixedValue(Vector3::zeros());
            u.boundary_field.y_max = BcType::FixedValue(Vector3::x());
            u.boundary_field.z_min = BcType::Slip;
            u.boundary_field.z_max = BcType::Slip;
            let mut p = VolScalarField::new(&mesh, 0.0);
            for _ in 0..5 {
                let divergence = advance(&mut u, &mut p, &mesh, &geom, 0.01, density, 0.001);
                assert!(divergence < 1e-8, "cavity divergence: {divergence}");
            }
            (u, p)
        };
        let (u1, p1) = run(1.0);
        let (u2, p2) = run(2.0);
        assert!(u1.internal_field[mesh.cell_idx(3, 7, 0)].x > 0.0);
        assert!(p1.internal_field.iter().any(|p| p.abs() > 1e-6));
        for i in 0..mesh.num_cells() {
            assert!((u1.internal_field[i] - u2.internal_field[i]).norm() < 1e-6);
            assert!((2.0 * p1.internal_field[i] - p2.internal_field[i]).abs() < 1e-5);
        }
    }
}
