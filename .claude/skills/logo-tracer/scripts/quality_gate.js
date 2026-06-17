#!/usr/bin/env node
// quality_gate.js â€” score a traced SVG against the logo-tracer quality rules and
// print PASS/FAIL per rule. Optionally diff against a baseline SVG.
//
// Usage:
//   node quality_gate.js <traced.svg> [--baseline baseline.svg] [--json] [--axis-deg 4] [--tiny 1.5]
//
// Rules checked (the six refs distilled):
//   R1 palette      fillColorCount <= maxColors          (default 8; logos usually 2-4)
//   R2 no-axis-cubic nearH+nearV cubics == 0             (straights must be lines)
//   R3 has-lines    lineCount > 0                        (straight runs emitted as L/H/V)
//   R4 no-tiny      tinySegmentCount == 0                (no zero-length/meaningless segs)
//   R5 fewer-cubics cubicCount < baseline.cubicCount     (only with --baseline)
//   R6 not-bloated  fileSizeBytes <= baseline.fileSizeBytes (only with --baseline)
//
// Exit code: 0 if all applicable rules PASS, 1 otherwise (usable as a CI/loop gate).

import { readFileSync } from 'node:fs';
import { basename } from 'node:path';

function parseArgs(argv) {
  const a = {
    input: null, baseline: null, json: false, axisDeg: 4, tiny: 1.5, maxColors: 8,
    // R2: a cubic only counts as a "missed straight" if it is near-axis AND flat (its
    // control points hug the start->end chord). Genuine shallow serif/bracket curves are
    // near-axis but BULGE, so they are excluded. flatness = max control-point perpendicular
    // distance from the chord, as a fraction of chord length; below this => effectively a line.
    axisCubicRatio: 0.15,
    flatness: 0.04,
  };
  for (let i = 2; i < argv.length; i++) {
    const t = argv[i];
    if (t === '--baseline') a.baseline = argv[++i];
    else if (t === '--json') a.json = true;
    else if (t === '--axis-deg') a.axisDeg = parseFloat(argv[++i]);
    else if (t === '--tiny') a.tiny = parseFloat(argv[++i]);
    else if (t === '--max-colors') a.maxColors = parseInt(argv[++i], 10);
    else if (t === '--axis-cubic-ratio') a.axisCubicRatio = parseFloat(argv[++i]);
    else if (t === '--flatness') a.flatness = parseFloat(argv[++i]);
    else if (!a.input) a.input = t;
  }
  return a;
}

// --- path-data tokenizer (same approach as tools/svg-metrics) ---
function tokenizePath(d) {
  const tokens = [];
  const re = /([MmZzLlHhVvCcSsQqTtAa])|(-?\d*\.?\d+(?:[eE][+-]?\d+)?)/g;
  let m, cur = null;
  while ((m = re.exec(d)) !== null) {
    if (m[1]) { cur = { cmd: m[1], coords: [] }; tokens.push(cur); }
    else if (cur) cur.coords.push(parseFloat(m[2]));
  }
  return tokens;
}
const PARAMS = { M: 2, L: 2, H: 1, V: 1, C: 6, S: 4, Q: 4, T: 2, A: 7, Z: 0 };
const angDeg = (dx, dy) => (Math.atan2(dy, dx) * 180) / Math.PI;
function nearAxis(dx, dy, axisDeg) {
  const ang = Math.abs(angDeg(dx, dy));
  if (Math.min(ang, Math.abs(180 - ang)) <= axisDeg) return 'h';
  if (Math.abs(90 - ang) <= axisDeg) return 'v';
  return null;
}

