import { Play, Square, Layers, Grid3x3, Wind } from 'lucide-react';

export default function Toolbar({
  activeSection,
  setActiveSection,
  isSimulating,
  onRun,
  onStop,
  layerVisibility,
  toggleLayer,
}) {
  return (
    <div className="pv-toolbar">
      <span className="pv-toolbar-brand">LiqVid</span>

      {/* Properties section selectors (like ParaView menus) */}
      {[
        { id: 'geometry', label: 'Geometry' },
        { id: 'boundaries', label: 'Boundaries' },
        { id: 'physics', label: 'Physics' },
      ].map((item) => (
        <button
          key={item.id}
          className={`pv-toolbar-btn ${activeSection === item.id ? 'active' : ''}`}
          onClick={() => setActiveSection(item.id)}
        >
          {item.label}
        </button>
      ))}

      <div className="pv-toolbar-separator" />

      {/* Visualization layer toggle icons */}
      {[
        { id: 'slice', icon: <Grid3x3 size={14} />, title: 'Cut Plane Slice' },
        { id: 'fluidBody', icon: <Layers size={14} />, title: '3D Fluid Body' },
        { id: 'vectors', icon: <Wind size={14} />, title: 'Vector Glyphs' },
      ].map((item) => (
        <button
          key={item.id}
          className={`pv-toolbar-btn ${layerVisibility[item.id] ? 'active' : ''}`}
          onClick={() => toggleLayer(item.id)}
          title={item.title}
        >
          {item.icon}
        </button>
      ))}

      <div className="pv-toolbar-spacer" />

      {/* Run / Stop buttons */}
      {isSimulating ? (
        <button className="pv-toolbar-stop" onClick={onStop}>
          <Square size={13} /> Stop
        </button>
      ) : (
        <button className="pv-toolbar-run" onClick={onRun}>
          <Play size={13} /> Run
        </button>
      )}
    </div>
  );
}
