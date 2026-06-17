// Logo Tracer web UI — decodes a PNG via canvas and runs the wasm logo tracer.
import init, { trace_logo, LogoOptions } from './pkg/logo_wasm.js';

const $ = (id) => document.getElementById(id);
const els = {
  drop: $('drop'), file: $('file'), status: $('status'), results: $('results'),
  srcImg: $('srcImg'), svgBox: $('svgBox'), metrics: $('metrics'),
  maxColors: $('maxColors'), colorPrecision: $('colorPrecision'),
  filterSpeckle: $('filterSpeckle'), cornerThreshold: $('cornerThreshold'),
  retrace: $('retrace'), download: $('download'), copy: $('copy'), reset: $('reset'),
};

let wasmReady = false;
let currentRGBA = null;     // { data: Uint8Array, width, height }
let currentSVG = '';
let currentName = 'logo';

// --- init wasm ---
(async () => {
  setStatus('Loading tracer…');
  try {
    await init();
    wasmReady = true;
    clearStatus();
  } catch (e) {
    setStatus('Failed to load the tracer engine. ' + e, true);
  }
})();

// --- helpers ---
function setStatus(msg, isError = false) {
  els.status.textContent = msg;
  els.status.classList.remove('hidden');
  // ink for normal status, error red per the design system
  els.status.style.color = isError ? '#ee0000' : '#171717';
}
function clearStatus() { els.status.classList.add('hidden'); }

// live-update the value labels
const bind = (input, label) => {
  const span = document.getElementById(label);
  const sync = () => { span.textContent = input.value; };
  input.addEventListener('input', sync); sync();
};
bind(els.maxColors, 'maxColorsVal');
bind(els.colorPrecision, 'colorPrecisionVal');
bind(els.filterSpeckle, 'filterSpeckleVal');
bind(els.cornerThreshold, 'cornerThresholdVal');

// --- file intake ---
els.drop.addEventListener('click', () => els.file.click());
els.file.addEventListener('change', (e) => { if (e.target.files[0]) loadFile(e.target.files[0]); });

['dragenter', 'dragover'].forEach((ev) =>
  els.drop.addEventListener(ev, (e) => { e.preventDefault(); els.drop.classList.add('drop-active'); }));
['dragleave', 'drop'].forEach((ev) =>
  els.drop.addEventListener(ev, (e) => { e.preventDefault(); els.drop.classList.remove('drop-active'); }));
els.drop.addEventListener('drop', (e) => {
  const f = e.dataTransfer.files[0];
  if (f && f.type.startsWith('image/')) loadFile(f);
});

async function loadFile(file) {
  currentName = (file.name || 'logo').replace(/\.[^.]+$/, '');
  const url = URL.createObjectURL(file);
  els.srcImg.src = url;

  const img = new Image();
  img.onload = () => {
    // decode to RGBA via an offscreen canvas
    const canvas = document.createElement('canvas');
    canvas.width = img.naturalWidth;
    canvas.height = img.naturalHeight;
    const ctx = canvas.getContext('2d', { willReadFrequently: true });
    ctx.drawImage(img, 0, 0);
    const { data } = ctx.getImageData(0, 0, canvas.width, canvas.height);
    currentRGBA = { data: new Uint8Array(data), width: canvas.width, height: canvas.height };
    URL.revokeObjectURL(url);
    els.results.classList.remove('hidden');
    trace();
  };
  img.onerror = () => setStatus('Could not read that image.', true);
  img.src = url;
}

// --- trace ---
function trace() {
  if (!wasmReady || !currentRGBA) return;
  setStatus('Tracing…');
  // let the status paint before the (sync) wasm call
  requestAnimationFrame(() => {
    try {
      const opts = new LogoOptions();
      opts.max_colors = parseInt(els.maxColors.value, 10);
      opts.color_precision = parseInt(els.colorPrecision.value, 10);
      opts.filter_speckle = parseInt(els.filterSpeckle.value, 10);
      opts.corner_threshold_deg = parseFloat(els.cornerThreshold.value);

      const t0 = performance.now();
      currentSVG = trace_logo(currentRGBA.data, currentRGBA.width, currentRGBA.height, opts);
      const ms = Math.round(performance.now() - t0);

      els.svgBox.innerHTML = currentSVG;
      // metrics
      const paths = (currentSVG.match(/<path/g) || []).length;
      const cubics = (currentSVG.match(/[Cc]/g) || []).length;
      const colors = new Set((currentSVG.match(/fill="[^"]*"/g) || [])).size;
      const kb = (new Blob([currentSVG]).size / 1024).toFixed(1);
      els.metrics.textContent = `${paths} paths · ${colors} colors · ${kb} KB · ${ms} ms`;
      clearStatus();
    } catch (e) {
      setStatus('Tracing failed: ' + e, true);
    }
  });
}

els.retrace.addEventListener('click', trace);

// --- download / copy / reset ---
els.download.addEventListener('click', () => {
  if (!currentSVG) return;
  const blob = new Blob([currentSVG], { type: 'image/svg+xml' });
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = `${currentName}.svg`;
  a.click();
  URL.revokeObjectURL(a.href);
});

els.copy.addEventListener('click', async () => {
  if (!currentSVG) return;
  try {
    await navigator.clipboard.writeText(currentSVG);
    els.copy.textContent = 'Copied!';
    setTimeout(() => (els.copy.textContent = 'Copy SVG code'), 1200);
  } catch { setStatus('Clipboard blocked by browser.', true); }
});

els.reset.addEventListener('click', () => {
  currentRGBA = null; currentSVG = '';
  els.results.classList.add('hidden');
  els.file.value = '';
  clearStatus();
});
