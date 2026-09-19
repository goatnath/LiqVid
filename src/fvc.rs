use crate::fields::{BcType, VolScalarField, VolVectorField};
use crate::geometry::Geometry;
use crate::mesh::Mesh;

use nalgebra::Vector3;

pub fn grad(scalar_field: &VolScalarField, mesh: &Mesh, geom: &Geometry) -> VolVectorField {
    let mut grad_field = VolVectorField::new(mesh, Vector3::new(0.0, 0.0, 0.0));

    for i in 0..(mesh.nx) {
        for j in 0..(mesh.ny) {
            for k in 0..(mesh.nz) {
                let center_idx = mesh.cell_idx(i, j, k);
                let p_center = scalar_field.internal_field[center_idx];

                //X direction
                let p_east = if i < mesh.nx - 1 {
                    if geom.is_solid[mesh.cell_idx(i + 1, j, k)] { p_center } else { (scalar_field.internal_field[mesh.cell_idx(i + 1, j, k)] + p_center) / 2.0 }
                } else {
                    match scalar_field.boundary_field.x_max {
                        BcType::FixedValue(v) => v,
                        BcType::ZeroGradient => p_center,
                    }
                };

                let p_west = if i > 0 {
                    if geom.is_solid[mesh.cell_idx(i - 1, j, k)] { p_center } else { (scalar_field.internal_field[mesh.cell_idx(i - 1, j, k)] + p_center) / 2.0 }
                } else {
                    match scalar_field.boundary_field.x_min {
                        BcType::FixedValue(v) => v,
                        BcType::ZeroGradient => p_center,
                    }
                };

                let dp_dx = (p_east - p_west) / mesh.dx;

                //Y direction
                let p_north = if j < mesh.ny - 1 {
                    if geom.is_solid[mesh.cell_idx(i, j + 1, k)] { p_center } else { (scalar_field.internal_field[mesh.cell_idx(i, j + 1, k)] + p_center) / 2.0 }
                } else {
                    match scalar_field.boundary_field.y_max {
                        BcType::FixedValue(v) => v,
                        BcType::ZeroGradient => p_center,
                    }
                };

                let p_south = if j > 0 {
                    if geom.is_solid[mesh.cell_idx(i, j - 1, k)] { p_center } else { (scalar_field.internal_field[mesh.cell_idx(i, j - 1, k)] + p_center) / 2.0 }
                } else {
                    match scalar_field.boundary_field.y_min {
                        BcType::FixedValue(v) => v,
                        BcType::ZeroGradient => p_center,
                    }
                };

                let dp_dy = (p_north - p_south) / mesh.dy;

                //Z direction
                let p_front = if k < mesh.nz - 1 {
                    if geom.is_solid[mesh.cell_idx(i, j, k + 1)] { p_center } else { (scalar_field.internal_field[mesh.cell_idx(i, j, k + 1)] + p_center) / 2.0 }
                } else {
                    match scalar_field.boundary_field.z_max {
                        BcType::FixedValue(v) => v,
                        BcType::ZeroGradient => p_center,
                    }
                };

                let p_back = if k > 0 {
                    if geom.is_solid[mesh.cell_idx(i, j, k - 1)] { p_center } else { (scalar_field.internal_field[mesh.cell_idx(i, j, k - 1)] + p_center) / 2.0 }
                } else {
                    match scalar_field.boundary_field.z_min {
                        BcType::FixedValue(v) => v,
                        BcType::ZeroGradient => p_center,
                    }
                };

                let dp_dz = (p_front - p_back) / mesh.dz;

                grad_field.internal_field[center_idx] = Vector3::new(dp_dx, dp_dy, dp_dz);
            }
        }
    }

    grad_field
}

