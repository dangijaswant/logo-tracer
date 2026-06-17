# Spec: Smooth-range Bézier fitting (STAGE 3)

**Refs:** bezier-primer.md (`#curvefitting`, `#pointcurves`, `#catmullfitting`, `#curvature`),
drawing-good-paths.md (magic triangle), bezier-method-ac.md (minimal nodes).
**Guide phase:** 8 (Kurbo). **Hard rule:** only smooth ranges reach the fitter.

## Goal
Fit each smooth, single-orientation range (between corners/inflections, with extrema nodes)
to the fewest cubics that stay within tolerance, with handles inside the magic triangle.

## Method options (best → cheapest)
1. **kurbo `fit_to_bezpath` / `fit_to_bezpath_opt`** (guide's intended path). Robust
   least-squares fit. Feed ONLY smooth ranges (corners pre-split). Configure tolerance.
2. **Least-squares cubic** (Pomax `#curvefitting`, matrix `T·M·C`) if not using kurbo.
3. **3-point / circular-arc cubic** (Pomax `#pointcurves` + ABC identity) for short arc-like
   ranges — cheap, good for rounded corners.
4. **Catmull-Rom → cubic** (prototype, tension k≈0.16) as a fallback; convert per
   `#catmullfitting`. Lowest quality; keep only as backstop.

## Constraints (validate every fit)
- **Magic triangle:** both handles on the same side, neither crossing the other or its
  elongation. Reject/clamp fits that violate → re-split the range.
- **Endpoints exact:** fitted segment endpoints must equal the range endpoints to sub-pixel;
  shared endpoints between adjacent ranges must match exactly (closed-path topology).
- **Tangent continuity** at smooth–smooth joins (G1); corners are intentional discontinuity.
- **No NaN/Inf**; guard degenerate (zero-length) input. Never hide a fit failure — return a
  typed error or use a documented fallback (guide Phase 8).

## Config
```rust
pub bezier_fit_tolerance: f64,
pub max_bezier_segments_per_range: usize,
```

## Pseudocode
```
fn fit_range(range) -> Vec<Cubic>:
    if range.is_line(line_fit_tolerance): return as_line(range)   // -> specs/lines.md
    cubics = kurbo_fit(range, bezier_fit_tolerance)
              .or_else(|| least_squares(range))
              .unwrap_or_else(|| catmull_rom(range))
    for c in &cubics: assert magic_triangle_ok(c) && finite(c)
    snap_endpoints_exact(cubics, range)
    return cubics
```

## Gate
`quality_gate.js`: cubic count low and concentrated on genuinely curved ranges; zero cubics
on straight runs; no tiny/zero-length segments; endpoints continuous.
