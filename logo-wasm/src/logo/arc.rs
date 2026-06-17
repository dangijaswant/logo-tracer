//! Circle-technique curve construction.
//!
//! Instead of guessing Bezier control points, a curved span that is well-approximated by a
//! circular arc is emitted AS an arc with a real center+radius. This follows the designer
//! "circle technique" (Albrecht Dürer letter construction, Apple golden-ratio circles):
//! curves are arcs, defined by 3 numbers (cx, cy, r), converted to exact cubics via
//! k = 4/3 * tan(theta/4). See docs/refs/circle-technique.md.

use visioncortex::PointF64;

/// A fitted circle: center + radius + max radial error over the input points.
#[derive(Debug, Clone, Copy)]
pub struct Circle {
    pub cx: f64,
    pub cy: f64,
    pub r: f64,
    pub max_err: f64,
}

/// Least-squares circle fit (deterministic 3x3 normal-equation solve).
/// From  x^2 + y^2 = 2a x + 2b y + c, with center (a,b) and r = sqrt(c + a^2 + b^2).
pub fn fit_circle(pts: &[PointF64]) -> Option<Circle> {
    let n = pts.len();
    if n < 3 {
        return None;
    }
    let (mut sx, mut sy, mut sxx, mut syy, mut sxy, mut sxz, mut syz, mut sz) =
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for p in pts {
        let z = p.x * p.x + p.y * p.y;
        sx += p.x;
        sy += p.y;
        sxx += p.x * p.x;
        syy += p.y * p.y;
        sxy += p.x * p.y;
        sxz += p.x * z;
        syz += p.y * z;
        sz += z;
    }
    let nf = n as f64;
    // Normal equations for [a, b, c] (a=2A, b=2B style folded): solve
    // | sxx sxy sx | |A|   | sxz |
    // | sxy syy sy | |B| = | syz |
    // | sx  sy  n  | |C|   | sz  |
    let m = [[sxx, sxy, sx], [sxy, syy, sy], [sx, sy, nf]];
    let rhs = [sxz, syz, sz];
    let sol = solve3(m, rhs)?;
    // model: z = A*x + B*y + C  with z=x^2+y^2  => center (A/2, B/2), r^2 = C + cx^2 + cy^2
    let cx = sol[0] / 2.0;
    let cy = sol[1] / 2.0;
    let r2 = sol[2] + cx * cx + cy * cy;
    if r2 <= 0.0 || !r2.is_finite() {
        return None;
    }
    let r = r2.sqrt();
    let mut max_err = 0.0_f64;
    for p in pts {
        let d = ((p.x - cx).powi(2) + (p.y - cy).powi(2)).sqrt();
        max_err = max_err.max((d - r).abs());
    }
    Some(Circle { cx, cy, r, max_err })
}

/// Solve a 3x3 linear system by Cramer's rule. Returns None if near-singular.
fn solve3(m: [[f64; 3]; 3], b: [f64; 3]) -> Option<[f64; 3]> {
    let det = det3(m);
    if det.abs() < 1e-9 {
        return None;
    }
    let mut out = [0.0; 3];
    for i in 0..3 {
        let mut mi = m;
        for r in 0..3 {
            mi[r][i] = b[r];
        }
        out[i] = det3(mi) / det;
    }
    Some(out)
}

fn det3(m: [[f64; 3]; 3]) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

/// Angle (radians) of point p relative to circle center.
fn angle_of(c: &Circle, p: PointF64) -> f64 {
    (p.y - c.cy).atan2(p.x - c.cx)
}

/// Emit a circular arc (center c, from angle a0 to a1 in the given sweep direction) as a
/// sequence of cubic Bezier segments, one per <=90deg quadrant, using the exact circle
/// control-point factor k = 4/3*tan(dtheta/4). Appends "C ..." commands to `out` starting
/// from the current point (which must already equal the arc's start point).
pub fn arc_to_cubics(c: &Circle, a0: f64, a1: f64, ccw: bool, precision: usize) -> String {
    use std::f64::consts::PI;
    let mut start = a0;
    // total sweep in [0, 2pi), in the chosen direction
    let mut sweep = if ccw { a1 - a0 } else { a0 - a1 };
    while sweep < 0.0 {
        sweep += 2.0 * PI;
    }
    if sweep < 1e-9 {
        return String::new();
    }
    let dir = if ccw { 1.0 } else { -1.0 };
    let max_step = PI / 2.0; // 90 degrees
    let n_seg = (sweep / max_step).ceil().max(1.0) as usize;
    let step = sweep / n_seg as f64;
    let k = 4.0 / 3.0 * (step / 4.0).tan();
    let mut d = String::new();
    for _ in 0..n_seg {
        let end = start + dir * step;
        let p0 = point_on(c, start);
        let p3 = point_on(c, end);
        // skip a sub-pixel arc segment (avoids tiny meaningless cubics)
        if ((p3.x - p0.x).powi(2) + (p3.y - p0.y).powi(2)).sqrt() < 1.5 {
            start = end;
            continue;
        }
        // unit tangents (perpendicular to radius), in sweep direction
        let t0 = tangent(start, ccw);
        let t1 = tangent(end, ccw);
        let c1 = PointF64 {
            x: p0.x + k * c.r * t0.x,
            y: p0.y + k * c.r * t0.y,
        };
        let c2 = PointF64 {
            x: p3.x - k * c.r * t1.x,
            y: p3.y - k * c.r * t1.y,
        };
        d.push_str(&format!(
            "C{} {} {} {} {} {} ",
            fnum(c1.x, precision),
            fnum(c1.y, precision),
            fnum(c2.x, precision),
            fnum(c2.y, precision),
            fnum(p3.x, precision),
            fnum(p3.y, precision)
        ));
        start = end;
    }
    d
}

