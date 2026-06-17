# Logo Tracer — Web (PNG → SVG, in-browser)

A static web app that converts a PNG logo into a clean, editable SVG entirely in the
browser via WebAssembly. No backend, no upload — the image never leaves the user's device.

## What's in this folder (the deployable bundle)
```
web/
├── index.html          # the UI
├── app.js              # decodes PNG via canvas, calls the wasm tracer
└── pkg/
    ├── logo_wasm.js        # wasm-bindgen JS glue (ES module)
    ├── logo_wasm_bg.wasm   # the tracer engine (~186 KB)
    └── logo_wasm.d.ts      # TypeScript types (optional)
```
These files are **all you deploy**. Everything runs client-side.

## Run locally
WASM must be served over HTTP (not `file://`). Any static server works, e.g.:
```powershell
# Python
python -m http.server 8088
# or Node (npx)
npx serve .
```
Then open http://localhost:8088/ . The server must send `.wasm` with
`Content-Type: application/wasm` (Python/serve do this automatically).

## Deploy to your site (static hosting)
Upload the **entire `web/` folder** to any static host:

- **Netlify / Vercel / Cloudflare Pages:** drag-drop the `web/` folder, or point the
  project at it. No build step needed.
- **GitHub Pages:** commit `web/` and enable Pages on that folder.
- **Your existing server (Apache/Nginx/IIS):** copy `web/` into the web root. Ensure the
  server serves `.wasm` as `application/wasm` (modern servers do; for old Nginx add
  `types { application/wasm wasm; }`).

### Embedding into an existing site
- Host the `pkg/` folder somewhere on your site and load `app.js` as a `<script type="module">`.
- Or iframe the standalone page: `<iframe src="/logo-tracer/index.html">`.

## Rebuilding the wasm (after changing the Rust tracer)
From `vtracer/logo-wasm/`:
```powershell
wasm-pack build --target web --release
# then copy the new pkg into the site:
Copy-Item .\pkg\logo_wasm.js,.\pkg\logo_wasm_bg.wasm,.\pkg\logo_wasm.d.ts ..\..\web\pkg\ -Force
```

## API (if you want to call the tracer yourself)
```js
import init, { trace_logo, LogoOptions } from './pkg/logo_wasm.js';
await init();
const opts = new LogoOptions();          // sensible logo defaults
opts.max_colors = 4;
opts.color_precision = 4;                // 1..8
opts.filter_speckle = 16;                // drop blobs smaller than NxN px
opts.corner_threshold_deg = 40;          // lower = more corners kept sharp
// rgba: Uint8Array of length width*height*4 (from canvas getImageData)
const svgString = trace_logo(rgba, width, height, opts);
```

## Notes
- PNG/JPEG/WebP decoding is done by the browser's `<canvas>` (the wasm has **no image
  codec** — keeps it small and portable). Transparency is preserved.
- The tracing result matches the native CLI (`vtracer --preset logo`) exactly.
- Tailwind is loaded from CDN for convenience. For production, replace the CDN `<script>`
  with a compiled Tailwind CSS file (the CDN prints a console warning).
