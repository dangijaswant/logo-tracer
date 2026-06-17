# LOCAL_SETUP.md — Custom VTracer Local Development

This document records the **exact commands and state** that succeeded on this Windows 11
machine while bringing up the unmodified VTracer baseline (Phases 0–2 of the project guide).

> Status at time of writing: **baseline tracing proven via the native CLI**. The browser
> web UI is **not yet running** because of a Webpack 4 / WASM incompatibility (see the
> "Known issue" section). Per the user's decision, the baseline is being verified
> headlessly through the `vtracer` command-line app, which uses the **same `visioncortex`
> tracing engine** as the web app.

---

## 1. Machine / toolchain (Phase 0)

| Tool                       | Version / value                          |
| -------------------------- | ---------------------------------------- |
| OS                         | Windows 11 (10.0.26200)                  |
| PowerShell                 | 5.1.26100                                |
| git                        | 2.46.0                                   |
| node                       | v22.19.0                                 |
| npm                        | 10.9.3                                   |
| rustc / cargo              | 1.96.0                                   |
| Rust host toolchain        | `stable-x86_64-pc-windows-msvc`          |
| Rust target (added)        | `wasm32-unknown-unknown`                 |
| MSVC compiler / linker     | `cl.exe` / `link.exe` (VS 2022 BuildTools, MSVC 14.44.35207) |
| Windows SDK                | 10.0.26100.0 (x64 libs present)          |
| wasm-pack                  | 0.15.0                                   |

### Prerequisites that had to be installed
The machine initially had **no Rust, no wasm-pack, and no C++ build tools**. Installed:

1. **Visual Studio 2022 Build Tools** with the **"Desktop development with C++"** workload
   (provides MSVC v143 `cl.exe`/`link.exe`) **plus the Windows 11 SDK (10.0.26100)**.
   The Windows SDK is mandatory — without its libs (`kernel32.lib`, `libucrt.lib`),
   linking fails even though `link.exe` exists.
2. **Rust** via rustup (default `stable-x86_64-pc-windows-msvc`), then
   `rustup target add wasm32-unknown-unknown`.
3. **wasm-pack** via `cargo install wasm-pack` (compiles from source; needs the MSVC
   linker from step 1, so install it last).

### Verification (proved the toolchain links end-to-end)
```powershell
& "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" -products * -property displayName
# -> Visual Studio Build Tools 2022
cargo new $env:TEMP\rust_link_test --bin
cargo build --manifest-path $env:TEMP\rust_link_test\Cargo.toml   # built + linked + ran "Hello, world!"
```

---

## 2. Workspace & repositories (Phase 1)

