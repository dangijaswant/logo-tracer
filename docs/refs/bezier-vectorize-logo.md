# The Box Method — Vectorizing a Logotype with Bézier Curves (logodesign.net)

**Source:** https://www.logodesign.net/how-to-bezier-curves-vectorize-logo-tutorial
**Title:** "How to Master Bezier Curves in Illustrator Using the Box Method"
**Role in this project:** the *logo reconstruction workflow*. This describes how a human
designer decides **where to place anchor points and how to orient handles** when redrawing
a logo. It's the practical, logo-specific complement to Glyphs' theory and Pomax's math.

---

## Core idea: the Box Method
> "Whenever you make a curvy/circular shape, the program gives you anchor points and
> handles. Everywhere the bounding box touches the outline, that's the most extreme curve —
> and that's where the anchor point goes."

For any shape:
1. Draw the **tight bounding box** around it.
2. The **anchor points go exactly where the box edges touch the outline** — i.e. at the
   **extrema** (topmost, bottommost, leftmost, rightmost points).
3. **Handles are axis-aligned at those points:** horizontal handles on horizontal
   (top/bottom) extrema, vertical handles on vertical (left/right) extrema. ("Hold Shift"
   in Illustrator = constrain handle to the axis.)
4. A circle → exactly **4 anchor points** (4 box-touch points). A letter "O" = outer box
   (4) + inner oval box (4) = **8 anchor points**, then subtract the counter.

This is the same "extremum points + orthogonal handles" rule as Glyphs, expressed as a
**hands-on reconstruction recipe**.

## Key rules extracted (directly actionable for our Rust port)
1. **Anchor placement = box-touch points = curve extrema.** Don't scatter anchors; put
   them only where the bbox meets the outline. (Matches Glyphs §"Extremum points" and
   Pomax `#extremities`/`#boundingbox`.)
2. **Handle orientation is axis-aligned at extrema:** horizontal tangent at top/bottom,
   vertical tangent at left/right. → enforce H/V tangents on the off-curve control points
   adjacent to extremum nodes.
3. **Handles must not cross the opposite line / opposite handle** — "it'll mess up the
   shape." (Same as Glyphs' "magic triangle".)
4. **Work opposing handles of two anchors together**, not one in isolation — keeps the
   shape balanced/symmetric.
5. **Corners are explicit exceptions:** "all horizontal and vertical, aside from those few
   little points on the corners of 'C' and 'A'." → straight/axis-aligned everywhere except
   genuine corners, which stay sharp. (Exactly our corner-split strategy.)
6. **Symmetry / uniformity:** anchor points "completely cover each letter" in a symmetric
   way. The box keeps the shape "uniform and scalable." → favor consistent, box-aligned
   anchor layouts across letters (feeds Material keylines reference).
7. **Trace over the source at low opacity (~10%)** so boundaries are crisp before plotting.
   → analogy for us: work from clean per-color masks, not the noisy raster directly.

## Two construction modes mentioned
- **Full Bézier (box method):** plot extrema anchors, pull axis-aligned handles to match
  boundary. More control; needed for non-circular/custom shapes.
- **Primitive shortcut:** for round letters, drop a **circle/ellipse primitive** and
  resize it to fit. Same result for simple shapes, fewer points.
  → This validates guide **Phase 10 (native primitive recognition)**: if a shape is really
  a circle/ellipse/rect, emit the primitive instead of a fitted path.

## (History, for context only — not implementation)
- Bézier curves rest on **Bernstein polynomials**; popularized by **Pierre Bézier**
  (Renault, 1960s). **Paul de Casteljau** (Citroën, 1959) developed them a year earlier but
  Citroën blocked publication. Cubic vs quadratic are the two common types; logotypes
  typically use **cubic**.

---

## Distilled mapping → Rust logo pipeline
| Box-Method step | Our implementation |
|---|---|
| Tight bbox per shape | compute bbox from extrema (Pomax `#boundingbox`) |
| Anchors at box-touch points | place nodes at x/y extrema (`B'(t)=0`) |
| Axis-aligned handles at extrema | constrain adjacent control points to H/V tangents |
| Corners kept sharp | corner-split before fitting; emit `L`/`H`/`V` |
| Handles never cross | validate fits against magic-triangle constraint |
| Circle shortcut | primitive recognition (Phase 10) for true round shapes |
| Symmetric, uniform anchors | shared keyline snapping (Material reference) |
