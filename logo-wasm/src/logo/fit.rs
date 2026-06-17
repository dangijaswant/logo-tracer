//! Curve fitting for logo mode using kurbo's least-squares Bezier fitter.
//!
//! The key idea (vs. the old Catmull-Rom-per-point approach): expose a contour as a
//! parametric curve and let `kurbo::fit_to_bezpath` find the FEWEST cubics that stay
//! within `accuracy`. One long smooth arc becomes 1-2 cubics instead of one-per-point.
//! Corners are reported via `break_cusp` so each smooth span is fitted independently and
//! the corner stays sharp.

use kurbo::{
    fit_to_bezpath, BezPath, CurveFitSample, ParamCurveFit, PathEl, Point as KPoint, Vec2,
};
use visioncortex::PointF64;

/// A closed polyline exposed to kurbo as a parametric curve, parameterized by arc length
/// over t in [0, 1]. `corners` marks vertices that are hard corners (cusps).
pub struct PolylineCurve {
    pts: Vec<PointF64>,
    corner: Vec<bool>,
    /// cumulative arc length at each vertex; cum[i] = length from pts[0] to pts[i].
    cum: Vec<f64>,
    total: f64,
}

impl PolylineCurve {
    /// `pts` is an OPEN polyline span (start..end, no wraparound). Used for per-span fitting.
    pub fn new_open(pts: Vec<PointF64>, corner: Vec<bool>) -> Self {
        Self::build(pts, corner, false)
    }

    fn build(pts: Vec<PointF64>, corner: Vec<bool>, closed: bool) -> Self {
        let n = pts.len();
        let mut cum = vec![0.0; n + 1];
        let segs = if closed { n } else { n.saturating_sub(1) };
        for i in 0..segs {
            let a = pts[i];
            let b = pts[(i + 1) % n];
            cum[i + 1] = cum[i] + ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt();
        }
        if !closed && n >= 1 {
            // last cum entry mirrors the final vertex (no closing segment)
            cum[n] = cum[n.saturating_sub(1)];
        }
        let total = cum[segs];
        Self {
            pts,
            corner,
            cum,
            total,
        }
    }

    /// Map t in [0,1] to (segment index i, fraction f within segment, edge direction).
    fn locate(&self, t: f64) -> (usize, f64) {
        let n = self.pts.len();
        if self.total <= 1e-12 {
            return (0, 0.0);
        }
        let target = (t.clamp(0.0, 1.0)) * self.total;
        // find segment i such that cum[i] <= target <= cum[i+1]
        let mut i = 0;
        while i + 1 < n && self.cum[i + 1] < target {
            i += 1;
        }
        let seg_len = self.cum[i + 1] - self.cum[i];
        let f = if seg_len > 1e-12 {
            (target - self.cum[i]) / seg_len
        } else {
            0.0
        };
        (i, f)
    }

    fn edge_dir(&self, i: usize) -> Vec2 {
        let n = self.pts.len();
        let a = self.pts[i];
        let b = self.pts[(i + 1) % n];
        let v = Vec2::new(b.x - a.x, b.y - a.y);
        let len = v.hypot();
        if len > 1e-12 {
            v / len
        } else {
            Vec2::new(1.0, 0.0)
        }
    }

    fn point_at(&self, i: usize, f: f64) -> KPoint {
        let n = self.pts.len();
        let a = self.pts[i];
        let b = self.pts[(i + 1) % n];
        KPoint::new(a.x + (b.x - a.x) * f, a.y + (b.y - a.y) * f)
    }
}

impl ParamCurveFit for PolylineCurve {
    fn sample_pt_tangent(&self, t: f64, sign: f64) -> CurveFitSample {
        let n = self.pts.len();
        let (i, f) = self.locate(t);
        let p = self.point_at(i, f);
        // At a vertex (f ~ 0 or ~1), tangent is ambiguous; use `sign` to pick the side.
        let tangent = if f < 1e-6 {
            if sign < 0.0 {
                self.edge_dir((i + n - 1) % n)
            } else {
                self.edge_dir(i)
            }
        } else if f > 1.0 - 1e-6 {
            if sign < 0.0 {
                self.edge_dir(i)
            } else {
                self.edge_dir((i + 1) % n)
            }
        } else {
            self.edge_dir(i)
        };
        CurveFitSample { p, tangent }
    }

    fn sample_pt_deriv(&self, t: f64) -> (KPoint, Vec2) {
        let (i, f) = self.locate(t);
        let p = self.point_at(i, f);
        // derivative magnitude scaled by total length so it integrates correctly over [0,1]
        let dir = self.edge_dir(i);
        (p, dir * self.total)
    }

