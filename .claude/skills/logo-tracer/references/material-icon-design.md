# Designing Icons — Material Design 3 (keylines & geometric consistency)

**Source:** https://m3.material.io/styles/icons/designing-icons
**Role in this project:** the *geometric-consistency* reference. Material's keyline grid and
metric rules tell us how to make multiple shapes in one logo/icon read as a **single
consistent system** — shared radii, aligned proportions, on-pixel placement. Used in our
port for snapping shapes to a shared sub-pixel grid and unifying corner radii / stroke
widths across letters.

---

## Design principles
- **Simplify for clarity and legibility.** Don't be overly literal; avoid complex icons.
- **Don't use delicate or loose organic shapes.** (Crisp geometry over wobble — exactly
  the wavy-edge problem we're fixing.)
- **Maintain a consistent visual style across the whole set/one icon.**

## Sizes & layout (live area / trim area)
- Standard icon = **24dp × 24dp**; design at 100% for pixel accuracy. Other optical sizes:
  20, 40, 48dp.
- **Live area** = 20×20dp where content should stay; **2dp padding** around the perimeter;
  **trim area** = full 24dp. Content may extend into padding for visual weight but **never
  past the trim area.**
- → Analogy for us: keep traced geometry within a defined safe region; don't let
  anti-alias debris push past the artwork bounds (component-area filtering, Phase 6.3).

## Grid and keyline shapes (the core takeaway)
- **Keyline shapes are the foundation of the grid.** Core shapes (circle, vertical
  rectangle 16×20dp, horizontal rectangle 20×16dp, square) act as **guidelines so all icons
  share consistent proportions.**
- **Position icons "on pixel."** Don't place an icon on a coordinate that isn't on-pixel.
- → Implementation: snap shape extrema/edges to a **shared keyline grid** (a small set of
  consistent x/y guide lines) and to whole/half-pixel coordinates. This unifies stem
  positions and widths across letters so the logo reads as one system, not N independently
  traced glyphs. Pairs with axis-snapping (guide Phase 7.3).

## Icon metrics
### Corners
- **Default corner radius = 2dp.**
- **Outlined style: interior corners are square (not rounded).**
- **For shapes ≤2dp wide, stroke corners shouldn't be rounded.**
- **Rounded style:** both exterior and interior radii rounded. **Sharp style:** both reduce
  from 2dp to 0dp.
- **Overly round corners reduce legibility.**
- → Implementation: **unify near-equal corner radii** to a shared value (consistency), and
  keep small/thin features' corners sharp. Don't over-round. (Feeds rounded-rect primitive
  recognition, Phase 10, with a consistent radius.)

### Weight and stroke
- Recommended stroke weight **2dp / regular (400)**; range thin(100)–bold(700).
- **Use consistent stroke weights**; **squared stroke terminals** (not rounded).
- → Implementation: **unify near-equal stem widths** across letters; keep terminals
  square/sharp unless the source is genuinely rounded.

### Complex shapes & optical corrections (geometry-preserving)
- When detail is needed, make **subtle optical corrections** — but they **must use the same
  geometric forms** all other icons are based on, **without skewing or distorting** them.
- Example: paperclip uses 1.5dp of the 2dp stroke area to fit multiple curves in 24dp.
- **Don't tilt, rotate, or make icons appear dimensional.**
- → Implementation: any optical tweak (see optical-corrections ref) stays within the
  established geometry; never shear/rotate to "fix" a shape.

---

## Distilled rules → Rust logo pipeline
1. **Shared keyline grid:** derive a small set of consistent guide coordinates per logo;
   snap shape edges/extrema to them.
2. **On-pixel placement:** round node coordinates to whole/half pixels (configurable
   precision; guide Phase 11).
3. **Unify corner radii:** cluster near-equal radii → one canonical radius per style.
4. **Unify stroke/stem widths:** cluster near-equal stem widths → consistent value.
5. **Keep terminals/interior corners square** unless source is truly rounded; don't
   over-round.
6. **No skew/rotation** as a correction; preserve the base geometry.
7. **Filter content to the artwork bounds**; drop sub-area debris.
