# Analysis — how the `D` was constructed from circles (3-step reference)

Reference SVGs: `step-1.svg`, `step-2.svg`, `step-3.svg`. This documents the construction
so the tracer can target the same clean result.

## Step 1 — two equal circles, horizontally offset
```
circle A: cx=355, cy=355, r=355   (outer)
circle B: cx=480, cy=355, r=355   (same radius, shifted +125 right)
```
Two **equal-radius** circles, same `cy`, offset by 125px. Their overlap region defines the
`D`'s bowl. The horizontal offset = how "fat" the D is.

## Step 2 — boolean subtraction
The circles are subtracted (evenodd fill rule). The two circles **intersect** at exactly:
```
(417.5, 5.484)  and  (417.5, 704.516)
```
So the vertical seam of the D is `x = 417.5`. The shape is the lens/region bounded by the
two circular arcs between those intersection points.

## Step 3 — the final `D` glyph (THE TARGET)
The whole letter is **ONE path = 6 cubic arcs**:
```
M62.5 710
C258.561 710 417.5 551.061 417.5 355      # OUTER bowl, bottom half  (r = 355)
C417.5 158.939 258.561 0 62.5 0           # OUTER bowl, top half      (r = 355)
C41.18 0 20.29 1.88 0 5.484               # top-left cap (stem joint)
C166.255 35.0134 292.5 180.264 292.5 355  # INNER counter, top half   (r = 292.5)
C292.5 529.736 166.255 674.987 0 704.516  # INNER counter, bottom half (r = 292.5)
C20.29 708.12 41.18 710 62.5 710 Z        # bottom-left cap (stem joint)
```

## The key facts for the tracer
1. **Outer bowl = ONE circle, radius 355.** Drawn as a **semicircle = 2 cubics** (top + bottom).
2. **Inner counter = ONE circle, radius 292.5** (= 355 − 62.5 stem width). Also **2 cubics**.
3. **2 tiny cubics** are the joints where the curves meet the flat left stem.
4. Total = **6 cubics**, **2 distinct radii** (outer 355, inner 292.5), concentric in y.
5. Each arc spans ~180°, split into 2 cubics at the 90° extrema (left/top/right/bottom) —
   exactly the `k = 4/3·tan(θ/4)` arc-to-cubic rule.

## Gap vs current tracer output
- Reference `D`: **6 cubics**, 2 radii.
- Current tracer `D` (`dyez-new-logo.svg` path[5]): **9 cubics**.
- The tracer is **over-segmenting the bowl** — fitting it as several smaller arcs instead of
  recognizing the whole bowl as ONE semicircle of a single radius.

## What this implies for the implementation
The arc fitter already does circle-fit + arc-to-cubic per span. To match the reference:
1. **Fit the largest possible arc per smooth span** — don't let a long bowl get split into
   multiple small-radius arcs. One contiguous near-constant-radius run = one arc.
2. **Snap/merge radii** so the outer bowl is one radius and the inner counter another
   (the radius-snapping step from the circle-technique ref).
3. Each arc → 2 cubics max per semicircle (already correct).

Target: a `D`/`O`/`C`/`G` bowl emits ~2 cubics per side, ~6 total per `D`, not 9–12.
