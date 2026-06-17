# Spec: Dominant palette & per-color masks (STAGE 2)

**Refs:** material-icon-design.md (consistent style, drop debris), optical-corrections.md
(figure/ground, closure). **Guide phase:** 6.2, 6.3.

## Goal
Collapse the image to a **small set of dominant opaque colors** (e.g. the 2 we want:
one dark, one light) and build clean per-color masks. This is what fixes "49 colors → 2".

## Dominant palette (deterministic)
1. Gather opaque / sufficiently-visible pixels (alpha above threshold — see keylines.md).
2. Cluster colors in a **perceptual space (OKLab/Lab)**, NOT raw RGB Euclidean.
3. Weight clusters by pixel count; **remove tiny clusters**.
4. **Merge clusters below `palette_merge_threshold`** (perceptual distance).
5. **Snap anti-aliased edge colors to the nearest dominant color** (kills the 47 grey
   fragments between black and white).
6. Cap at `max_colors`.

## Per-color masks (Phase 6.3)
For each dominant color: binary mask → connected components → **discard components below
`min_component_area`** → extract outer contours + holes → keep color + layer. This separates
geometry reconstruction from color clustering and preserves **counters (figure/ground)**.

## Engine mapping
The existing `Runner`/`RunnerConfig` already clusters by color (`is_same_color_a =
color_precision_loss`) and filters by area (`good_min_area = filter_speckle_area`). For logo
mode, tighten these and add a post-cluster **palette snap** so residue colors collapse to the
dominant set before serialization.

## Config
```rust
pub max_colors: usize,
pub palette_merge_threshold: f64,
pub preserve_transparency: bool,
pub min_component_area: usize,
```
Do NOT hardcode the colors (e.g. the DYEZ black/white) — detect them.

## Gate
`quality_gate.js`: `fillColorCount` → small (target 2 for DYEZ); no tiny color fragments;
transparent background stays transparent.