    fn break_cusp(&self, range: std::ops::Range<f64>) -> Option<f64> {
        // Report the t of any corner vertex strictly inside the range.
        let n = self.pts.len();
        for i in 0..n {
            if !self.corner[i] {
                continue;
            }
            let t = if self.total > 1e-12 {
                self.cum[i] / self.total
            } else {
                continue;
            };
            // strictly inside, with a small epsilon to avoid endpoint re-reporting
            if t > range.start + 1e-6 && t < range.end - 1e-6 {
                return Some(t);
            }
        }
        None
    }
}

/// Fit a closed polyline to a Bezier path and return SVG path-data.
/// `accuracy` is the kurbo Fréchet tolerance (pixels). `line_tol` is the max chord
/// deviation under which a fitted cubic is collapsed to a straight line; it must scale
/// with image size or small curves get flattened. Straight spans come back as lines.
pub fn fit_contour_to_svg(
    pts: &[PointF64],
    corner: &[bool],
    accuracy: f64,
    line_tol: f64,
    precision: usize,
) -> Option<String> {
    fit_contour_to_svg_arc(pts, corner, accuracy, line_tol, accuracy, precision)
}

/// Span-based fit using the circle technique. The contour is split at corners; each span is
/// emitted as (1) a straight line if nearly straight, (2) circular-arc cubics if it fits a
/// circle within `arc_tol`, else (3) a kurbo least-squares Bezier fit. This makes curves
/// come from real geometry (center+radius) instead of guessed control points.
pub fn fit_contour_to_svg_arc(
    pts: &[PointF64],
    corner: &[bool],
    accuracy: f64,
    line_tol: f64,
    arc_tol: f64,
    precision: usize,
) -> Option<String> {
    let n = pts.len();
    if n < 3 {
        return None;
    }
    // corner indices (split points). If no corners, the whole closed contour is one span.
    let corner_idx: Vec<usize> = (0..n).filter(|&i| corner[i]).collect();

    // Build the list of spans as index ranges along the closed contour.
    let spans: Vec<Vec<usize>> = if corner_idx.is_empty() {
        // one closed span: all points, wrapping back to start
        let mut v: Vec<usize> = (0..n).collect();
        v.push(0);
        vec![v]
    } else {
        let mut out = Vec::new();
        for w in 0..corner_idx.len() {
            let a = corner_idx[w];
            let b = corner_idx[(w + 1) % corner_idx.len()];
            let mut span = Vec::new();
            let mut i = a;
            loop {
                span.push(i);
                if i == b {
                    break;
                }
                i = (i + 1) % n;
            }
            out.push(span);
        }
        out
    };

    // The path MUST start at the first span's first point (a corner when corners exist),
    // not pts[0] -- otherwise a stray segment is drawn from pts[0] to the first span.
    let start_idx = spans[0][0];
    let mut d = format!(
        "M{} {} ",
        fmt_num(pts[start_idx].x, precision),
        fmt_num(pts[start_idx].y, precision)
    );
    let start_pt = pts[start_idx];
    // Classify each span, merge adjacent same-circle arcs (the reference-D insight), emit.
    let segs: Vec<Segment> = spans
        .iter()
        .map(|span| {
            let span_pts: Vec<PointF64> = span.iter().map(|&i| pts[i]).collect();
            classify_span(&span_pts, accuracy, line_tol, arc_tol)
        })
        .collect();
    let segs = merge_arcs(segs, arc_tol);
    emit_segments(&mut d, &segs, line_tol, precision);
    // The fitted geometry can land a fraction of a pixel off the start point, leaving a tiny
    // `Z` closing segment. Append an explicit line back to the exact start so the loop closes
    // cleanly (the duplicate is collapsed: if we are already there, no visible segment), then
    // rely on `Z`. We only add it when the last emitted point differs from the start.
    if let Some(last) = last_point_of(&d) {
        let gap = ((last.0 - start_pt.x).powi(2) + (last.1 - start_pt.y).powi(2)).sqrt();
        if gap > 1e-6 {
            // overwrite: snap the final coordinate to the start so Z is zero-length
            replace_last_point(&mut d, start_pt, precision);
        }
    }
    d.push_str("Z ");
    Some(d)
}

/// Parse the last "x y" coordinate pair emitted in a path-data string.
fn last_point_of(d: &str) -> Option<(f64, f64)> {
    let nums: Vec<f64> = d
        .split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<f64>().ok())
        .collect();
    if nums.len() >= 2 {
        Some((nums[nums.len() - 2], nums[nums.len() - 1]))
    } else {
        None
    }
}

