# Changelog

All notable changes to this project are documented in this file.

## [0.1.0] - 2026-11-08

### Added
- MIT license at the repo root (the `logo-wasm` crate already declared MIT OR Apache-2.0).
- CI workflow (`.github/workflows/ci.yml`): runs `cargo test` for `logo-wasm` and the
  svg-metrics test suite on push to `master` and on every PR.
- 4 public-API contract tests in `logo-wasm/src/lib.rs` (input validation + output shape);
  the crate's suite is now 17 tests, all green.
- svg-metrics test suite (`tools/svg-metrics/test/`, 9 tests, node:test, no deps); the
  tool's internal functions are now exported and `main()` only runs when executed
  directly, so the module is importable.
- Contributor tooling: `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, issue + PR templates,
  Dependabot (GitHub Actions), root `package.json` so `npm test` runs both suites.
- First tagged release (v0.1.0) so consumers can pin a version.

### Changed
- README rewritten: options table, testing instructions, toolchain notes, CI + license badges.
