# Logo Tracer — PNG → clean SVG (in-browser)

Convert a PNG logo into a clean, editable SVG entirely in the browser via WebAssembly.
No backend, no upload — the image never leaves the user's device.

The tracer emits straight edges as lines, keeps corners sharp, fits curves from real
circular-arc geometry (the "circle technique"), and collapses anti-aliased colors to a
small dominant palette — producing compact, designer-quality SVGs from raster logos.

[![CI](https://github.com/dangijaswant/logo-tracer/actions/workflows/ci.yml/badge.svg)](https://github.com/dangijaswant/logo-tracer/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Live demo
**https://logo-tracer.pages.dev** (Cloudflare Pages)

## Quick start (local)

The prebuilt engine is committed at `web/pkg/`, so no build is required:

```bash
cd web
python -m http.server 8088     # or: npx serve .
# open http://localhost:8088/
```

Drop a PNG in, tune the options, download the SVG. All client-side.

## Repo layout

```
web/                  deployable static app (what Cloudflare Pages serves)
├── index.html / app.js / styles.css
├── README.md         web/deploy notes
└── pkg/              prebuilt wasm engine (committed so it deploys without a build)
    ├── logo_wasm.js
    └── logo_wasm_bg.wasm

logo-wasm/            the Rust source that compiles to the wasm
├── src/lib.rs        wasm-bindgen entry: trace_logo(rgba, w, h, opts) -> svg
└── src/logo/         the logo pipeline: mod.rs (geometry), fit.rs (bézier), arc.rs (circle)

tools/svg-metrics/    structural metrics tool for tracing output (has its own test suite)
docs/refs/            design references the tracer is built on
.claude/skills/       logo-tracer skill (specs + quality gate)
```

## How it works
1. **JS** decodes the PNG to raw RGBA via an offscreen `<canvas>` (no image codec in wasm).
2. **wasm** (`logo-wasm`) runs the pipeline: color clustering → small-palette snap →
   contour corner-splitting → line / circular-arc / Bézier fitting → SVG serialization.
3. The SVG is previewed and downloadable, all client-side.

## Options (all have sensible defaults matching the CLI `--preset logo`)

| Option | Default | What it does |
|---|---|---|
| `max_colors` | 4 | cap on the dominant palette size |
| `palette_merge_threshold` | 90·90·3 | squared-RGB cutoff for merging near-duplicate palette entries |
| `filter_speckle` | 16 | drop subpaths whose area is below this (residual specks) |
| `color_precision` | 4 | significant color bits (1..=8) used in clustering |
| `corner_threshold_deg` | 40 | turn angle above which a vertex is treated as a sharp corner |
| `precision` | 2 | decimal places in the emitted SVG coordinates |

## Testing

```bash
npm test
# or, piece by piece:
cd logo-wasm && cargo test          # geometry + public-API contract tests
node --test tools/svg-metrics/test/ # svg-metrics parser tests
```

The Rust suite pins the path-shape behavior (lines vs. cubics, corner handling,
speck dropping) and the public `trace_logo` API contract. See [CONTRIBUTING.md](CONTRIBUTING.md).

## Rebuilding the wasm
The `logo-wasm` crate depends on the `visioncortex` tracing engine (from crates.io).

```bash
cd logo-wasm
wasm-pack build --target web --release
cp pkg/logo_wasm.js pkg/logo_wasm_bg.wasm ../web/pkg/
```

Requirements: Rust with the `wasm32-unknown-unknown` target, and `wasm-pack`.
Toolchain notes (Windows 11 + MSVC + wasm-pack) are in `docs/LOCAL_SETUP.md`.

## Deploy (Cloudflare Pages)
```bash
npx wrangler pages deploy web --project-name logo-tracer
```

## Contributing
See [CONTRIBUTING.md](CONTRIBUTING.md). Good first issues: label the repo with
`good first issue` when you add one. Code of conduct: [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## License
[MIT](LICENSE). Built on the `visioncortex`/VTracer tracing engine (MIT OR Apache-2.0).
