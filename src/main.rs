use axum::{
    Json, Router,
    response::sse::{Event, Sse},
    routing::post,
};
use futures::stream::Stream;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tower_http::cors::CorsLayer;

use crate::fields::{VolScalarField, VolVectorField};
use crate::mesh::Mesh;
mod fields;
mod fvc;
mod geometry;
mod mesh;
mod piso;
mod benchmark;

#[derive(Deserialize, Debug)]
pub struct Vector3Def {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Deserialize, Debug)]
pub struct InletRequest {
    pub id: usize,
    pub position: Vector3Def,
    pub velocity: Vector3Def,
}

#[derive(Deserialize, Debug)]
pub struct SimulationRequest {
    #[serde(rename = "kinematicViscosity")]
    pub kinematic_viscosity: f64,
    pub density: f64,
    pub inlets: Vec<InletRequest>,
    pub stl_base64: Option<String>,
    #[serde(default)]
    pub geometry: Option<serde_json::Value>,
    #[serde(default)]
    pub boundaries: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct SimulationResponse {
    pub status: String,
    pub message: String,
}

#[tokio::main]
async fn main() {
    println!("Initializing LiqVid Physics API...");

    // Allow the React frontend to make requests to this backend
    let cors = CorsLayer::permissive();

    let app = Router::new()
        .route("/simulate", post(run_simulation))
        .layer(cors);

    println!("Server listening on http://127.0.0.1:3000");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

// This function is triggered whenever the React UI sends a request
async fn run_simulation(
    Json(payload): Json<SimulationRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("\n--- NEW SIMULATION REQUEST ---");
    println!("Density: {}", payload.density);
    println!("Viscosity: {}", payload.kinematic_viscosity);
    println!("Active Inlets: {}", payload.inlets.len());

    for inlet in &payload.inlets {
        println!(
            " - Inlet #{}: Position({:?}) Velocity({:?})",
            inlet.id, inlet.position, inlet.velocity
        );
    }

    let (tx, rx) = mpsc::channel(100);

    tokio::spawn(async move {
        // Send initial message
        let _ = tx
            .send(Ok(
                Event::default().data("Initializing mesh and boundaries...")
            ))
            .await;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let nx = 50;
        let ny = 50;
        let nz = 50;

        let mesh = Mesh::new(nx, ny, nz, 100.0, 100.0, 100.0);
        let mut p = VolScalarField::new(&mesh, 0.0);
        let mut u = VolVectorField::new(&mesh, Vector3::new(0.0, 0.0, 0.0));
        let dt = 0.1;

        let mut geom = geometry::Geometry::new(&mesh);

        if let Some(ref base64_str) = payload.stl_base64 {
            let _ = tx.send(Ok(Event::default().data("Voxelizing STL Geometry... (this may take a few seconds)"))).await;
            use base64::{Engine as _, engine::general_purpose};
            if let Ok(bytes) = general_purpose::STANDARD.decode(base64_str) {
                if let Err(e) = geom.load_stl_from_bytes(&mesh, &bytes) {
                    println!("Error loading STL: {}", e);
                } else {
                    let _ = tx.send(Ok(Event::default().data("STL successfully loaded into CFD Mesh!"))).await;
                }
            }
        }

        if payload.geometry.as_ref().and_then(|g| g.get("type")).and_then(|v| v.as_str()) != Some("stl") {
            apply_geometry_config(&mut geom, &mesh, payload.geometry.as_ref());
        }

        let _ = tx
            .send(Ok(
                Event::default().data("Starting PISO Mass Conservation Loop....")
            ))
            .await;
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        for step in 1..=100 {
            // Apply the boundary values selected in the UI on every timestep.
            apply_boundary_conditions(&mut u, &mut p, &mesh, payload.boundaries.as_ref());

            let nu = payload.kinematic_viscosity;
            let laplacian_u = fvc::laplacian_vector(&u, &mesh, &geom);
            let convection_u = fvc::convect(&u, &mesh, &geom);

            for i in 0..mesh.num_cells() {
                if !geom.is_solid[i] {
                    u.internal_field[i] += (laplacian_u.internal_field[i] * nu - convection_u.internal_field[i]) * dt;
                }
            }

            // --- NO SLIP BOUNDARY CONDITION ---
            // If the fluid is inside a solid wall, it stops moving.
            for i in 0..mesh.num_cells() {
                if geom.is_solid[i] {
                    u.internal_field[i] = Vector3::new(0.0, 0.0, 0.0);
                }
            }

            let div_u = fvc::div(&u, &mesh, &geom);

            let mut source = VolScalarField::new(&mesh, 0.0);
            let mut max_div_u = 0.0_f64;

            for i in 0..mesh.num_cells() {
                if geom.is_solid[i] { continue; }
                let div_val = div_u.internal_field[i];
                source.internal_field[i] = div_val / dt;

                if div_val.abs() > max_div_u {
                    max_div_u = div_val.abs();
                }
            }
            piso::solve_pressure_poisson(&mut p, &mesh, &source, &geom, 100);

            let grad_p = fvc::grad(&p, &mesh, &geom);

            for i in 0..mesh.num_cells() {
                if !geom.is_solid[i] {
                    u.internal_field[i] -= grad_p.internal_field[i] * dt;
                }
            }
            
            // --- NO SLIP BOUNDARY CONDITION (AGAIN) ---
            // The pressure gradient might have accidentally pushed fluid into the wall,
            // so we strictly enforce it again here.
            for i in 0..mesh.num_cells() {
                if geom.is_solid[i] {
                    u.internal_field[i] = Vector3::new(0.0, 0.0, 0.0);
                }
            }

            let msg = format!("Time Step {:03}: Max Divergence = {:.6}", step, max_div_u);
            if tx.send(Ok(Event::default().data(msg))).await.is_err() {
                println!("Client disconnected. Halting simulation early.");
                break;
            }

            // --- STREAM LIVE 3D VOLUMETRIC DATA ---
            // Send a [FRAME] message every 2 timesteps for real-time animation in the UI
            if step % 2 == 0 {
                let mid_k = mesh.nz / 2;
                let mut slice_grid = Vec::with_capacity(mesh.nx * mesh.ny);
                let mut pressure_slice = Vec::with_capacity(mesh.nx * mesh.ny);
                
                for fj in 0..mesh.ny {
                    for fi in 0..mesh.nx {
                        let fidx = mesh.cell_idx(fi, fj, mid_k);
                        if geom.is_solid[fidx] {
                            slice_grid.push(-1.0);
                            pressure_slice.push(-999.0);
                        } else {
                            let vel = u.internal_field[fidx];
                            let mag = (vel.x * vel.x + vel.y * vel.y + vel.z * vel.z).sqrt();
                            slice_grid.push(mag);
                            pressure_slice.push(p.internal_field[fidx]);
                        }
                    }
                }

                let step_stride = 2_usize;
                let cell_w = mesh.dx * (step_stride as f64);
                let cell_h = mesh.dy * (step_stride as f64);
                let cell_d = mesh.dz * (step_stride as f64);

                let mut frame_cells = Vec::new();
                for fi in (0..mesh.nx).step_by(step_stride) {
                    for fj in (0..mesh.ny).step_by(step_stride) {
                        for fk in (0..mesh.nz).step_by(step_stride) {
                            let fidx = mesh.cell_idx(fi, fj, fk);
                            if geom.is_solid[fidx] { continue; }
                            
                            let vel = u.internal_field[fidx];
                            let mag = (vel.x * vel.x + vel.y * vel.y + vel.z * vel.z).sqrt();
                            frame_cells.push(serde_json::json!({
                                "x": (fi as f64 + (step_stride as f64) / 2.0) * mesh.dx - (mesh.nx as f64 * mesh.dx) / 2.0,
                                "y": (fj as f64 + (step_stride as f64) / 2.0) * mesh.dy - (mesh.ny as f64 * mesh.dy) / 2.0,
                                "z": (fk as f64 + (step_stride as f64) / 2.0) * mesh.dz - (mesh.nz as f64 * mesh.dz) / 2.0,
                                "mag": mag, "vx": vel.x, "vy": vel.y, "vz": vel.z, "p": p.internal_field[fidx]
                            }));
                        }
                    }
                }

                if !frame_cells.is_empty() || !slice_grid.is_empty() {
                    let mut vel_max = 0.0_f64;
                    let mut p_min = f64::MAX;
                    let mut p_max = f64::MIN;
                    let mut vel_sum = 0.0_f64;
                    let mut fluid_count = 0_usize;
                    for i in 0..mesh.num_cells() {
                        if !geom.is_solid[i] {
                            let vel = u.internal_field[i];
                            let mag = (vel.x * vel.x + vel.y * vel.y + vel.z * vel.z).sqrt();
                            if mag > vel_max { vel_max = mag; }
                            vel_sum += mag;
                            let pv = p.internal_field[i];
                            if pv < p_min { p_min = pv; }
                            if pv > p_max { p_max = pv; }
                            fluid_count += 1;
                        }
                    }
                    let vel_avg = if fluid_count > 0 { vel_sum / fluid_count as f64 } else { 0.0 };
                    if p_min == f64::MAX { p_min = 0.0; }
                    if p_max == f64::MIN { p_max = 0.0; }

                    if let Ok(frame_json) = serde_json::to_string(&serde_json::json!({
                        "step": step,
                        "total_steps": 100,
                        "cells": frame_cells,
                        "slice": slice_grid,
                        "pressure_slice": pressure_slice,
                        "nx": mesh.nx,
                        "ny": mesh.ny,
                        "nz": mesh.nz,
                        "cell_w": cell_w,
                        "cell_h": cell_h,
                        "cell_d": cell_d,
                        "max_div": max_div_u,
                        "vel_max": vel_max,
                        "vel_avg": vel_avg,
                        "p_min": p_min,
                        "p_max": p_max,
                        "fluid_cells": fluid_count,
                        "nu": payload.kinematic_viscosity
                    })) {
                        let msg = format!("[FRAME]{}", frame_json);
                        let _ = tx.send(Ok(Event::default().data(msg))).await;
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        let _ = tx
            .send(Ok(Event::default().data(
                "Simulation completed successfully! Divergence minimised",
            )))
            .await;

        // --- NEW: SEND THE VISUALIZATION SLICE ---
        let _ = tx.send(Ok(Event::default().data("Generating 3D Heatmap Slice..."))).await;
        
        let mut slice_data = Vec::new();
        let k = nz / 2; // Slice right down the middle of the Z axis

        for i in 0..mesh.nx {
            for j in 0..mesh.ny {
                let idx = mesh.cell_idx(i, j, k);
                let vel = u.internal_field[idx];
                
                // Calculate the magnitude (speed) of the vector
                let mag = (vel.x * vel.x + vel.y * vel.y + vel.z * vel.z).sqrt();
                
                slice_data.push(serde_json::json!({
                    "x": (i as f64 + 0.5) * mesh.dx - 50.0,
                    "y": (j as f64 + 0.5) * mesh.dy - 50.0,
                    "z": 0.0, // The UI draws this flat on the Z=0 plane
                    "mag": mag
                }));
            }
        }

        // Format as JSON and send to React with the [SLICE] prefix
        if let Ok(slice_json) = serde_json::to_string(&slice_data) {
            let msg = format!("[SLICE]{}", slice_json);
            let _ = tx.send(Ok(Event::default().data(msg))).await;
        }

        // The stream ends when the tx goes out of scope and is dropped.
    });

    Sse::new(ReceiverStream::new(rx))
}


fn number(config: &serde_json::Value, key: &str, fallback: f64) -> f64 {
    config.get(key).and_then(|v| v.as_f64()).unwrap_or(fallback)
}

fn apply_geometry_config(
    geom: &mut geometry::Geometry,
    mesh: &Mesh,
    config: Option<&serde_json::Value>,
) {
    let Some(config) = config else { return; };
    let kind = config.get("type").and_then(|v| v.as_str()).unwrap_or("box");
    for i in 0..mesh.nx {
        for j in 0..mesh.ny {
            for k in 0..mesh.nz {
                let x = (i as f64 + 0.5) * mesh.dx;
                let y = (j as f64 + 0.5) * mesh.dy;
                let z = (k as f64 + 0.5) * mesh.dz;
                let solid = match kind {
                    "channel_cylinder" => {
                        let cx = number(config, "center_x", 35.0);
                        let cy = number(config, "center_y", 50.0);
                        let radius = number(config, "radius", 12.0);
                        (x - cx).powi(2) + (y - cy).powi(2) <= radius.powi(2)
                    }
                    "step" => x < number(config, "step_length", 30.0)
                        && y < number(config, "step_height", 40.0),
                    "box_obstacle" => {
                        x >= number(config, "min_x", 35.0)
                            && x <= number(config, "max_x", 65.0)
                            && y >= number(config, "min_y", 35.0)
                            && y <= number(config, "max_y", 65.0)
                            && z >= number(config, "min_z", 10.0)
                            && z <= number(config, "max_z", 50.0)
                    }
                    _ => false,
                };
                let idx = mesh.cell_idx(i, j, k);
                geom.is_solid[idx] = solid;
                geom.is_surface[idx] = false;
            }
        }
    }
    for i in 0..mesh.nx {
        for j in 0..mesh.ny {
            for k in 0..mesh.nz {
                let idx = mesh.cell_idx(i, j, k);
                if geom.is_solid[idx] { continue; }
                let neighbors = [
                    if i > 0 { Some(mesh.cell_idx(i-1,j,k)) } else { None },
                    if i+1 < mesh.nx { Some(mesh.cell_idx(i+1,j,k)) } else { None },
                    if j > 0 { Some(mesh.cell_idx(i,j-1,k)) } else { None },
                    if j+1 < mesh.ny { Some(mesh.cell_idx(i,j+1,k)) } else { None },
                    if k > 0 { Some(mesh.cell_idx(i,j,k-1)) } else { None },
                    if k+1 < mesh.nz { Some(mesh.cell_idx(i,j,k+1)) } else { None },
                ];
                geom.is_surface[idx] = neighbors.iter().flatten().any(|n| geom.is_solid[*n]);
            }
        }
    }
}

fn apply_boundary_conditions(
    u: &mut VolVectorField,
    p: &mut VolScalarField,
    mesh: &Mesh,
    config: Option<&serde_json::Value>,
) {
    use crate::fields::BcType;
    let zero = nalgebra::Vector3::new(0.0, 0.0, 0.0);
    let default = serde_json::Value::Null;
    let config = config.unwrap_or(&default);
    let mut velocity = [zero; 6];
    let mut fixed_velocity = [false; 6];
    let mut fixed_pressure = [None; 6];
    let faces = ["x_min", "x_max", "y_min", "y_max", "z_min", "z_max"];
    for (face_index, face) in faces.iter().enumerate() {
        let bc = config.get(*face).unwrap_or(&default);
        let kind = bc.get("type").and_then(|v| v.as_str()).unwrap_or("no_slip");
        match kind {
            "inlet" | "moving_wall" => {
                let v = bc.get("velocity").unwrap_or(&default);
                velocity[face_index] = nalgebra::Vector3::new(
                    number(v, "x", 1.0), number(v, "y", 0.0), number(v, "z", 0.0)
                );
                fixed_velocity[face_index] = true;
            }
            "no_slip" => {
                velocity[face_index] = zero;
                fixed_velocity[face_index] = true;
            }
            _ => {}
        }
        if kind == "outlet" {
            fixed_pressure[face_index] = Some(number(bc, "pressure", 0.0));
        }
    }

    u.boundary_field.x_min = if fixed_velocity[0] { BcType::FixedValue(velocity[0]) } else { BcType::ZeroGradient };
    u.boundary_field.x_max = if fixed_velocity[1] { BcType::FixedValue(velocity[1]) } else { BcType::ZeroGradient };
    u.boundary_field.y_min = if fixed_velocity[2] { BcType::FixedValue(velocity[2]) } else { BcType::ZeroGradient };
    u.boundary_field.y_max = if fixed_velocity[3] { BcType::FixedValue(velocity[3]) } else { BcType::ZeroGradient };
    u.boundary_field.z_min = if fixed_velocity[4] { BcType::FixedValue(velocity[4]) } else { BcType::ZeroGradient };
    u.boundary_field.z_max = if fixed_velocity[5] { BcType::FixedValue(velocity[5]) } else { BcType::ZeroGradient };

    p.boundary_field.x_min = fixed_pressure[0].map(BcType::FixedValue).unwrap_or(BcType::ZeroGradient);
    p.boundary_field.x_max = fixed_pressure[1].map(BcType::FixedValue).unwrap_or(BcType::ZeroGradient);
    p.boundary_field.y_min = fixed_pressure[2].map(BcType::FixedValue).unwrap_or(BcType::ZeroGradient);
    p.boundary_field.y_max = fixed_pressure[3].map(BcType::FixedValue).unwrap_or(BcType::ZeroGradient);
    p.boundary_field.z_min = fixed_pressure[4].map(BcType::FixedValue).unwrap_or(BcType::ZeroGradient);
    p.boundary_field.z_max = fixed_pressure[5].map(BcType::FixedValue).unwrap_or(BcType::ZeroGradient);

    // Cell-centred boundary representation: enforce prescribed velocities on edge cells.
    for i in 0..mesh.nx {
        for j in 0..mesh.ny {
            for k in 0..mesh.nz {
                let faces_here = [
                    (i == 0, 0usize), (i + 1 == mesh.nx, 1),
                    (j == 0, 2), (j + 1 == mesh.ny, 3),
                    (k == 0, 4), (k + 1 == mesh.nz, 5),
                ];
                for (on_face, f) in faces_here {
                    if on_face && fixed_velocity[f] {
                        let idx = mesh.cell_idx(i, j, k);
                        u.internal_field[idx] = velocity[f];
                    }
                }
            }
        }
    }
}
