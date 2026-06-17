#!/usr/bin/env node
// svg-metrics — structural metrics for traced SVGs (logo-tracer project, Phase 4).
//
// Parses the SVG structurally:
//   - element counts by tag (path, rect, circle, ellipse, polygon, ...)
//   - path-data command counts via a real tokenizer (not a loose regex over the whole file)
//   - H / V / L line counts, plus near-horizontal / near-vertical CUBIC detection
//   - unique fill colors
//   - total path-data length
//   - count of segments shorter than a configurable threshold
//   - SVG file size in bytes
//
// Usage:
//   node index.js <input.svg> [--out report.json] [--tiny-threshold 1.5] [--axis-deg 4]
//
// Exit code 0 on success; prints a human summary to stderr and the JSON report to stdout
// (also written to --out when provided).

import { readFileSync, writeFileSync } from 'node:fs';
import { basename } from 'node:path';

function parseArgs(argv) {
  const args = { input: null, out: null, tinyThreshold: 1.5, axisDeg: 4 };
  for (let i = 2; i < argv.length; i++) {
    const a = argv[i];
    if (a === '--out') args.out = argv[++i];
    else if (a === '--tiny-threshold') args.tinyThreshold = parseFloat(argv[++i]);
    else if (a === '--axis-deg') args.axisDeg = parseFloat(argv[++i]);
    else if (!args.input) args.input = a;
  }
  return args;
}

// --- structural element scan ---------------------------------------------
// Lightweight tag scan: count opening tags by name. Good enough for the flat
// SVGs vtracer emits (it does not nest shapes), and avoids a heavy XML dep.
function countElements(svg) {
  const counts = {};
  const re = /<([a-zA-Z][\w:-]*)\b/g;
  let m;
  while ((m = re.exec(svg)) !== null) {
    const tag = m[1];
    if (tag === '?xml' || tag === '!--') continue;
    counts[tag] = (counts[tag] || 0) + 1;
  }
  return counts;
}

function uniqueFills(svg) {
  const fills = new Set();
  const re = /fill\s*=\s*"([^"]*)"/g;
  let m;
  while ((m = re.exec(svg)) !== null) {
    const v = m[1].trim();
    if (v && v.toLowerCase() !== 'none') fills.add(v.toLowerCase());
  }
  return [...fills];
}

function getViewBoxOrSize(svg) {
  const vb = svg.match(/viewBox\s*=\s*"([^"]+)"/i);
  if (vb) return { viewBox: vb[1].trim() };
  const w = svg.match(/\bwidth\s*=\s*"([^"]+)"/i);
  const h = svg.match(/\bheight\s*=\s*"([^"]+)"/i);
  return { width: w ? w[1] : null, height: h ? h[1] : null };
}

// --- path-data tokenizer --------------------------------------------------
// Tokenizes a `d` attribute into [{cmd, coords:[...numbers]}] using a real
// scanner so we never mis-split on negative signs, scientific notation, or
// missing whitespace ("M0 0C..." with no space before C).
function tokenizePath(d) {
  const tokens = [];
  const re = /([MmZzLlHhVvCcSsQqTtAa])|(-?\d*\.?\d+(?:[eE][+-]?\d+)?)/g;
  let m;
  let cur = null;
  while ((m = re.exec(d)) !== null) {
    if (m[1]) {
      cur = { cmd: m[1], coords: [] };
      tokens.push(cur);
    } else if (cur) {
      cur.coords.push(parseFloat(m[2]));
    }
  }
  return tokens;
}

const PARAMS = { M: 2, L: 2, H: 1, V: 1, C: 6, S: 4, Q: 4, T: 2, A: 7, Z: 0 };

function angleDegFrom(dx, dy) {
  // angle of the vector, folded into [0,90] distance-from-axis terms handled by caller
  return (Math.atan2(dy, dx) * 180) / Math.PI;
}

function nearAxis(dx, dy, axisDeg) {
  // returns 'h' if near-horizontal, 'v' if near-vertical, else null
  const ang = Math.abs(angleDegFrom(dx, dy)); // 0..180
  const fromH = Math.min(ang, Math.abs(180 - ang));      // distance to horizontal
  const fromV = Math.abs(90 - ang);                       // distance to vertical
  if (fromH <= axisDeg) return 'h';
  if (fromV <= axisDeg) return 'v';
  return null;
}

