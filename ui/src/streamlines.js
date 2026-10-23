// Instantaneous streamlines: RK4 integration in arc length, not particle time.
// The grid is cell centered, X fastest, and null blocks represent obstacles.
export function sampleVelocity(grid, point) {
  const { dimensions: n, origin, spacing, velocity } = grid;
  const q = point.map((v, a) => (v - origin[a]) / spacing[a]);
  if (q.some((v, a) => !Number.isFinite(v) || v < 0 || v > n[a] - 1)) return null;
  const lo = q.map((v, a) => Math.min(Math.floor(v), n[a] - 2));
  const t = q.map((v, a) => v - lo[a]);
  const result = [0, 0, 0];
  for (let z = 0; z < 2; z++) for (let y = 0; y < 2; y++) for (let x = 0; x < 2; x++) {
    const weight = (x ? t[0] : 1 - t[0]) * (y ? t[1] : 1 - t[1]) * (z ? t[2] : 1 - t[2]);
    if (weight < 1e-12) continue;
    const v = velocity[lo[0] + x + n[0] * (lo[1] + y + n[1] * (lo[2] + z))];
    if (!v || !v.every(Number.isFinite)) return null;
    for (let a = 0; a < 3; a++) result[a] += v[a] * weight;
  }
  return result;
}

const offset = (p, v, h) => p.map((c, a) => c + v[a] * h);

export function traceStreamline(grid, seed, { step = Math.min(...grid.spacing) * 0.2, maxSteps = 400, direction = 1 } = {}) {
  const tangent = (point) => {
    const v = sampleVelocity(grid, point);
    const speed = v && Math.hypot(...v);
    return speed > 1e-7 ? v.map(c => direction * c / speed) : null;
  };
  const points = [];
  let p = seed;
  for (let i = 0; i <= maxSteps; i++) {
    const v = sampleVelocity(grid, p);
    const a = tangent(p);
    if (!a) break;
    points.push({ position: p, speed: Math.hypot(...v) });
    const b = tangent(offset(p, a, step / 2));
    if (!b) break;
    const c = tangent(offset(p, b, step / 2));
    if (!c) break;
    const d = tangent(offset(p, c, step));
    if (!d) break;
    p = p.map((value, axis) => value + step * (a[axis] + 2*b[axis] + 2*c[axis] + d[axis]) / 6);
    // Closed orbits need only one circuit; the step cap also bounds long paths.
    if (i > 12 && Math.hypot(...p.map((v, axis) => v - seed[axis])) < step * 0.75) break;
  }
  return points;
}

export function seedStreamlines(grid, boundaries = {}) {
  const lo = grid.origin;
  const hi = lo.map((v, a) => v + (grid.dimensions[a] - 1) * grid.spacing[a]);
  const seeds = [];
  for (const [face, bc] of Object.entries(boundaries)) {
    if (bc.type !== 'inlet') continue;
    const axis = { x: 0, y: 1, z: 2 }[face[0]];
    const lowFace = face.endsWith('min');
    const component = bc.velocity?.[face[0]] || 0;
    if ((lowFace ? component : -component) <= 0) continue;
    const other = [0, 1, 2].filter(a => a !== axis);
    for (let j = 0; j < 8; j++) for (let i = 0; i < 8; i++) {
      const p = [...lo];
      p[axis] = lowFace ? lo[axis] + 1e-5 : hi[axis] - 1e-5;
      p[other[0]] = lo[other[0]] + (hi[other[0]] - lo[other[0]]) * (i + 0.5) / 8;
      p[other[1]] = lo[other[1]] + (hi[other[1]] - lo[other[1]]) * (j + 0.5) / 8;
      seeds.push(p);
    }
  }
  if (seeds.length) return { seeds, bidirectional: false };
  // Closed cavities have no inlet. Distribute interior seeds across all axes.
  for (let k = 0; k < 4; k++) for (let j = 0; j < 4; j++) for (let i = 0; i < 4; i++) {
    seeds.push([i,j,k].map((v,a) => lo[a] + (hi[a] - lo[a]) * (v + 0.5) / 4));
  }
  return { seeds, bidirectional: true };
}

export function buildStreamlines(grid, boundaries) {
  if (!grid || grid.dimensions.some(n => n < 2)) return [];
  const { seeds, bidirectional } = seedStreamlines(grid, boundaries);
  return seeds.map(seed => {
    const forward = traceStreamline(grid, seed);
    if (!bidirectional) return forward;
    const backward = traceStreamline(grid, seed, { direction: -1 });
    return [...backward.slice(1).reverse(), ...forward];
  }).filter(points => points.length > 1);
}
