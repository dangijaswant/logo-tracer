# The Bézier Game — bezier.method.ac (pen-tool discipline / minimal nodes)

**Source:** https://bezier.method.ac/
**Title:** "The Bézier Game — A game to help you master the pen tool"
**Role in this project:** the *interactive trainer* that drills the single most important
habit for clean outlines: **use the fewest possible nodes, placed at extrema, with
axis-aligned handles.** It's the hands-on, gamified version of the Glyphs + Box-Method
rules.

> Capture note: the game is **canvas/WebGL-rendered**; its per-level instructions are drawn
> on the canvas, not in the DOM, so they can't be text-scraped. The DOM only exposes the
> intro/keyboard-confirmation. The one scrapable lesson string captures the whole ethos:
> **"The ideal solution has 0 nodes. Think you can do better? Try again."**
> (i.e. on a straight segment, an extra node is *penalized* — minimum nodes wins.)

---

## What the game teaches (well-documented mechanics)
The game scores you on each shape by **how few nodes you use** and how well the curve
matches the target. The lessons, in order of difficulty, drill:

1. **Straight lines need no curve nodes / no handles.** Adding nodes to a straight run is
   marked as non-ideal. → straight runs = `L`/`H`/`V`, zero control points.
2. **Place nodes at extrema** (top/bottom/left/right of every curve) — the box-touch points.
   Anywhere else and you need more nodes to compensate. → nodes at x/y extrema only.
3. **One node per corner**, kept sharp. → corner-split, no smoothing across a corner.
4. **Axis-align handles at extrema** (horizontal at top/bottom, vertical at left/right).
   → constrain off-curve control points to H/V tangents at extremum nodes.
5. **Don't over-node curves.** A smooth arc usually needs just its two extremum endpoints
   and balanced handles — not a chain of nodes. → minimize segments per smooth range.
6. **Closed shapes** with clean joins. → closed contours, exact endpoints.

These are the same principles as `drawing-good-paths.md` (Glyphs) and
`bezier-vectorize-logo.md` (Box Method), expressed as a pass/fail skill drill. The value
here is the **scoring philosophy**: *fewer nodes + extrema placement = correct*, which is
exactly the objective function our Rust fitter should optimize.

---

## Direct relevance to our metrics (the game IS our objective)
Our `svg-metrics` tool already measures the things the game scores:
- `cubicCount` / segment counts → "fewest nodes" (lower is better, to a point).
- `nearHorizontalCubicCount` / `nearVerticalCubicCount` → straight runs wrongly drawn as
  curves (the game penalizes exactly this).
- `tinySegmentCount` → superfluous/zero-length nodes (penalized).
- `lineCount` (L/H/V) → straight runs done right (rewarded).

So the logo-tracer's success target == "would this pass The Bézier Game with a near-minimal
node count?"

---

## Distilled rules → Rust logo pipeline
1. **Minimum nodes:** after fitting, run a reduction pass that removes any node whose
   removal keeps fit error within tolerance (Glyphs "delete one node at a time" idea).
2. **Extrema-only node placement** for curves; **one node per corner**; **none on straights**.
3. **Axis-aligned handles** at extrema.
4. Treat node count + fit error as the **objective to minimize** (mirrors the game's score),
   bounded by the magic-triangle / no-self-intersection constraints.
