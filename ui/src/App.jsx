import { useState, useRef, useMemo, useCallback } from 'react';
import { getCfdColor } from './colormap';
import Toolbar from './Toolbar';
import PipelineBrowser from './PipelineBrowser';
import PropertiesPanel from './PropertiesPanel';
import Viewport from './Viewport';
import StatusBar from './StatusBar';

export default function App() {
  const fileInputRef = useRef(null);
  const abortRef = useRef(null);
  const [stlFileUrl, setStlFileUrl] = useState(null);
  const [stlBase64, setStlBase64] = useState(null);

  // Properties panel context
  const [activeSection, setActiveSection] = useState('geometry');

  // Pipeline Browser layer visibility
  const [layerVisibility, setLayerVisibility] = useState({
    domain: true,
    geometry: true,
    slice: true,
    fluidBody: true,
    vectors: false,
  });
  const [selectedLayer, setSelectedLayer] = useState('geometry');

  const toggleLayer = useCallback((id) => {
    setLayerVisibility((prev) => ({ ...prev, [id]: !prev[id] }));
  }, []);

  // Geometry Configuration
  const [geometryType, setGeometryType] = useState('channel_cylinder');
  const [cylinderRadius, setCylinderRadius] = useState(12.0);
  const [cylinderPos, setCylinderPos] = useState({ x: 35.0, y: 50.0 });
  const [stepParams, setStepParams] = useState({ length: 30.0, height: 40.0 });
  const [boxParams, setBoxParams] = useState({
    min_x: 35.0, max_x: 65.0, min_y: 35.0, max_y: 65.0, min_z: 10.0, max_z: 50.0,
  });

  // Boundary Conditions
  const [boundaries, setBoundaries] = useState({
    x_min: { type: 'inlet', velocity: { x: 1.5, y: 0.0, z: 0.0 } },
    x_max: { type: 'outlet', pressure: 0.0 },
    y_min: { type: 'no_slip' },
    y_max: { type: 'no_slip' },
    z_min: { type: 'slip' },
    z_max: { type: 'slip' },
  });

  // Fluid Properties
  const [fluidProperties, setFluidProperties] = useState({
    kinematicViscosity: 0.001,
    density: 1.0,
  });

  // Simulation State
  const [isSimulating, setIsSimulating] = useState(false);
  const [simStatus, setSimStatus] = useState('idle');
  const [frameData, setFrameData] = useState(null);
  const [obstacleVoxels, setObstacleVoxels] = useState([]);
  const [currentStep, setCurrentStep] = useState(0);
  const [maxDiv, setMaxDiv] = useState(0);
  const [activeField, setActiveField] = useState('velocity_mag');
  const [convergenceHistory, setConvergenceHistory] = useState([]);

  // Preset Handler
  const applyPreset = (preset) => {
    setGeometryType(preset);
    if (preset === 'box') {
      setBoundaries({
        x_min: { type: 'inlet', velocity: { x: 1.0, y: 0.0, z: 0.0 } },
        x_max: { type: 'outlet', pressure: 0.0 },
        y_min: { type: 'no_slip' },
        y_max: { type: 'no_slip' },
        z_min: { type: 'slip' },
        z_max: { type: 'slip' },
      });
    } else if (preset === 'channel_cylinder') {
      setCylinderRadius(12.0);
      setCylinderPos({ x: 35.0, y: 50.0 });
      setBoundaries({
        x_min: { type: 'inlet', velocity: { x: 2.0, y: 0.0, z: 0.0 } },
        x_max: { type: 'outlet', pressure: 0.0 },
        y_min: { type: 'no_slip' },
        y_max: { type: 'no_slip' },
        z_min: { type: 'slip' },
        z_max: { type: 'slip' },
      });
    } else if (preset === 'step') {
      setStepParams({ length: 30.0, height: 40.0 });
      setBoundaries({
        x_min: { type: 'inlet', velocity: { x: 1.5, y: 0.0, z: 0.0 } },
        x_max: { type: 'outlet', pressure: 0.0 },
        y_min: { type: 'no_slip' },
        y_max: { type: 'no_slip' },
        z_min: { type: 'slip' },
        z_max: { type: 'slip' },
      });
    } else if (preset === 'cavity') {
      setBoundaries({
        x_min: { type: 'no_slip' },
        x_max: { type: 'no_slip' },
        y_min: { type: 'no_slip' },
        y_max: { type: 'moving_wall', velocity: { x: 1.0, y: 0.0, z: 0.0 } },
        z_min: { type: 'slip' },
        z_max: { type: 'slip' },
      });
    }
  };

  const handleFileUpload = (event) => {
    const file = event.target.files[0];
    if (file) {
      const url = URL.createObjectURL(file);
      setStlFileUrl(url);
      setGeometryType('stl');
      const reader = new FileReader();
      reader.onload = (e) => {
        const base64 = e.target.result.split(',')[1];
        setStlBase64(base64);
      };
      reader.readAsDataURL(file);
    }
  };

  const updateBoundary = (face, field, value) => {
    setBoundaries((prev) => {
      const current = prev[face] || { type: 'no_slip' };
      if (field === 'type') {
        let newBc = { type: value };
        if (value === 'inlet') newBc.velocity = { x: 1.0, y: 0.0, z: 0.0 };
        if (value === 'outlet') newBc.pressure = 0.0;
        if (value === 'moving_wall') newBc.velocity = { x: 1.0, y: 0.0, z: 0.0 };
        return { ...prev, [face]: newBc };
      }
      if (field.startsWith('velocity.')) {
        const axis = field.split('.')[1];
        return {
          ...prev,
          [face]: {
            ...current,
            velocity: { ...(current.velocity || { x: 0, y: 0, z: 0 }), [axis]: parseFloat(value) || 0 },
          },
        };
      }
      if (field === 'pressure') {
        return {
          ...prev,
          [face]: { ...current, pressure: parseFloat(value) || 0 },
        };
      }
      return prev;
    });
  };

  // ─── Simulation with AbortController ───
  const submitSimulation = async () => {
    const controller = new AbortController();
    abortRef.current = controller;

    let geoPayload = { type: 'box' };
    if (geometryType === 'channel_cylinder') {
      geoPayload = { type: 'channel_cylinder', radius: cylinderRadius, center_x: cylinderPos.x, center_y: cylinderPos.y };
    } else if (geometryType === 'step') {
      geoPayload = { type: 'step', step_length: stepParams.length, step_height: stepParams.height };
    } else if (geometryType === 'box_obstacle') {
      geoPayload = { type: 'box_obstacle', ...boxParams };
    } else if (geometryType === 'stl' && stlBase64) {
      geoPayload = { type: 'stl', stl_base64: stlBase64 };
    }

    const payload = {
      kinematicViscosity: fluidProperties.kinematicViscosity,
      density: fluidProperties.density,
      geometry: geoPayload,
      boundaries: boundaries,
      inlets: [],
      stl_base64: stlBase64,
    };

    setFrameData(null);
    setObstacleVoxels([]);
    setCurrentStep(0);
    setConvergenceHistory([]);
    setMaxDiv(0);
    setIsSimulating(true);
    setSimStatus('running');

    try {
      const response = await fetch('http://127.0.0.1:3000/simulate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
        signal: controller.signal,
      });

      if (!response.body) throw new Error('ReadableStream not supported.');

      const reader = response.body.getReader();
      const decoder = new TextDecoder('utf-8');
      let buffer = '';

      while (true) {
        const { value, done } = await reader.read();
        if (done) {
          setIsSimulating(false);
          setSimStatus('complete');
          break;
        }
        buffer += decoder.decode(value, { stream: true });
        let lines = buffer.split('\n\n');
        buffer = lines.pop();

        for (let line of lines) {
          if (line.startsWith('data:')) {
            const data = line.substring(line.indexOf(':') + 1).trimStart();
            if (data.startsWith('[OBSTACLES]')) {
              try {
                const voxels = JSON.parse(data.replace('[OBSTACLES]', ''));
                setObstacleVoxels(voxels);
              } catch (_) { /* malformed SSE chunk — ignore */ }
            } else if (data.startsWith('[FRAME]')) {
              try {
                const frame = JSON.parse(data.replace('[FRAME]', ''));
                setFrameData(frame);
                if (frame.max_div !== undefined) {
                  setCurrentStep(frame.step || 0);
                  setMaxDiv(frame.max_div || 0);
                  setConvergenceHistory((prev) => [...prev, {
                    step: frame.step,
                    maxDiv: frame.max_div,
                    velMax: frame.vel_max,
                  }]);
                }
              } catch (_) { /* malformed SSE chunk — ignore */ }
            } else {
              // Parse timestep info from log messages
              const stepMatch = data.match(/Time Step (\d+):.*Divergence = ([\d.]+)/);
              if (stepMatch) {
                setCurrentStep(parseInt(stepMatch[1]));
                setMaxDiv(parseFloat(stepMatch[2]));
              }
            }
          }
        }
      }
    } catch (error) {
      if (error.name === 'AbortError') {
        setSimStatus('idle');
      } else {
        setSimStatus('error');
      }
      setIsSimulating(false);
    }
  };

  const stopSimulation = () => {
    if (abortRef.current) {
      abortRef.current.abort();
      setIsSimulating(false);
      setSimStatus('idle');
    }
  };

  // Compute dynamic color scale bounds
  const { minMag, maxMag } = useMemo(() => {
    let currentMin = 0.0;
    let currentMax = 1.0;

    let inletNominal = 1.0;
    if (boundaries.x_min?.type === 'inlet' && boundaries.x_min.velocity?.x) {
      inletNominal = Math.abs(boundaries.x_min.velocity.x);
    }

    if (frameData?.slice && frameData.slice.length > 0) {
      const valid = frameData.slice.filter((v) => v >= 0);
      if (valid.length > 0) {
        currentMin = Math.min(...valid);
        currentMax = Math.max(...valid);
      }
    } else if (frameData?.cells && frameData.cells.length > 0) {
      const mags = frameData.cells.map((c) => c.mag);
      currentMin = Math.min(...mags);
      currentMax = Math.max(...mags);
    }

    const maxBound = Math.max(currentMax, inletNominal * 1.4, 0.5);
    const minBound = Math.min(currentMin, 0.0);
    return { minMag: minBound, maxMag: maxBound };
  }, [frameData, boundaries]);

  return (
    <div className="pv-app">
      <Toolbar
        activeSection={activeSection}
        setActiveSection={setActiveSection}
        isSimulating={isSimulating}
        onRun={submitSimulation}
        onStop={stopSimulation}
        layerVisibility={layerVisibility}
        toggleLayer={toggleLayer}
      />

      <div className="pv-main">
        <div className="pv-left-panel">
          <PipelineBrowser
            layerVisibility={layerVisibility}
            toggleLayer={toggleLayer}
            selectedLayer={selectedLayer}
            setSelectedLayer={setSelectedLayer}
          />
          <PropertiesPanel
            activeSection={activeSection}
            geometryType={geometryType}
            setGeometryType={setGeometryType}
            cylinderRadius={cylinderRadius}
            setCylinderRadius={setCylinderRadius}
            cylinderPos={cylinderPos}
            setCylinderPos={setCylinderPos}
            stepParams={stepParams}
            setStepParams={setStepParams}
            boxParams={boxParams}
            setBoxParams={setBoxParams}
            applyPreset={applyPreset}
            fileInputRef={fileInputRef}
            handleFileUpload={handleFileUpload}
            boundaries={boundaries}
            updateBoundary={updateBoundary}
            fluidProperties={fluidProperties}
            setFluidProperties={setFluidProperties}
          />
          <FieldSelector activeField={activeField} setActiveField={setActiveField} />
          <StatsPanel frameData={frameData} />
          <ConvergencePlot history={convergenceHistory} />
        </div>

        <Viewport
          layerVisibility={layerVisibility}
          boundaries={boundaries}
          geometryType={geometryType}
          cylinderRadius={cylinderRadius}
          cylinderPos={cylinderPos}
          stepParams={stepParams}
          boxParams={boxParams}
          obstacleVoxels={obstacleVoxels}
          stlFileUrl={stlFileUrl}
          frameData={frameData}
          minMag={minMag}
          maxMag={maxMag}
          activeField={activeField}
        />
      </div>

      <StatusBar
        currentStep={currentStep}
        totalSteps={frameData?.total_steps || 100}
        maxDiv={maxDiv}
        numCells={frameData?.fluid_cells || 50 * 50 * 30}
        simStatus={simStatus}
      />
    </div>
  );
}