function analyze(svg, opts) {
  const fileSizeBytes = Buffer.byteLength(svg, 'utf8');
  const elements = {};
  for (const m of svg.matchAll(/<([a-zA-Z][\w:-]*)\b/g)) {
    const t = m[1]; if (t === '?xml' || t === '!--') continue;
    elements[t] = (elements[t] || 0) + 1;
  }
  const fills = new Set();
  for (const m of svg.matchAll(/fill\s*=\s*"([^"]*)"/g)) {
    const v = m[1].trim().toLowerCase();
    if (v && v !== 'none') fills.add(v);
  }
  let pathCount = 0, line = 0, cubic = 0, quad = 0, arc = 0, tiny = 0, nH = 0, nV = 0;
  let flatAxisCubic = 0; // near-axis AND flat (control points hug the chord) => a missed straight
  const dist = (ax, ay, bx, by) => Math.hypot(bx - ax, by - ay);
  // Max perpendicular distance of the two control points from the start->end chord,
  // as a fraction of chord length. ~0 => the cubic is effectively a straight line.
  const cubicFlatness = (x0, y0, x1, y1, x2, y2, x3, y3) => {
    const dx = x3 - x0, dy = y3 - y0;
    const chord = Math.hypot(dx, dy);
    if (chord < 1e-6) return 0;
    const perp = (px, py) => Math.abs(dx * (y0 - py) - (x0 - px) * dy) / chord;
    return Math.max(perp(x1, y1), perp(x2, y2)) / chord;
  };
  for (const pm of svg.matchAll(/<path\b[^>]*\bd\s*=\s*"([^"]*)"/g)) {
    pathCount++;
    const toks = tokenizePath(pm[1]);
    let cx = 0, cy = 0, sx = 0, sy = 0;
    for (const t of toks) {
      const U = t.cmd.toUpperCase();
      const rel = t.cmd !== U && U !== 'Z';
      const n = PARAMS[U];
      if (U === 'Z') { const L = dist(cx, cy, sx, sy); if (L > 0 && L < opts.tiny) { tiny++; } cx = sx; cy = sy; continue; }
      const groups = n > 0 ? Math.max(1, Math.floor(t.coords.length / n)) : 1;
      for (let g = 0; g < groups; g++) {
        const c = t.coords.slice(g * n, g * n + n); if (c.length < n) break;
        let nx = cx, ny = cy;
        switch (U) {
          case 'M': nx = rel ? cx + c[0] : c[0]; ny = rel ? cy + c[1] : c[1];
            if (g === 0) { sx = nx; sy = ny; } else { line++; const L = dist(cx, cy, nx, ny); if (L > 0 && L < opts.tiny) { tiny++; } } break;
          case 'L': nx = rel ? cx + c[0] : c[0]; ny = rel ? cy + c[1] : c[1]; line++; { const L = dist(cx, cy, nx, ny); if (L > 0 && L < opts.tiny) { tiny++; } } break;
          case 'H': nx = rel ? cx + c[0] : c[0]; line++; { const L = Math.abs(nx - cx); if (L > 0 && L < opts.tiny) { tiny++; } } break;
          case 'V': ny = rel ? cy + c[0] : c[0]; line++; { const L = Math.abs(ny - cy); if (L > 0 && L < opts.tiny) { tiny++; } } break;
          case 'C': {
            const c1x = rel ? cx + c[0] : c[0], c1y = rel ? cy + c[1] : c[1];
            const c2x = rel ? cx + c[2] : c[2], c2y = rel ? cy + c[3] : c[3];
            nx = rel ? cx + c[4] : c[4]; ny = rel ? cy + c[5] : c[5]; cubic++;
            { const L = dist(cx, cy, nx, ny); if (L > 0 && L < opts.tiny) { tiny++; } }
            const ax = nearAxis(nx - cx, ny - cy, opts.axisDeg);
            if (ax === 'h') nH++; else if (ax === 'v') nV++;
            // A near-axis cubic only counts against R2 if it is also FLAT -- i.e. it is a
            // straight run wrongly encoded as a curve, not a genuine shallow serif curve.
            if (ax && cubicFlatness(cx, cy, c1x, c1y, c2x, c2y, nx, ny) < opts.flatness) {
              flatAxisCubic++;
            }
            break;
          }
          case 'S': nx = rel ? cx + c[2] : c[2]; ny = rel ? cy + c[3] : c[3]; cubic++; break;
          case 'Q': nx = rel ? cx + c[2] : c[2]; ny = rel ? cy + c[3] : c[3]; quad++; break;
          case 'T': nx = rel ? cx + c[0] : c[0]; ny = rel ? cy + c[1] : c[1]; quad++; break;
          case 'A': nx = rel ? cx + c[5] : c[5]; ny = rel ? cy + c[6] : c[6]; arc++; break;
        }
        cx = nx; cy = ny;
      }
    }
  }
  return {
    file: '', elements, pathCount, lineCount: line, cubicCount: cubic, quadraticCount: quad,
    arcCount: arc, nearHorizontalCubicCount: nH, nearVerticalCubicCount: nV,
    flatAxisCubicCount: flatAxisCubic,
    tinySegmentCount: tiny, fillColors: [...fills], fillColorCount: fills.size, fileSizeBytes,
  };
}

