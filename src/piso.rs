use std::vec;

use crate::fields::{BcType, VolScalarField};
use crate::mesh::Mesh;

fn apply_laplacian(p: &VolScalarField, mesh: &Mesh, input: &[f64], is_delta: bool) -> Vec<f64> {
    let mut output = vec![0.0; input.len()];

    for i in 0..mesh.nx {
        for j in 0..mesh.ny {
            for k in 0..mesh.nz {
                let c_idx = mesh.cell_idx(i, j, k);
                let center = input[c_idx];

                let v_east = if i < mesh.nx - 1 {
                    input[mesh.cell_idx(i + 1, j, k)]
                } else {
                    match p.boundary_field.x_max {
                        BcType::FixedValue(v) => {
                            if is_delta {
                                0.0
                            } else {
                                v
                            }
                        }
                        BcType::ZeroGradient => center,
                    }
                };

                let v_west = if i > 0 {
                    input[mesh.cell_idx(i - 1, j, k)]
                } else {
                    match p.boundary_field.x_min {
                        BcType::FixedValue(v) => {
                            if is_delta {
                                0.0
                            } else {
                                v
                            }
                        }
                        BcType::ZeroGradient => center,
                    }
                };
                let v_north = if j < mesh.ny - 1 {
                    input[mesh.cell_idx(i, j + 1, k)]
                } else {
                    match p.boundary_field.y_max {
                        BcType::FixedValue(v) => {
                            if is_delta {
                                0.0
                            } else {
                                v
                            }
                        }
                        BcType::ZeroGradient => center,
                    }
                };
                let v_south = if j > 0 {
                    input[mesh.cell_idx(i, j - 1, k)]
                } else {
                    match p.boundary_field.y_min {
                        BcType::FixedValue(v) => {
                            if is_delta {
                                0.0
                            } else {
                                v
                            }
                        }
                        BcType::ZeroGradient => center,
                    }
                };
                let v_front = if k < mesh.nz - 1 {
                    input[mesh.cell_idx(i, j, k + 1)]
                } else {
                    match p.boundary_field.z_max {
                        BcType::FixedValue(v) => {
                            if is_delta {
                                0.0
                            } else {
                                v
                            }
                        }
                        BcType::ZeroGradient => center,
                    }
                };
                let v_back = if k > 0 {
                    input[mesh.cell_idx(i, j, k - 1)]
                } else {
                    match p.boundary_field.z_min {
                        BcType::FixedValue(v) => {
                            if is_delta {
                                0.0
                            } else {
                                v
                            }
                        }
                        BcType::ZeroGradient => center,
                    }
                };

                let d2x = (v_east - 2.0 * center + v_west) / (mesh.dx * mesh.dx);
                let d2y = (v_north - 2.0 * center + v_south) / (mesh.dy * mesh.dy);
                let d2z = (v_front - 2.0 * center + v_back) / (mesh.dz * mesh.dz);

                output[c_idx] = d2x + d2y + d2z;
            }
        }
    }
    output
}

pub fn solve_pressure_poisson(
    p: &mut VolScalarField,
    mesh: &Mesh,
    source: &VolScalarField,
    max_iter: usize,
) {
    let n_cells = mesh.nx * mesh.ny * mesh.nz;
    let tolerance = 1e-6;
    let ap0 = apply_laplacian(p, mesh, &p.internal_field, false);

    let mut r = vec![0.0; n_cells];
    for i in 0..n_cells {
        r[i] = source.internal_field[i] - ap0[i];
    }

    let mut d = r.clone();

    let mut r_dot_r: f64 = r.iter().map(|&x| x * x).sum();

    for _ in 0..max_iter {
        if r_dot_r < tolerance {
            break;
        }
        let q = apply_laplacian(p, mesh, &d, true);

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
