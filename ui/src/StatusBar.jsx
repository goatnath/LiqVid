export default function StatusBar({ currentStep, totalSteps, maxDiv, numCells, simStatus }) {
  const statusLabels = {
    idle: 'Ready',
    running: 'Running',
    complete: 'Complete',
    error: 'Error',
  };

  return (
    <div className="pv-statusbar">
      <div className="pv-statusbar-item">
        Time Step: {currentStep}/{totalSteps}
      </div>
      <div className="pv-statusbar-item">
        Max ∇·U: {maxDiv.toFixed(6)}
      </div>
      <div className="pv-statusbar-item">
        Cells: {numCells.toLocaleString()}
      </div>
      <div className="pv-statusbar-item">
        Solver: PISO-CG
      </div>
      <div style={{ flex: 1 }} />
      <div className="pv-statusbar-item">
        <span className={`pv-status-dot ${simStatus}`} />
        {statusLabels[simStatus] || 'Ready'}
      </div>
    </div>
  );
}