fn point_on(c: &Circle, a: f64) -> PointF64 {
    PointF64 {
        x: c.cx + c.r * a.cos(),
        y: c.cy + c.r * a.sin(),
    }
}

/// Unit tangent at angle `a`, pointing in the sweep direction.
fn tangent(a: f64, ccw: bool) -> PointF64 {
    let (s, co) = (a.sin(), a.cos());
    if ccw {
        PointF64 { x: -s, y: co }
    } else {
        PointF64 { x: s, y: -co }
    }
}

fn fnum(v: f64, precision: usize) -> String {
    let s = format!("{:.*}", precision, v);
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

/// Try to represent a smooth span of contour points as a circular arc. Returns
/// (circle, start_angle, end_angle, ccw) if the span fits a circle within `tol` radial
/// error and spans a meaningful angle. The caller emits it via `arc_to_cubics`.
pub fn try_arc(pts: &[PointF64], tol: f64) -> Option<(Circle, f64, f64, bool)> {
    if pts.len() < 4 {
        return None;
    }
    let c = fit_circle(pts)?;
    if c.max_err > tol {
        return None;
    }
    let a0 = angle_of(&c, pts[0]);
    let a1 = angle_of(&c, pts[pts.len() - 1]);
    // determine sweep direction from the middle point
    let mid = angle_of(&c, pts[pts.len() / 2]);
    let ccw = {
        use std::f64::consts::PI;
        let norm = |mut x: f64| {
            while x < 0.0 {
                x += 2.0 * PI;
            }
            while x >= 2.0 * PI {
                x -= 2.0 * PI;
            }
            x
        };
        // is mid between a0..a1 going ccw?
        let ccw_sweep = norm(a1 - a0);
        let ccw_mid = norm(mid - a0);
        ccw_mid <= ccw_sweep
    };
    use std::f64::consts::PI;
    let norm = |mut x: f64| {
        while x < 0.0 {
            x += 2.0 * PI;
        }
        while x >= 2.0 * PI {
            x -= 2.0 * PI;
        }
        x
    };
    // require a minimum arc angle so we don't arc-ify near-straight spans
    let sweep = {
        let mut s = if ccw { a1 - a0 } else { a0 - a1 };
        while s < 0.0 {
            s += 2.0 * PI;
        }
        s
    };
    if !(0.20..=PI * 1.95).contains(&sweep) {
        // too small (treat as line) or a near-full circle (ambiguous endpoints; real spans
        // are corner-split so this only excludes degenerate full loops). Let kurbo handle it.
        return None;
    }
    // VALIDATE DIRECTION: every span point's angular position from a0 (in the chosen
    // direction) must lie monotonically within [0, sweep]. A wrong-way arc puts points
    // outside the sweep -> reject and let kurbo handle it. This kills the slash artifacts.
    let mut prev = 0.0;
    for (j, p) in pts.iter().enumerate() {
        let ai = angle_of(&c, *p);
        let rel = if ccw { norm(ai - a0) } else { norm(a0 - ai) };
        // allow a small overshoot tolerance at the very ends
        if rel > sweep + 0.15 {
            return None;
        }
        // monotonic progression (points must advance along the arc, not jump backward)
        if j > 0 && rel + 0.15 < prev {
            return None;
        }
        prev = rel;
    }
    Some((c, a0, a1, ccw))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f64, y: f64) -> PointF64 {
        PointF64 { x, y }
    }

    #[test]
    fn fits_a_circle_accurately() {
        let r = 50.0;
        let pts: Vec<PointF64> = (0..24)
            .map(|i| {
                let a = i as f64 / 24.0 * std::f64::consts::TAU;
                p(10.0 + r * a.cos(), 20.0 + r * a.sin())
            })
            .collect();
        let c = fit_circle(&pts).unwrap();
        assert!((c.cx - 10.0).abs() < 1e-6);
        assert!((c.cy - 20.0).abs() < 1e-6);
        assert!((c.r - r).abs() < 1e-6);
        assert!(c.max_err < 1e-6);
    }

    #[test]
    fn rejects_a_straight_line() {
        let pts: Vec<PointF64> = (0..10).map(|i| p(i as f64 * 5.0, 0.0)).collect();
        // a line either fails the circle fit (huge radius) or huge error; try_arc rejects it
        assert!(try_arc(&pts, 1.0).is_none());
    }

    #[test]
    fn quarter_arc_emits_one_cubic() {
        // quarter circle from angle 0 to pi/2
        let c = Circle {
            cx: 0.0,
            cy: 0.0,
            r: 100.0,
            max_err: 0.0,
        };
        let d = arc_to_cubics(&c, 0.0, std::f64::consts::FRAC_PI_2, true, 2);
        assert_eq!(d.matches('C').count(), 1, "a 90deg arc is one cubic: {d}");
    }
}
