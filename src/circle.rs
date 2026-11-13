#[derive(Clone, Copy)]
struct Circle { x: f64, y: f64, r2: f64 }

impl Circle {
    fn contains(self, p: [f64; 2]) -> bool {
        let dx = p[0] - self.x;
        let dy = p[1] - self.y;
        let scale = self.r2.max(dx * dx + dy * dy).max(1.0);
        dx * dx + dy * dy <= self.r2 + 1e-12 * scale
    }
}

fn diameter(a: [f64; 2], b: [f64; 2]) -> Circle {
    let x = (a[0] + b[0]) * 0.5;
    let y = (a[1] + b[1]) * 0.5;
    Circle { x, y, r2: (a[0] - x).powi(2) + (a[1] - y).powi(2) }
}

fn circumcircle(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> Option<Circle> {
    let d = 2.0 * (a[0] * (b[1] - c[1]) + b[0] * (c[1] - a[1]) + c[0] * (a[1] - b[1]));
    let scale = (a[0].abs() + a[1].abs() + b[0].abs() + b[1].abs() + c[0].abs() + c[1].abs()).max(1.0);
    if d.abs() <= 1e-14 * scale * scale { return None; }
    let aa = a[0] * a[0] + a[1] * a[1];
    let bb = b[0] * b[0] + b[1] * b[1];
    let cc = c[0] * c[0] + c[1] * c[1];
    let x = (aa * (b[1] - c[1]) + bb * (c[1] - a[1]) + cc * (a[1] - b[1])) / d;
    let y = (aa * (c[0] - b[0]) + bb * (a[0] - c[0]) + cc * (b[0] - a[0])) / d;
    Some(Circle { x, y, r2: (x - a[0]).powi(2) + (y - a[1]).powi(2) })
}

/// Plus petit cercle englobant (algorithme incrémental, ordre pseudo-aléatoire déterministe).
pub(crate) fn minimum_enclosing_circle(points: &[[f64; 2]]) -> ([f64; 2], f64) {
    if points.is_empty() { return ([0.0, 0.0], 0.0); }
    let mut shuffled = points.to_vec();
    let mut state = (points.len() as u64).wrapping_add(0x9e3779b97f4a7c15);
    for i in (1..shuffled.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        shuffled.swap(i, (state as usize) % (i + 1));
    }

    let mut circle: Option<Circle> = None;
    for i in 0..shuffled.len() {
        if circle.map_or(false, |c| c.contains(shuffled[i])) { continue; }
        circle = Some(Circle { x: shuffled[i][0], y: shuffled[i][1], r2: 0.0 });
        for j in 0..i {
            if circle.unwrap().contains(shuffled[j]) { continue; }
            circle = Some(diameter(shuffled[i], shuffled[j]));
            for k in 0..j {
                if circle.unwrap().contains(shuffled[k]) { continue; }
                circle = Some(circumcircle(shuffled[i], shuffled[j], shuffled[k]).unwrap_or_else(|| {
                    let candidates = [diameter(shuffled[i], shuffled[j]), diameter(shuffled[i], shuffled[k]), diameter(shuffled[j], shuffled[k])];
                    *candidates.iter().max_by(|a, b| a.r2.total_cmp(&b.r2)).unwrap()
                }));
            }
        }
    }
    let c = circle.unwrap();
    ([c.x, c.y], c.r2.max(0.0).sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn circle_through_triangle() {
        let (center, radius) = minimum_enclosing_circle(&[[0.0, 0.0], [2.0, 0.0], [1.0, 1.0]]);
        assert!((center[0] - 1.0).abs() < 1e-10);
        assert!((center[1] - 0.0).abs() < 1e-10);
        assert!((radius - 1.0).abs() < 1e-10);
    }

    #[test]
    fn obtuse_triangle_uses_longest_side() {
        let (center, radius) = minimum_enclosing_circle(&[[0.0, 0.0], [4.0, 0.0], [1.0, 0.5]]);
        assert!((center[0] - 2.0).abs() < 1e-10);
        assert!(radius >= 2.0);
    }
}
