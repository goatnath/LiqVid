use axum::http::StatusCode;
use axum::{
    Json, Router,
    response::sse::{Event, Sse},
    routing::post,
};
use futures::stream::Stream;
use nalgebra::Vector3;
use std::convert::Infallible;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tower_http::cors::CorsLayer;

use liqvid::fields::{VolScalarField, VolVectorField};
use liqvid::{fields, geometry, mesh, piso};
mod visualization;

use liqvid::config::SimulationRequest;

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
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)> {
    payload
        .validate()
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    let mesh = payload.mesh();
    // Parse and voxelize before opening the stream so invalid uploads return HTTP errors.
    let (payload, mesh, geom) = tokio::task::spawn_blocking(move || {
        let geom = payload.build_geometry(&mesh)?;
        Ok::<_, String>((payload, mesh, geom))
    })
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "geometry worker failed".into(),
        )
    })?
    .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let (tx, rx) = mpsc::channel(4);

    tokio::task::spawn_blocking(move || {
        let mut p = VolScalarField::new(&mesh, 0.0);
        let mut u = VolVectorField::new(&mesh, Vector3::zeros());
        payload.boundaries.apply(&mut u, &mut p);
        let dt = 0.1;

        for step in 1..=100 {
            if tx.is_closed() {
                return;
            }
            let max_div_u = piso::advance(
                &mut u,
                &mut p,
                &mesh,
                &geom,
                payload.kinematic_viscosity,
                payload.density,
                dt,
            );
            if !u
                .internal_field
                .iter()
                .all(|v| v.iter().all(|c| c.is_finite()))
                || !p.internal_field.iter().all(|v| v.is_finite())
            {
                let _ = tx.blocking_send(Ok(Event::default()
                    .data("[ERROR]Simulation became non-finite; reduce velocity or viscosity.")));
                return;
            }
            let msg = format!("Time Step {:03}: Max Divergence = {:.6}", step, max_div_u);
            if tx.blocking_send(Ok(Event::default().data(msg))).is_err() {
                println!("Client disconnected. Halting simulation early.");
                return;
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
                            if geom.is_solid[fidx] {
                                continue;
                            }

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
                            if mag > vel_max {
                                vel_max = mag;
                            }
                            vel_sum += mag;
                            let pv = p.internal_field[i];
                            if pv < p_min {
                                p_min = pv;
                            }
                            if pv > p_max {
                                p_max = pv;
                            }
                            fluid_count += 1;
                        }
                    }
                    let vel_avg = if fluid_count > 0 {
                        vel_sum / fluid_count as f64
                    } else {
                        0.0
                    };
                    if p_min == f64::MAX {
                        p_min = 0.0;
                    }
                    if p_max == f64::MIN {
                        p_max = 0.0;
                    }

                    if let Ok(frame_json) = serde_json::to_string(&serde_json::json!({
                        "step": step,
                        "total_steps": 100,
                        "cells": frame_cells,
                        "flow_grid": visualization::flow_grid(&mesh, &geom, &u, step_stride),
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
                        let _ = tx.blocking_send(Ok(Event::default().data(msg)));
                    }
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        let _ = tx.blocking_send(Ok(
            Event::default().data("Simulation completed successfully!")
        ));
    });

    Ok(Sse::new(ReceiverStream::new(rx)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use futures::StreamExt;
    use serde_json::json;

    fn request(geometry: serde_json::Value) -> SimulationRequest {
        serde_json::from_value(json!({
            "kinematicViscosity": 0.01, "density": 1.0, "geometry": geometry,
            "boundaries": {
                "x_min": {"type": "inlet", "velocity": {"x": 1.5, "y": 0.0, "z": 0.0}},
                "x_max": {"type": "outlet", "pressure": 0.0},
                "y_min": {"type": "no_slip"}, "y_max": {"type": "no_slip"},
                "z_min": {"type": "slip"}, "z_max": {"type": "slip"}
            }
        }))
        .unwrap()
    }

    #[tokio::test]
    async fn invalid_upload_returns_error_before_streaming() {
        let response =
            run_simulation(Json(request(json!({"type": "stl", "stl_base64": "bad!"})))).await;
        match response {
            Err((status, message)) => {
                assert_eq!(status, StatusCode::BAD_REQUEST);
                assert!(message.contains("STL"));
            }
            Ok(_) => panic!("invalid upload was accepted"),
        }
    }

    #[tokio::test]
    async fn streamed_frame_reflects_requested_geometry_and_domain() {
        let response = run_simulation(Json(request(
            json!({"type": "step", "step_length": 30.0, "step_height": 40.0}),
        )))
        .await
        .unwrap()
        .into_response();
        let mut stream = response.into_body().into_data_stream();
        let frame = tokio::time::timeout(std::time::Duration::from_secs(30), async {
            while let Some(bytes) = stream.next().await {
                let text = String::from_utf8(bytes.unwrap().to_vec()).unwrap();
                if let Some((_, data)) = text.split_once("[FRAME]") {
                    return serde_json::from_str::<serde_json::Value>(data.trim()).unwrap();
                }
            }
            panic!("stream ended without a frame");
        })
        .await
        .unwrap();
        assert_eq!(frame["nx"], 50);
        assert_eq!(frame["ny"], 50);
        assert_eq!(frame["nz"], 30);
        assert_eq!(frame["flow_grid"]["dimensions"], json!([25, 25, 15]));
        assert_eq!(
            frame["flow_grid"]["velocity"].as_array().unwrap().len(),
            25 * 25 * 15
        );
        assert_eq!(frame["flow_grid"]["velocity"][0], serde_json::Value::Null);
        assert_eq!(frame["flow_grid"]["origin"], json!([-48.0, -48.0, -28.0]));
        assert_eq!(frame["fluid_cells"], 50 * 50 * 30 - 15 * 20 * 30);
        assert_eq!(frame["slice"][0], -1.0);
        assert!(frame["vel_max"].as_f64().unwrap() > 0.0);
        assert!(
            frame["cells"]
                .as_array()
                .unwrap()
                .iter()
                .all(|cell| cell["z"].as_f64().unwrap().abs() <= 30.0)
        );
        // Dropping the response stops the worker on its next send or timestep.
        drop(stream);
    }
}
