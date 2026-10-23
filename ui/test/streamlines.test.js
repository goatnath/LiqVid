import test from 'node:test';
import assert from 'node:assert/strict';
import { sampleVelocity, traceStreamline, seedStreamlines, buildStreamlines } from '../src/streamlines.js';

function grid(dimensions, origin, spacing, field) {
  const velocity = [];
  for (let z = 0; z < dimensions[2]; z++) for (let y = 0; y < dimensions[1]; y++) for (let x = 0; x < dimensions[0]; x++) {
    velocity.push(field(...[x,y,z].map((v,a) => origin[a] + v * spacing[a])));
  }
  return { dimensions, origin, spacing, velocity };
}

test('trilinear interpolation reproduces an affine 3D vector field', () => {
  const field = (x,y,z) => [2*x+y, y-z, z+3*x];
  const g = grid([5,6,7], [-2,-3,-4], [1,2,3], field);
  const point = [-0.3,1.2,2.4];
  sampleVelocity(g,point).forEach((v,a) => assert.ok(Math.abs(v-field(...point)[a]) < 1e-10));
  assert.equal(sampleVelocity(g,[-3,0,0]), null);
  assert.deepEqual(sampleVelocity(g,[-2,-3,-4]), field(-2,-3,-4));
});

test('uniform flow gives straight paths and terminates at the grid boundary', () => {
  const g = grid([11,5,5], [0,0,0], [1,1,1], () => [2,0,0]);
  const points = traceStreamline(g,[0.01,2,2]);
  assert.ok(points.length > 40 && points.length < 60);
  assert.ok(points.at(-1).position[0] > 9.5);
  for (const p of points) { assert.equal(p.position[1],2); assert.equal(p.position[2],2); assert.equal(p.speed,2); }
});

test('RK4 follows an analytic helix with nonzero Z velocity', () => {
  const g = grid([11,11,11], [-5,-5,0], [1,1,1], (x,y) => [-y,x,1]);
  const points = traceStreamline(g,[3,0,1], { step: 0.1, maxSteps: 100 });
  assert.equal(points.length,101);
  const angle = 10/Math.sqrt(10);
  const expected = [3*Math.cos(angle), 3*Math.sin(angle), 1+angle];
  points.at(-1).position.forEach((v,a) => assert.ok(Math.abs(v-expected[a]) < 1e-5));
});

test('solid blocks stop integration and zero velocity produces no path', () => {
  const g = grid([11,5,5], [0,0,0], [1,1,1], x => x === 5 ? null : [1,0,0]);
  const points = traceStreamline(g,[0.01,2,2]);
  assert.ok(points.at(-1).position[0] <= 4);
  assert.equal(sampleVelocity(g,[4.5,2,2]),null);
  assert.equal(traceStreamline(g,[5,2,2]).length,0);
  const still = grid([3,3,3], [0,0,0], [1,1,1], () => [0,0,0]);
  assert.equal(buildStreamlines(still,{}).length,0);
});

test('all inlet orientations seed 64 paths across both transverse axes', () => {
  const g = grid([11,11,11], [-5,-5,-5], [1,1,1], () => [1,1,1]);
  for (const [axis,name] of ['x','y','z'].entries()) for (const side of ['min','max']) {
    const { seeds, bidirectional } = seedStreamlines(g, { [`${name}_${side}`]: { type: 'inlet', velocity: { [name]: side === 'min' ? 1 : -1 } } });
    assert.equal(seeds.length,64);
    assert.equal(bidirectional,false);
    for (const p of seeds) assert.ok(Math.abs(p[axis] - (side === 'min' ? -5 : 5)) < 0.001);
    for (const a of [0,1,2].filter(a => a !== axis)) assert.equal(new Set(seeds.map(p => p[a])).size,8);
  }
  const cavity = seedStreamlines(g,{});
  assert.equal(cavity.seeds.length,64);
  assert.equal(cavity.bidirectional,true);
});
