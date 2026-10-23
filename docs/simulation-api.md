# Simulation configuration

`POST /simulate` accepts JSON and returns server-sent events. Geometry and all
six face boundary conditions are required. The mesh is currently fixed at
50 × 50 × 30 cells over a 100 × 100 × 60 domain. Geometry coordinates start at
(0, 0, 0); visualization coordinates are shifted by (-50, -50, -30).

```json
{
  "kinematicViscosity": 0.001,
  "density": 1.0,
  "geometry": {
    "type": "channel_cylinder",
    "radius": 12.0,
    "center_x": 35.0,
    "center_y": 50.0
  },
  "boundaries": {
    "x_min": { "type": "inlet", "velocity": { "x": 1.5, "y": 0.0, "z": 0.0 } },
    "x_max": { "type": "outlet", "pressure": 0.0 },
    "y_min": { "type": "no_slip" },
    "y_max": { "type": "no_slip" },
    "z_min": { "type": "slip" },
    "z_max": { "type": "slip" }
  }
}
```

Geometry types:

- `box` and `cavity`: empty fluid domains; the face conditions define the flow.
- `channel_cylinder`: `radius`, `center_x`, `center_y`; spans the Z direction.
- `step`: `step_length`, `step_height`; starts at the origin and spans Z.
- `box_obstacle`: `min_x`, `max_x`, `min_y`, `max_y`, `min_z`, `max_z`.
- `stl`: `stl_base64` containing the file in domain coordinates. Selecting a
  different preset does not reuse a previous STL upload.

Face types are `inlet` (prescribed velocity), `outlet` (prescribed pressure and
zero-gradient velocity), `no_slip` (stationary wall), `slip` (zero normal velocity
and zero tangential gradient), and `moving_wall` (prescribed tangential velocity).
Non-outlet faces use zero-gradient pressure. Solid voxel faces impose no-slip
velocity and zero-gradient pressure; the pressure solver excludes solid cells.

Density must be positive and viscosity nonnegative. Pressure is in physical
units: the Poisson source uses density and the velocity correction divides by
density. At fixed kinematic viscosity and velocity boundaries, changing a uniform
density scales pressure while preserving velocity. A closed cavity's pressure is
determined only up to an additive constant.

This replaces the old request fields `inlets` and top-level `stl_base64`.
Unknown fields and geometry/boundary types are rejected. Invalid parameters and
STL uploads return HTTP errors before streaming. Numerical failure is reported
as a `[ERROR]` event; successful frames retain the `[FRAME]` JSON format.
The worker uses a bounded channel and stops when the response is dropped.

Each frame also contains `flow_grid`, a regular block-averaged velocity grid
for visualization: `dimensions`, centered-world `origin`, `spacing`, and an
X-fastest `velocity` array of `[vx, vy, vz]` values. A `null` entry means at least
one original cell in that block is solid. The current stride of 2 produces
25 × 25 × 15 samples. This intentionally conservative mask can end streamlines
slightly before an obstacle surface.

The explicit timestep (0.1), step count (100), and pressure iteration cap (100)
remain fixed. Completing these steps does not establish steady-state convergence.
The [standalone Re=100 cavity benchmark](validation/README.md) measures reference
agreement, steady change, and discrete divergence; its documented runs pass all
three criteria. A completed API run or passing unit tests do not establish a
validated CFD solution.
