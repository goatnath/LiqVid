import { Canvas } from '@react-three/fiber';
import { OrbitControls, GizmoHelper, GizmoViewcube } from '@react-three/drei';
import * as THREE from 'three';
import DomainBoundaryVisualizer from './DomainBoundaryVisualizer';
import { ContinuousFluidSlice, ContinuousFluidBody, FlowVectorGlyphs } from './FluidVisualization';

function AxisIndicator() {
  return <axesHelper args={[15]} />;
}

export default function Viewport({
  layerVisibility,
  boundaries,
  geometryType,
  cylinderRadius,
  cylinderPos,
  stepParams,
  boxParams,
  obstacleVoxels,
  stlFileUrl,
  frameData,
  minMag,
  maxMag,
  activeField,
}) {
  return (
    <div className="pv-viewport">
      <Canvas
        camera={{ position: [90, 80, 110], fov: 45 }}
        gl={{ antialias: true }}
        onCreated={({ gl }) => {
          gl.setClearColor('#1a1a1a');
        }}
      >
        <ambientLight intensity={0.6} />
        <directionalLight position={[100, 100, 50]} intensity={1.0} />
        <directionalLight position={[-100, -100, -50]} intensity={0.4} />

        <OrbitControls makeDefault />

        {/* ParaView-style orientation gizmo */}
        <GizmoHelper alignment="bottom-right" margin={[70, 70]}>
          <GizmoViewcube
            color="#444"
            textColor="#ddd"
            hoverColor="#5b9bd5"
            strokeColor="#666"
            opacity={0.85}
          />
        </GizmoHelper>

        {/* Origin axes */}
        <AxisIndicator />

        {/* Domain wireframe + geometry + obstacles */}
        {(layerVisibility.domain || layerVisibility.geometry) && (
          <DomainBoundaryVisualizer
            boundaries={boundaries}
            geometryType={layerVisibility.geometry ? geometryType : 'box'}
            cylinderRadius={cylinderRadius}
            cylinderPos={cylinderPos}
            stepParams={stepParams}
            boxParams={boxParams}
            obstacleVoxels={layerVisibility.geometry ? obstacleVoxels : []}
            stlUrl={stlFileUrl}
          />
        )}

        {/* Cut plane slice */}
        {frameData && layerVisibility.slice && frameData.slice && (
          <ContinuousFluidSlice
            sliceData={frameData.slice}
            pressureSlice={frameData.pressure_slice}
            nx={frameData.nx || 50}
            ny={frameData.ny || 50}
            minMag={minMag}
            maxMag={maxMag}
            sliceZ={0}
            activeField={activeField}
          />
        )}

        {/* 3D Fluid body */}
        {frameData && layerVisibility.fluidBody && frameData.cells && (
          <ContinuousFluidBody
            cells={frameData.cells}
            cellW={frameData.cell_w || 4.0}
            cellH={frameData.cell_h || 4.0}
            cellD={frameData.cell_d || 4.0}
            minMag={minMag}
            maxMag={maxMag}
          />
        )}

        {/* Vector glyphs */}
        {frameData && layerVisibility.vectors && frameData.cells && (
          <FlowVectorGlyphs
            cells={frameData.cells}
            minMag={minMag}
            maxMag={maxMag}
            stepStride={2}
          />
        )}
      </Canvas>

      {/* Vertical Color Legend */}
      <div className="pv-colorbar" style={{ top: '40px', right: '16px' }}>
        <div style={{ position: 'relative', display: 'flex' }}>
          <div>
            <div className="pv-colorbar-gradient" />
          </div>
          <div className="pv-colorbar-ticks">
            <span className="pv-colorbar-tick">{maxMag.toFixed(2)}</span>
            <span className="pv-colorbar-tick">{((minMag + maxMag) * 0.75).toFixed(2)}</span>
            <span className="pv-colorbar-tick">{((minMag + maxMag) * 0.5).toFixed(2)}</span>
            <span className="pv-colorbar-tick">{((minMag + maxMag) * 0.25).toFixed(2)}</span>
            <span className="pv-colorbar-tick">{minMag.toFixed(2)}</span>
          </div>
        </div>
        <div className="pv-colorbar-title">{activeField === 'pressure' ? 'P [Pa]' : '|U| [m/s]'}</div>
      </div>
    </div>
  );
}
