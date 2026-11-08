# Contributing

Thanks for taking a look. This is a browser tool: it traces a PNG logo into a clean,
compact SVG entirely client-side via a Rust/WASM engine. The tracer is built on the
`visioncortex` tracing engine; the logo-specific geometry (corner splitting, axis snapping,
circular-arc + Bézier fitting, small-palette snap) lives in `logo-wasm/src/logo/`.

## Repo layout

```
web/                  deployable static app (this is what Cloudflare Pages serves)
├── index.html / app.js / styles.css
├── pkg/              prebuilt wasm engine (committed so it deploys without a build)
logo-wasm/            the Rust source that compiles to the wasm
├── src/lib.rs        wasm-bindgen entry: trace_logo(rgba, w, h, opts) -> svg
└── src/logo/         the logo pipeline: mod.rs (geometry), fit.rs (bézier), arc.rs (circle)
tools/svg-metrics/    structural metrics tool for tracing output
docs/refs/            design references the tracer is built on
.claude/skills/       logo-tracer skill (specs + quality gate)
```

## Local development

Run the web app:

```bash
cd web
python -m http.server 8088     # or: npx serve .
# open http://localhost:8088/
```

No build is required — the prebuilt engine is committed at `web/pkg/`.

Rebuild the wasm engine (after changing `logo-wasm/`):

```bash
cd logo-wasm
wasm-pack build --target web --release
cp pkg/logo_wasm.js pkg/logo_wasm_bg.wasm ../web/pkg/
```

Requirements: Rust toolchain with the `wasm32-unknown-unknown` target, and `wasm-pack`.
See `docs/LOCAL_SETUP.md` for the exact toolchain notes (Windows 11 + MSVC + wasm-pack).

Run the test suites (Rust + svg-metrics):

```bash
npm test
# or, piece by piece:
cd logo-wasm && cargo test
node --test tools/svg-metrics/test/
```

The CI workflow (`.github/workflows/ci.yml`) runs both on every PR.

## Working on the geometry

`logo-wasm/src/logo/mod.rs` is pure geometry + string building. When you change a
tunable in `LogoConfig`, add or update a test in the `#[cfg(test)]` block so the
expected path shape (lines vs. cubics, corner handling, speck dropping) stays pinned.
The design rationale is in `docs/refs/` and `.claude/skills/logo-tracer/specs/`.

## Pull requests

- Keep PRs small; one change per PR.
- For `logo-wasm/` changes: run `cargo test`, and rebuild the committed wasm
  (`wasm-pack build` → copy into `web/pkg/`) so the static demo reflects your change.
- Fill in the PR template; especially "How I verified".

## Reporting issues

Use the issue template. For tracing-quality bugs, attach a source PNG and the SVG you
got, plus the option values you used (max colors, color precision, speckle filter,
corner threshold).
