//! Pure geometry of the join lines drawn on the canvas (no GTK), so it can be
//! unit-tested like the domain.

pub type Point = (f64, f64);

/// Cubic Bézier control points for a join line: horizontal tangents at both
/// ends, bending at the horizontal midpoint.
pub fn link_curve(from: Point, to: Point) -> [Point; 4] {
    let mid = (from.0 + to.0) / 2.0;
    [from, (mid, from.1), (mid, to.1), to]
}

pub fn bezier_point(p: &[Point; 4], t: f64) -> Point {
    let mt = 1.0 - t;
    let (a, b, c, d) = (
        mt.powi(3),
        3.0 * mt.powi(2) * t,
        3.0 * mt * t.powi(2),
        t.powi(3),
    );
    (
        a * p[0].0 + b * p[1].0 + c * p[2].0 + d * p[3].0,
        a * p[0].1 + b * p[1].1 + c * p[2].1 + d * p[3].1,
    )
}

/// X of the card edge a join line attaches to: the edge facing the other
/// card, so the line hugs the outside of both cards instead of cutting
/// across one of them. `own` and `facing` are `(x, width)` of each card.
pub fn facing_edge_x(own: (f64, f64), facing: (f64, f64)) -> f64 {
    let own_center = own.0 + own.1 / 2.0;
    let facing_center = facing.0 + facing.1 / 2.0;
    if own_center <= facing_center {
        own.0 + own.1
    } else {
        own.0
    }
}

/// `y` kept within `[top, top + height]`, inset so a line never attaches on
/// the very border; `y` as is when the span is too small to inset.
pub fn clamp_to_span(y: f64, top: f64, height: f64) -> f64 {
    const INSET: f64 = 6.0;
    if height <= 2.0 * INSET {
        return y;
    }
    y.clamp(top + INSET, top + height - INSET)
}

/// Index of the curve passing closest to `(x, y)`, if any lies within
/// `tolerance` pixels (sampled at 21 points per curve).
pub fn hit_test(curves: &[[Point; 4]], x: f64, y: f64, tolerance: f64) -> Option<usize> {
    let mut best = None;
    let mut best_dist = tolerance;
    for (index, curve) in curves.iter().enumerate() {
        for i in 0..=20 {
            let (px, py) = bezier_point(curve, i as f64 / 20.0);
            let dist = ((px - x).powi(2) + (py - y).powi(2)).sqrt();
            if dist < best_dist {
                best_dist = dist;
                best = Some(index);
            }
        }
    }
    best
}
