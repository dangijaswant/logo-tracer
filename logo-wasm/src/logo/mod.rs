//! Logo-mode geometry cleanup (behind the `logo` preset).
//!
//! A native Rust port of the `_clean_svg.py` prototype. Operates on a flattened,
//! closed polyline (`Vec<PointF64>`) for one contour and produces an SVG path-data
//! fragment that:
//!   * draws straight runs as lines (`L`) with axis-snapping for crisp H/V stems,
//!   * keeps detected corners sharp,
//!   * fits smooth runs with cubic Beziers (Catmull-Rom tangents, zeroed at corners).
//!
//! This module is pure geometry + string building; it does not touch the default
//! tracing path. See `docs/refs/` and `.claude/skills/logo-tracer/specs/` for the
//! design rationale (Glyphs/Pomax/Box-Method/Material/optical/Bezier-Game).

pub mod arc;
pub mod fit;

use std::collections::HashMap;
use visioncortex::{Color, PointF64};

/// Tunables for the logo cleanup. Defaults mirror the validated Python prototype.
#[derive(Debug, Clone, Copy)]
pub struct LogoConfig {
    /// RDP simplification tolerance in pixels.
    pub rdp_epsilon: f64,
    /// Axis-snap angle tolerance in degrees (near-H/V edges snap to exact H/V).
    pub axis_snap_deg: f64,
    /// Corner turn-angle threshold in degrees (sharper turns become corners).
    pub corner_threshold_deg: f64,
    /// Drop subpaths whose total length is below this (residual specks).
    pub min_subpath_len: f64,
    /// kurbo fit accuracy (Fréchet tolerance, pixels). Larger => fewer, looser curves.
    pub fit_accuracy: f64,
    /// Max chord deviation under which a fitted cubic collapses to a line. Must scale with
    /// image size; too large flattens small curves into facets.
    pub line_collapse_tol: f64,
    /// Coordinate output precision (decimal places).
    pub precision: usize,
}

impl Default for LogoConfig {
    fn default() -> Self {
        Self {
            // Polygon-mode contours are dense; simplify harder than the prototype and
            // drop more residual specks to keep node/path counts down.
            rdp_epsilon: 2.0,
            axis_snap_deg: 6.0,
            corner_threshold_deg: 42.0,
            min_subpath_len: 24.0,
            // "Favor clean": a slightly looser tolerance lets kurbo collapse arcs into
            // very few cubics. Scaled to image size by the converter.
            fit_accuracy: 1.5,
            line_collapse_tol: 0.75,
            precision: 2,
        }
    }
}

#[inline]
fn dist(a: PointF64, b: PointF64) -> f64 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}

/// Total length of a (closed) polyline.
fn polyline_len(pts: &[PointF64], closed: bool) -> f64 {
    if pts.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0;
    for i in 0..pts.len() - 1 {
        total += dist(pts[i], pts[i + 1]);
    }
    if closed {
        total += dist(pts[pts.len() - 1], pts[0]);
    }
    total
}

/// Ramer-Douglas-Peucker simplification (open polyline).
/// Mirrors the prototype's `rdp`.
pub fn rdp(pts: &[PointF64], eps: f64) -> Vec<PointF64> {
    if pts.len() < 3 {
        return pts.to_vec();
    }
    let start = pts[0];
    let end = pts[pts.len() - 1];
    let d = PointF64 {
        x: end.x - start.x,
        y: end.y - start.y,
    };
    let nrm = (d.x * d.x + d.y * d.y).sqrt();

    let mut idx = 0;
    let mut max_d = 0.0;
    for (i, p) in pts.iter().enumerate() {
        let dperp = if nrm < 1e-9 {
            ((p.x - start.x).powi(2) + (p.y - start.y).powi(2)).sqrt()
        } else {
            // perpendicular distance to the chord start->end
            (d.x * (start.y - p.y) - (start.x - p.x) * d.y).abs() / nrm
        };
        if dperp > max_d {
            max_d = dperp;
            idx = i;
        }
    }

    if max_d > eps {
        let mut left = rdp(&pts[..=idx], eps);
        let right = rdp(&pts[idx..], eps);
        left.pop(); // avoid duplicating the shared point
        left.extend(right);
        left
    } else {
        vec![start, end]
    }
}

