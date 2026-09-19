import { ChevronRight } from 'lucide-react';
import { useState } from 'react';

const FIELDS = [
  { id: 'velocity_mag', label: 'Velocity Magnitude |U|', unit: 'm/s' },
  { id: 'pressure', label: 'Pressure', unit: 'Pa' },
  { id: 'vx', label: 'Velocity X (Ux)', unit: 'm/s' },
  { id: 'vy', label: 'Velocity Y (Uy)', unit: 'm/s' },
  { id: 'vz', label: 'Velocity Z (Uz)', unit: 'm/s' },
];

export default function FieldSelector({ activeField, setActiveField }) {
  const [open, setOpen] = useState(true);
  const currentField = FIELDS.find(f => f.id === activeField) || FIELDS[0];

  return (
    <div className="pv-section">
      <div className="pv-section-header" onClick={() => setOpen(!open)}>
        <ChevronRight size={12} className={`pv-section-chevron ${open ? 'open' : ''}`} />
        Coloring: {currentField.label}
      </div>
      {open && (
        <div className="pv-section-body">
          {FIELDS.map((f) => (
            <label
              key={f.id}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                fontSize: '11px',
                cursor: 'pointer',
                padding: '2px 0',
                color: activeField === f.id ? '#5b9bd5' : '#e0e0e0',
                fontWeight: activeField === f.id ? 600 : 400,
              }}
            >
              <input
                type="radio"
                name="field"
                checked={activeField === f.id}
                onChange={() => setActiveField(f.id)}
                style={{ accentColor: '#5b9bd5' }}
              />
              {f.label} <span style={{ color: '#999', fontSize: '10px' }}>({f.unit})</span>
            </label>
          ))}
        </div>
      )}
    </div>
  );
}