Workspace root (user-chosen, instead of the guide's default `D:\projects\custom-vtracer`):

```text
A:\image tracingg\custom-vtracer\
├── vtracer\          (git: feature/logo-tracer, from fd9cdb0)
├── visioncortex\     (git: feature/logo-tracer, from 01ec7c2)
├── test-assets\
├── test-output\
└── docs\
```

Commands that succeeded:
```powershell
New-Item -ItemType Directory -Path 'A:\image tracingg\custom-vtracer' -Force
Set-Location 'A:\image tracingg\custom-vtracer'
git clone https://github.com/visioncortex/vtracer.git
git clone https://github.com/visioncortex/visioncortex.git
git -C .\vtracer      switch -c feature/logo-tracer
git -C .\visioncortex switch -c feature/logo-tracer
```

### Repo structure notes (inspected, not assumed)
- `vtracer` is a Cargo **workspace** of two members: `cmdapp` and `webapp`.
  - `cmdapp` → the native CLI (crate name `vtracer`, v0.6.12). Depends on `visioncortex = 0.8.8`.
  - `webapp` → a `cdylib` built with wasm-pack (crate `vtracer-webapp`, v0.4.0).
    Depends on `visioncortex = 0.8.1`.
- **Neither member currently uses a local path dependency on `visioncortex`** — both pull
  the published crate from crates.io (resolved to `visioncortex 0.8.10` at build time).
  Wiring the local `..\..\visioncortex` checkout is **Phase 3** and has **not** been done yet.
- Frontend lives in `webapp\app` (Webpack 4; scripts `start` = `webpack-dev-server`,
  `build` = `webpack`).

---

## 3. Building the unmodified baseline (Phase 2)

### 3a. WASM build — SUCCEEDED
```powershell
$env:Path += ";$HOME\.cargo\bin"
Set-Location 'A:\image tracingg\custom-vtracer\vtracer\webapp'
wasm-pack build --dev      # -> pkg\vtracer_webapp_bg.wasm + JS bindings, exit 0
```

### 3b. Frontend install — SUCCEEDED
```powershell
Set-Location 'A:\image tracingg\custom-vtracer\vtracer\webapp\app'
npm install                # exit 0 (audit warnings from old Webpack 4 stack; NOT fixed, to preserve baseline)
```

### 3c. Dev server — STARTS, but fails to compile WASM (see Known issue)
```powershell
$env:NODE_OPTIONS="--openssl-legacy-provider"   # required: Node 22 + Webpack 4 (OpenSSL legacy)
npm run start                                    # server runs at http://localhost:8080/ but webpack errors on the wasm
```

### 3d. Native CLI build — SUCCEEDED (used for the headless baseline)
```powershell
$env:Path += ";$HOME\.cargo\bin"
Set-Location 'A:\image tracingg\custom-vtracer\vtracer'
cargo build --release -p vtracer        # -> target\release\vtracer.exe, exit 0
```

End-to-end SVG generation was proven by tracing a stand-in PNG; it produced a valid SVG
with a correct `viewBox`/dimensions and `<path>` data (`Conversion successful.`, exit 0).

Default CLI invocation (matches the web app's default `color`/`spline` behavior — the
"existing VTracer result" the project aims to improve):
```powershell
.\target\release\vtracer.exe -i <input.png> -o <output.svg>
```

---

## Known issue — Webpack 4 cannot parse modern wasm-bindgen output

**Symptom** (browser dev server, `npm run start`):
```
ERROR in ../pkg/vtracer_webapp_bg.wasm
Module parse failed: Internal failure: parseVec could not cast the value
... unexpected end
```

**Root cause:** The frontend uses **Webpack 4**, whose built-in `@webassemblyjs` decoder
is too old to parse the WASM binary emitted by current **wasm-bindgen (0.2.125)**. The
`pkg` is built for the *bundler* target (`import * as wasm from "...wasm"`), so Webpack
itself tries to decode the `.wasm` and chokes on its newer binary format. This is
environment/version drift, **not** a bug in vtracer.

**Decision:** Defer the browser UI fix. Verify the baseline headlessly via `vtracer.exe`
(same `visioncortex` engine). When the web UI is needed, the least-invasive fix is to
rebuild with `wasm-pack build --target web` and add a few lines of `init()` glue to
`app/index.js` + `app/bootstrap.js` (no tracing/algorithm changes). Other options:
pin an older wasm-bindgen, or upgrade the frontend to Webpack 5 (`asyncWebAssembly`).

---

## Node 22 note
Node 22 is newer than this repo's Webpack 4 config expects. The dev server requires
`$env:NODE_OPTIONS="--openssl-legacy-provider"` (session-only; not set globally) to avoid
`ERR_OSSL_EVP_UNSUPPORTED`. This is applied only when launching the dev server.

---

## Outstanding before Phase 5 (algorithm work)
- [ ] Obtain the real **`news-cg.png`** logo from the user and copy it to
      `test-assets\news-cg.png`.
- [ ] Generate `test-output\news-cg-baseline.svg` with the default CLI config.
- [ ] (Phase 3) Wire the local `visioncortex` path dependency + prove it is used.
- [ ] (Optional) Fix the web UI WASM loading when the browser preview is needed.

**Do not start Phase 5+ (algorithm changes) until the user confirms the baseline is working.**
