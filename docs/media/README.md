# Demo media

`cylinder-ui.png` and `cylinder-flow.gif` are captures of the actual React/Three.js
UI rendering SSE snapshots from the Rust solver. They are not generated mockups
or reference-solution plots.

Settings used:

- Code: `f253eb9`, with documentation edits only in the working tree.
- Cylinder Flow preset selected explicitly: radius 12, center (35, 50), spanning Z.
- Domain: 100 × 100 × 60; grid: 50 × 50 × 30.
- Inlet: X-min, velocity (2, 0, 0); outlet: X-max, pressure 0.
- Y faces: no-slip; Z faces: slip.
- Kinematic viscosity: 0.001; density: 1; timestep: 0.1.
- Layers: domain, geometry, streamlines, and arrows enabled; cut plane and voxels disabled.
- Browser: headless Firefox with WebGL, 1440 × 1000 viewport, default camera.
- Screenshot: around step 83. The GIF uses 14 captures from approximately steps
  5–83, resized to 1008 × 700 and quantized to 128 colors. Playback timing is
  illustrative and does not represent simulation time or solver throughput.

To recreate the view, start the release backend and frontend as described in the
README, select Cylinder Flow, click Run, and capture the viewport while frames
arrive. For this capture an isolated release build used port 3001 because a
backend was already running on port 3000; browser requests were routed to the
isolated backend. The temporary copy changed only the bind address, not the
solver settings or data. The frontend used port 5174 because 5173 was occupied.

These images demonstrate the UI and transient field evolution. They do not
establish steady-state convergence, mass conservation to a specified tolerance,
or agreement with a validated CFD reference.
