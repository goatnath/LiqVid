# Viewing 3D flow

Streamlines and vector arrows are enabled by default. The Pipeline Browser and
toolbar independently toggle these layers, the Z-mid cut plane, and the optional
voxel context. Voxel instances now include the entire received frame, with no
5,000-cell cutoff. Their low opacity provides context rather than a solid block.
Arrow glyphs use regular sampling across the entire cell list.

The streamline view seeds 64 points across each inflowing inlet face, interpolates
the streamed velocity grid in three dimensions, and uses fourth-order Runge–Kutta
integration. Closed cases without an inflowing inlet use 64 interior seeds and
trace both directions. Paths terminate at the grid boundary, solids, near-zero
velocity, a closed loop, or a fixed integration limit. Integration steps are
smaller than the grid spacing. A conservative coarse obstacle mask prevents
interpolation through solid blocks, at the cost of ending some paths early.

These are instantaneous streamlines recomputed for each snapshot, colored by
speed. They are not particle trajectories accumulated through simulation time.
The arrows show flow direction. Streamline and arrow colors always use speed;
the existing field selector applies to the cut plane.

The cylinder and step presets span the domain's Z dimension. Uniform inflow and
slip boundaries can therefore produce nearly identical Z slices. Visualization
preserves that field; it does not invent out-of-plane motion. Select **Box
Obstacle** or upload a finite 3D STL to explore flow around an object in Z. STL
coordinates must lie in the solver's 0–100, 0–100, 0–60 domain; the preview uses
the same coordinates.

The voxel context is not volume raymarching, and no isosurface or Q-criterion
extraction is performed. Those are separate visualization modes.

Run `npm test` in `ui/` to check interpolation, analytic 3D streamline accuracy,
obstacle and stagnation stopping, and seeding on all six inlet orientations.
