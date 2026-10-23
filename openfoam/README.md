# OpenFOAM / snappyHexMesh mesh-generation case

This is a starter OpenFOAM case for generating a body-fitted mesh around an uploaded STL obstacle. The block domain is 100 × 100 × 60 units, matching LiqVid's structured solver domain and the UI's centered coordinates.

## Run

1. Install a compatible OpenFOAM distribution.
2. Copy the closed, watertight STL to `constant/triSurface/obstacle.stl`.
3. From this directory run:

   ```sh
   mkdir -p constant/triSurface
   blockMesh
   surfaceCheck constant/triSurface/obstacle.stl
   snappyHexMesh -overwrite
   checkMesh
   ```

4. Inspect the generated mesh with `paraFoam`.

The `locationInMesh` point in `system/snappyHexMeshDict` must lie in the fluid region of your particular STL; change it if the obstacle occupies that point. Surface refinement is set to levels 2–3 and layer addition is disabled for this starter case.

**Integration boundary:** this case currently generates a mesh in OpenFOAM. LiqVid's Rust solver still uses its own structured voxel mesh; it does not yet read OpenFOAM polyMesh files. Do not treat snappyHexMesh output as being used by the Rust solver until a polyMesh importer or an OpenFOAM solver backend is added.
