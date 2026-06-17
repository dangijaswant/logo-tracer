# Pipeline Map — where each rule plugs into the real trace

Code reference: `vtracer/cmdapp/src/converter.rs :: color_image_to_svg`
(the webapp mirrors this in `webapp/src/conversion/color_image.rs` as a tick state machine).

## The actual stages (color mode)

```
read_image -> ColorImage (RGBA)
   │
   ▼  STAGE 1: KEYING
should_key_image(img) ? find_unused_color_in_image -> set transparent px to key_color
   │        (logo rule: alpha-edge cleanup, semi-transparent -> nearest opaque color)
   ▼  STAGE 2: CLUSTERING
Runner::new(RunnerConfig{ good_min_area = filter_speckle_area,
                          is_same_color_a = color_precision_loss,
                          deepen_diff = layer_difference, key_color, ... }).run()
   │        (logo rule: small dominant palette; good_min_area drops debris)
   ▼  STAGE 3: CONTOUR -> PATH   ◀── most logo work happens here
for cluster in view.clusters_output.rev():
    paths = cluster.to_compound_path(
              &view, false, config.mode,        // mode = Spline | Polygon | None
              config.corner_threshold,          // radians (deg2rad in config)
              config.length_threshold,
              config.max_iterations,
              config.splice_threshold)
   │        (logo rules: corner-split -> extrema nodes -> smooth ranges -> fit;
   │         emit lines on straight runs; magic-triangle handle constraint)
   ▼  STAGE 4: SERIALIZE
svg.add_path(paths, cluster.residue_color())    // SvgFile, path_precision rounding
   │        (logo rules: emit L/H/V; primitive recognition; optical pass)
   ▼
write_svg
```

## Insertion strategy (port the Python prototype natively)

The Python prototype (`test-output/metrics/_clean_svg.py`) runs as a **post-process on the
finished SVG**. Porting to Rust means moving each Python step to its native stage:

| Python step | Native Rust home | Notes |
|---|---|---|
| keep dark glyphs + bg, drop tiny subpaths (`L<8`) | STAGE 2 (`good_min_area`) + STAGE 3 entry | use cluster area + subpath length |
| `flatten` path -> polyline | STAGE 3, inside/around `to_compound_path` | visioncortex already has the pixel contour (`PathI32`); flatten only if starting from splines |
| `rdp` simplify | STAGE 3 | a `path_analysis::simplify` step before corner detect |
| `corner_flags` (turn angle) | STAGE 3 | `corner_threshold` already exists; see `specs/corners.md` |
| `axis_snap` near H/V | STAGE 3 (geometry) + STAGE 4 (emit) | snap then emit `H`/`V`; see `specs/lines.md` |
| `emit_subpath` (L at corners, cubic on smooth) | STAGE 3 fit + STAGE 4 serialize | see `specs/fitting.md`, `specs/lines.md` |
| 2-color merged path | STAGE 2 palette + STAGE 4 | see `specs/palette-masks.md` |

### Two viable implementation routes (pick per effort/risk)
- **Route A — engine-native (preferred long term):** add a `visioncortex/src/path_analysis/`
  module and a logo path in `to_compound_path` (or a sibling `to_logo_path`). Highest
  quality, runs in WASM, behind the `logo` preset. Matches guide Phases 6-11.
- **Route B — post-process bridge (fast, lower risk):** keep tracing as-is, add a Rust
  post-process that re-analyzes the emitted compound paths (flatten→RDP→corner→snap→fit) —
  a direct port of `_clean_svg.py` into the cmdapp/webapp, gated by the preset. Easier to
  land incrementally; promote pieces into the engine later.

Either way: **the quality gate (`scripts/quality_gate.js`) is the same**, so versions are
comparable.

## Config knobs already present (reuse, don't reinvent)
From `cmdapp/src/config.rs`:
`color_mode, hierarchical, filter_speckle, color_precision, layer_difference, mode,
corner_threshold, length_threshold, max_iterations, splice_threshold, path_precision`.
New logo-only fields (alpha_threshold, max_colors, line_fit_tolerance, axis_snap_angle_deg,
bezier_fit_tolerance, etc.) should follow the same struct style — see the guide's Phases 6-8.