function analyzePathData(d, opts) {
  const tinyT = opts.tinyThreshold;
  const axisDeg = opts.axisDeg;
  const tokens = tokenizePath(d);

  const cmdCounts = {};
  let move = 0, line = 0, horiz = 0, vert = 0, cubic = 0, quad = 0, arc = 0;
  let tinySegments = 0;
  let nearHCubic = 0, nearVCubic = 0;

  // current point + subpath start, tracking absolute positions for length checks
  let cx = 0, cy = 0, sx = 0, sy = 0;

  function dist(ax, ay, bx, by) {
    return Math.hypot(bx - ax, by - ay);
  }

  for (const t of tokens) {
    const upper = t.cmd.toUpperCase();
    const isRel = t.cmd !== upper && upper !== 'Z';
    const n = PARAMS[upper];
    cmdCounts[t.cmd] = (cmdCounts[t.cmd] || 0) + 1;

    if (upper === 'Z') {
      const segLen = dist(cx, cy, sx, sy);
      if (segLen > 0 && segLen < tinyT) tinySegments++;
      cx = sx; cy = sy;
      continue;
    }

    // a command can carry multiple coordinate groups (implicit repeat)
    const groups = n > 0 ? Math.max(1, Math.floor(t.coords.length / n)) : 1;
    for (let g = 0; g < groups; g++) {
      const c = t.coords.slice(g * n, g * n + n);
      if (c.length < n) break;
      let nx = cx, ny = cy;

      switch (upper) {
        case 'M': {
          nx = isRel ? cx + c[0] : c[0];
          ny = isRel ? cy + c[1] : c[1];
          // first M of a subpath sets the start; implicit repeats are treated as L
          if (g === 0) { move++; sx = nx; sy = ny; }
          else { line++; const L = dist(cx, cy, nx, ny); if (L > 0 && L < tinyT) tinySegments++; }
          break;
        }
        case 'L': {
          nx = isRel ? cx + c[0] : c[0];
          ny = isRel ? cy + c[1] : c[1];
          line++;
          const L = dist(cx, cy, nx, ny); if (L > 0 && L < tinyT) tinySegments++;
          break;
        }
        case 'H': {
          nx = isRel ? cx + c[0] : c[0]; ny = cy;
          horiz++; line++;
          const L = Math.abs(nx - cx); if (L > 0 && L < tinyT) tinySegments++;
          break;
        }
        case 'V': {
          nx = cx; ny = isRel ? cy + c[0] : c[0];
          vert++; line++;
          const L = Math.abs(ny - cy); if (L > 0 && L < tinyT) tinySegments++;
          break;
        }
        case 'C': {
          nx = isRel ? cx + c[4] : c[4];
          ny = isRel ? cy + c[5] : c[5];
          cubic++;
          const dx = nx - cx, dy = ny - cy;
          const ax = nearAxis(dx, dy, axisDeg);
          if (ax === 'h') nearHCubic++; else if (ax === 'v') nearVCubic++;
          const L = dist(cx, cy, nx, ny); if (L > 0 && L < tinyT) tinySegments++;
          break;
        }
        case 'S': {
          nx = isRel ? cx + c[2] : c[2];
          ny = isRel ? cy + c[3] : c[3];
          cubic++;
          const L = dist(cx, cy, nx, ny); if (L > 0 && L < tinyT) tinySegments++;
          break;
        }
        case 'Q': {
          nx = isRel ? cx + c[2] : c[2];
          ny = isRel ? cy + c[3] : c[3];
          quad++;
          const L = dist(cx, cy, nx, ny); if (L > 0 && L < tinyT) tinySegments++;
          break;
        }
        case 'T': {
          nx = isRel ? cx + c[0] : c[0];
          ny = isRel ? cy + c[1] : c[1];
          quad++;
          const L = dist(cx, cy, nx, ny); if (L > 0 && L < tinyT) tinySegments++;
          break;
        }
        case 'A': {
          nx = isRel ? cx + c[5] : c[5];
          ny = isRel ? cy + c[6] : c[6];
          arc++;
          const L = dist(cx, cy, nx, ny); if (L > 0 && L < tinyT) tinySegments++;
          break;
        }
      }
      cx = nx; cy = ny;
    }
  }

  return {
    cmdCounts, move, line, horiz, vert, cubic, quad, arc,
    tinySegments, nearHCubic, nearVCubic,
  };
}

