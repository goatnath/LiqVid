# LiqVid

A small 3D fluid simulation project written from scratch in Rust, with a React/Three.js frontend for visualization. It solves the incompressible Navier-Stokes equations on a structured grid using a simplified pressure-projection (PISO-style) approach, loosely inspired by OpenFOAM's `icoFoam` solver.

This is a learning/portfolio project, not a production CFD tool. It's meant to demonstrate an understanding of the core numerical building blocks of a Navier-Stokes solver, not to compete with OpenFOAM, SU2, or other mature CFD software.

## What it does

- **Structured 3D grid**: a uniform grid mapped to flat 1D arrays for the velocity and pressure fields.
- **Momentum solve**: convection and viscous diffusion terms for the velocity field.
- **Pressure Poisson equation**: solved with Jacobi iteration to enforce a divergence-free velocity field.
- **STL geometry support**: loads `.stl` files, voxelizes them against the grid using Möller–Trumbore ray-triangle intersection, and applies no-slip boundary conditions on solid cells.
- **Live visualization**: the Rust backend streams 2D cross-sections of the velocity field over Server-Sent Events; the frontend renders them as a real-time heatmap and lets you click to place inlets.

## What it doesn't do (yet)

Being upfront about this because "CFD solver" can imply more than what's here:

- **No convergence/accuracy validation.** There's no comparison against known analytical solutions or benchmark cases (e.g. lid-driven cavity), so correctness is "it runs and looks physically plausible," not verified.
- **Jacobi iteration for pressure is slow to converge.** No multigrid, no conjugate gradient — fine for a small demo grid, not scalable.
- **Single-threaded, uniform grid only.** No adaptive mesh, no parallelization.
- **No automated tests.**

These are the natural next steps if this becomes more than a demo.

## Getting Started

Requires [Rust](https://rustup.rs/) and [Node.js](https://nodejs.org/).

```bash
# Clone the repository
git clone https://github.com/goatnath/LiqVid.git
cd LiqVid

# Start the Rust solver (backend)
cargo run

# In a separate terminal, start the visualizer (frontend)
cd ui
npm install
npm run dev
```

Open the local Vite URL (usually `http://localhost:5173`), upload an `.stl` file, click on the model to place a flow inlet, and run the simulation to see the velocity field update live.

## Why I built this

I wanted to learn the deep mathematics and architecture of Computational Fluid Dynamics (CFD) by building it myself, rather than just treating industry-standard software like OpenFOAM as a magical black box. Writing the momentum predictor and mass conservation loops from scratch was the best way to truly understand what happens under the hood of a Navier-Stokes solver.

## Roadmap / possible next steps

- [ ] Validate against a known benchmark flow (e.g. lid-driven cavity at low Reynolds number)
- [ ] Replace Jacobi with conjugate gradient or multigrid for the pressure solve
- [ ] Add unit tests for the mesh, boundary conditions, and Poisson solver
- [ ] Support non-uniform / adaptive grids

## License

MIT License
