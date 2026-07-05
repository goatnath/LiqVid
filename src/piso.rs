use crate::fields::{BcType, VolScalarField};
use crate::mesh::Mesh;

pub fn solve_pressure_poisson(
    p: &mut VolScalarField,
    mesh: &Mesh,
    source: &VolScalarField,
    max_iterations: usize,
) {
    let mut p_new = p.internal_field.clone();

    let denominator =
        2.0 * (1.0 / (mesh.dx * mesh.dx) + 1.0 / (mesh.dy * mesh.dy) + 1.0 / (mesh.dz * mesh.dz));

    for _ in 0..max_iterations {
        for i in 0..mesh.nx {
            for j in 0..mesh.ny {
                for k in 0..mesh.nz {
                    let c_idx = mesh.cell_idx(i, j, k);
                    let p_old = &p.internal_field;

                    let p_east = if i < mesh.nx - 1 {
                        p_old[mesh.cell_idx(i + 1, j, k)]
                    } else {
                        match p.boundary_field.x_max {
                            BcType::FixedValue(v) => v,
                            BcType::ZeroGradient => p_old[c_idx],
                        }
                    };

                    let p_west = if i > 0 {
                        p_old[mesh.cell_idx(i - 1, j, k)]
                    } else {
                        match p.boundary_field.x_min {
                            BcType::FixedValue(v) => v,
                            BcType::ZeroGradient => p_old[c_idx],
                        }
                    };

                    let p_north = if j < mesh.ny - 1 {
                        p_old[mesh.cell_idx(i, j + 1, k)]
                    } else {
                        match p.boundary_field.y_max {
                            BcType::FixedValue(v) => v,
                            BcType::ZeroGradient => p_old[c_idx],
                        }
                    };

                    let p_south = if j > 0 {
                        p_old[mesh.cell_idx(i, j - 1, k)]
                    } else {
                        match p.boundary_field.y_min {
                            BcType::FixedValue(v) => v,
                            BcType::ZeroGradient => p_old[c_idx],
                        }
                    };

                    let p_front = if k < mesh.nz - 1 {
                        p_old[mesh.cell_idx(i, j, k + 1)]
                    } else {
                        match p.boundary_field.z_max {
                            BcType::FixedValue(v) => v,
                            BcType::ZeroGradient => p_old[c_idx],
                        }
                    };

                    let p_back = if k > 0 {
                        p_old[mesh.cell_idx(i, j, k - 1)]
                    } else {
                        match p.boundary_field.z_min {
                            BcType::FixedValue(v) => v,
                            BcType::ZeroGradient => p_old[c_idx],
                        }
                    };

                    let sum_x = (p_east + p_west) / (mesh.dx * mesh.dx);
                    let sum_y = (p_north + p_south) / (mesh.dy * mesh.dy);
                    let sum_z = (p_front + p_back) / (mesh.dz * mesh.dz);

                    p_new[c_idx] =
                        (sum_x + sum_y + sum_z - source.internal_field[c_idx]) / denominator;
                }
            }
        }
        p.internal_field.copy_from_slice(&p_new);
    }
}
