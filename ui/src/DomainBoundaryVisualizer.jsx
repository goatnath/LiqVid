import { useLoader } from '@react-three/fiber';
import { Center } from '@react-three/drei';
import { STLLoader } from 'three/examples/jsm/loaders/STLLoader';
import * as THREE from 'three';

// STL model loader component
function StlModel({ fileUrl }) {
  const geometry = useLoader(STLLoader, fileUrl);
  return (
    <mesh geometry={geometry}>
      <meshStandardMaterial color="#475569" roughness={0.4} metalness={0.3} />
    </mesh>
  );
}

// 3D Domain Bounding Box and Boundary Face Highlights
export default function DomainBoundaryVisualizer({ boundaries, geometryType, cylinderRadius, cylinderPos, stepParams, boxParams, obstacleVoxels, stlUrl }) {
  const domainSize = [100, 100, 60];

  const getWallColor = (bc) => {
    if (!bc) return 'rgba(100, 116, 139, 0.15)';
    switch (bc.type) {
      case 'inlet': return 'rgba(34, 197, 94, 0.35)';
      case 'outlet': return 'rgba(239, 68, 68, 0.35)';
      case 'no_slip': return 'rgba(71, 85, 105, 0.25)';
      case 'slip': return 'rgba(6, 182, 212, 0.25)';
      case 'moving_wall': return 'rgba(234, 179, 8, 0.4)';
      default: return 'rgba(100, 116, 139, 0.15)';
    }
  };

  return (
    <group>
      {/* Outer Bounding Box Wireframe */}
      <mesh>
        <boxGeometry args={domainSize} />
        <meshBasicMaterial color="#888888" wireframe={true} transparent opacity={0.4} />
      </mesh>

      {/* X_min (Left / West Wall) */}
      <mesh position={[-50, 0, 0]} rotation={[0, Math.PI / 2, 0]}>
        <planeGeometry args={[60, 100]} />
        <meshBasicMaterial color={getWallColor(boundaries.x_min)} transparent opacity={0.12} side={THREE.DoubleSide} />
      </mesh>

      {/* X_max (Right / East Wall) */}
      <mesh position={[50, 0, 0]} rotation={[0, Math.PI / 2, 0]}>
        <planeGeometry args={[60, 100]} />
        <meshBasicMaterial color={getWallColor(boundaries.x_max)} transparent opacity={0.12} side={THREE.DoubleSide} />
      </mesh>

      {/* Y_min (Bottom / South Wall) */}
      <mesh position={[0, -50, 0]} rotation={[Math.PI / 2, 0, 0]}>
        <planeGeometry args={[100, 60]} />
        <meshBasicMaterial color={getWallColor(boundaries.y_min)} transparent opacity={0.12} side={THREE.DoubleSide} />
      </mesh>

      {/* Y_max (Top / North Wall) */}
      <mesh position={[0, 50, 0]} rotation={[Math.PI / 2, 0, 0]}>
        <planeGeometry args={[100, 60]} />
        <meshBasicMaterial color={getWallColor(boundaries.y_max)} transparent opacity={0.12} side={THREE.DoubleSide} />
      </mesh>

      {/* Z_min (Back Wall) */}
      <mesh position={[0, 0, -30]}>
        <planeGeometry args={[100, 100]} />
        <meshBasicMaterial color={getWallColor(boundaries.z_min)} transparent opacity={0.08} side={THREE.DoubleSide} />
      </mesh>

      {/* Z_max (Front Wall) */}
      <mesh position={[0, 0, 30]}>
        <planeGeometry args={[100, 100]} />
        <meshBasicMaterial color={getWallColor(boundaries.z_max)} transparent opacity={0.08} side={THREE.DoubleSide} />
      </mesh>

      {/* Parametric 3D Obstacle Rendering */}
      {geometryType === 'channel_cylinder' && (
        <mesh position={[cylinderPos.x - 50, cylinderPos.y - 50, 0]} rotation={[Math.PI / 2, 0, 0]}>
          <cylinderGeometry args={[cylinderRadius, cylinderRadius, 60, 32]} />
          <meshStandardMaterial color="#475569" roughness={0.3} metalness={0.4} />
        </mesh>
      )}

      {geometryType === 'step' && (
        <mesh position={[stepParams.length / 2 - 50, stepParams.height / 2 - 50, 0]}>
          <boxGeometry args={[stepParams.length, stepParams.height, 60]} />
          <meshStandardMaterial color="#475569" roughness={0.3} metalness={0.4} />
        </mesh>
      )}

      {geometryType === 'box_obstacle' && (
        <mesh
          position={[
            (boxParams.min_x + boxParams.max_x) / 2 - 50,
            (boxParams.min_y + boxParams.max_y) / 2 - 50,
            (boxParams.min_z + boxParams.max_z) / 2 - 30,
          ]}
        >
          <boxGeometry
            args={[
              boxParams.max_x - boxParams.min_x,
              boxParams.max_y - boxParams.min_y,
              boxParams.max_z - boxParams.min_z,
            ]}
          />
          <meshStandardMaterial color="#475569" roughness={0.3} metalness={0.4} />
        </mesh>
      )}

      {geometryType === 'stl' && stlUrl && (
        <Center>
          <StlModel fileUrl={stlUrl} />
        </Center>
      )}

      {/* Voxelized Obstacles returned by solver */}
      {obstacleVoxels &&
        obstacleVoxels.length > 0 &&
        geometryType !== 'channel_cylinder' &&
        geometryType !== 'step' &&
        geometryType !== 'stl' && (
          <group>
            {obstacleVoxels.map((vox, idx) => (
              <mesh key={idx} position={[vox.x, vox.y, vox.z]}>
                <boxGeometry args={[2, 2, 2]} />
                <meshStandardMaterial color="#334155" />
              </mesh>
            ))}
          </group>
        )}
    </group>
  );
}