/// Snap near-horizontal / near-vertical edges of a CLOSED polyline to exact H/V,
/// preserving corner positions by moving both endpoints to the shared midpoint.
/// Mirrors the prototype's `axis_snap`.
pub fn axis_snap(pts: &mut [PointF64], ang_tol_deg: f64) {
    let n = pts.len();
    if n < 2 {
        return;
    }
    for i in 0..n {
        let a = pts[i];
        let b = pts[(i + 1) % n];
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        if dx == 0.0 && dy == 0.0 {
            continue;
        }
        let ang = dy.atan2(dx).to_degrees();
        let from_h = ang.abs().min((ang.abs() - 180.0).abs());
        let from_v = (ang.abs() - 90.0).abs();
        if from_h < ang_tol_deg {
            let my = (a.y + b.y) / 2.0;
            pts[i].y = my;
            pts[(i + 1) % n].y = my;
        } else if from_v < ang_tol_deg {
            let mx = (a.x + b.x) / 2.0;
            pts[i].x = mx;
            pts[(i + 1) % n].x = mx;
        }
    }
}

/// Flag vertices of a CLOSED polyline whose turn angle exceeds the threshold as
/// sharp corners. Mirrors the prototype's `corner_flags`.
pub fn corner_flags(pts: &[PointF64], thresh_deg: f64) -> Vec<bool> {
    let n = pts.len();
    let mut flags = vec![false; n];
    if n < 3 {
        return vec![true; n];
    }
    for i in 0..n {
        let p0 = pts[(i + n - 1) % n];
        let p1 = pts[i];
        let p2 = pts[(i + 1) % n];
        let v1 = PointF64 {
            x: p1.x - p0.x,
            y: p1.y - p0.y,
        };
        let v2 = PointF64 {
            x: p2.x - p1.x,
            y: p2.y - p1.y,
        };
        let a1 = (v1.x * v1.x + v1.y * v1.y).sqrt();
        let a2 = (v2.x * v2.x + v2.y * v2.y).sqrt();
        if a1 < 1e-6 || a2 < 1e-6 {
            flags[i] = true;
            continue;
        }
        let cosang = ((v1.x * v2.x + v1.y * v2.y) / (a1 * a2)).clamp(-1.0, 1.0);
        let turn = cosang.acos().to_degrees();
        if turn > thresh_deg {
            flags[i] = true;
        }
    }
    flags
}

/// Full cleanup of one contour polyline -> SVG path-data fragment (or None if dropped).
/// Input is treated as a closed polyline in absolute coordinates.
pub fn clean_contour(mut pts: Vec<PointF64>, cfg: &LogoConfig) -> Option<String> {
    if polyline_len(&pts, true) < cfg.min_subpath_len {
        return None;
    }
    // drop a duplicated closing point if present
    if pts.len() >= 2 && dist(pts[0], pts[pts.len() - 1]) < 1e-6 {
        pts.pop();
    }
    let simplified = rdp(&pts, cfg.rdp_epsilon);
    if simplified.len() < 3 {
        return None;
    }
    let mut s = simplified;
    axis_snap(&mut s, cfg.axis_snap_deg);
    let flags = corner_flags(&s, cfg.corner_threshold_deg);
    // Fit with kurbo: least-squares Bezier fitting emits the FEWEST cubics per smooth span
    // (one arc -> 1-2 cubics) and straight spans as lines. This replaces the old
    // Catmull-Rom-per-point emitter that produced dense clusters of tiny cubics.
    fit::fit_contour_to_svg(
        &s,
        &flags,
        cfg.fit_accuracy,
        cfg.line_collapse_tol,
        cfg.precision,
    )
}

/// Squared RGB distance between two colors.
fn color_dist2(a: Color, b: Color) -> i32 {
    let dr = a.r as i32 - b.r as i32;
    let dg = a.g as i32 - b.g as i32;
    let db = a.b as i32 - b.b as i32;
    dr * dr + dg * dg + db * db
}

/// Snap a color to the nearest entry of a dominant palette (collapses anti-alias
/// fragments to the dominant colors). `merge_threshold` is a squared-distance cutoff.
pub fn snap_to_palette(color: Color, palette: &[Color]) -> Color {
    palette
        .iter()
        .copied()
        .min_by_key(|&p| color_dist2(color, p))
        .unwrap_or(color)
}

