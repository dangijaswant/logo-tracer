# Logo Tracer — PNG → clean SVG (in-browser)

Convert a PNG logo into a clean, editable SVG entirely in the browser via WebAssembly.
No backend, no upload — the image never leaves the user's device.

The tracer emits straight edges as lines, keeps corners sharp, fits curves from real
circular-arc geometry (the "circle technique"), and collapses anti-aliased colors to a
small dominant palette — producing compact, designer-quality SVGs from raster logos.

## Live demo
**https://logo-tracer.pages.dev** (Cloudflare Pages). To run locally:

```bash
cd web
python -m http.server 8088     # or: npx serve .
# open http://localhost:8088/
```

## Repo layout
```
web/                     # the deployable static app (this is what Cloudflare serves)
├── index.html
├── app.js               # decodes PNG via <canvas>, calls the wasm tracer
├── README.md            # web/deploy notes
└── pkg/                 # prebuilt wasm engine (committed so it deploys without a build)
    ├── logo_wasm.js
    └── logo_wasm_bg.wasm

logo-wasm/               # the Rust source that compiles to the wasm
└── src/
    ├── lib.rs           # wasm-bindgen entry: trace_logo(rgba, w, h, opts) -> svg
    └── logo/            # the logo pipeline: mod.rs, fit.rs, arc.rs

docs/refs/               # design references the tracer is built on
.claude/skills/          # logo-tracer skill (specs + quality gate)
```

## How it works
1. **JS** decodes the PNG to raw RGBA via an offscreen `<canvas>` (no image codec in wasm).
2. **wasm** (`logo-wasm`) runs the pipeline: color clustering → small-palette snap →
   contour corner-splitting → line / circular-arc / Bézier fitting → SVG serialization.
3. The SVG is previewed and downloadable, all client-side.

## Rebuilding the wasm
The `logo-wasm` crate depends on the `visioncortex` tracing engine. With that available:

```bash
cd logo-wasm
wasm-pack build --target web --release
cp pkg/logo_wasm.js pkg/logo_wasm_bg.wasm ../web/pkg/
```

## Deploy (Cloudflare Pages)
```bash
npx wrangler pages deploy web --project-name logo-tracer
```

## License
Built on visioncortex/VTracer (MIT/Apache-2.0). Tracing pipeline and web app © the author.
