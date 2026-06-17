# Drawing Good Paths — Glyphs.app (human-quality outline spec)

**Source:** https://glyphsapp.com/learn/drawing-good-paths
**Author:** Rainer Erich Scheichelbauer (mekkablue), Glyphs.app
**Role in this project:** the *human-quality specification* for outlines. These are
the rules a type designer follows so a glyph outline is clean, minimal, and renders
well. We use them as acceptance rules for the Rust logo-tracer's node placement and
curve emission.

---

## 1. The magic triangle (curve segment quality)
- A **segment** = everything between two adjacent **on-curve points (nodes)**.
- Two segment types: **straight lines** (just the two nodes) and **curves** (two nodes
  + **two** off-curve Bézier control points / "handles" / BCPs).
- **Every curve segment must fit inside a triangle** formed by its two end nodes and the
  point where the *elongations of the two handles* intersect.
- **Handles must never extend outside that triangle.** A handle must not cross the other
  handle, nor cross the elongation of the other handle. Violations cause unnecessary
  inflections, cusps, or self-intersections — and trouble for rasterizers.
- A curve segment **always has two BCPs, never one.** (Illustrator hides the second BCP
  inside a node; bad — no two points should share coordinates, and you lose control.)

**→ Implementation rule for our fitter:** when fitting a cubic to a smooth contour range,
keep both control points inside the magic triangle (handles on the same side, not
crossing). Reject/clamp fits whose handles cross — they indicate the node split is wrong.

## 2. Inflections
- The magic triangle implies a curve segment should have a **single orientation**
  (not switch CW/CCW) — i.e., avoid inflections **within one segment**.
- **Prefer NO inflection** for S/tilde-like shapes when both handles are orthogonal
  (vertical/horizontal). Fewer points = fewer problems.
- If a flowing S truly needs it, split at the **inflection point** (add an on-curve node
  there) rather than letting one segment inflect.
- Avoid inflections where possible — they cause interpolation "kinks".

**→ Implementation rule:** detect inflection points (sign change of curvature) and **split
the contour there** into separate single-orientation segments, the same way we split at
corners. One segment = one orientation.

## 3. Path orientation and order
- **Outer paths counter-clockwise; counters (holes) clockwise.** (This is the winding
  convention Glyphs uses; SVG uses `fill-rule` instead, but orientation still matters for
  even-odd/nonzero correctness.)
- "Correct Path Direction" also **re-orders paths and resets the start point** of each
  closed path → useful for *deterministic, stable output* (snapshot tests).

**→ Implementation rule:** enforce consistent winding (outer vs hole) and a deterministic
start-point rule so serialization is stable/snapshot-testable (matches guide Phase 11).

## 4. Self-intersection
- An exported path **must not intersect itself**. Remove overlaps before final output.
- Watch for **double overlaps** (an overlap inside an overlap) → shows as tiny white gaps,
  often where a curve segment meets a straight line and bends slightly the wrong way.

**→ Implementation rule:** after fitting, validate no self-intersection; watch the
curve→line join (the exact case our corner-split + line emission must get right).

## 5. Superfluous points & wrong node types ("Tidy Up Paths")
- Remove **superfluous nodes and BCPs** — anything not needed to display the shape
  (e.g., handles on a straight segment).
- Fix **node type**: corner (sharp) vs smooth (curve) point where appropriate.
- Remove **zero-length segments** (two points at identical coordinates).

**→ Implementation rule (directly maps to our metrics):** emit straight runs as `L`/`H`/`V`
with NO control points; classify each node corner-vs-smooth; drop zero-length / tiny
segments. (Our svg-metrics already counts `tinySegmentCount` and near-axis cubics.)

## 6. Extremum points (KEY rule for our node placement)
- **Insert nodes at the x and y extremes** of paths — the spots with completely horizontal
  or vertical tangents.
- Doing so **often lets you delete intermediate points** → simpler paths, fewer points,
  smaller files. (Delete one node at a time; the tool reconstructs the segment.)
- Extremum points are also required for stem hinting.
- **Pro tip — avoid an extremum that would create a "shallow curve"** (only a few units
  deep): integer coordinates can't represent it well and it messes up the shape.

**→ Implementation rule (most important takeaway):** our node-placement strategy = put
on-curve nodes at **corners + curve extrema (H/V tangents) + inflections**, and nowhere
else. This is the core of "human-quality" output and replaces the prototype's uniform
Catmull-Rom node spacing. Skip an extremum if the resulting curve would be shallower than
a small threshold.

## 7. Open vs closed
- **All paths must be closed.** Open paths are ignored at export.

**→ Implementation rule:** guarantee closed contours with exactly matching start/end
endpoints (guide Phase 8: "preserve closed-path topology; ensure endpoints match exactly").

## 8. Vectors out of bounds
- Stray off-canvas debris produces absurd sidebearings — delete it.

**→ Implementation rule:** discard tiny/out-of-bounds connected components (guide Phase
6.3: "discard components below minimum area").

---

## Distilled checklist for the Rust logo fitter
1. Nodes go ONLY at: **corners**, **curve extrema (H/V tangents)**, **inflections**.
2. Straight runs → `L`/`H`/`V`, no handles.
3. Each curve segment: single orientation, both handles inside the magic triangle.
4. No zero-length / shallow / tiny segments.
5. Closed contours, exact matching endpoints, consistent winding, deterministic start point.
6. No self-intersection; verify the curve→line joins at corners.
7. Discard sub-minimum-area debris.
