import { useState } from 'react';
import { ChevronRight, Upload, Settings } from 'lucide-react';

function Section({ title, defaultOpen = false, children }) {
  const [open, setOpen] = useState(defaultOpen);
  return (
    <div className="pv-section">
      <div className="pv-section-header" onClick={() => setOpen(!open)}>
        <ChevronRight size={12} className={`pv-section-chevron ${open ? 'open' : ''}`} />
        {title}
      </div>
      {open && <div className="pv-section-body">{children}</div>}
    </div>
  );
}

export default function PropertiesPanel({
  activeSection,
  // Geometry
  geometryType,
  setGeometryType,
  cylinderRadius,
  setCylinderRadius,
  cylinderPos,
  setCylinderPos,
  stepParams,
  setStepParams,
  boxParams,
  setBoxParams,
  applyPreset,
  fileInputRef,
  handleFileUpload,
  // Boundaries
  boundaries,
  updateBoundary,
  // Physics
  fluidProperties,
  setFluidProperties,
}) {
  const wallLabels = {
    x_min: 'X_min (Inflow)',
    x_max: 'X_max (Outflow)',
    y_min: 'Y_min (Floor)',
    y_max: 'Y_max (Ceiling)',
    z_min: 'Z_min (Back)',
    z_max: 'Z_max (Front)',
  };

  return (
    <div className="pv-properties">
      <div className="pv-properties-header">
        <Settings size={13} />
        Properties
      </div>

      {/* GEOMETRY SECTION */}
      {activeSection === 'geometry' && (
        <>
          <Section title="Model Presets" defaultOpen={true}>
            <div className="pv-preset-grid">
              {[
                { id: 'channel_cylinder', label: 'Cylinder Flow' },
                { id: 'step', label: 'Backward Step' },
                { id: 'box_obstacle', label: 'Box Obstacle' },
                { id: 'box', label: 'Straight Channel' },
                { id: 'cavity', label: 'Driven Cavity' },
                { id: 'stl', label: 'Custom STL' },
              ].map((item) => (
                <button
                  key={item.id}
                  className={`pv-preset-btn ${geometryType === item.id ? 'active' : ''}`}
                  onClick={() => applyPreset(item.id)}
                >
                  {item.label}
                </button>
              ))}
            </div>
          </Section>

          {geometryType === 'channel_cylinder' && (
            <Section title="Cylinder Parameters" defaultOpen={true}>
              <div className="pv-field">
                <span className="pv-field-label">Radius (mm)</span>
                <input
                  type="range" min="4" max="25" step="1"
                  value={cylinderRadius}
                  onChange={(e) => setCylinderRadius(parseFloat(e.target.value))}
                />
                <input
                  type="number" className="pv-field-input" style={{ width: '50px', flex: 'none' }}
                  value={cylinderRadius}
                  onChange={(e) => setCylinderRadius(parseFloat(e.target.value) || 4)}
                />
              </div>
              <div className="pv-field">
                <span className="pv-field-label">Position X (mm)</span>
                <input
                  type="range" min="20" max="70" step="1"
                  value={cylinderPos.x}
                  onChange={(e) => setCylinderPos({ ...cylinderPos, x: parseFloat(e.target.value) })}
                />
                <input
                  type="number" className="pv-field-input" style={{ width: '50px', flex: 'none' }}
                  value={cylinderPos.x}
                  onChange={(e) => setCylinderPos({ ...cylinderPos, x: parseFloat(e.target.value) || 20 })}
                />
              </div>
            </Section>
          )}

          {geometryType === 'step' && (
            <Section title="Step Parameters" defaultOpen={true}>
              <div className="pv-field">
                <span className="pv-field-label">Step Length (mm)</span>
                <input
                  type="range" min="10" max="60"
                  value={stepParams.length}
                  onChange={(e) => setStepParams({ ...stepParams, length: parseFloat(e.target.value) })}
                />
                <input
                  type="number" className="pv-field-input" style={{ width: '50px', flex: 'none' }}
                  value={stepParams.length}
                  onChange={(e) => setStepParams({ ...stepParams, length: parseFloat(e.target.value) || 10 })}
                />
              </div>
              <div className="pv-field">
                <span className="pv-field-label">Step Height (mm)</span>
                <input
                  type="range" min="10" max="60"
                  value={stepParams.height}
                  onChange={(e) => setStepParams({ ...stepParams, height: parseFloat(e.target.value) })}
                />
                <input
                  type="number" className="pv-field-input" style={{ width: '50px', flex: 'none' }}
                  value={stepParams.height}
                  onChange={(e) => setStepParams({ ...stepParams, height: parseFloat(e.target.value) || 10 })}
                />
              </div>
            </Section>
          )}

          {geometryType === 'box_obstacle' && (
            <Section title="Box Obstacle Parameters" defaultOpen={true}>
              <div className="pv-field">
                <span className="pv-field-label">X Range</span>
                <div className="pv-vector-inputs">
                  <input type="number" value={boxParams.min_x} onChange={(e) => setBoxParams({ ...boxParams, min_x: parseFloat(e.target.value) || 0 })} />
                  <input type="number" value={boxParams.max_x} onChange={(e) => setBoxParams({ ...boxParams, max_x: parseFloat(e.target.value) || 0 })} />
                </div>
              </div>
              <div className="pv-field">
                <span className="pv-field-label">Y Range</span>
                <div className="pv-vector-inputs">
                  <input type="number" value={boxParams.min_y} onChange={(e) => setBoxParams({ ...boxParams, min_y: parseFloat(e.target.value) || 0 })} />
                  <input type="number" value={boxParams.max_y} onChange={(e) => setBoxParams({ ...boxParams, max_y: parseFloat(e.target.value) || 0 })} />
                </div>
              </div>
              <div className="pv-field">
                <span className="pv-field-label">Z Range</span>
                <div className="pv-vector-inputs">
                  <input type="number" value={boxParams.min_z} onChange={(e) => setBoxParams({ ...boxParams, min_z: parseFloat(e.target.value) || 0 })} />
                  <input type="number" value={boxParams.max_z} onChange={(e) => setBoxParams({ ...boxParams, max_z: parseFloat(e.target.value) || 0 })} />
                </div>
              </div>
            </Section>
          )}

          {geometryType === 'stl' && (
            <Section title="STL Upload" defaultOpen={true}>
              <input type="file" accept=".stl" ref={fileInputRef} onChange={handleFileUpload} style={{ display: 'none' }} />
              <button className="pv-upload-btn" onClick={() => fileInputRef.current?.click()}>
                <Upload size={14} /> Upload STL File
              </button>
            </Section>
          )}
        </>
      )}

      {/* BOUNDARIES SECTION */}
      {activeSection === 'boundaries' && (
        <Section title="Wall Boundary Conditions" defaultOpen={true}>
          {['x_min', 'x_max', 'y_min', 'y_max', 'z_min', 'z_max'].map((face) => {
            const bc = boundaries[face] || { type: 'no_slip' };
            return (
              <div key={face} className="pv-bc-card">
                <div className="pv-bc-row">
                  <span className="pv-bc-label">{wallLabels[face]}</span>
                  <select
                    className="pv-field-select"
                    style={{ flex: 'none', width: '120px' }}
                    value={bc.type}
                    onChange={(e) => updateBoundary(face, 'type', e.target.value)}
                  >
                    <option value="inlet">Inlet</option>
                    <option value="outlet">Outlet</option>
                    <option value="no_slip">No-Slip Wall</option>
                    <option value="slip">Slip / Symmetry</option>
                    <option value="moving_wall">Moving Wall</option>
                  </select>
                </div>
                {(bc.type === 'inlet' || bc.type === 'moving_wall') && (
                  <div className="pv-field">
                    <span className="pv-field-label">Velocity (m/s)</span>
                    <div className="pv-vector-inputs">
                      <input type="number" step="0.1" placeholder="Ux" value={bc.velocity?.x ?? 1.0} onChange={(e) => updateBoundary(face, 'velocity.x', e.target.value)} />
                      <input type="number" step="0.1" placeholder="Uy" value={bc.velocity?.y ?? 0.0} onChange={(e) => updateBoundary(face, 'velocity.y', e.target.value)} />
                      <input type="number" step="0.1" placeholder="Uz" value={bc.velocity?.z ?? 0.0} onChange={(e) => updateBoundary(face, 'velocity.z', e.target.value)} />
                    </div>
                  </div>
                )}
                {bc.type === 'outlet' && (
                  <div className="pv-field">
                    <span className="pv-field-label">Pressure (Pa)</span>
                    <input type="number" className="pv-field-input" step="0.1" style={{ width: '80px', flex: 'none' }} value={bc.pressure ?? 0.0} onChange={(e) => updateBoundary(face, 'pressure', e.target.value)} />
                  </div>
                )}
              </div>
            );
          })}
        </Section>
      )}

      {/* PHYSICS SECTION */}
      {activeSection === 'physics' && (
        <Section title="Fluid Physical Properties" defaultOpen={true}>
          <div className="pv-field">
            <span className="pv-field-label">Viscosity ν (m²/s)</span>
            <input type="number" className="pv-field-input" step="0.0001" value={fluidProperties.kinematicViscosity} onChange={(e) => setFluidProperties({ ...fluidProperties, kinematicViscosity: parseFloat(e.target.value) || 0.001 })} />
          </div>
          <div className="pv-field">
            <span className="pv-field-label">Density ρ (kg/m³)</span>
            <input type="number" className="pv-field-input" value={fluidProperties.density} onChange={(e) => setFluidProperties({ ...fluidProperties, density: parseFloat(e.target.value) || 1.0 })} />
          </div>
          <div style={{ fontSize: '11px', color: '#999', padding: '4px 0', borderTop: '1px solid #555', marginTop: '4px' }}>
            Solver: PISO Fractional Step + Conjugate Gradient
          </div>
        </Section>
      )}
    </div>
  );
}
