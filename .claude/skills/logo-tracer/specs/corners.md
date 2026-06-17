# Spec: Corner detection & splitting (STAGE 3)

**Refs:** drawing-good-paths.md (§corner vs smooth), bezier-vectorize-logo.md (corners are
the exception), bezier-method-ac.md (one node per corner).
**Guide phase:** 7.1. **Hard rule:** never fit a corner-containing contour as one smooth curve.

## Goal
Find genuine corners on a (denoised, simplified) contour and split the contour into ranges
at them. Smooth ranges go to the fitter; corners become exact `L`/`H`/`V` joins.

## Signals (use more than one — don't trust a single noisy point)
1. **Tangent-angle change** at vertex i: angle between incoming `v1 = p[i]-p[i-1]` and
   outgoing `v2 = p[i+1]-p[i]`. Corner if turn > `corner_threshold`.
2. **Support length** on both sides: require a minimum run of near-collinear points each
   side (`corner_support_radius`) so a single jaggy pixel isn't a corner.
3. **Curvature**: local curvature spike (see bezier-primer.md curvature).
4. **Removal error**: if deleting the vertex barely changes the fit, it wasn't a corner.

## Pseudocode (matches prototype `corner_flags`, hardened)
```
fn corner_flags(pts, thresh_deg, support):
    n = len(pts); flags = [false; n]
    for i in 0..n:
        v1 = pts[i] - pts[(i-1+n)%n]
        v2 = pts[(i+1)%n] - pts[i]
        if |v1|<eps or |v2|<eps { flags[i]=true; continue }   // cusp/zero-length
        turn = acos(clamp(dot(v1,v2)/(|v1|*|v2|), -1, 1)).deg()
        if turn > thresh_deg
           and run_collinear_before(pts,i,support)
           and run_collinear_after(pts,i,support):
            flags[i] = true
    enforce_min_separation(flags, corner_min_separation)   // no two corners too close
    return flags
```

## Config
```rust
pub corner_angle_threshold_deg: f64, // reuse cmdapp corner_threshold (deg2rad'd)
pub corner_support_radius: usize,
pub corner_min_separation: f64,
```

## In the engine
visioncortex already takes `corner_threshold` into `to_compound_path`. For logo mode, run
the multi-signal detector on the simplified polyline BEFORE spline fitting, then split into
ranges and pass only smooth ranges to the fitter (`specs/fitting.md`).

## Gate
After this stage, `quality_gate.js` should show letter corners preserved (no rounding) and
NO cubic spanning a corner. Visually: sharp at 10x zoom.
