# The Circle Technique — DesignMantic (geometric curve construction)

**Source:** https://www.designmantic.com/how-to/how-to-use-circle-technique-to-make-your-logo
**Role in this project:** the *geometric construction* reference. It answers a specific
problem in our fitter — instead of **guessing Bézier control points with ad-hoc math**, the
right curves come from **circular arcs with a real center and radius**, snapped to a
construction grid. This is how designers (and historically Albrecht Dürer) build clean logo
and letter curves.

> Viewed reference images (chrome-devtools):
> - **Apple logo** built from circles whose radii follow the Fibonacci/golden sequence
>   (13, 8, 5, 3, 2, 1), each tangent to the next. Every curve of the mark is a circular arc.
> - **Albrecht Dürer's Latin alphabet**: each letter inside a **square grid**; bowls of
>   B/D/O/P/Q/R are **circular arcs** with defined centers/radii; stems are exact H/V lines.
> - **Squirrel illustration**: the whole silhouette is overlapping circles — body, head,
>   tail curl, ear are all circular arcs.

---

## The core principle (what to actually do)
A clean curved outline is **not** a freehand Bézier. It is built from:
1. **A construction grid / bounding box** per shape (the "box method", Material keylines).
2. **Circular arcs** for the curved parts — each arc has a **center, radius, start angle,
   end angle**. Curves of similar character share radii (golden-ratio / consistent sizes).
3. **Straight lines** (exact H/V on the grid) for the straight parts.
4. **Tangent continuity** where an arc meets a line or another arc (the construction circles
   are placed tangent to each other).

So a letter `O` = two arcs (or one circle). A `D` = one vertical stem (line) + one arc bowl.
A `C` = one arc with two line/terminal caps. Few primitives, exact geometry.

## Why this fixes our fitter's weakness
Our current fitter feeds the raw (possibly jagged) pixel contour to kurbo's least-squares
Bézier fit and tunes tolerances by eye. That works, but:
- it has **no notion that a span is a circular arc**, so it can wobble or facet;
- control points are derived from the data fit, not from a clean center/radius;
- tolerances are guessed per image size.

The circle technique says: **when a contour span is well-approximated by a circular arc,
emit it AS an arc** (exact center+radius) rather than a fitted free Bézier. Arcs are:
- defined by 3 numbers (center x/y + radius), not 6 (two control points) → fewer DOF, no
  wobble;
- naturally smooth and symmetric;
- convertible to either an SVG `<path>` `A` (elliptical-arc) command or to a single clean
  cubic per ≤90° quadrant with the **exact** control-point distance
  `k = (4/3)·tan(θ/4)·r` (Pomax `#circles_cubic`) — no guessing.

## Concrete algorithm (maps onto our pipeline, Phase 7/8/10)
For each **smooth span** (already split at corners):
1. **Fit a circle to the span's points** (least-squares circle: solve for center+radius
   that minimizes radial error). This is deterministic linear algebra, not guessing.
2. **Measure max radial deviation** of the span points from that circle.
   - If deviation ≤ `arc_tolerance` → the span **is an arc**. Emit it as an arc:
     - either an SVG `A rx ry 0 large sweep x y` elliptical-arc command, or
     - one cubic per ≤90° sub-span using `k = (4/3)·tan(Δθ/4)` × r for exact handle length,
       with handles **tangent to the circle** (perpendicular to the radius).
   - Else → fall back to the current kurbo least-squares Bézier fit (genuine free-form curve).
3. **Snap radii**: cluster near-equal arc radii across the logo to a shared value (Material
   keyline consistency / golden-ratio idea) so all bowls/corners match.
4. **Tangent join**: where an arc meets a line, the line should be tangent to the arc
   (already true if the corner split is right).

This is exactly the guide's **Phase 10 (native primitive recognition: circle/ellipse)** plus
**Phase 8 fitting**, made the *primary* path for curved spans instead of an afterthought.

## Least-squares circle fit (the deterministic math, no guessing)
Given points (xi, yi), fit center (a,b) and radius r by solving the linear system from
`xi² + yi² = 2·a·xi + 2·b·yi + (r² − a² − b²)`:
- Let `z_i = xi² + yi²`. Solve `[Σxi², Σxixi y..]` normal equations for `(a, b, c)` where
  `c = r² − a² − b²`; then `r = sqrt(c + a² + b²)`.
- Radial error of a point = `| hypot(xi−a, yi−b) − r |`. Max over the span = the arc test.
This is O(n) accumulation + a 3×3 solve. Fully deterministic.

## Exact arc→cubic (Pomax, no guessing)
For a circular arc of radius `r` spanning angle `θ ≤ 90°` from angle `φ0`:
```
k = (4/3) * tan(θ/4)            // control-point distance factor
P0 = center + r*(cos φ0, sin φ0)
P3 = center + r*(cos φ1, sin φ1)         // φ1 = φ0 + θ
T0 = (-sin φ0,  cos φ0)                   // unit tangent at P0
T1 = (-sin φ1,  cos φ1)
C1 = P0 + k*r*T0
C2 = P3 - k*r*T1
```
Split arcs > 90° at quadrant boundaries (extrema), which also satisfies the
"nodes at extrema" rule from Glyphs/Box-Method.

---

## Distilled rules → our Rust fitter
1. Per smooth span: **least-squares circle fit** → if radial error ≤ tol, it's an arc.
2. Emit arcs as **exact arc-derived cubics** (k = 4/3·tan(θ/4)·r, handles tangent), split at
   quadrant extrema. Else fall back to kurbo free fit.
3. **Snap/cluster radii** across the logo for consistency.
4. Straights stay exact H/V/L lines (unchanged).
5. Result: curves come from real geometry (center+radius), not guessed control points —
   smoother, wobble-free, fewer DOF, and tolerances are about *radial error*, a meaningful
   physical quantity, not eyeballed pixel fudge.