function main() {
  const args = parseArgs(process.argv);
  if (!args.input) {
    console.error('usage: node index.js <input.svg> [--out report.json] [--tiny-threshold N] [--axis-deg N]');
    process.exit(2);
  }

  const svg = readFileSync(args.input, 'utf8');
  const fileSizeBytes = Buffer.byteLength(svg, 'utf8');

  const elements = countElements(svg);
  const fills = uniqueFills(svg);
  const sizeInfo = getViewBoxOrSize(svg);

  // gather all path `d` attributes
  const dRe = /<path\b[^>]*\bd\s*=\s*"([^"]*)"/g;
  const agg = {
    pathCount: 0, moveCount: 0, lineCount: 0,
    horizontalCount: 0, verticalCount: 0,
    cubicCount: 0, quadraticCount: 0, arcCount: 0,
    tinySegmentCount: 0, nearHorizontalCubicCount: 0, nearVerticalCubicCount: 0,
    totalPathDataLength: 0,
  };
  const cmdTotals = {};
  let m;
  while ((m = dRe.exec(svg)) !== null) {
    agg.pathCount++;
    agg.totalPathDataLength += m[1].length;
    const a = analyzePathData(m[1], args);
    agg.moveCount += a.move;
    agg.lineCount += a.line;
    agg.horizontalCount += a.horiz;
    agg.verticalCount += a.vert;
    agg.cubicCount += a.cubic;
    agg.quadraticCount += a.quad;
    agg.arcCount += a.arc;
    agg.tinySegmentCount += a.tinySegments;
    agg.nearHorizontalCubicCount += a.nearHCubic;
    agg.nearVerticalCubicCount += a.nearVCubic;
    for (const [k, v] of Object.entries(a.cmdCounts)) cmdTotals[k] = (cmdTotals[k] || 0) + v;
  }

  const report = {
    file: basename(args.input),
    elements,
    pathCount: agg.pathCount,
    moveCount: agg.moveCount,
    lineCount: agg.lineCount,
    horizontalCount: agg.horizontalCount,
    verticalCount: agg.verticalCount,
    cubicCount: agg.cubicCount,
    quadraticCount: agg.quadraticCount,
    arcCount: agg.arcCount,
    nearHorizontalCubicCount: agg.nearHorizontalCubicCount,
    nearVerticalCubicCount: agg.nearVerticalCubicCount,
    tinySegmentCount: agg.tinySegmentCount,
    tinyThreshold: args.tinyThreshold,
    axisSnapDeg: args.axisDeg,
    fillColors: fills,
    fillColorCount: fills.length,
    sizeInfo,
    rawCommandTotals: cmdTotals,
    totalPathDataLength: agg.totalPathDataLength,
    fileSizeBytes,
  };

  const json = JSON.stringify(report, null, 2);
  if (args.out) writeFileSync(args.out, json + '\n', 'utf8');
  process.stdout.write(json + '\n');

  // human summary -> stderr (so stdout stays clean JSON)
  const s = report;
  console.error(
    `\n[svg-metrics] ${s.file}\n` +
    `  paths=${s.pathCount}  fills=${s.fillColorCount} ${JSON.stringify(s.fillColors)}\n` +
    `  cubic=${s.cubicCount}  quad=${s.quadraticCount}  arc=${s.arcCount}\n` +
    `  lines L/H/V=${s.lineCount}/${s.horizontalCount}/${s.verticalCount}\n` +
    `  near-axis cubics  H=${s.nearHorizontalCubicCount}  V=${s.nearVerticalCubicCount}  (these SHOULD be lines)\n` +
    `  tiny segments (<${s.tinyThreshold})=${s.tinySegmentCount}\n` +
    `  pathDataLen=${s.totalPathDataLength}  fileBytes=${s.fileSizeBytes}\n`
  );
}

main();
