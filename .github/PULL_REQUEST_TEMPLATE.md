## What changes
One or two sentences.

## Why
What problem or annoyance does this fix?

## How to verify
- [ ] `cargo test` passes (required for `logo-wasm/` changes)
- [ ] `node --test tools/svg-metrics/test/` passes (required for `tools/` changes)
- [ ] For engine changes: rebuilt the committed wasm (`wasm-pack build --target web --release` → copy into `web/pkg/`) and checked the demo against a sample PNG
- [ ] For web/ changes: served `web/` locally and traced a sample logo

## Notes
_Deployment notes for Cloudflare Pages if relevant, otherwise delete._
