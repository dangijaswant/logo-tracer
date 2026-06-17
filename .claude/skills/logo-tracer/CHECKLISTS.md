# Per-stage checklists (pass/fail gates)

Run `scripts/quality_gate.js <svg>` for the automated version. These are the human gates
for each stage. A stage is "done" only when its gate passes AND default tracing is unchanged.

## STAGE 1 — Keying / alpha  (spec: keylines.md)
- [ ] Transparent background remains transparent in output.
- [ ] No white/black halo color fragments around former transparent edges.
- [ ] Semi-transparent pixels assigned to nearest dominant color (no blended grays).
- [ ] (DYEZ is opaque → expect ~no-op; test alpha on a transparent fixture.)

## STAGE 2 — Clustering / palette  (spec: palette-masks.md)
- [ ] `fillColorCount` small (target 2 for DYEZ: one dark, one light).
- [ ] No tiny accidental color fragments (the 47 grays are gone).
- [ ] Sub-min-area components discarded; counters/holes preserved.
- [ ] Colors detected, not hardcoded.

## STAGE 3 — Contour → path  (specs: corners, extrema, fitting, lines)
- [ ] Contours split at corners BEFORE fitting (no cubic spans a corner).
- [ ] Letter corners sharp at 10x zoom; intended rounded edges still smooth.
- [ ] Straight runs are `L`/`H`/`V`, not cubics.
- [ ] FLAT near-axis cubics are a small fraction of total cubics (R2: <= ~15%). R2 only
      counts a near-axis cubic if it is ALSO flat (control points hug the start->end chord)
      = a straight wrongly encoded as a curve. Genuine shallow serif/letter curves bulge off
      the chord and are NOT counted. A large flat-axis count => fix the line detection.
- [ ] Nodes only at corners + extrema + inflections; node/cubic count near minimal.
- [ ] Curve handles axis-aligned at extrema; inside magic triangle (no crossing).
- [ ] No zero-length / shallow / tiny segments.

## STAGE 4 — Serialize  (specs: lines, primitives, optical)
- [ ] Correct `viewBox`; coordinates rounded to `path_precision`.
- [ ] Consecutive collinear lines merged; redundant transforms avoided.
- [ ] Primitives (`rect/circle/...`) only where error ≤ path; custom curves kept.
- [ ] Optical corrections off/minimal by default; nothing skewed/rotated.
- [ ] Closed paths, exact matching endpoints, deterministic output (snapshot-stable).

## CROSS-CUTTING (every change)
- [ ] Default / photo / poster / pixel tracing NOT regressed (byte-compare baseline).
- [ ] Improvement shown in `quality_gate.js` metrics vs baseline (not just by eye).
- [ ] Behavior behind the `logo` preset / explicit config only.
- [ ] Valid in Chrome, Firefox, Inkscape, project preview.

## Reference targets (DYEZ fixture)
| metric | baseline (before) | Python target (`news-cg-clean.svg`) |
|---|---|---|
| paths | 54 | 1 |
| fill colors | 49 | 2 |
| cubics | 530 | 96 |
| lines (L/H/V) | 0 | 46 |
| near-axis cubics | 256 (134H+122V) | 14 |
| tiny segments | 18 | 0 |
| file bytes | 20223 | 5308 |
