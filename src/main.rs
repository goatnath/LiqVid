use nalgebra::Vector3;

use crate::fields::{VolScalarField, VolVectorField};
use crate::mesh::Mesh;
mod fields;
mod fvc;
mod mesh;
mod piso;

fn main() {
    println!("Welcome to goatFoam!");

    let mesh = Mesh::new(10, 10, 10, 1.0, 1.0, 1.0);
    println!("Created mesh with {} cells", mesh.num_cells());

    let mut p = VolScalarField::new(&mesh, 0.0);
    let mut u = VolVectorField::new(&mesh, Vector3::new(0.0, 0.0, 0.0));
    let center_idx = mesh.cell_idx(5, 5, 5);
    u.internal_field[center_idx] = Vector3::new(10.0, 0.0, 0.0);

    println!("Before PISO");
    let initial_div = fvc::div(&u, &mesh);
    println!(
        "Divergence at center: {:.3}",
        initial_div.internal_field[center_idx]
    );
    println!(
        "Divergence to the right: {:.3}",
        initial_div.internal_field[mesh.cell_idx(6, 5, 5)]
    );

    let dt = 0.01;
    let rho = 1.0;

    let div_u = fvc::div(&u, &mesh);

    let mut source = VolScalarField::new(&mesh, 0.0);

    for i in 0..mesh.num_cells() {
        source.internal_field[i] = (rho / dt) * div_u.internal_field[i];
    }

    piso::solve_pressure_poisson(&mut p, &mesh, &source, 500);

    let grad_p = fvc::grad(&p, &mesh);

    for i in 0..mesh.num_cells() {
        u.internal_field[i] -= grad_p.internal_field[i] * (dt / rho);
    }

    println!("After PISO");
    let final_div = fvc::div(&u, &mesh);
    println!(
        "Divergence at center: {:.3}",
        final_div.internal_field[center_idx]
    );
    println!(
        "Divergence to the right: {:.3}",
        final_div.internal_field[mesh.cell_idx(6, 5, 5)]
    );

    println!("\ngoatFoam done");
}
