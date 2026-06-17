# Spec: Straight-line detection, axis snapping, L/H/V emission (STAGE 3 → 4)

**Refs:** bezier-vectorize-logo.md (axis-aligned), material-icon-design.md (keylines,
on-pixel), drawing-good-paths.md (no handles on straights).
**Guide phase:** 7.2, 7.3, 11. **Hard rule:** straight runs are lines, never cubics.

## Line detection (per contour range)
A range is a line when **max perpendicular distance to the chord ≤ `line_fit_tolerance`**
AND **angle variance ≤ tolerance**. (Use the RDP perpendicular-distance metric from the
prototype's `rdp`.)
- Emit **`H`** if the chord is near-horizontal, **`V`** if near-vertical, else **`L`**.

## Axis snapping (logo mode only)
When a line is within `axis_snap_angle_deg` of horizontal/vertical:
- snap both endpoints to a shared coordinate (set equal y for H, equal x for V) — prototype
  `axis_snap` sets both to the midpoint;
- **preserve corner positions** (snap the edge between corners, don't move the corners apart);
- **avoid cumulative drift** — snap against absolute coordinates, not incrementally.

## Keyline consistency (Material)
Optionally snap snapped edges to a small shared **keyline set** (consistent stem x's and
baseline/cap y's across letters) and round to whole/half pixels (`path_precision`). Keeps
stems uniform so the logo reads as one system. Tolerance-bounded (see optical.md) — do not
over-snap.

## Emission (STAGE 4 serializer)
- Straight run → `H`/`V`/`L`, **no control points**.
- Merge consecutive collinear lines into one.
- Remove zero-length segments.
- Round coordinates to `path_precision`.

## Config
```rust
pub line_fit_tolerance: f64,
pub axis_snap_angle_deg: f64,
pub axis_snap_distance: f64,
```
Axis snapping MUST be off outside logo mode unless explicitly requested.

## Gate (this is the headline metric)
`quality_gate.js`:
- R2 counts only **flat** near-axis cubics (control points hug the start->end chord = a
  straight wrongly drawn as a curve), allowed <= ~15% of total cubics. Genuine shallow
  serif/letter curves are near-axis but BULGE off the chord, so they are excluded. A large
  flat-axis count means straight runs were left as cubics — strengthen the line detection.
- `lineCount` (L/H/V) > 0 and substantial.
- No wavy straight edges at 10x zoom.
Baseline had 134 near-H + 122 near-V cubics and 0 lines — those must invert.
