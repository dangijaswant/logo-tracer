# A Primer on Bézier Curves — Pomax (math implementation reference)

**Source:** https://pomax.github.io/bezierinfo/
**Author:** Mike "Pomax" Kamermans (interactive, open-source primer)
**Role in this project:** the *mathematical implementation reference* for the Rust
logo-tracer's curve work. When we need the actual formula/algorithm (extrema, fitting,
inflections, arcs), this is the canonical source. Sections below are the ones that map
directly to our pipeline; the full primer has 48 sections.

---

## Sections most relevant to the logo-tracer

### Derivatives (`#derivatives`) — foundation for everything
- **The derivative of an nth-degree Bézier is an (n−1)-degree Bézier.**
- For a cubic (weights w0..w3), the derivative has 3 weights:
  `w'0 = 3(w1−w0)`, `w'1 = 3(w2−w1)`, `w'2 = 3(w3−w2)`.
- → We use this to get tangents (for corner/smooth classification) and to find extrema.

### Finding extremities: root finding (`#extremities`) — KEY for node placement
- Extrema = where a **component derivative is zero**: solve `B'(t) = 0` per axis (x and y).
- Cubic's derivative is a **quadratic** → solve with the **quadratic formula**.
- Guard: if the leading term is ~0, fall back (no division by zero).
- → This is how we place on-curve nodes at **x/y extrema** (the "extremum points" rule
  from Glyphs). Implement `extrema_t(curve) -> [t...]` for x and y, then split there.

### Tangents and normals (`#pointvectors`)
- Tangent at t = normalized derivative; normal = perpendicular.
- → Used for corner detection (tangent-angle change) and for constraining handle
  directions to horizontal/vertical at extrema.

### Curve inflections (`#inflections`) — split smooth runs here too
- Inflection = where **curvature C(t) = 0** (the osculating circle flips sides), i.e. an
  S-curve's midpoint.
- `C(t)` = cross product of the **first and second derivatives**.
- **Trick:** axis-align the curve first (translate start to origin, rotate so end is on
  the x-axis); this zeroes contributions and makes `C(t)=0` far easier to solve.
- → We split contour ranges at inflections (matches Glyphs "one segment = one
  orientation"). Combine with corner-splitting before fitting.

### Bounding boxes (`#boundingbox`) — the "Box Method" math
- Once you have extrema + endpoints, min/max over x and y gives the **axis-aligned
  bounding box**.
- Tight (curve-aligned) boxes need the align step first.
- → Backs the "Box Method" reconstruction workflow: compute each shape's bbox from its
  extrema, then reason about keylines/alignment relative to that box.

### Curve fitting (`#curvefitting`) — replace the prototype's plain Catmull-Rom
- Automated fitting = **least-squares polynomial regression**: pick an appropriate order,
  then minimize **summed squared distance** between data points and the curve.
- Uses the **matrix form** `T · M · C` (powers · basis matrix · coordinates), following
  Jim Herold's "Least Squares Bézier Fit".
- Needs reasonable `t` values per data point (parameterization) — see point-curves /
  arc-length below for getting them without guessing.
- → For each **smooth contour range** we do a least-squares cubic fit instead of naive
  Catmull-Rom, giving cleaner, more controllable handles. (kurbo's `fit_to_bezpath`
  implements a robust variant — guide Phase 8.)

### Creating a curve from three points (`#pointcurves`) + ABC identity (`#abc`)
- **ABC / projection identity:** for any on-curve point at parameter t, the ratio of
  distances |A→B| : |B→C| is **fixed by t alone** (independent of where start/end/control
  points are). Lets you reconstruct a curve from 3 points + a tangent by running de
  Casteljau **in reverse**.
- Cubic-from-3-points: assume the arc ≈ **circular arc through the 3 points**, fit a
  circle (perpendicular bisectors of two chords meet at center), take the tangent at the
  middle point (perpendicular to center line), then place e1/e2 on that tangent.
- → Useful fast path when a smooth range is short/arc-like: build a good cubic from
  endpoints + one interior sample without full least-squares.

### Catmull-Rom ↔ Bézier (`#catmullconv`, `#catmullfitting`)
- Catmull-Rom and cubic Bézier are inter-convertible via fixed conversion formulae.
- A Catmull-Rom needs ≥4 points; "from 3 points" = make the cubic Bézier, then convert.
- → Our prototype uses Catmull-Rom→cubic with tension k≈0.16. Keep as a cheap fallback,
  but prefer extrema-aware least-squares fitting for quality.

### Arc length, approximate (`#arclengthapprox`) + tracing at fixed intervals (`#tracing`)
- **Approximate arc length** = flatten the curve to segments and sum linear distances;
  error shrinks as segment count grows (integer part agrees even at low counts).
- → Used to (a) decide sampling density when flattening contours, and (b) drop tiny
  subpaths below a length threshold (the prototype drops subpaths with length < 8).

### Curvature & smooth joins (`#curvature`)
- A smooth join needs shared endpoint **and matching tangents** (G1); matching higher
  derivatives → smoother (G2…). But derivative *values* depend on parameterization, so the
  invariant quantity is **curvature reparameterized by distance**.
- → When we rejoin smooth ranges with line ranges at corners, we enforce exact endpoint
  match (mandatory) and, at smooth-smooth joins, tangent continuity for clean transitions.

### Flattening (`#flattening`) & splitting (`#splitting`)
- de Casteljau gives stable curve **splitting** at any t, and **flattening** to polylines.
- → Flatten cubics to polylines (prototype's `flatten`) for RDP/corner analysis;
  split at chosen t (extrema/inflections/corners) for re-fitting.

---

## Distilled algorithm map (Pomax → our Rust pipeline)
1. **Flatten** contour cubics → polyline (de Casteljau / fixed-interval sampling).
2. **Corner detect** via tangent-angle change (derivative section).
3. **Extrema** via `B'(t)=0` per axis (quadratic formula) → candidate node positions.
4. **Inflections** via `C(t)=0` (cross of 1st/2nd derivative, axis-aligned) → split points.
5. **Split** contour at corners + inflections into single-orientation smooth ranges.
6. **Fit** each smooth range with least-squares cubic (matrix form) honoring extrema nodes
   and H/V tangents; fall back to 3-point/Catmull-Rom for short arcs.
7. **Join** ranges: exact shared endpoints; tangent continuity at smooth joins.
8. **Bounding box** from extrema → feeds Box-Method keyline alignment.
9. **Arc length** to drop tiny subpaths and choose sampling density.

> Licensing note: the primer's code is MIT/credited; we take *ideas/formulae*, not
> verbatim code. Any adapted snippet must keep attribution (guide Phase 9 rule).
