# LiqVid roadmap

This checklist describes the current implementation. Completed items are distinct
from planned validation; a green CI run does not establish benchmark accuracy.

## P0 — correctness and visual blockers

- [x] Replace the 5,000-voxel cap with capacity based on the received cell list.
- [x] Lower voxel opacity and make the layer optional; default to streamlines and arrows.
- [x] Connect geometry and six-face boundaries to the solver; exclude solid cells from CG.
- [x] Use consistent physical pressure for constant density and test density scaling.
- [x] Evaluate boundary values per face rather than overwriting corner cell velocities in a face-order loop.
- [x] Run `cargo test` and frontend numerical tests in CI.
- [ ] Add a corner/edge regression that checks simultaneous inlet and wall face fluxes.
- [x] Measure full-domain maximum divergence for the Re=100 cavity.
- [x] Repair the projection with a compatible discrete divergence/gradient composition, including boundary/solid behavior; add conservation regressions.
- [ ] Check compatibility and pressure gauge handling for closed or disconnected fluid regions and incompatible prescribed fluxes.

The density formulation is ∇²p = (ρ/Δt)∇·u* followed by
u = u* − (Δt/ρ)∇p for uniform density. The Laplacian needs no additional density
factor. Switching to kinematic pressure is an alternative representation, not a
required correction; it would also require converting outlet and display units.

## P1 — validation, documentation, and usability

- [x] Update README grid, architecture, boundary workflow, visualization, tests, and OpenFOAM scope.
- [x] Keep FVC stencil loops in readable multiline form.
- [x] Enable arrows by default and add 3D RK4 streamlines.
- [x] Capture an actual UI screenshot and cylinder-flow animation for the README; include capture settings and avoid presenting them as validation evidence.
- [x] Wire `src/benchmark.rs` into the shared library, executable example, and `cargo test`.
- [x] Run a cavity at Re = UL/ν = 100 with explicit dimensions, lid speed, timestep, resolution, convergence criterion, and pressure gauge.
- [x] Sample the normalized vertical centerline velocity, compare with the reference table, and report maximum error and steady-change/divergence diagnostics.
- [x] Add a reproducible comparison plot to the README with measured errors and the divergence criterion.
- [x] Resolve the failed cavity discrete divergence criterion; broader solver validation remains open. See [benchmark results](validation/README.md).
- [ ] Add a Z-position slider and XY/XZ/YZ orientation selection with corresponding backend field data.
- [ ] Implement Ux/Uy/Uz cut-plane data and colors; use a shared field range for the texture and legend, including negative values and a separate solid mask.
- [x] Capture the default streamline/arrow UI running a cylinder case in a browser with WebGL.
- [ ] Add a browser regression for voxel instance-capacity growth and shrinkage.

## P2 — performance and scalability

- [ ] Add and measure Jacobi-preconditioned CG on representative cases.
- [ ] Parallelize suitable cell loops, for example with Rayon; benchmark speedups and check numerical tolerances.
- [ ] Accept validated grid/domain/run settings through the API and expose them in the UI; enforce resource limits and timestep stability checks.
- [x] Seed instantaneous 3D streamlines at inlets and integrate through the velocity grid using RK4.
- [ ] Add particle pathlines using velocity fields across simulation time.
- [ ] Explore isosurfaces, Q-criterion, and volume raymarching as separate visualization modes.
- [ ] Investigate nonuniform meshes and OpenFOAM mesh/backend integration. The existing starter case only generates meshes outside the Rust solver.
