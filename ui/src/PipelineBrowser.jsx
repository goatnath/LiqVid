import { Eye, EyeOff, Database, Box, Layers, Grid3x3, Wind } from 'lucide-react';

const LAYERS = [
  { id: 'domain', label: 'Domain Wireframe', icon: <Box size={13} /> },
  { id: 'geometry', label: 'Geometry', icon: <Database size={13} /> },
  { id: 'slice', label: 'Cut Plane (Z-mid)', icon: <Grid3x3 size={13} /> },
  { id: 'fluidBody', label: 'Fluid Body (3D)', icon: <Layers size={13} /> },
  { id: 'vectors', label: 'Vector Glyphs', icon: <Wind size={13} /> },
];

export default function PipelineBrowser({
  layerVisibility,
  toggleLayer,
  selectedLayer,
  setSelectedLayer,
}) {
  return (
    <div className="pv-pipeline">
      <div className="pv-pipeline-header">
        <Database size={13} />
        Pipeline Browser
      </div>
      {LAYERS.map((layer) => (
        <div
          key={layer.id}
          className={`pv-pipeline-item ${selectedLayer === layer.id ? 'selected' : ''}`}
          onClick={() => setSelectedLayer(layer.id)}
        >
          <span
            className={`pv-pipeline-eye ${layerVisibility[layer.id] ? 'visible' : ''}`}
            onClick={(e) => {
              e.stopPropagation();
              toggleLayer(layer.id);
            }}
          >
            {layerVisibility[layer.id] ? <Eye size={14} /> : <EyeOff size={14} />}
          </span>
          <span className="pv-pipeline-icon">{layer.icon}</span>
          <span className="pv-pipeline-label">{layer.label}</span>
        </div>
      ))}
    </div>
  );
}