pub fn div(vector_field: &VolVectorField, mesh: &Mesh, geom: &Geometry) -> VolScalarField {
    let mut div_field = VolScalarField::new(mesh, 0.0);
    for i in 0..mesh.nx {
        for j in 0..mesh.ny {
            for k in 0..mesh.nz {
                let c_idx = mesh.cell_idx(i, j, k);
                let u_center = vector_field.internal_field[c_idx];

                let u_east = if i < mesh.nx - 1 {
                    if geom.is_solid[mesh.cell_idx(i + 1, j, k)] { 0.0 } else { (vector_field.internal_field[mesh.cell_idx(i + 1, j, k)].x + u_center.x) / 2.0 }
                } else {
                    match vector_field.boundary_field.x_max {
                        BcType::FixedValue(v) => v.x,
                        BcType::ZeroGradient => u_center.x,
                    }
                };

                let u_west = if i > 0 {
                    if geom.is_solid[mesh.cell_idx(i - 1, j, k)] { 0.0 } else { (vector_field.internal_field[mesh.cell_idx(i - 1, j, k)].x + u_center.x) / 2.0 }
                } else {
                    match vector_field.boundary_field.x_min {
                        BcType::FixedValue(v) => v.x,
                        BcType::ZeroGradient => u_center.x,
                    }
                };
                let du_dx = (u_east - u_west) / mesh.dx;

                let v_north = if j < mesh.ny - 1 {
                    if geom.is_solid[mesh.cell_idx(i, j + 1, k)] { 0.0 } else { (vector_field.internal_field[mesh.cell_idx(i, j + 1, k)].y + u_center.y) / 2.0 }
                } else {
                    match vector_field.boundary_field.y_max {
                        BcType::FixedValue(v) => v.y,
                        BcType::ZeroGradient => u_center.y,
                    }
                };
                let v_south = if j > 0 {
                    if geom.is_solid[mesh.cell_idx(i, j - 1, k)] { 0.0 } else { (vector_field.internal_field[mesh.cell_idx(i, j - 1, k)].y + u_center.y) / 2.0 }
                } else {
                    match vector_field.boundary_field.y_min {
                        BcType::FixedValue(v) => v.y,
                        BcType::ZeroGradient => u_center.y,
                    }
                };
                let du_dy = (v_north - v_south) / mesh.dy;
                let w_front = if k < mesh.nz - 1 {
                    if geom.is_solid[mesh.cell_idx(i, j, k + 1)] { 0.0 } else { (vector_field.internal_field[mesh.cell_idx(i, j, k + 1)].z + u_center.z) / 2.0 }
                } else {
                    match vector_field.boundary_field.z_max {
                        BcType::FixedValue(v) => v.z,
                        BcType::ZeroGradient => u_center.z,
                    }
                };
                let w_back = if k > 0 {
                    if geom.is_solid[mesh.cell_idx(i, j, k - 1)] { 0.0 } else { (vector_field.internal_field[mesh.cell_idx(i, j, k - 1)].z + u_center.z) / 2.0 }
                } else {
                    match vector_field.boundary_field.z_min {
                        BcType::FixedValue(v) => v.z,
                        BcType::ZeroGradient => u_center.z,
                    }
                };
                let du_dz = (w_front - w_back) / mesh.dz;

                div_field.internal_field[c_idx] = du_dx + du_dy + du_dz;
            }
        }
    }
    div_field
}

