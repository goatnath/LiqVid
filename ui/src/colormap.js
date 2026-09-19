// Scientific colormap utilities for CFD visualization

// Helper: HSL to RGB conversion
export function hslToRgb(h, s, l) {
  let r, g, b;
  if (s === 0) {
    r = g = b = l;
  } else {
    const hue2rgb = (p, q, t) => {
      if (t < 0) t += 1;
      if (t > 1) t -= 1;
      if (t < 1 / 6) return p + (q - p) * 6 * t;
      if (t < 1 / 2) return q;
      if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
      return p;
    };
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;
    r = hue2rgb(p, q, h + 1 / 3);
    g = hue2rgb(p, q, h);
    b = hue2rgb(p, q, h - 1 / 3);
  }
  return [Math.round(r * 255), Math.round(g * 255), Math.round(b * 255)];
}

// Scientific colormap generator (Blue -> Cyan -> Green -> Yellow -> Red)
export function getCfdColor(val, minVal, maxVal) {
  const span = Math.max(0.001, maxVal - minVal);
  const norm = Math.max(0.0, Math.min(1.0, (val - minVal) / span));
  const hue = (1.0 - norm) * 240.0; // 240 deg (Blue) -> 0 deg (Red)
  const lightness = 40 + norm * 15;
  const rgb = hslToRgb(hue / 360, 1.0, lightness / 100);
  return {
    hsl: `hsl(${hue}, 100%, ${lightness}%)`,
    hue,
    norm,
    rgb,
  };
}
