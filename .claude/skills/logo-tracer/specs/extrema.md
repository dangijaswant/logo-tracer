# Spec: Extrema & inflection node placement (STAGE 3)

**Refs:** drawing-good-paths.md (§Extremum points — KEY), bezier-vectorize-logo.md (Box
Method: anchors at bbox-touch points), bezier-primer.md (`#extremities`, `#inflections`).
**Guide phase:** 7 (analysis), 8 (fitting input). This is the heart of "human-quality".

## Goal
On each smooth range, put on-curve nodes at **x/y extrema** (horizontal/vertical tangents)
and split at **inflections** so every emitted segment is single-orientation. Then delete
intermediate points — extrema + corners are usually enough.

## Extrema (per cubic segment), from Pomax
Derivative of a cubic is a quadratic; solve `B'(t)=0` per axis with the quadratic formula:
```
// cubic weights p0,p1,p2,p3 (one axis at a time)
a = 3(p1-p0); b = 3(p2-p1); c = 3(p3-p2)          // 1st-derivative control points
A = a - 2b + c; B = 2(b - a); C = a               // quadratic At^2+Bt+C
roots = solve_quadratic(A,B,C) ∩ (0,1)            // guard A~0 -> linear -> -C/B
```
Do this for x and y; each root t is an extremum → candidate node (Box-Method "box-touch").
**Skip an extremum that would create a shallow curve** (depth < few units) — Glyphs pro tip.

## Inflections, from Pomax (`#inflections`)
Inflection = curvature `C(t)=0`, where `C(t)` is the cross product of 1st and 2nd
derivatives. Easier after **axis-aligning** the curve (translate start→origin, rotate end
onto x-axis). Split the range at each inflection so no single segment inflects.

## Handle orientation (Box Method + Glyphs)
At an extremum node the tangent is axis-aligned, so the two adjacent control points must be
**horizontal** (top/bottom extrema) or **vertical** (left/right extrema). Enforce this when
emitting handles — it's what makes curves look intentional.

## Pseudocode
```
fn place_nodes(range):
    nodes = [range.start, range.end]
    for seg in range.cubics():
        for t in extrema_t(seg.x) ∪ extrema_t(seg.y):
            if curve_depth_at(seg,t) >= shallow_min: nodes.push(seg.point(t))
        for t in inflection_t(seg): nodes.push(seg.point(t))  // also a split point
    nodes = dedup_sort(nodes)
    reduce(nodes)   // delete any node whose removal keeps fit error <= tol (Glyphs idea)
    return nodes
```

## Gate
Fewer, well-placed nodes; cubic count drops sharply; remaining cubics sit between extrema
with axis-aligned handles. `quality_gate.js` cubic count should approach the Python target.
