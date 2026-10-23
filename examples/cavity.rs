//! Re=100 cavity benchmark using the same numerical core as the web API.
use liqvid::{
    benchmark::GHIA_RE100_U,
    fields::{VolScalarField, VolVectorField},
    geometry::Geometry,
    mesh::Mesh,
    piso,
};
use nalgebra::Vector3;
use serde_json::json;
use std::{error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let mut n = 32_usize;
    let mut dt = 0.002_f64;
    let mut max_steps = 30000_usize;
    let mut output = PathBuf::from("docs/validation/cavity-re100");
    let mut check = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--n" => n = args.next().ok_or("missing --n value")?.parse()?,
            "--dt" => dt = args.next().ok_or("missing --dt value")?.parse()?,
            "--max-steps" => max_steps = args.next().ok_or("missing --max-steps value")?.parse()?,
            "--output" => output = args.next().ok_or("missing --output value")?.into(),
            "--check" => check = true,
            "--help" => {
                println!(
                    "cargo run --release --example cavity -- [--n 32] [--dt 0.002] [--max-steps 30000] [--output DIR] [--check]"
                );
                return Ok(());
            }
            _ => return Err(format!("unknown option: {arg}").into()),
        }
    }
    if !(8..=256).contains(&n) || n % 2 != 0 || !dt.is_finite() || dt <= 0.0 || max_steps == 0 {
        return Err(
            "require even 8 <= n <= 256, positive finite dt, and positive max-steps".into(),
        );
    }
    // Conservative explicit diffusion/advection limits for U=L=1 and nu=0.01.
    if dt > (0.2 / n as f64).min(0.2 / (0.01 * (n * n) as f64)) {
        return Err(
            "dt exceeds the benchmark's conservative explicit stability limit; reduce --dt".into(),
        );
    }
    let mesh = Mesh::new(n, n, 1, 1.0, 1.0, 1.0 / n as f64);
    let geom = Geometry::new(&mesh);
    let mut u = VolVectorField::new(&mesh, Vector3::zeros());
    let mut p = VolScalarField::new(&mesh, 0.0);
    // Use the cavity preset's actual six-face mapping, without the UI's fixed scale.
    let boundaries: liqvid::config::Boundaries = serde_json::from_value(json!({
        "x_min":{"type":"no_slip"}, "x_max":{"type":"no_slip"},
        "y_min":{"type":"no_slip"}, "y_max":{"type":"moving_wall","velocity":{"x":1.0,"y":0.0,"z":0.0}},
        "z_min":{"type":"slip"}, "z_max":{"type":"slip"}
    }))?;
    boundaries.apply(&mut u, &mut p);
    let steady_tolerance = 1e-6;
    let reference_tolerance = 0.05;
    let divergence_tolerance = 0.01;
    let mut history = Vec::new();
    let mut stable_steps = 0;
    let mut converged = false;
    let mut last_rate = f64::INFINITY;
    let mut max_div = f64::INFINITY;
    let mut step = 0;
    for iteration in 1..=max_steps {
        let old = u.internal_field.clone();
        max_div = piso::advance(&mut u, &mut p, &mesh, &geom, 0.01, 1.0, dt);
        // All-Neumann pressure has a constant nullspace. Set the mean to zero.
        let mean = p.internal_field.iter().sum::<f64>() / mesh.num_cells() as f64;
        for value in &mut p.internal_field {
            *value -= mean;
        }
        last_rate = old
            .iter()
            .zip(&u.internal_field)
            .map(|(a, b)| (b - a).amax() / dt)
            .fold(0.0, f64::max);
        if !max_div.is_finite()
            || !last_rate.is_finite()
            || !u
                .internal_field
                .iter()
                .all(|v| v.iter().all(|c| c.is_finite()))
            || !p.internal_field.iter().all(|v| v.is_finite())
        {
            return Err(format!("non-finite solution at step {iteration}").into());
        }
        step = iteration;
        if iteration % 100 == 0 {
            history.push(json!({"step":iteration,"time":iteration as f64*dt,"velocity_change_rate":last_rate,"max_divergence":max_div}));
        }
        if iteration % 1000 == 0 {
            eprintln!(
                "step={iteration} t={:.3} change/dt={last_rate:.3e} max_div={max_div:.3e}",
                iteration as f64 * dt
            );
        }
        if last_rate < steady_tolerance {
            stable_steps += 1;
        } else {
            stable_steps = 0;
        }
        if stable_steps >= 100 {
            converged = true;
            break;
        }
    }
    let mut profile = vec![(0.0, 0.0)];
    for j in 0..n {
        let velocity = (u.internal_field[mesh.cell_idx(n / 2 - 1, j, 0)].x
            + u.internal_field[mesh.cell_idx(n / 2, j, 0)].x)
            / 2.0;
        profile.push(((j as f64 + 0.5) / n as f64, velocity));
    }
    profile.push((1.0, 1.0));
    let comparisons = liqvid::benchmark::compare_centerline(&profile).ok_or("invalid profile")?;
    let max_error = liqvid::benchmark::max_centerline_error(&profile).ok_or("invalid profile")?;
    let squared_error = comparisons
        .iter()
        .map(|p| p.absolute_error.powi(2))
        .sum::<f64>();
    let passed = converged && max_error <= reference_tolerance && max_div <= divergence_tolerance;
    let result = json!({
        "schema_version":1, "case":"lid_driven_cavity", "reynolds":100,
        "dimensions":[n,n,1], "domain":[1.0,1.0,1.0/n as f64], "lid_speed":1.0,"density":1.0,"kinematic_viscosity":0.01,
        "dt":dt,"max_steps":max_steps,"steps":step,"time":step as f64*dt,
        "pressure_gauge":"zero spatial mean after each step", "z_condition":"slip; homogeneous extruded 2D case",
        "solver":"piso::advance, unchanged from web API; compatible D(G(p)) pressure CG cap 100, absolute L2 tolerance 1e-6",
        "criteria":{"steady_change_rate":steady_tolerance,"consecutive_steady_steps":100,"max_centerline_error":reference_tolerance,"max_divergence":divergence_tolerance},
        "converged":converged,"velocity_change_rate":last_rate,"max_divergence":max_div,
        "max_centerline_error":max_error,"rms_centerline_error":(squared_error/GHIA_RE100_U.len() as f64).sqrt(),"passed":passed,
        "reference":"Ghia, Ghia & Shin (1982), Table I, Re=100; DOI: 10.1016/0021-9991(82)90058-4",
        "profile":profile.iter().map(|&(y,u)|json!({"y":y,"u":u})).collect::<Vec<_>>(), "comparison":comparisons,"history":history
    });
    fs::create_dir_all(&output)?;
    fs::write(
        output.join("results.json"),
        serde_json::to_string_pretty(&result)? + "\n",
    )?;
    println!(
        "Re=100 {n}x{n}x1: steps={step}, steady={converged}, max_error={max_error:.6}, max_div={max_div:.3e}, passed={passed}"
    );
    println!("Results: {}", output.join("results.json").display());
    if check && !passed {
        std::process::exit(2);
    }
    Ok(())
}
