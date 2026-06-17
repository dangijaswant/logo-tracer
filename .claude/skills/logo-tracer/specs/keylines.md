# Spec: Alpha normalization (STAGE 1) + keyline grid note

**Refs:** material-icon-design.md (live/trim area, on-pixel), optical-corrections.md.
**Guide phase:** 6.1. (Keyline *snapping* lives in optical.md; this file covers the STAGE 1
alpha cleanup + the grid concept.)

## Alpha normalization (STAGE 1, before clustering)
Goal: no white/black halos around transparent PNG edges; ignore fully-transparent RGB
garbage; handle semi-transparent anti-aliasing correctly.

Rules:
1. Pixels below `alpha_threshold` → fully transparent.
2. Fully transparent pixels **do not participate** in color clustering.
3. Semi-transparent pixels are associated with the **nearest dominant opaque color**
   (not blended toward background → prevents halos).
4. Preserve geometric edge position as much as possible.

Engine mapping: the existing `should_key_image`/`find_unused_color_in_image`/`key_color`
machinery handles full transparency. For logo mode, add an `alpha_threshold` pre-pass and an
`alpha_edge_snap` that reassigns semi-transparent edge pixels to the nearest dominant color
(works with `specs/palette-masks.md`).

```rust
pub alpha_threshold: u8,
pub alpha_edge_snap: bool,
```

> NOTE for the DYEZ fixture: it is opaque (Format24bppRgb, no alpha), so STAGE 1 is nearly a
> no-op for it. The big wins for DYEZ are STAGE 2 (palette) and STAGE 3 (lines/corners).
> Use a transparent fixture (`transparent-icon.png`) to exercise this spec.

## Keyline grid (concept; snapping is in optical.md)
The "live area / trim area / on-pixel" idea from Material: keep artwork within bounds, place
geometry on consistent guides. We derive a per-logo keyline set (shared stem x's, baseline/
cap y's) and snap near-misses — tolerance-bounded — to make stems and heights consistent.

## Gate
`quality_gate.js`: transparent stays transparent; no halo fragments in `fillColors`; edges
land on consistent coordinates.
