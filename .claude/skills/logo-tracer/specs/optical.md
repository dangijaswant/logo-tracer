# Spec: Optical corrections + keyline snapping (STAGE 4, final, optional)

**Refs:** optical-corrections.md (overshoot, irradiation, gestalt), material-icon-design.md
(keylines, on-pixel), drawing-good-paths.md (don't over-tidy).
**Guide phase:** post-11. **Rule:** optional, off/minimal by default, last, never skew/rotate.

## Why this is last and optional
The tracer's first job is **faithful reconstruction**. Optical corrections intentionally
**break the grid** to look right to the eye, so they must be toggleable and conservative.
Over-snapping destroys intentional optical balance (the Google-G lesson).

## Keyline snapping (consistency, tolerance-bounded)
- Build a small shared **keyline set** (consistent stem x's, baseline/cap y's) from the
  detected shapes; snap near-miss edges to it. **Tolerance-bounded** — only near-misses.
- Round node coordinates to whole/half pixels (`path_precision`).
- Unify near-equal **corner radii** and **stem widths** to canonical values.

## Optical corrections (each a toggle, default ~off)
- **Overshoot:** let round/pointed extrema (O, C, A apex) extend a small configurable amount
  past shared baseline/cap-line instead of snapping flush. Default ~0.
- **Irradiation / knockout:** for an inverted (light-on-dark) variant, apply a small inward
  offset so it looks optically equal to dark-on-light. Default off.
- **Preserve figure/ground & closure:** never fill counters or break negative space.

## Config
```rust
pub keyline_snap: bool,
pub overshoot: f64,          // default 0
pub knockout_thinning: f64,  // default 0
```

## Gate
`quality_gate.js`: with corrections OFF, output is faithful (visual similarity not worse than
baseline). With corrections ON, stems/radii are consistent and nothing is skewed/rotated.
Document any optical setting used (guide rule: measurable + documented).
