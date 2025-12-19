use super::core::{Point3, Tolerance, Vec3};

pub trait Curve3 {
    fn point_at(&self, t: f64) -> Point3;

    #[must_use]
    fn domain(&self) -> (f64, f64) {
        (0.0, 1.0)
    }

    #[must_use]
    fn is_closed(&self) -> bool {
        false
    }

    #[must_use]
    fn derivative_at(&self, t: f64) -> Vec3 {
        let (a, b) = self.domain();
        let span = b - a;
        if !span.is_finite() || span == 0.0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }

        let h = Tolerance::DERIVATIVE.relative_to(span);
        if !h.is_finite() || h == 0.0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }

        let t0 = (t - h).max(a);
        let t1 = (t + h).min(b);
        if t1 == t0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }

        let p0 = self.point_at(t0);
        let p1 = self.point_at(t1);
        p1.sub_point(p0).mul_scalar(1.0 / (t1 - t0))
    }

    #[must_use]
    fn second_derivative_at(&self, t: f64) -> Vec3 {
        let (a, b) = self.domain();
        let span = b - a;
        if !span.is_finite() || span == 0.0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }

        let h = Tolerance::SECOND_DERIVATIVE.relative_to(span);
        if !h.is_finite() || h == 0.0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }

        let t0 = (t - h).max(a);
        let t2 = (t + h).min(b);
        if t2 == t0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }
        let tm = 0.5 * (t0 + t2);
        let dt = tm - t0;
        if dt == 0.0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }

        let p0 = self.point_at(t0);
        let p1 = self.point_at(tm);
        let p2 = self.point_at(t2);
        vec3_from_points(p0, p1, p2).mul_scalar(1.0 / (dt * dt))
    }

    #[must_use]
    fn curvature_at(&self, t: f64) -> Option<f64> {
        let d1 = self.derivative_at(t);
        let d2 = self.second_derivative_at(t);
        let denom = d1.length();
        if denom <= 0.0 || !denom.is_finite() {
            return None;
        }
        let num = d1.cross(d2).length();
        let k = num / (denom * denom * denom);
        if k.is_finite() { Some(k) } else { None }
    }

    /// Returns the unit tangent vector at parameter `t`.
    /// Returns `None` if the derivative is zero or degenerate.
    #[must_use]
    fn tangent_at(&self, t: f64) -> Option<Vec3> {
        self.derivative_at(t).normalized()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line3 {
    pub start: Point3,
    pub end: Point3,
}

impl Line3 {
    #[must_use]
    pub const fn new(start: Point3, end: Point3) -> Self {
        Self { start, end }
    }

    #[must_use]
    pub const fn direction(self) -> Vec3 {
        self.end.sub_point(self.start)
    }
}

impl Curve3 for Line3 {
    fn point_at(&self, t: f64) -> Point3 {
        let dir = self.direction();
        self.start.add_vec(dir.mul_scalar(t))
    }

    fn derivative_at(&self, _t: f64) -> Vec3 {
        self.direction()
    }

    fn second_derivative_at(&self, _t: f64) -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Polyline3 {
    points: Vec<Point3>,
    cumulative_lengths: Vec<f64>,
    total_length: f64,
    closed: bool,
}

impl Polyline3 {
    pub fn new(mut points: Vec<Point3>, closed: bool) -> Result<Self, String> {
        if points.len() < 2 {
            return Err("polyline requires at least 2 points".to_string());
        }
        if closed && points.len() > 2 {
            if points.first() == points.last() {
                points.pop();
            }
        }

        let mut cumulative_lengths = Vec::with_capacity(points.len());
        cumulative_lengths.push(0.0);
        let mut total = 0.0;
        for window in points.windows(2) {
            total += window[1].sub_point(window[0]).length();
            cumulative_lengths.push(total);
        }

        if closed {
            total += points
                .first()
                .copied()
                .zip(points.last().copied())
                .map(|(first, last)| first.sub_point(last).length())
                .unwrap_or(0.0);
        }

        Ok(Self {
            points,
            cumulative_lengths,
            total_length: total,
            closed,
        })
    }

    #[must_use]
    pub fn points(&self) -> &[Point3] {
        &self.points
    }

    #[must_use]
    pub const fn is_closed(&self) -> bool {
        self.closed
    }
}

impl Curve3 for Polyline3 {
    fn point_at(&self, t: f64) -> Point3 {
        if self.points.len() == 1 {
            return self.points[0];
        }

        if self.total_length <= 0.0 || !self.total_length.is_finite() {
            return self.points[0];
        }

        let mut target = (t.clamp(0.0, 1.0)) * self.total_length;

        let last_index = self.points.len() - 1;
        if target >= self.cumulative_lengths[last_index] {
            if !self.closed {
                return self.points[last_index];
            }

            let last = self.points[last_index];
            let first = self.points[0];
            let segment_length = first.sub_point(last).length();
            if segment_length == 0.0 {
                return last;
            }
            let ratio = ((target - self.cumulative_lengths[last_index]) / segment_length).clamp(0.0, 1.0);
            return lerp_point(last, first, ratio);
        }

        let idx = match self
            .cumulative_lengths
            .binary_search_by(|value| value.total_cmp(&target))
        {
            Ok(i) => i,
            Err(i) => i.max(1) - 1,
        };

        let seg_start = self.points[idx];
        let seg_end = self.points[idx + 1];
        let seg_len = seg_end.sub_point(seg_start).length();
        if seg_len == 0.0 {
            return seg_start;
        }
        target -= self.cumulative_lengths[idx];
        lerp_point(seg_start, seg_end, (target / seg_len).clamp(0.0, 1.0))
    }

    fn is_closed(&self) -> bool {
        self.closed
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle3 {
    pub center: Point3,
    pub x_axis: Vec3,
    pub y_axis: Vec3,
    pub radius: f64,
}

impl Circle3 {
    #[must_use]
    pub fn new(center: Point3, normal: Vec3, radius: f64) -> Self {
        let (x_axis, y_axis) = frame_axes_from_normal(normal);
        Self {
            center,
            x_axis,
            y_axis,
            radius,
        }
    }

    #[must_use]
    pub fn from_center_xaxis_normal(center: Point3, x_axis: Vec3, normal: Vec3, radius: f64) -> Self {
        let (x_axis, y_axis) = frame_axes_from_xaxis_normal(x_axis, normal);
        Self {
            center,
            x_axis,
            y_axis,
            radius,
        }
    }
}

impl Curve3 for Circle3 {
    fn point_at(&self, t: f64) -> Point3 {
        let (t0, t1) = self.domain();
        if t == t1 {
            return self.point_at(t0);
        }
        let u = ((t - t0) / (t1 - t0)).clamp(0.0, 1.0);
        let angle = std::f64::consts::TAU * u;
        self.center
            .add_vec(self.x_axis.mul_scalar(self.radius * angle.cos()))
            .add_vec(self.y_axis.mul_scalar(self.radius * angle.sin()))
    }

    fn is_closed(&self) -> bool {
        true
    }

    fn derivative_at(&self, t: f64) -> Vec3 {
        let (t0, t1) = self.domain();
        if t1 == t0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }
        let u = ((t - t0) / (t1 - t0)).clamp(0.0, 1.0);
        let angle = std::f64::consts::TAU * u;
        let dtheta_dt = std::f64::consts::TAU / (t1 - t0);
        let dx = self.x_axis.mul_scalar(-self.radius * angle.sin());
        let dy = self.y_axis.mul_scalar(self.radius * angle.cos());
        dx.add(dy).mul_scalar(dtheta_dt)
    }

    fn second_derivative_at(&self, t: f64) -> Vec3 {
        let (t0, t1) = self.domain();
        if t1 == t0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }
        let u = ((t - t0) / (t1 - t0)).clamp(0.0, 1.0);
        let angle = std::f64::consts::TAU * u;
        let dtheta_dt = std::f64::consts::TAU / (t1 - t0);
        let dd = self
            .x_axis
            .mul_scalar(-self.radius * angle.cos())
            .add(self.y_axis.mul_scalar(-self.radius * angle.sin()));
        dd.mul_scalar(dtheta_dt * dtheta_dt)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arc3 {
    pub center: Point3,
    pub x_axis: Vec3,
    pub y_axis: Vec3,
    pub radius: f64,
    pub start_angle: f64,
    pub sweep_angle: f64,
}

impl Arc3 {
    #[must_use]
    pub fn new(center: Point3, normal: Vec3, radius: f64, start_angle: f64, sweep_angle: f64) -> Self {
        let (x_axis, y_axis) = frame_axes_from_normal(normal);
        Self {
            center,
            x_axis,
            y_axis,
            radius,
            start_angle,
            sweep_angle,
        }
    }

    #[must_use]
    pub fn from_center_xaxis_normal(
        center: Point3,
        x_axis: Vec3,
        normal: Vec3,
        radius: f64,
        start_angle: f64,
        sweep_angle: f64,
    ) -> Self {
        let (x_axis, y_axis) = frame_axes_from_xaxis_normal(x_axis, normal);
        Self {
            center,
            x_axis,
            y_axis,
            radius,
            start_angle,
            sweep_angle,
        }
    }
}

impl Curve3 for Arc3 {
    fn point_at(&self, t: f64) -> Point3 {
        let (t0, t1) = self.domain();
        let u = if t1 == t0 {
            0.0
        } else {
            ((t - t0) / (t1 - t0)).clamp(0.0, 1.0)
        };
        let angle = self.start_angle + self.sweep_angle * u;
        self.center
            .add_vec(self.x_axis.mul_scalar(self.radius * angle.cos()))
            .add_vec(self.y_axis.mul_scalar(self.radius * angle.sin()))
    }

    fn derivative_at(&self, t: f64) -> Vec3 {
        let (t0, t1) = self.domain();
        if t1 == t0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }
        let u = ((t - t0) / (t1 - t0)).clamp(0.0, 1.0);
        let angle = self.start_angle + self.sweep_angle * u;
        let dtheta_dt = self.sweep_angle / (t1 - t0);
        let dx = self.x_axis.mul_scalar(-self.radius * angle.sin());
        let dy = self.y_axis.mul_scalar(self.radius * angle.cos());
        dx.add(dy).mul_scalar(dtheta_dt)
    }

    fn second_derivative_at(&self, t: f64) -> Vec3 {
        let (t0, t1) = self.domain();
        if t1 == t0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }
        let u = ((t - t0) / (t1 - t0)).clamp(0.0, 1.0);
        let angle = self.start_angle + self.sweep_angle * u;
        let dtheta_dt = self.sweep_angle / (t1 - t0);
        let dd = self
            .x_axis
            .mul_scalar(-self.radius * angle.cos())
            .add(self.y_axis.mul_scalar(-self.radius * angle.sin()));
        dd.mul_scalar(dtheta_dt * dtheta_dt)
    }

    fn is_closed(&self) -> bool {
        // An arc is closed if its sweep covers a full circle (2π) within tolerance
        const FULL_CIRCLE_TOLERANCE: f64 = 1e-9;
        (self.sweep_angle.abs() - std::f64::consts::TAU).abs() < FULL_CIRCLE_TOLERANCE
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ellipse3 {
    pub center: Point3,
    pub x_axis: Vec3,
    pub y_axis: Vec3,
    pub radius_x: f64,
    pub radius_y: f64,
}

impl Ellipse3 {
    #[must_use]
    pub fn new(center: Point3, x_axis: Vec3, y_axis: Vec3, radius_x: f64, radius_y: f64) -> Self {
        let (x_axis, y_axis) = frame_axes_from_xy(x_axis, y_axis);
        Self {
            center,
            x_axis,
            y_axis,
            radius_x,
            radius_y,
        }
    }

    #[must_use]
    pub fn from_normal(center: Point3, normal: Vec3, radius_x: f64, radius_y: f64) -> Self {
        let (x_axis, y_axis) = frame_axes_from_normal(normal);
        Self {
            center,
            x_axis,
            y_axis,
            radius_x,
            radius_y,
        }
    }
}

impl Curve3 for Ellipse3 {
    fn point_at(&self, t: f64) -> Point3 {
        let (t0, t1) = self.domain();
        if t == t1 {
            return self.point_at(t0);
        }
        let u = ((t - t0) / (t1 - t0)).clamp(0.0, 1.0);
        let angle = std::f64::consts::TAU * u;
        self.center
            .add_vec(self.x_axis.mul_scalar(self.radius_x * angle.cos()))
            .add_vec(self.y_axis.mul_scalar(self.radius_y * angle.sin()))
    }

    fn is_closed(&self) -> bool {
        true
    }

    fn derivative_at(&self, t: f64) -> Vec3 {
        let (t0, t1) = self.domain();
        if t1 == t0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }
        let u = ((t - t0) / (t1 - t0)).clamp(0.0, 1.0);
        let angle = std::f64::consts::TAU * u;
        let dtheta_dt = std::f64::consts::TAU / (t1 - t0);
        let dx = self.x_axis.mul_scalar(-self.radius_x * angle.sin());
        let dy = self.y_axis.mul_scalar(self.radius_y * angle.cos());
        dx.add(dy).mul_scalar(dtheta_dt)
    }

    fn second_derivative_at(&self, t: f64) -> Vec3 {
        let (t0, t1) = self.domain();
        if t1 == t0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }
        let u = ((t - t0) / (t1 - t0)).clamp(0.0, 1.0);
        let angle = std::f64::consts::TAU * u;
        let dtheta_dt = std::f64::consts::TAU / (t1 - t0);
        let dd = self
            .x_axis
            .mul_scalar(-self.radius_x * angle.cos())
            .add(self.y_axis.mul_scalar(-self.radius_y * angle.sin()));
        dd.mul_scalar(dtheta_dt * dtheta_dt)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuadraticBezier3 {
    pub p0: Point3,
    pub p1: Point3,
    pub p2: Point3,
}

impl QuadraticBezier3 {
    #[must_use]
    pub const fn new(p0: Point3, p1: Point3, p2: Point3) -> Self {
        Self { p0, p1, p2 }
    }
}

impl Curve3 for QuadraticBezier3 {
    fn point_at(&self, t: f64) -> Point3 {
        let t = t.clamp(0.0, 1.0);
        let u = 1.0 - t;
        point_weighted_sum(
            self.p0,
            u * u,
            self.p1,
            2.0 * u * t,
            self.p2,
            t * t,
        )
    }

    fn derivative_at(&self, t: f64) -> Vec3 {
        let t = t.clamp(0.0, 1.0);
        let u = 1.0 - t;
        let a = self.p1.sub_point(self.p0);
        let b = self.p2.sub_point(self.p1);
        a.mul_scalar(2.0 * u).add(b.mul_scalar(2.0 * t))
    }

    fn second_derivative_at(&self, _t: f64) -> Vec3 {
        let p0 = self.p0;
        let p1 = self.p1;
        let p2 = self.p2;
        Vec3::new(
            2.0 * (p2.x - 2.0 * p1.x + p0.x),
            2.0 * (p2.y - 2.0 * p1.y + p0.y),
            2.0 * (p2.z - 2.0 * p1.z + p0.z),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubicBezier3 {
    pub p0: Point3,
    pub p1: Point3,
    pub p2: Point3,
    pub p3: Point3,
}

impl CubicBezier3 {
    #[must_use]
    pub const fn new(p0: Point3, p1: Point3, p2: Point3, p3: Point3) -> Self {
        Self { p0, p1, p2, p3 }
    }
}

impl Curve3 for CubicBezier3 {
    fn point_at(&self, t: f64) -> Point3 {
        let t = t.clamp(0.0, 1.0);
        let u = 1.0 - t;
        let u2 = u * u;
        let t2 = t * t;
        point_weighted_sum4(
            self.p0,
            u2 * u,
            self.p1,
            3.0 * u2 * t,
            self.p2,
            3.0 * u * t2,
            self.p3,
            t2 * t,
        )
    }

    fn derivative_at(&self, t: f64) -> Vec3 {
        let t = t.clamp(0.0, 1.0);
        let u = 1.0 - t;
        let a = self.p1.sub_point(self.p0);
        let b = self.p2.sub_point(self.p1);
        let c = self.p3.sub_point(self.p2);
        a.mul_scalar(3.0 * u * u)
            .add(b.mul_scalar(6.0 * u * t))
            .add(c.mul_scalar(3.0 * t * t))
    }

    fn second_derivative_at(&self, t: f64) -> Vec3 {
        let t = t.clamp(0.0, 1.0);
        let u = 1.0 - t;
        let a = vec3_bezier_second(self.p0, self.p1, self.p2);
        let b = vec3_bezier_second(self.p1, self.p2, self.p3);
        a.mul_scalar(6.0 * u).add(b.mul_scalar(6.0 * t))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NurbsCurve3 {
    pub degree: usize,
    pub control_points: Vec<Point3>,
    pub knots: Vec<f64>,
    pub weights: Option<Vec<f64>>,
}

impl NurbsCurve3 {
    pub fn new(
        degree: usize,
        control_points: Vec<Point3>,
        knots: Vec<f64>,
        weights: Option<Vec<f64>>,
    ) -> Result<Self, String> {
        if control_points.len() < 2 {
            return Err("nurbs curve requires at least 2 control points".to_string());
        }
        if degree == 0 {
            return Err("nurbs curve degree must be >= 1".to_string());
        }
        if degree >= control_points.len() {
            return Err("nurbs curve degree must be < control point count".to_string());
        }

        let expected_knot_len = control_points.len() + degree + 1;
        if knots.len() != expected_knot_len {
            return Err(format!(
                "nurbs curve knot length must be {}, got {}",
                expected_knot_len,
                knots.len()
            ));
        }

        if let Some(ref weights) = weights {
            if weights.len() != control_points.len() {
                return Err("nurbs curve weights length must match control point count".to_string());
            }
            if weights.iter().any(|w| !w.is_finite() || *w <= 0.0) {
                return Err("nurbs curve weights must be finite and > 0".to_string());
            }
        }

        if !is_non_decreasing(&knots) {
            return Err("nurbs curve knots must be non-decreasing".to_string());
        }

        Ok(Self {
            degree,
            control_points,
            knots,
            weights,
        })
    }

    #[must_use]
    pub fn knot_multiplicities(&self, tol: Tolerance) -> Vec<(f64, usize)> {
        if self.knots.is_empty() {
            return Vec::new();
        }

        let mut result = Vec::new();
        let mut current = self.knots[0];
        let mut count = 1usize;
        for &k in &self.knots[1..] {
            if tol.approx_eq_f64(k, current) {
                count += 1;
            } else {
                result.push((current, count));
                current = k;
                count = 1;
            }
        }
        result.push((current, count));
        result
    }

    #[must_use]
    pub fn continuity_order_at_knot(&self, knot: f64, tol: Tolerance) -> Option<usize> {
        let (a, b) = self.domain();
        if tol.approx_eq_f64(knot, a) || tol.approx_eq_f64(knot, b) {
            return None;
        }

        let multiplicity = self
            .knot_multiplicities(tol)
            .into_iter()
            .find(|(value, _count)| tol.approx_eq_f64(*value, knot))
            .map(|(_value, count)| count)?;

        if multiplicity >= self.degree + 1 {
            None
        } else {
            Some(self.degree - multiplicity)
        }
    }

    #[must_use]
    pub fn is_tangent_continuous_at_knot(&self, knot: f64, tol: Tolerance) -> bool {
        self.continuity_order_at_knot(knot, tol)
            .is_some_and(|order| order >= 1)
    }

    #[must_use]
    pub fn is_closed_with_tol(&self, tol: Tolerance) -> bool {
        let (a, b) = self.domain();
        let span = b - a;
        if !span.is_finite() || span == 0.0 {
            return false;
        }
        tol.approx_eq_point3(self.point_at(a), self.point_at(b))
    }
}

impl Curve3 for NurbsCurve3 {
    fn point_at(&self, t: f64) -> Point3 {
        if self.control_points.is_empty() {
            return Point3::new(0.0, 0.0, 0.0);
        }

        let p = self.degree;
        if p == 0 || p >= self.control_points.len() {
            return self.control_points[0];
        }

        let expected_knot_len = self.control_points.len() + p + 1;
        if self.control_points.len() < 2 || self.knots.len() != expected_knot_len || !is_non_decreasing(&self.knots) {
            return self.control_points[0];
        }

        let (a, b) = self.domain();
        let u = if t <= a {
            a
        } else if t >= b {
            b
        } else {
            t
        };

        let n = self.control_points.len() - 1;
        let span = find_span(n, p, u, &self.knots);

        if let Some(weights) = self
            .weights
            .as_ref()
            .filter(|weights| weights.len() == self.control_points.len())
        {
            let mut d = Vec::with_capacity(p + 1);
            for j in 0..=p {
                let index = span - p + j;
                let w = weights[index];
                let point = self.control_points[index];
                d.push(HPoint4::new(point.x * w, point.y * w, point.z * w, w));
            }
            de_boor(&mut d, span, p, u, &self.knots);
            d[p].to_point3().unwrap_or_else(|| self.control_points[0])
        } else {
            let mut d = Vec::with_capacity(p + 1);
            for j in 0..=p {
                let index = span - p + j;
                let point = self.control_points[index];
                d.push(HPoint4::new(point.x, point.y, point.z, 1.0));
            }
            de_boor(&mut d, span, p, u, &self.knots);
            Point3::new(d[p].x, d[p].y, d[p].z)
        }
    }

    fn domain(&self) -> (f64, f64) {
        if self.control_points.is_empty() || self.knots.is_empty() {
            return (0.0, 0.0);
        }

        let p = self.degree;
        let expected_knot_len = self.control_points.len() + p + 1;
        if p == 0
            || p >= self.control_points.len()
            || self.knots.len() != expected_knot_len
            || !is_non_decreasing(&self.knots)
        {
            return (0.0, 0.0);
        }

        let start = self.knots[p];
        let end = self.knots[self.control_points.len()];
        (start, end)
    }

    fn is_closed(&self) -> bool {
        self.is_closed_with_tol(Tolerance::default_geom())
    }

    fn derivative_at(&self, t: f64) -> Vec3 {
        // Analytic derivative using de Boor algorithm on the hodograph (derivative curve)
        // For a B-spline of degree p, the derivative is a B-spline of degree p-1
        if self.control_points.is_empty() || self.degree == 0 {
            return Vec3::new(0.0, 0.0, 0.0);
        }

        let p = self.degree;
        if p >= self.control_points.len() {
            return Vec3::new(0.0, 0.0, 0.0);
        }

        let expected_knot_len = self.control_points.len() + p + 1;
        if self.control_points.len() < 2
            || self.knots.len() != expected_knot_len
            || !is_non_decreasing(&self.knots)
        {
            return Vec3::new(0.0, 0.0, 0.0);
        }

        let (a, b) = self.domain();
        let u = t.clamp(a, b);

        let n = self.control_points.len() - 1;
        let span = find_span(n, p, u, &self.knots);

        // Evaluate curve and first derivative using de Boor with derivative extension
        if let Some(weights) = self
            .weights
            .as_ref()
            .filter(|w| w.len() == self.control_points.len())
        {
            // Rational case: use quotient rule on homogeneous coordinates
            // C(u) = A(u) / w(u) where A is the vector part and w is the weight
            // C'(u) = (A'(u) * w(u) - A(u) * w'(u)) / w(u)^2
            let mut d = Vec::with_capacity(p + 1);
            for j in 0..=p {
                let index = span - p + j;
                let w = weights[index];
                let pt = self.control_points[index];
                d.push(HPoint4::new(pt.x * w, pt.y * w, pt.z * w, w));
            }

            // Compute derivative control points for the homogeneous curve
            let mut d_prime = Vec::with_capacity(p);
            for j in 0..p {
                let i = span - p + j;
                let denom = self.knots[i + p + 1] - self.knots[i + 1];
                let factor = if denom.abs() > 1e-14 {
                    (p as f64) / denom
                } else {
                    0.0
                };
                d_prime.push(HPoint4::new(
                    (d[j + 1].x - d[j].x) * factor,
                    (d[j + 1].y - d[j].y) * factor,
                    (d[j + 1].z - d[j].z) * factor,
                    (d[j + 1].w - d[j].w) * factor,
                ));
            }

            // Evaluate the curve value
            de_boor(&mut d, span, p, u, &self.knots);
            let curve_val = d[p];

            // Evaluate the derivative of homogeneous curve (degree p-1)
            if p >= 1 && !d_prime.is_empty() {
                de_boor(&mut d_prime, span, p - 1, u, &self.knots);
                let deriv_hom = d_prime[p - 1];

                // Apply quotient rule: C'(u) = (A' * w - A * w') / w^2
                let w = curve_val.w;
                let w_prime = deriv_hom.w;
                if w.abs() > 1e-14 {
                    let w_sq = w * w;
                    return Vec3::new(
                        (deriv_hom.x * w - curve_val.x * w_prime) / w_sq,
                        (deriv_hom.y * w - curve_val.y * w_prime) / w_sq,
                        (deriv_hom.z * w - curve_val.z * w_prime) / w_sq,
                    );
                }
            }
            Vec3::new(0.0, 0.0, 0.0)
        } else {
            // Non-rational case: direct derivative of B-spline
            // The derivative control points are: Q_i = p * (P_{i+1} - P_i) / (u_{i+p+1} - u_{i+1})
            if p == 0 {
                return Vec3::new(0.0, 0.0, 0.0);
            }

            let mut d_prime = Vec::with_capacity(p);
            for j in 0..p {
                let i = span - p + j;
                let denom = self.knots[i + p + 1] - self.knots[i + 1];
                let factor = if denom.abs() > 1e-14 {
                    (p as f64) / denom
                } else {
                    0.0
                };
                let p0 = self.control_points[i];
                let p1 = self.control_points[i + 1];
                d_prime.push(HPoint4::new(
                    (p1.x - p0.x) * factor,
                    (p1.y - p0.y) * factor,
                    (p1.z - p0.z) * factor,
                    1.0,
                ));
            }

            de_boor(&mut d_prime, span, p - 1, u, &self.knots);
            Vec3::new(d_prime[p - 1].x, d_prime[p - 1].y, d_prime[p - 1].z)
        }
    }
}

#[must_use]
pub fn tessellate_curve_uniform(curve: &impl Curve3, steps: usize) -> Vec<Point3> {
    let steps = steps.max(1);
    let denom = steps as f64;
    let (t0, t1) = curve.domain();
    let span = t1 - t0;
    if !span.is_finite() || span == 0.0 {
        return vec![curve.point_at(t0)];
    }
    if curve.is_closed() {
        (0..steps)
            .map(|i| {
                let u = i as f64 / denom;
                curve.point_at(t0 + span * u)
            })
            .collect()
    } else {
        (0..=steps)
            .map(|i| {
                let u = i as f64 / denom;
                curve.point_at(t0 + span * u)
            })
            .collect()
    }
}

fn lerp_point(a: Point3, b: Point3, t: f64) -> Point3 {
    Point3::new(
        a.x + (b.x - a.x) * t,
        a.y + (b.y - a.y) * t,
        a.z + (b.z - a.z) * t,
    )
}

fn point_weighted_sum(p0: Point3, w0: f64, p1: Point3, w1: f64, p2: Point3, w2: f64) -> Point3 {
    Point3::new(
        p0.x * w0 + p1.x * w1 + p2.x * w2,
        p0.y * w0 + p1.y * w1 + p2.y * w2,
        p0.z * w0 + p1.z * w1 + p2.z * w2,
    )
}

fn point_weighted_sum4(
    p0: Point3,
    w0: f64,
    p1: Point3,
    w1: f64,
    p2: Point3,
    w2: f64,
    p3: Point3,
    w3: f64,
) -> Point3 {
    Point3::new(
        p0.x * w0 + p1.x * w1 + p2.x * w2 + p3.x * w3,
        p0.y * w0 + p1.y * w1 + p2.y * w2 + p3.y * w3,
        p0.z * w0 + p1.z * w1 + p2.z * w2 + p3.z * w3,
    )
}

fn vec3_bezier_second(p0: Point3, p1: Point3, p2: Point3) -> Vec3 {
    Vec3::new(p2.x - 2.0 * p1.x + p0.x, p2.y - 2.0 * p1.y + p0.y, p2.z - 2.0 * p1.z + p0.z)
}

fn vec3_from_points(p0: Point3, p1: Point3, p2: Point3) -> Vec3 {
    Vec3::new(
        p0.x - 2.0 * p1.x + p2.x,
        p0.y - 2.0 * p1.y + p2.y,
        p0.z - 2.0 * p1.z + p2.z,
    )
}

fn frame_axes_from_normal(normal: Vec3) -> (Vec3, Vec3) {
    let z = normal.normalized().unwrap_or_else(|| Vec3::new(0.0, 0.0, 1.0));
    let x = orthogonal_unit_vector(z);
    let y = z.cross(x).normalized().unwrap_or_else(|| Vec3::new(0.0, 1.0, 0.0));
    (x, y)
}

fn frame_axes_from_xaxis_normal(x_axis: Vec3, normal: Vec3) -> (Vec3, Vec3) {
    let z = normal.normalized().unwrap_or_else(|| Vec3::new(0.0, 0.0, 1.0));
    let projected = x_axis.sub(z.mul_scalar(x_axis.dot(z)));
    let x = projected
        .normalized()
        .unwrap_or_else(|| orthogonal_unit_vector(z));
    let y = z.cross(x).normalized().unwrap_or_else(|| Vec3::new(0.0, 1.0, 0.0));
    (x, y)
}

fn frame_axes_from_xy(x_axis: Vec3, y_axis: Vec3) -> (Vec3, Vec3) {
    let x = x_axis.normalized().unwrap_or_else(|| Vec3::new(1.0, 0.0, 0.0));
    let z = x.cross(y_axis)
        .normalized()
        .unwrap_or_else(|| Vec3::new(0.0, 0.0, 1.0));
    let y = z.cross(x).normalized().unwrap_or_else(|| Vec3::new(0.0, 1.0, 0.0));
    (x, y)
}

fn orthogonal_unit_vector(reference: Vec3) -> Vec3 {
    let candidate = if reference.x.abs() < reference.y.abs() {
        Vec3::new(0.0, -reference.z, reference.y)
    } else {
        Vec3::new(-reference.z, 0.0, reference.x)
    };

    candidate
        .normalized()
        .unwrap_or_else(|| Vec3::new(1.0, 0.0, 0.0))
}

fn is_non_decreasing(knots: &[f64]) -> bool {
    knots.windows(2).all(|w| w[0] <= w[1])
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct HPoint4 {
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}

impl HPoint4 {
    const fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }

    fn lerp(self, rhs: Self, t: f64) -> Self {
        let s = 1.0 - t;
        Self::new(
            self.x * s + rhs.x * t,
            self.y * s + rhs.y * t,
            self.z * s + rhs.z * t,
            self.w * s + rhs.w * t,
        )
    }

    fn to_point3(self) -> Option<Point3> {
        if self.w.is_finite() && self.w != 0.0 {
            Some(Point3::new(self.x / self.w, self.y / self.w, self.z / self.w))
        } else {
            None
        }
    }
}

fn find_span(n: usize, p: usize, u: f64, knots: &[f64]) -> usize {
    if u >= knots[n + 1] {
        return n;
    }
    if u <= knots[p] {
        return p;
    }

    let mut low = p;
    let mut high = n + 1;
    let mut mid = (low + high) / 2;
    while u < knots[mid] || u >= knots[mid + 1] {
        if u < knots[mid] {
            high = mid;
        } else {
            low = mid;
        }
        mid = (low + high) / 2;
    }
    mid
}

fn de_boor(d: &mut [HPoint4], span: usize, p: usize, u: f64, knots: &[f64]) {
    for r in 1..=p {
        for j in (r..=p).rev() {
            let i = span - p + j;
            let denom = knots[i + p + 1 - r] - knots[i];
            let alpha = if denom == 0.0 { 0.0 } else { (u - knots[i]) / denom };
            d[j] = d[j - 1].lerp(d[j], alpha);
        }
    }
}
