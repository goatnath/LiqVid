import { useRef, useMemo, useEffect } from 'react';
import * as THREE from 'three';
import { getCfdColor } from './colormap';

// Continuous 2D Cut Plane — supports multiple field types
export function ContinuousFluidSlice({ sliceData, pressureSlice, nx = 50, ny = 50, minMag = 0.0, maxMag = 1.0, sliceZ = 0, activeField = 'velocity_mag' }) {
  const canvasRef = useRef(null);
  const textureRef = useRef(null);

  if (!canvasRef.current) {
    canvasRef.current = document.createElement('canvas');
  }
  if (!textureRef.current) {
    const tex = new THREE.CanvasTexture(canvasRef.current);
    tex.magFilter = THREE.LinearFilter;
    tex.minFilter = THREE.LinearFilter;
    textureRef.current = tex;
  }

  useMemo(() => {
    const isPressure = activeField === 'pressure';
    const data = isPressure ? pressureSlice : sliceData;
    if (!data || data.length === 0) return;

    const canvas = canvasRef.current;
    canvas.width = nx;
    canvas.height = ny;
    const ctx = canvas.getContext('2d');
    const imgData = ctx.createImageData(nx, ny);

    // Compute local min/max for the selected field
    let fieldMin = Infinity;
    let fieldMax = -Infinity;
    const solidMarker = isPressure ? -999.0 : -1.0;
    for (let i = 0; i < data.length; i++) {
      if (data[i] > solidMarker + 0.5) {
        if (data[i] < fieldMin) fieldMin = data[i];
        if (data[i] > fieldMax) fieldMax = data[i];
      }
    }
    if (!isFinite(fieldMin)) fieldMin = 0;
    if (!isFinite(fieldMax)) fieldMax = 1;

    const useMin = isPressure ? fieldMin : minMag;
    const useMax = isPressure ? fieldMax : maxMag;

    for (let j = 0; j < ny; j++) {
      for (let i = 0; i < nx; i++) {
        const idx = j * nx + i;
        const val = data[idx];
        const pixelIdx = (j * nx + i) * 4;

        if (val <= solidMarker + 0.5) {
          imgData.data[pixelIdx] = 30;
          imgData.data[pixelIdx + 1] = 30;
          imgData.data[pixelIdx + 2] = 30;
          imgData.data[pixelIdx + 3] = 255;
        } else {
          const { rgb, norm } = getCfdColor(val, useMin, useMax);
          imgData.data[pixelIdx] = rgb[0];
          imgData.data[pixelIdx + 1] = rgb[1];
          imgData.data[pixelIdx + 2] = rgb[2];
          imgData.data[pixelIdx + 3] = 230;
        }
      }
    }

    ctx.putImageData(imgData, 0, 0);
    textureRef.current.needsUpdate = true;
  }, [sliceData, pressureSlice, nx, ny, minMag, maxMag, activeField]);

  useEffect(() => {
    return () => {
      if (textureRef.current) {
        textureRef.current.dispose();
        textureRef.current = null;
      }
    };
  }, []);

  const data = activeField === 'pressure' ? pressureSlice : sliceData;
  if (!data || data.length === 0) return null;

  return (
    <mesh position={[0, 0, sliceZ]}>
      <planeGeometry args={[100, 100]} />
      <meshBasicMaterial map={textureRef.current} transparent opacity={0.95} side={THREE.DoubleSide} depthWrite={false} />
    </mesh>
  );
}

// Continuous 3D Fluid Body using InstancedMesh for performance
// FIXED: Uses InstancedMesh instead of mapping thousands of individual meshes
export function ContinuousFluidBody({ cells, cellW = 4.0, cellH = 4.0, cellD = 4.0, minMag = 0.0, maxMag = 1.0 }) {
  const meshRef = useRef();
  const tempObject = useMemo(() => new THREE.Object3D(), []);
  const maxCount = 5000; // Upper bound for instance count

  useEffect(() => {
    if (!meshRef.current || !cells || cells.length === 0) return;

    const count = Math.min(cells.length, maxCount);
    meshRef.current.count = count;

    for (let i = 0; i < count; i++) {
      const cell = cells[i];
      tempObject.position.set(cell.x, cell.y, cell.z);
      tempObject.updateMatrix();
      meshRef.current.setMatrixAt(i, tempObject.matrix);

      const { norm } = getCfdColor(cell.mag, minMag, maxMag);
      const hue = (1.0 - norm) * 240.0;
      const lightness = (40 + norm * 15) / 100;
      const color = new THREE.Color().setHSL(hue / 360, 1.0, lightness);
      meshRef.current.setColorAt(i, color);
    }

    meshRef.current.instanceMatrix.needsUpdate = true;
    if (meshRef.current.instanceColor) {
      meshRef.current.instanceColor.needsUpdate = true;
    }
  }, [cells, cellW, cellH, cellD, minMag, maxMag, tempObject]);

  if (!cells || cells.length === 0) return null;

  return (
    <instancedMesh ref={meshRef} args={[null, null, maxCount]} frustumCulled={false}>
      <boxGeometry args={[cellW * 1.02, cellH * 1.02, cellD * 1.02]} />
      <meshStandardMaterial
        transparent
        opacity={0.6}
        roughness={0.2}
        metalness={0.1}
        depthWrite={false}
        vertexColors
      />
    </instancedMesh>
  );
}

// Directional Vector Glyphs using InstancedMesh
// FIXED: Uses InstancedMesh instead of mapping individual meshes
export function FlowVectorGlyphs({ cells, minMag = 0.0, maxMag = 1.0, stepStride = 2 }) {
  const meshRef = useRef();
  const tempObject = useMemo(() => new THREE.Object3D(), []);
  const maxCount = 2500;

  const sampled = useMemo(() => {
    if (!cells) return [];
    return cells.filter((c, i) => i % stepStride === 0 && c.mag > 0.01);
  }, [cells, stepStride]);

  useEffect(() => {
    if (!meshRef.current || sampled.length === 0) return;

    const count = Math.min(sampled.length, maxCount);
    meshRef.current.count = count;

    const up = new THREE.Vector3(0, 1, 0);
    const quat = new THREE.Quaternion();

    for (let i = 0; i < count; i++) {
      const cell = sampled[i];
      const dir = new THREE.Vector3(cell.vx, cell.vy, cell.vz).normalize();
      const { norm } = getCfdColor(cell.mag, minMag, maxMag);
      const len = Math.min(7.0, 2.0 + norm * 5.0);

      quat.setFromUnitVectors(up, dir);
      tempObject.position.set(cell.x, cell.y, cell.z);
      tempObject.quaternion.copy(quat);
      tempObject.scale.set(1, len, 1);
      tempObject.updateMatrix();
      meshRef.current.setMatrixAt(i, tempObject.matrix);

      const hue = (1.0 - norm) * 240.0;
      const lightness = (40 + norm * 15) / 100;
      const color = new THREE.Color().setHSL(hue / 360, 1.0, lightness);
      meshRef.current.setColorAt(i, color);
    }

    meshRef.current.instanceMatrix.needsUpdate = true;
    if (meshRef.current.instanceColor) {
      meshRef.current.instanceColor.needsUpdate = true;
    }
  }, [sampled, minMag, maxMag, tempObject]);

  if (sampled.length === 0) return null;

  return (
    <instancedMesh ref={meshRef} args={[null, null, maxCount]} frustumCulled={false}>
      <cylinderGeometry args={[0.3, 0.3, 1, 8]} />
      <meshBasicMaterial vertexColors />
    </instancedMesh>
  );
}
