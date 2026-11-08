// Tests for tools/svg-metrics/index.js — the structural SVG metrics tool.
// Uses the built-in node:test runner (no deps): `node --test tools/svg-metrics/test/`

import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  parseArgs,
  countElements,
  uniqueFills,
  getViewBoxOrSize,
  tokenizePath,
  analyzePathData,
  nearAxis,
} from '../index.js';

const opts = () => ({ tinyThreshold: 1.5, axisDeg: 4 });

test('parseArgs: positional input + flags', () => {
  const a = parseArgs(['node', 'index.js', 'a.svg', '--out', 'r.json', '--tiny-threshold', '2']);
  assert.equal(a.input, 'a.svg');
  assert.equal(a.out, 'r.json');
  assert.equal(a.tinyThreshold, 2);
});

test('countElements: counts opening tags, skips xml/doctype noise', () => {
  const svg = `<?xml version="1.0"?><svg><path/><path/><rect/></svg>`;
  const counts = countElements(svg);
  assert.equal(counts.svg, 1);
  assert.equal(counts.path, 2);
  assert.equal(counts.rect, 1);
});

test('uniqueFills: dedupes and drops "none" case-insensitively', () => {
  const svg = `<path fill="red"/><path fill="RED"/><path fill="none"/><path fill="blue"/>`;
  const fills = uniqueFills(svg);
  assert.deepEqual(fills, ['red', 'blue']);
});

test('getViewBoxOrSize: prefers viewBox, falls back to width/height', () => {
  assert.deepEqual(getViewBoxOrSize('<svg viewBox="0 0 100 100">'), { viewBox: '0 0 100 100' });
  assert.deepEqual(getViewBoxOrSize('<svg width="50" height="60">'), { width: '50', height: '60' });
});

test('tokenizePath: handles missing whitespace and negative numbers', () => {
  const tokens = tokenizePath('M0 0L10-5C1 2 3 4 5 6Z');
  // M, L, C, Z each open a token; C collects 6 coords
  const c = tokens.find((t) => t.cmd === 'C');
  assert.equal(c.coords.length, 6);
  assert.equal(tokens.some((t) => t.cmd === 'Z'), true);
});

test('analyzePathData: near-horizontal cubic is flagged (should be a line)', () => {
  // A cubic whose start->end delta is nearly horizontal over a long run.
  const d = 'M0 0 C 100 -0.5, 200 0.5, 300 0';
  const a = analyzePathData(d, opts());
  assert.equal(a.cubic, 1);
  assert.equal(a.nearHCubic, 1, 'long near-horizontal cubic must be flagged');
});

test('analyzePathData: counts H and V segments as lines', () => {
  const a = analyzePathData('M0 0H10V10Z', opts());
  assert.equal(a.horiz, 1);
  assert.equal(a.vert, 1);
  assert.equal(a.line, 2, 'H and V both count toward total line segments');
});

test('analyzePathData: tiny closing segment is counted', () => {
  // Subpath: M to (0,0), L to (0,0.5) — a 0.5px segment below the 1.5 threshold.
  const a = analyzePathData('M0 0L0 0.5Z', opts());
  assert.equal(a.tinySegments, 2, 'the 0.5px L and its Z closure are both tiny');
});

test('nearAxis: classifies h / v / null', () => {
  assert.equal(nearAxis(50, 0, 4), 'h');
  assert.equal(nearAxis(0, 50, 4), 'v');
  assert.equal(nearAxis(30, 30, 4), null, '45deg is not near an axis');
});