/// Replace the final coordinate pair in `d` with `p` (used to snap the closing point).
fn replace_last_point(d: &mut String, p: PointF64, precision: usize) {
    // find the byte index of the start of the last two numbers
    let bytes = d.as_bytes();
    let mut i = bytes.len();
    let is_num = |c: u8| c.is_ascii_digit() || c == b'.' || c == b'-';
    // trim trailing spaces
    while i > 0 && (bytes[i - 1] == b' ') {
        i -= 1;
    }
    let mut nums_seen = 0;
    let mut idx = i;
    while idx > 0 {
        // skip a number
        while idx > 0 && is_num(bytes[idx - 1]) {
            idx -= 1;
        }
        nums_seen += 1;
        if nums_seen == 2 {
            break;
        }
        // skip separators between the two numbers
        while idx > 0 && !is_num(bytes[idx - 1]) {
            idx -= 1;
        }
    }
    d.truncate(idx);
    d.push_str(&format!(
        "{} {} ",
        fmt_num(p.x, precision),
        fmt_num(p.y, precision)
    ));
}

/// A classified contour span: a straight line, a circular arc, or a free-form curve.
enum Segment {
    Line {
        end: PointF64,
    },
    Arc {
        circle: crate::logo::arc::Circle,
        a0: f64,
        a1: f64,
        ccw: bool,
        /// the raw points, kept so a merged arc can be re-validated against them
        pts: Vec<PointF64>,
    },
    Curve {
        bez: BezPath,
    },
    /// sub-pixel span, dropped (emits nothing)
    Skip,
}

/// Classify one span as Line / Arc / Curve (the circle technique), without emitting yet.
fn classify_span(
    span: &[PointF64],
    accuracy: f64,
    line_tol: f64,
    arc_tol: f64,
) -> Segment {
    if span.len() < 2 {
        return Segment::Skip;
    }
    let start = span[0];
    let end = span[span.len() - 1];
    let chord = ((end.x - start.x).powi(2) + (end.y - start.y).powi(2)).sqrt();
    if chord < 1.5 {
        return Segment::Skip;
    }
    let kstart = KPoint::new(start.x, start.y);
    let kend = KPoint::new(end.x, end.y);

    // 1) straight?
    let mut max_perp = 0.0_f64;
    for p in span.iter() {
        max_perp = max_perp.max(perp_dist(KPoint::new(p.x, p.y), kstart, kend));
    }
    if max_perp <= line_tol {
        return Segment::Line { end };
    }

    // 2) circular arc?
    if let Some((circle, a0, a1, ccw)) = crate::logo::arc::try_arc(span, arc_tol) {
        return Segment::Arc {
            circle,
            a0,
            a1,
            ccw,
            pts: span.to_vec(),
        };
    }

    // 3) free-form curve via kurbo
    let no_corners = vec![false; span.len()];
    let curve = PolylineCurve::new_open(span.to_vec(), no_corners);
    let bez = fit_to_bezpath(&curve, accuracy);
    Segment::Curve { bez }
}

/// Merge consecutive `Arc` segments that lie on the same circle (center+radius within
/// tolerance) into a single larger arc. This is the "circle technique" insight from the
/// reference D: a whole bowl is ONE circle, so adjacent same-radius arcs become one arc
/// (then re-emitted as the minimal cubics) instead of several small ones.
fn merge_arcs(segs: Vec<Segment>, arc_tol: f64) -> Vec<Segment> {
    let mut out: Vec<Segment> = Vec::new();
    for seg in segs {
        if let Segment::Arc {
            circle, ccw, pts, ..
        } = &seg
        {
            if let Some(Segment::Arc {
                circle: pc,
                pts: ppts,
                ccw: pccw,
                ..
            }) = out.last()
            {
                let same_center = ((circle.cx - pc.cx).powi(2) + (circle.cy - pc.cy).powi(2))
                    .sqrt()
                    < (arc_tol + 2.0);
                let same_radius = (circle.r - pc.r).abs() < (arc_tol + 2.0);
                if same_center && same_radius && ccw == pccw {
                    // re-fit a single circle to the combined points and re-derive the arc
                    let mut combined = ppts.clone();
                    combined.extend(pts.iter().copied());
                    if let Some((mc, ma0, ma1, mccw)) =
                        crate::logo::arc::try_arc(&combined, arc_tol + 1.5)
                    {
                        out.pop();
                        out.push(Segment::Arc {
                            circle: mc,
                            a0: ma0,
                            a1: ma1,
                            ccw: mccw,
                            pts: combined,
                        });
                        continue;
                    }
                }
            }
        }
        out.push(seg);
    }
    out
}

/// Serialize classified segments into the path `d` (after the initial M).
fn emit_segments(d: &mut String, segs: &[Segment], line_tol: f64, precision: usize) {
    for seg in segs {
        match seg {
            Segment::Skip => {}
            Segment::Line { end } => {
                d.push_str(&format!(
                    "L{} {} ",
                    fmt_num(end.x, precision),
                    fmt_num(end.y, precision)
                ));
            }
            Segment::Arc {
                circle, a0, a1, ccw, ..
            } => {
                d.push_str(&crate::logo::arc::arc_to_cubics(circle, *a0, *a1, *ccw, precision));
            }
            Segment::Curve { bez } => {
                append_bezpath_tail(d, bez, line_tol, precision);
            }
        }
    }
}

