//! WebAssembly entry point for the logo-mode PNG->SVG tracer.
//!
//! JS decodes the PNG to raw RGBA via a canvas and calls `trace_logo(rgba, w, h, opts)`.
//! This crate runs the same logo pipeline as the CLI (`--preset logo`) and returns the SVG
//! string. No `image` crate / filesystem is used, so it builds for wasm32.

mod logo;

use logo::{LogoConfig, LogoPath};
use visioncortex::color_clusters::{KeyingAction, Runner, RunnerConfig, HIERARCHICAL_MAX};
use visioncortex::{Color, ColorImage, CompoundPathElement, PointF64};
use wasm_bindgen::prelude::*;

/// Tunable options passed from JS (all optional; sensible defaults applied).
#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct LogoOptions {
    pub max_colors: usize,
    pub palette_merge_threshold: i32,
    pub filter_speckle: usize,
    pub color_precision: i32, // 1..=8 significant bits
    pub corner_threshold_deg: f64,
    pub precision: u32,
}

#[wasm_bindgen]
impl LogoOptions {
    /// Defaults matching the CLI `--preset logo`.
    #[wasm_bindgen(constructor)]
    pub fn new() -> LogoOptions {
        LogoOptions {
            max_colors: 4,
            palette_merge_threshold: 90 * 90 * 3,
            filter_speckle: 16,
            color_precision: 4,
            corner_threshold_deg: 40.0,
            precision: 2,
        }
    }
}

impl Default for LogoOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Trace raw RGBA pixels (length must be w*h*4) into a logo-mode SVG string.
#[wasm_bindgen]
pub fn trace_logo(rgba: &[u8], width: usize, height: usize, opts: LogoOptions) -> String {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    if rgba.len() != width * height * 4 || width == 0 || height == 0 {
        return String::from("<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>");
    }

    let mut img = ColorImage {
        pixels: rgba.to_vec(),
        width,
        height,
    };

    // --- keying: replace fully-transparent pixels with an unused key color ---
    let key_color = if should_key_image(&img) {
        if let Ok(key) = find_unused_color_in_image(&img) {
            for y in 0..height {
                for x in 0..width {
                    if img.get_pixel(x, y).a == 0 {
                        img.set_pixel(x, y, &key);
                    }
                }
            }
            key
        } else {
            Color::default()
        }
    } else {
        Color::default()
    };

    // --- clustering (color mode, polygon-style contours via to_compound_path) ---
    let color_precision_loss = 8 - opts.color_precision;
    let filter_speckle_area = opts.filter_speckle * opts.filter_speckle;
    let runner = Runner::new(
        RunnerConfig {
            diagonal: false,
            hierarchical: HIERARCHICAL_MAX,
            batch_size: 25600,
            good_min_area: filter_speckle_area,
            good_max_area: width * height,
            is_same_color_a: color_precision_loss,
            is_same_color_b: 1,
            deepen_diff: 16,
            hollow_neighbours: 1,
            key_color,
            keying_action: KeyingAction::Discard,
        },
        img,
    );
    let clusters = runner.run();
    let view = clusters.view();

    // --- size-aware logo config (mirrors the CLI converter) ---
    let ref_dim = width.max(height) as f64;
    let defaults = LogoConfig::default();
    let scale = (ref_dim / 1254.0).max(0.12);
    let fit_accuracy = (ref_dim * 0.004).clamp(1.2, 3.0);
    let logo_cfg = LogoConfig {
        precision: opts.precision as usize,
        corner_threshold_deg: opts.corner_threshold_deg,
        rdp_epsilon: (defaults.rdp_epsilon * scale).clamp(0.3, 1.0),
        min_subpath_len: (defaults.min_subpath_len * scale).max(4.0),
        fit_accuracy,
        line_collapse_tol: (fit_accuracy * 0.5).clamp(0.4, 1.2),
        ..defaults
    };

    // Polygon mode for clean polyline contours.
    let corner_threshold = opts.corner_threshold_deg / 180.0 * std::f64::consts::PI;
    let mode = visioncortex::PathSimplifyMode::Polygon;

    struct RawPath {
        d: String,
        color: Color,
        area: usize,
    }
    let mut raw: Vec<RawPath> = Vec::new();
    for &cluster_index in view.clusters_output.iter().rev() {
        let cluster = view.get_cluster(cluster_index);
        let color = cluster.residue_color();
        let area = cluster.area();
        let compound = cluster.to_compound_path(&view, false, mode, corner_threshold, 4.0, 10, 45.0);
        for element in compound.iter() {
            let pts: Vec<PointF64> = match element {
                CompoundPathElement::PathI32(p) => p
                    .path
                    .iter()
                    .map(|q| PointF64 {
                        x: q.x as f64,
                        y: q.y as f64,
                    })
                    .collect(),
                CompoundPathElement::PathF64(p) => p.path.clone(),
                CompoundPathElement::Spline(_) => continue,
            };
            if let Some(d) = logo::clean_contour(pts, &logo_cfg) {
                raw.push(RawPath { d, color, area });
            }
        }
    }

    // --- palette snap to a small dominant set ---
    let weighted: Vec<(Color, usize)> = raw.iter().map(|r| (r.color, r.area)).collect();
    let palette = logo::dominant_palette(&weighted, opts.max_colors, opts.palette_merge_threshold);
    let paths: Vec<LogoPath> = raw
        .into_iter()
        .map(|r| LogoPath {
            d: r.d,
            color: logo::snap_to_palette(r.color, &palette),
        })
        .collect();

    logo::logo_svg(width, height, &paths, "logo-wasm")
}

// --- keying helpers (ported from the CLI converter; no `image` dep) ---

const KEYING_THRESHOLD: f32 = 0.2;

fn color_exists_in_image(img: &ColorImage, color: Color) -> bool {
    for y in 0..img.height {
        for x in 0..img.width {
            let p = img.get_pixel(x, y);
            if p.r == color.r && p.g == color.g && p.b == color.b {
                return true;
            }
        }
    }
    false
}

fn find_unused_color_in_image(img: &ColorImage) -> Result<Color, String> {
    let candidates = [
        Color::new(255, 0, 0),
        Color::new(0, 255, 0),
        Color::new(0, 0, 255),
        Color::new(255, 255, 0),
        Color::new(0, 255, 255),
        Color::new(255, 0, 255),
        Color::new(128, 128, 128),
        Color::new(1, 2, 3),
    ];
    for c in candidates {
        if !color_exists_in_image(img, c) {
            return Ok(c);
        }
    }
    Err(String::from("no unused color"))
}

fn should_key_image(img: &ColorImage) -> bool {
    if img.width == 0 || img.height == 0 {
        return false;
    }
    let threshold = ((img.width * 2) as f32 * KEYING_THRESHOLD) as usize;
    let mut n = 0;
    let ys = [
        0,
        img.height / 4,
        img.height / 2,
        3 * img.height / 4,
        img.height - 1,
    ];
    for y in ys {
        for x in 0..img.width {
            if img.get_pixel(x, y).a == 0 {
                n += 1;
            }
            if n >= threshold {
                return true;
            }
        }
    }
    false
}
