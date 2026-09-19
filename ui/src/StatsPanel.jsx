import { Activity } from 'lucide-react';

export default function StatsPanel({ frameData }) {
  if (!frameData) return null;

  const stats = [
    { label: 'Max |U|', value: (frameData.vel_max ?? 0).toFixed(4), unit: 'm/s' },
    { label: 'Avg |U|', value: (frameData.vel_avg ?? 0).toFixed(4), unit: 'm/s' },
    { label: 'Pressure Range', value: `${(frameData.p_min ?? 0).toFixed(2)} — ${(frameData.p_max ?? 0).toFixed(2)}`, unit: 'Pa' },
    { label: 'Max ∇·U', value: (frameData.max_div ?? 0).toExponential(3), unit: '' },
    { label: 'Fluid Cells', value: (frameData.fluid_cells ?? 0).toLocaleString(), unit: '' },
    { label: 'Re (approx)', value: frameData.vel_max && frameData.nu ? (frameData.vel_max * 100 / frameData.nu).toFixed(0) : '—', unit: '' },
  ];

  return (
    <div className="pv-section">
      <div className="pv-section-header" style={{ cursor: 'default' }}>
        <Activity size={12} />
        Flow Statistics
      </div>
      <div className="pv-section-body" style={{ gap: '3px' }}>
        {stats.map((s) => (
          <div key={s.label} style={{ display: 'flex', justifyContent: 'space-between', fontSize: '11px' }}>
            <span style={{ color: '#999' }}>{s.label}</span>
            <span style={{ fontFamily: 'JetBrains Mono, monospace', color: '#e0e0e0' }}>
              {s.value} <span style={{ color: '#777', fontSize: '9px' }}>{s.unit}</span>
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
