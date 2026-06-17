# Spec: Native primitive recognition (STAGE 4)

**Refs:** bezier-vectorize-logo.md (circle shortcut), material-icon-design.md (keyline
shapes: circle/rect), bezier-primer.md (`#circles`, `#boundingbox`).
**Guide phase:** 10. **Rule:** only replace a path when the primitive is equal-or-better error.

## Goal
After fitting, try to replace a generic path with a native SVG element
(`<rect>/<circle>/<ellipse>/<rounded rect>/<polygon>`) when the geometry really is that shape.

## Recognizers (small, independent)
- **Rectangle:** 4 dominant straight sides; opposite sides parallel; adjacent ~perpendicular;
  low fit error.
- **Rounded rect:** 4 straight sides + 4 corner arcs with **consistent radii** (unify per
  material-icon-design.md).
- **Circle:** center+radius fit with low radial variance.
- **Ellipse:** stable center, axes, rotation; low geometric error.
- **Polygon:** all sides straight (emit `<polygon>` / line-only path).

## Acceptance
Replace ONLY if the primitive's max geometric error ≤ the fitted path's error (or within a
small slack). Otherwise keep the path. **Do not force intentional custom curves into
primitives** (guide: don't turn the blue panels' custom right edges into `<rect>`).

## Engine mapping
Runs on the per-cluster geometry just before `svg.add_path`. Emit into the typed SVG model
(see optical.md / guide Phase 11 `SvgGeometry` enum) rather than always a raw path.

## Gate
`quality_gate.js`: `elements` shows `rect/circle/...` where appropriate; visual error not
worse than baseline; SVG still valid in Chrome/Firefox/Inkscape.
