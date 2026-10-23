use std::vec;

use crate::fields::{BcType, VolScalarField};
use crate::geometry::Geometry;
use crate::mesh::Mesh;

fn apply_laplacian(
    p: &VolScalarField,
    mesh: &Mesh,
    geom: &Geometry,
    input: &[f64],
    is_delta: bool,
) -> Vec<f64> {
    let mut output = vec![0.0; input.len()];
    for i in 0..mesh.nx {
        for j in 0..mesh.ny {
            for k in 0..mesh.nz {
                let idx = mesh.cell_idx(i, j, k);
                if geom.is_solid[idx] { continue; }
                let center = input[idx];
                let neighbor = |ni: usize, nj: usize, nk: usize| {
                    let nidx = mesh.cell_idx(ni, nj, nk);
                    if geom.is_solid[nidx] { center } else { input[nidx] }
                };
                let east = if i + 1 < mesh.nx { neighbor(i+1,j,k) } else {
                    match p.boundary_field.x_max { BcType::FixedValue(v) => if is_delta { 0.0 } else { v }, BcType::ZeroGradient => center }
                };
                let west = if i > 0 { neighbor(i-1,j,k) } else {
                    match p.boundary_field.x_min { BcType::FixedValue(v) => if is_delta { 0.0 } else { v }, BcType::ZeroGradient => center }
                };
                let north = if j + 1 < mesh.ny { neighbor(i,j+1,k) } else {
                    match p.boundary_field.y_max { BcType::FixedValue(v) => if is_delta { 0.0 } else { v }, BcType::ZeroGradient => center }
                };
                let south = if j > 0 { neighbor(i,j-1,k) } else {
                    match p.boundary_field.y_min { BcType::FixedValue(v) => if is_delta { 0.0 } else { v }, BcType::ZeroGradient => center }
                };
                let front = if k + 1 < mesh.nz { neighbor(i,j,k+1) } else {
                    match p.boundary_field.z_max { BcType::FixedValue(v) => if is_delta { 0.0 } else { v }, BcType::ZeroGradient => center }
                };
                let back = if k > 0 { neighbor(i,j,k-1) } else {
                    match p.boundary_field.z_min { BcType::FixedValue(v) => if is_delta { 0.0 } else { v }, BcType::ZeroGradient => center }
                };
                output[idx] = (east - 2.0*center + west)/(mesh.dx*mesh.dx)
                    + (north - 2.0*center + south)/(mesh.dy*mesh.dy)
                    + (front - 2.0*center + back)/(mesh.dz*mesh.dz);
            }
        }
    }
    output
}

pub fn solve_pressure_poisson(
    p: &mut VolScalarField,
    mesh: &Mesh,
    source: &VolScalarField,
    geom: &Geometry,
    max_iter: usize,
) {
    let n = mesh.num_cells();
    let tolerance = 1e-12;
    let ap = apply_laplacian(p, mesh, geom, &p.internal_field, false);
    let mut r = vec![0.0; n];
    for i in 0..n {
        if !geom.is_solid[i] { r[i] = source.internal_field[i] - ap[i]; }
    }
    let mut d = r.clone();
    let mut rr: f64 = r.iter().map(|x| x*x).sum();
    for _ in 0..max_iter {
        if rr < tolerance { break; }
        let q = apply_laplacian(p, mesh, geom, &d, true);
        let mut dq = 0.0;
        for i in 0..n { if !geom.is_solid[i] { dq += d[i]*q[i]; } }
        if dq.abs() < 1e-20 { break; }
        let alpha = rr / dq;
        let mut next_rr = 0.0;
        for i in 0..n {
            if geom.is_solid[i] { p.internal_field[i] = 0.0; r[i] = 0.0; d[i] = 0.0; continue; }
            p.internal_field[i] += alpha*d[i];
            r[i] -= alpha*q[i];
            next_rr += r[i]*r[i];
        }
        let beta = if rr > 0.0 { next_rr / rr } else { 0.0 };
        for i in 0..n {
            if !geom.is_solid[i] { d[i] = r[i] + beta*d[i]; }
        }
        rr = next_rr;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Geometry;

    #[test]
    fn solid_cells_are_excluded_from_pressure_operator_and_solution() {
        let mesh = Mesh::new(3, 3, 3, 3.0, 3.0, 3.0);
        let mut geom = Geometry::new(&mesh);
        let solid = mesh.cell_idx(1, 1, 1);
        geom.is_solid[solid] = true;
        let mut p = VolScalarField::new(&mesh, 0.0);
        let mut source = VolScalarField::new(&mesh, 1.0);
        source.internal_field[solid] = 1000.0;
        solve_pressure_poisson(&mut p, &mesh, &source, &geom, 20);
        assert_eq!(p.internal_field[solid], 0.0);
        assert!(p.internal_field.iter().enumerate().all(|(i, v)| i == solid || v.is_finite()));
    }
}
