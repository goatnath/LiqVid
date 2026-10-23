#[allow(dead_code)]
pub struct Mesh {
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    pub dx: f64,
    pub dy: f64,
    pub dz: f64,
    pub lx: f64,
    pub ly: f64,
    pub lz: f64,
}

impl Mesh {
    pub fn new(nx: usize, ny: usize, nz: usize, lx: f64, ly: f64, lz: f64) -> Self {
        Mesh {
            nx,
            ny,
            nz,
            dx: lx / nx as f64,
            dy: ly / ny as f64,
            dz: lz / nz as f64,
            lx,
            ly,
            lz,
        }
    }

    pub fn num_cells(&self) -> usize {
        self.nx * self.ny * self.nz
    }
    pub fn cell_idx(&self, i: usize, j: usize, k: usize) -> usize {
        i + j * self.nx + k * self.nx * self.ny
    }

    pub fn neighbors(&self, i: usize, j: usize, k: usize) -> [Option<usize>; 6] {
        [
            (i > 0).then(|| self.cell_idx(i - 1, j, k)),
            (i + 1 < self.nx).then(|| self.cell_idx(i + 1, j, k)),
            (j > 0).then(|| self.cell_idx(i, j - 1, k)),
            (j + 1 < self.ny).then(|| self.cell_idx(i, j + 1, k)),
            (k > 0).then(|| self.cell_idx(i, j, k - 1)),
            (k + 1 < self.nz).then(|| self.cell_idx(i, j, k + 1)),
        ]
    }
}