fn fmt_num(v: f64, precision: usize) -> String {
    let s = format!("{:.*}", precision, v);
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

/// Perpendicular distance of point `c` from the segment a->b.
fn perp_dist(c: KPoint, a: KPoint, b: KPoint) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1e-9 {
        return ((c.x - a.x).powi(2) + (c.y - a.y).powi(2)).sqrt();
    }
    (dx * (a.y - c.y) - (a.x - c.x) * dy).abs() / len
}

/// Append a fitted span's segments to an existing `d` string, skipping the leading MoveTo
/// (the span continues from the current path position). Collapses flat cubics to lines.
fn append_bezpath_tail(d: &mut String, bez: &BezPath, line_tol: f64, p: usize) {
    let mut cur = KPoint::new(0.0, 0.0);
    let seg_len = |a: KPoint, b: KPoint| ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt();
    for el in bez.elements() {
        match el {
            PathEl::MoveTo(pt) => {
                cur = *pt; // skip emitting; we already have a current point
            }
            PathEl::LineTo(pt) => {
                if seg_len(cur, *pt) >= 1.5 {
                    d.push_str(&format!("L{} {} ", fmt_num(pt.x, p), fmt_num(pt.y, p)));
                    cur = *pt;
                }
            }
            PathEl::QuadTo(c, pt) => {
                if seg_len(cur, *pt) < 1.5 {
                    // skip tiny segment
                } else if perp_dist(*c, cur, *pt) <= line_tol {
                    d.push_str(&format!("L{} {} ", fmt_num(pt.x, p), fmt_num(pt.y, p)));
                } else {
                    d.push_str(&format!(
                        "Q{} {} {} {} ",
                        fmt_num(c.x, p),
                        fmt_num(c.y, p),
                        fmt_num(pt.x, p),
                        fmt_num(pt.y, p)
                    ));
                }
                cur = *pt;
            }
            PathEl::CurveTo(c1, c2, pt) => {
                if seg_len(cur, *pt) < 1.5 {
                    // skip tiny segment
                } else if perp_dist(*c1, cur, *pt) <= line_tol
                    && perp_dist(*c2, cur, *pt) <= line_tol
                {
                    d.push_str(&format!("L{} {} ", fmt_num(pt.x, p), fmt_num(pt.y, p)));
                } else {
                    d.push_str(&format!(
                        "C{} {} {} {} {} {} ",
                        fmt_num(c1.x, p),
                        fmt_num(c1.y, p),
                        fmt_num(c2.x, p),
                        fmt_num(c2.y, p),
                        fmt_num(pt.x, p),
                        fmt_num(pt.y, p)
                    ));
                }
                cur = *pt;
            }
            PathEl::ClosePath => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f64, y: f64) -> PointF64 {
        PointF64 { x, y }
    }

    #[test]
    fn square_fits_to_lines_only() {
        let sq = vec![p(0.0, 0.0), p(100.0, 0.0), p(100.0, 100.0), p(0.0, 100.0)];
        let corners = vec![true, true, true, true];
        let d = fit_contour_to_svg(&sq, &corners, 1.0, 0.75, 2).unwrap();
        assert!(!d.contains('C'), "a square must have no cubics: {d}");
        assert!(d.contains('L'));
    }

    #[test]
    fn circle_fits_to_few_cubics() {
        // sample a circle densely (like a pixel contour) -> should fit with FEW cubics,
        // not one per sample point. This is the whole point of the rework. We model a
        // realistic post-corner-split case: a closed shape made of a half-circle arc (top)
        // closed by a straight diameter (bottom), with the two ends marked as corners.
        let n = 33;
        let r = 100.0;
        let mut pts: Vec<PointF64> = (0..n)
            .map(|i| {
                let a = i as f64 / (n - 1) as f64 * std::f64::consts::PI; // 0..pi (half circle)
                p(r * a.cos(), r * a.sin())
            })
            .collect();
        // ends of the half-circle are corners (where it meets the diameter)
        let mut corners = vec![false; pts.len()];
        corners[0] = true;
        corners[pts.len() - 1] = true;
        // (the straight diameter back to start is implied by the closing Z)
        let _ = &mut pts;
        let d = fit_contour_to_svg(&pts, &corners, 0.5, 0.3, 2).unwrap();
        let cubics = d.matches('C').count();
        assert!(
            (1..=6).contains(&cubics),
            "a half circle should fit in 1..=6 cubics, got {cubics}: {d}"
        );
    }
}
