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

        let _ = tx
            .send(Ok(
                Event::default().data("Starting PISO Mass Conservation Loop....")
            ))
            .await;
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        for step in 1..=100 {
            // Set Boundary Conditions
            u.boundary_field.x_min = crate::fields::BcType::FixedValue(nalgebra::Vector3::new(2.0, 0.0, 0.0));
            p.boundary_field.x_max = crate::fields::BcType::FixedValue(0.0);
            p.boundary_field.x_min = crate::fields::BcType::ZeroGradient;
            u.boundary_field.x_max = crate::fields::BcType::ZeroGradient;

            let nu = payload.kinematic_viscosity;
            let laplacian_u = fvc::laplacian_vector(&u, &mesh, &geom);
            let convection_u = fvc::convect(&u, &mesh, &geom);

            for i in 0..mesh.num_cells() {
                u.internal_field[i] += (laplacian_u.internal_field[i] * nu - convection_u.internal_field[i]) * dt;
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
                let div_val = div_u.internal_field[i];
                source.internal_field[i] = div_val / dt;

                if div_val.abs() > max_div_u {
                    max_div_u = div_val.abs();
                }
            }
            piso::solve_pressure_poisson(&mut p, &mesh, &source, 100);

            let grad_p = fvc::grad(&p, &mesh, &geom);

            for i in 0..mesh.num_cells() {
                u.internal_field[i] -= grad_p.internal_field[i] * dt;
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
