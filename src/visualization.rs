use crate::{fields::VolVectorField, geometry::Geometry, mesh::Mesh};
use serde::Serialize;

#[derive(Serialize)]
pub struct FlowGrid {
    dimensions: [usize; 3],
    origin: [f64; 3],
    spacing: [f64; 3],
    // X varies fastest. Null blocks contain solid cells and stop integration.
    velocity: Vec<Option<[f64; 3]>>,
}

pub fn flow_grid(mesh: &Mesh, geom: &Geometry, u: &VolVectorField, stride: usize) -> FlowGrid {
    let dimensions = [
        mesh.nx.div_ceil(stride),
        mesh.ny.div_ceil(stride),
        mesh.nz.div_ceil(stride),
    ];
    let mut velocity = Vec::with_capacity(dimensions.iter().product());
    for k in (0..mesh.nz).step_by(stride) {
        for j in (0..mesh.ny).step_by(stride) {
            for i in (0..mesh.nx).step_by(stride) {
                let mut solid = false;
                let mut sum = nalgebra::Vector3::zeros();
                let mut count = 0;
                for kk in k..(k + stride).min(mesh.nz) {
                    for jj in j..(j + stride).min(mesh.ny) {
                        for ii in i..(i + stride).min(mesh.nx) {
                            let idx = mesh.cell_idx(ii, jj, kk);
                            solid |= geom.is_solid[idx];
                            sum += u.internal_field[idx];
                            count += 1;
                        }
                    }
                }
                velocity.push(if solid {
                    None
                } else {
                    let v = sum / count as f64;
                    Some([v.x, v.y, v.z])
                });
            }
        }
    }
    FlowGrid {
        dimensions,
        origin: [
            stride as f64 * mesh.dx / 2.0 - mesh.lx / 2.0,
            stride as f64 * mesh.dy / 2.0 - mesh.ly / 2.0,
            stride as f64 * mesh.dz / 2.0 - mesh.lz / 2.0,
        ],
        spacing: [
            stride as f64 * mesh.dx,
            stride as f64 * mesh.dy,
            stride as f64 * mesh.dz,
        ],
        velocity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector3;

    #[test]
    fn grid_averages_blocks_in_x_fastest_order_and_masks_any_solid() {
        let mesh = Mesh::new(4, 4, 4, 4.0, 4.0, 4.0);
        let mut geom = Geometry::new(&mesh);
        let mut u = VolVectorField::new(&mesh, Vector3::zeros());
        for k in 0..4 {
            for j in 0..4 {
                for i in 0..4 {
                    u.internal_field[mesh.cell_idx(i, j, k)] =
                        Vector3::new(i as f64, j as f64, k as f64);
                }
            }
        }
        // A solid away from the sampled corner must still block its coarse voxel.
        geom.is_solid[mesh.cell_idx(3, 3, 3)] = true;
        let grid = flow_grid(&mesh, &geom, &u, 2);
        assert_eq!(grid.dimensions, [2, 2, 2]);
        assert_eq!(grid.origin, [-1.0, -1.0, -1.0]);
        assert_eq!(grid.spacing, [2.0, 2.0, 2.0]);
        assert_eq!(grid.velocity[0], Some([0.5, 0.5, 0.5]));
        assert_eq!(grid.velocity[1], Some([2.5, 0.5, 0.5]));
        assert_eq!(grid.velocity[7], None);
    }
}