/// Build a small dominant palette from weighted colors (weight = pixel/area count).
/// Greedy: take heaviest colors, merge any within `merge_threshold` (squared RGB).
pub fn dominant_palette(
    weighted: &[(Color, usize)],
    max_colors: usize,
    merge_threshold: i32,
) -> Vec<Color> {
    // accumulate weight per exact color first
    let mut totals: HashMap<(u8, u8, u8), usize> = HashMap::new();
    for &(c, w) in weighted {
        *totals.entry((c.r, c.g, c.b)).or_insert(0) += w;
    }
    let mut sorted: Vec<(Color, usize)> = totals
        .into_iter()
        .map(|((r, g, b), w)| (Color::new(r, g, b), w))
        .collect();
    sorted.sort_by_key(|&(_, w)| std::cmp::Reverse(w));

    let mut palette: Vec<Color> = Vec::new();
    for (c, _) in sorted {
        if palette.len() >= max_colors {
            break;
        }
        if palette.iter().all(|&p| color_dist2(c, p) > merge_threshold) {
            palette.push(c);
        }
    }
    palette
}

/// A cleaned logo path: SVG `d` fragment + its (palette-snapped) fill color.
pub struct LogoPath {
    pub d: String,
    pub color: Color,
}

/// Serialize cleaned logo paths into a complete SVG document.
pub fn logo_svg(width: usize, height: usize, paths: &[LogoPath], generator: &str) -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str(&format!("<!-- Generator: {} (logo mode) -->\n", generator));
    out.push_str(&format!(
        "<svg version=\"1.1\" xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">\n",
        width, height, width, height
    ));
    for p in paths {
        out.push_str(&format!(
            "<path d=\"{}\" fill=\"{}\"/>\n",
            p.d.trim_end(),
            p.color.to_hex_string()
        ));
    }
    out.push_str("</svg>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f64, y: f64) -> PointF64 {
        PointF64 { x, y }
    }

    #[test]
    fn rdp_collapses_collinear_points() {
        // a straight run of points should reduce to its two endpoints
        let line = vec![p(0.0, 0.0), p(1.0, 0.0), p(2.0, 0.0), p(3.0, 0.0)];
        let out = rdp(&line, 0.5);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0], p(0.0, 0.0));
        assert_eq!(out[1], p(3.0, 0.0));
    }

    #[test]
    fn rdp_keeps_a_real_bend() {
        let bend = vec![p(0.0, 0.0), p(1.0, 1.0), p(2.0, 0.0)];
        let out = rdp(&bend, 0.5);
        assert_eq!(out.len(), 3); // the apex must survive
    }

    #[test]
    fn axis_snap_makes_near_horizontal_exact() {
        // closed square-ish loop with a slightly-off horizontal top edge
        let mut sq = vec![p(0.0, 0.05), p(10.0, -0.05), p(10.0, 10.0), p(0.0, 10.0)];
        axis_snap(&mut sq, 4.0);
        // top edge endpoints should now share a y
        assert!((sq[0].y - sq[1].y).abs() < 1e-9);
    }

    #[test]
    fn corner_flags_marks_square_corners() {
        let sq = vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)];
        let flags = corner_flags(&sq, 42.0);
        // every vertex of a square is a 90-degree corner
        assert!(flags.iter().all(|&f| f));
    }

    #[test]
    fn corner_flags_ignores_gentle_curve() {
        // points along a shallow arc: small turn angles, no corners
        let arc: Vec<PointF64> = (0..12)
            .map(|i| {
                let t = i as f64 / 12.0 * std::f64::consts::TAU;
                p(50.0 * t.cos(), 50.0 * t.sin())
            })
            .collect();
        let flags = corner_flags(&arc, 42.0);
        assert!(flags.iter().all(|&f| !f));
    }

    #[test]
    fn square_emits_only_lines() {
        let sq = vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)];
        let d = clean_contour(sq, &LogoConfig::default()).unwrap();
        assert!(d.contains('L'));
        assert!(!d.contains('C')); // a pure square has no curves
        assert!(d.trim_end().ends_with('Z'));
    }

    #[test]
    fn straight_run_with_noncorner_nodes_stays_lines() {
        // A large rectangle whose long edges have extra near-collinear midpoints (not
        // corners). The kurbo fit must keep the straight edges as lines, not cubics.
        let rect = vec![
            p(0.0, 0.0),
            p(0.0, 150.0), // midpoint on left edge (collinear)
            p(0.0, 300.0),
            p(120.0, 300.0),
            p(120.0, 150.0), // midpoint on right edge (collinear)
            p(120.0, 0.0),
        ];
        let d = clean_contour(rect, &LogoConfig::default()).unwrap();
        assert!(
            !d.contains('C'),
            "straight edges must not become cubics: {d}"
        );
        assert!(d.contains('L'));
    }

    #[test]
    fn tiny_contour_is_dropped() {
        let speck = vec![p(0.0, 0.0), p(1.0, 0.0), p(1.0, 1.0), p(0.0, 1.0)];
        assert!(clean_contour(speck, &LogoConfig::default()).is_none());
    }
}