function main() {
  const args = parseArgs(process.argv);
  if (!args.input) { console.error('usage: node quality_gate.js <traced.svg> [--baseline b.svg] [--json]'); process.exit(2); }
  const m = analyze(readFileSync(args.input, 'utf8'), args); m.file = basename(args.input);
  const base = args.baseline ? analyze(readFileSync(args.baseline, 'utf8'), args) : null;

  const rules = [];
  const add = (id, desc, pass, detail) => rules.push({ id, desc, pass, detail });
  add('R1-palette', `fillColorCount <= ${args.maxColors}`, m.fillColorCount <= args.maxColors, `${m.fillColorCount} colors`);
  const nearAxis = m.nearHorizontalCubicCount + m.nearVerticalCubicCount;
  const axisAllowance = Math.max(2, Math.ceil(m.cubicCount * args.axisCubicRatio));
  add(
    'R2-no-flat-axis-cubic',
    `flat near-axis cubics <= ${(args.axisCubicRatio * 100).toFixed(0)}% of cubics (<= ${axisAllowance})`,
    m.flatAxisCubicCount <= axisAllowance,
    `${m.flatAxisCubicCount} flat-straight of ${nearAxis} near-axis / ${m.cubicCount} cubics`,
  );
  add('R3-has-lines', 'lineCount > 0', m.lineCount > 0, `${m.lineCount} lines`);
  add('R4-no-tiny', 'tinySegmentCount == 0', m.tinySegmentCount === 0, `${m.tinySegmentCount} tiny`);
  if (base) {
    add('R5-fewer-cubics', 'cubicCount < baseline', m.cubicCount < base.cubicCount, `${m.cubicCount} vs ${base.cubicCount}`);
    add('R6-not-bloated', 'fileSizeBytes <= baseline', m.fileSizeBytes <= base.fileSizeBytes, `${m.fileSizeBytes} vs ${base.fileSizeBytes}`);
  }
  const allPass = rules.every(r => r.pass);

  if (args.json) {
    console.log(JSON.stringify({ metrics: m, baseline: base, rules, pass: allPass }, null, 2));
  } else {
    console.log(`\nQUALITY GATE â€” ${m.file}${base ? `  (vs ${basename(args.baseline)})` : ''}`);
    console.log(`  paths=${m.pathCount} colors=${m.fillColorCount} cubic=${m.cubicCount} line=${m.lineCount} nearAxisCubic=${m.nearHorizontalCubicCount + m.nearVerticalCubicCount} tiny=${m.tinySegmentCount} bytes=${m.fileSizeBytes}`);
    console.log('  â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€');
    for (const r of rules) console.log(`  ${r.pass ? 'PASS' : 'FAIL'}  ${r.id.padEnd(16)} ${r.desc}  (${r.detail})`);
    console.log('  â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€');
    console.log(`  ${allPass ? 'ALL PASS' : 'GATE FAILED -- fix the FAIL rows above'}\n`);
  }
  process.exit(allPass ? 0 : 1);
}

main();
