import { useRef, useEffect } from 'react';

export default function ConvergencePlot({ history }) {
  const canvasRef = useRef(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || history.length < 2) return;

    const ctx = canvas.getContext('2d');
    const W = canvas.width;
    const H = canvas.height;
    const pad = { top: 20, right: 10, bottom: 20, left: 45 };
    const plotW = W - pad.left - pad.right;
    const plotH = H - pad.top - pad.bottom;

    ctx.fillStyle = '#2a2a2a';
    ctx.fillRect(0, 0, W, H);

    // Compute ranges
    const divValues = history.map(h => h.maxDiv).filter(v => v > 0);
    if (divValues.length < 2) return;

    const minLog = Math.log10(Math.max(1e-12, Math.min(...divValues)));
    const maxLog = Math.log10(Math.max(...divValues));
    const logRange = Math.max(0.1, maxLog - minLog);

    // Grid lines
    ctx.strokeStyle = '#444';
    ctx.lineWidth = 0.5;
    for (let i = 0; i <= 4; i++) {
      const y = pad.top + (i / 4) * plotH;
      ctx.beginPath();
      ctx.moveTo(pad.left, y);
      ctx.lineTo(pad.left + plotW, y);
      ctx.stroke();

      const logVal = maxLog - (i / 4) * logRange;
      ctx.fillStyle = '#999';
      ctx.font = '9px JetBrains Mono, monospace';
      ctx.textAlign = 'right';
      ctx.fillText(`1e${logVal.toFixed(0)}`, pad.left - 4, y + 3);
    }

    // Axes
    ctx.strokeStyle = '#666';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(pad.left, pad.top);
    ctx.lineTo(pad.left, pad.top + plotH);
    ctx.lineTo(pad.left + plotW, pad.top + plotH);
    ctx.stroke();

    // Divergence line
    ctx.strokeStyle = '#5bbd5b';
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    for (let i = 0; i < divValues.length; i++) {
      const x = pad.left + (i / Math.max(1, divValues.length - 1)) * plotW;
      const logV = Math.log10(Math.max(1e-12, divValues[i]));
      const y = pad.top + plotH - ((logV - minLog) / logRange) * plotH;
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
    ctx.stroke();

    // Labels
    ctx.fillStyle = '#999';
    ctx.font = '10px sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText('Iteration', pad.left + plotW / 2, H - 2);

    ctx.save();
    ctx.translate(10, pad.top + plotH / 2);
    ctx.rotate(-Math.PI / 2);
    ctx.fillText('Max ∇·U', 0, 0);
    ctx.restore();

    // Title
    ctx.fillStyle = '#ccc';
    ctx.font = '10px sans-serif';
    ctx.textAlign = 'left';
    ctx.fillText('Convergence History', pad.left, 12);

  }, [history]);

  return (
    <div className="pv-section">
      <div className="pv-section-header" style={{ cursor: 'default' }}>
        Convergence Plot
      </div>
      <div style={{ padding: '4px' }}>
        <canvas
          ref={canvasRef}
          width={290}
          height={150}
          style={{ width: '100%', borderRadius: '2px' }}
        />
      </div>
    </div>
  );
}
