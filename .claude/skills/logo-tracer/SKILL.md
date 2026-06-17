---
name: logo-tracer
description: Craft and quality-gate logo/icon SVG tracing so straight edges become lines, corners stay sharp, curves use minimal extrema-placed nodes, and colors collapse to a small palette. This skill should be used when implementing, tuning, or reviewing the custom VTracer "Logo Mode" pipeline (visioncortex/vtracer), when porting the Python clean-up prototype to Rust, or when deciding where in the trace to apply path-quality rules. Provides per-stage implementation specs, a metrics quality-gate, and six design references (Glyphs, Pomax Bézier, Box Method, Material keylines, optical corrections, the Bézier Game).
license: MIT
---

# Logo Tracer

A specialized guide for producing **human-quality logo/icon traces** in the custom VTracer
pipeline. It turns six design references into (a) concrete Rust implementation specs and
(b) an automated quality gate, and tells you **exactly where in the trace each rule applies**.

## The one objective (everything else serves this)
Place on-curve nodes **only at corners + curve extrema (H/V tangents) + inflections**;
draw straight runs as **`L`/`H`/`V`** (never cubics); keep curve handles **axis-aligned and
inside the magic triangle**; **minimize node count**; collapse to a **small palette**; then
optionally snap to shared keylines and apply optical corrections **last**.

> Litmus test: *"Would this path pass The Bézier Game with a near-minimal node count?"*

## When to use
- Implementing/tuning the `logo` preset in `cmdapp` or `webapp` (visioncortex engine).
- Porting `test-output/metrics/_clean_svg.py` (the working prototype) to Rust.
- Reviewing a traced SVG's quality, or deciding which stage a problem belongs to.

## Where it plugs into the trace (pipeline stage map)
The real pipeline is in `vtracer/cmdapp/src/converter.rs :: color_image_to_svg`. Apply the
rules **between** these stages — see `specs/pipeline-map.md` for code-level detail.

| # | Trace stage (code) | Logo rule to apply | Spec |
|---|---|---|---|
| 1 | keying — `should_key_image` / key color | alpha-edge cleanup; semi-transparent → nearest opaque | `specs/keylines.md` (alpha note) |
| 2 | clustering — `Runner::run()` (`good_min_area`, `color_precision_loss`) | small dominant palette; drop sub-area debris | `specs/palette-masks.md` |
| 3 | **contour → path — `cluster.to_compound_path(corner_threshold, length_threshold, max_iterations, splice_threshold)`** | **CORE: corner-split → extrema nodes → smooth ranges → fit; emit lines on straights** | `specs/corners.md`, `specs/extrema.md`, `specs/fitting.md`, `specs/lines.md` |
| 4 | serialize — `svg.add_path` / `SvgFile` | emit `L`/`H`/`V`; recognize primitives; optical pass | `specs/primitives.md`, `specs/optical.md` |

**Most work happens at stage 3** (`to_compound_path`). Do NOT pass a contour containing hard
corners straight into a smooth fitter — split at corners first (guide rule #11).

## How to use it mid-trace (the loop)
Treat each change as a measurable loop, not a guess:

1. **Pick the stage** for the problem (use the table). A wavy vertical edge = stage 3 line
   detection; 49 colors = stage 2 palette; halos = stage 1 alpha.
2. **Load the matching spec** in `specs/` and implement the smallest change there.
3. **Trace the fixture** and **run the quality gate**:
   ```powershell
   node .claude/skills/logo-tracer/scripts/quality_gate.js <traced.svg>
   ```
   It scores the SVG against the rules and prints PASS/FAIL **per rule** (near-axis cubics,
   line usage, palette size, tiny segments, node count) vs the baseline.
4. **Compare versions**: baseline vs your output vs the Python target
   (`test-output/metrics/news-cg-clean.svg`). Improvement must show in the metrics, not just
   by eye (project guide rule #10 + #47: measurable before/after).
5. **Loop** until the gate passes and default (non-logo) tracing is unchanged.

See `CHECKLISTS.md` for the per-stage pass/fail gates.

## References (load as needed — don't read all up front)
Detailed design rationale lives in `references/` (one file per source). Pull the relevant
one when a spec cites it:
- `references/drawing-good-paths.md` — Glyphs: magic triangle, extrema, minimal nodes (human spec)
- `references/bezier-primer.md` — Pomax: extrema `B'(t)=0`, inflections `C(t)=0`, least-squares fit (math)
- `references/bezier-vectorize-logo.md` — Box Method: anchors at bbox-touch points (workflow)
- `references/material-icon-design.md` — Material: keyline grid, on-pixel, unified radii (consistency)
- `references/optical-corrections.md` — Logo Geek: overshoot, irradiation (optional final pass)
- `references/bezier-method-ac.md` — Bézier Game: minimal-node objective (scoring philosophy)

## Hard rules (never violate)
1. Default/photo/poster/pixel tracing must not regress — logo behavior stays behind the preset.
2. Never fit a corner-containing contour as one smooth curve — split at corners first.
3. Straight runs are lines, not cubics. No zero-length / shallow / tiny segments.
4. Closed contours, exact matching endpoints, consistent winding, deterministic output.
5. Optical corrections are optional, off/minimal by default, applied last, never skew/rotate.
6. Prove every improvement with `quality_gate.js` metrics before claiming it.