pub fn laplacian_vector(vector_field: &VolVectorField, mesh: &Mesh, geom: &Geometry) -> VolVectorField {
    let mut laplacian_field = VolVectorField::new(mesh, Vector3::new(0.0, 0.0, 0.0));

    for i in 0..mesh.nx {
        for j in 0..mesh.ny {
            for k in 0..mesh.nz {
                let c_idx = mesh.cell_idx(i, j, k);
                let u_c = vector_field.internal_field[c_idx];

                // For solid neighbors, mirror the center value (wall boundary)
                let u_e = if i < mesh.nx - 1 {
                    let idx = mesh.cell_idx(i + 1, j, k);
                    if geom.is_solid[idx] { u_c } else { vector_field.internal_field[idx] }
                } else {
                    u_c
                };
                let u_w = if i > 0 {
                    let idx = mesh.cell_idx(i - 1, j, k);
                    if geom.is_solid[idx] { u_c } else { vector_field.internal_field[idx] }
                } else {
                    u_c
                };
                let u_n = if j < mesh.ny - 1 {
                    let idx = mesh.cell_idx(i, j + 1, k);
                    if geom.is_solid[idx] { u_c } else { vector_field.internal_field[idx] }
                } else {
                    u_c
                };
                let u_s = if j > 0 {
                    let idx = mesh.cell_idx(i, j - 1, k);
                    if geom.is_solid[idx] { u_c } else { vector_field.internal_field[idx] }
                } else {
                    u_c
                };
                let u_f = if k < mesh.nz - 1 {
                    let idx = mesh.cell_idx(i, j, k + 1);
                    if geom.is_solid[idx] { u_c } else { vector_field.internal_field[idx] }
                } else {
                    u_c
                };
                let u_b = if k > 0 {
                    let idx = mesh.cell_idx(i, j, k - 1);
                    if geom.is_solid[idx] { u_c } else { vector_field.internal_field[idx] }
                } else {
                    u_c
                };
                let d2u_dx2 = (u_e - 2.0 * u_c + u_w) / (mesh.dx * mesh.dx);
                let d2u_dy2 = (u_n - 2.0 * u_c + u_s) / (mesh.dy * mesh.dy);
                let d2u_dz2 = (u_f - 2.0 * u_c + u_b) / (mesh.dz * mesh.dz);

                laplacian_field.internal_field[c_idx] = d2u_dx2 + d2u_dy2 + d2u_dz2;
            }
        }
    }

    laplacian_field
}

pub fn convect(vector_field: &VolVectorField, mesh: &Mesh, geom: &Geometry) -> VolVectorField {
    let mut convect_field = VolVectorField::new(mesh, Vector3::new(0.0, 0.0, 0.0));

    for i in 0..mesh.nx {
        for j in 0..mesh.ny {
            for k in 0..mesh.nz {
                let c_idx = mesh.cell_idx(i, j, k);
                let u_c = vector_field.internal_field[c_idx];

                // Get neighbors — treat solid neighbors as walls (mirror center value)
                let u_e = if i < mesh.nx - 1 {
                    let idx = mesh.cell_idx(i + 1, j, k);
                    if geom.is_solid[idx] { u_c } else { vector_field.internal_field[idx] }
                } else { u_c };
                let u_w = if i > 0 {
                    let idx = mesh.cell_idx(i - 1, j, k);
                    if geom.is_solid[idx] { u_c } else { vector_field.internal_field[idx] }
                } else { u_c };

                let u_n = if j < mesh.ny - 1 {
                    let idx = mesh.cell_idx(i, j + 1, k);
                    if geom.is_solid[idx] { u_c } else { vector_field.internal_field[idx] }
                } else { u_c };
                let u_s = if j > 0 {
                    let idx = mesh.cell_idx(i, j - 1, k);
                    if geom.is_solid[idx] { u_c } else { vector_field.internal_field[idx] }
                } else { u_c };

                let u_f = if k < mesh.nz - 1 {
                    let idx = mesh.cell_idx(i, j, k + 1);
                    if geom.is_solid[idx] { u_c } else { vector_field.internal_field[idx] }
                } else { u_c };
                let u_b = if k > 0 {
                    let idx = mesh.cell_idx(i, j, k - 1);
                    if geom.is_solid[idx] { u_c } else { vector_field.internal_field[idx] }
                } else { u_c };

                // Calculate spatial derivatives (Central Differencing)
                // Note: du_dx is a Vector3 containing (du/dx, dv/dx, dw/dx)
                let du_dx = if u_c.x >= 0.0 { (u_c - u_w) / mesh.dx } else { (u_e - u_c) / mesh.dx };
                let du_dy = if u_c.y >= 0.0 { (u_c - u_s) / mesh.dy } else { (u_n - u_c) / mesh.dy };
                let du_dz = if u_c.z >= 0.0 { (u_c - u_b) / mesh.dz } else { (u_f - u_c) / mesh.dz };

                // (U • ∇) U = u*(dU/dx) + v*(dU/dy) + w*(dU/dz)
                let convection = du_dx * u_c.x + du_dy * u_c.y + du_dz * u_c.z;

                convect_field.internal_field[c_idx] = convection;
            }
        }
    }
    convect_field
}
