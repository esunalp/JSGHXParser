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

    /// Creates a B-spline curve that interpolates exactly through the given points.
    ///
    /// Uses chord-length parameterization and global curve interpolation (solving a
    /// tridiagonal system) to compute control points such that the resulting curve
    /// passes through all input points.
    ///
    /// # Arguments
    /// * `points` - Points to interpolate through (at least 2 required).
    /// * `degree` - Desired curve degree (1 = linear, 2 = quadratic, 3 = cubic, etc.).
    ///              Will be clamped to valid range [1, n-1] where n = points.len().
    /// * `closed` - If true, creates a closed (periodic) curve that smoothly wraps around.
    ///
    /// # Returns
    /// A [`NurbsCurve3`] that passes through all input points, or an error if construction fails.
    ///
    /// # Example
    /// ```ignore
    /// use ghx_engine::geom::{Point3, NurbsCurve3};
    ///
    /// let points = vec![
    ///     Point3::new(0.0, 0.0, 0.0),
    ///     Point3::new(1.0, 1.0, 0.0),
    ///     Point3::new(2.0, 0.0, 0.0),
    ///     Point3::new(3.0, 1.0, 0.0),
    /// ];
    /// let curve = NurbsCurve3::interpolate_through_points(&points, 3, false).unwrap();
    /// // The curve now passes exactly through each of the 4 points
    /// ```
    pub fn interpolate_through_points(
        points: &[Point3],
        degree: usize,
        closed: bool,
    ) -> Result<Self, String> {
        if points.len() < 2 {
            return Err("interpolation requires at least 2 points".to_string());
        }

        // For only 2 points, use linear interpolation (degree 1)
        if points.len() == 2 {
            let knots = vec![0.0, 0.0, 1.0, 1.0];
            return Self::new(1, points.to_vec(), knots, None);
        }

        let n = points.len();
        // Clamp degree to valid range: must be >= 1 and < n
        let p = degree.clamp(1, n - 1);

        if closed {
            Self::interpolate_closed(points, p)
        } else {
            Self::interpolate_open(points, p)
        }
    }

    /// Open curve interpolation using global curve fitting.
    fn interpolate_open(points: &[Point3], degree: usize) -> Result<Self, String> {
        let p = degree;

        // Step 1: Compute chord-length parameterization
        let params = chord_length_parameters(points, false);

        // Step 2: Compute knot vector using averaging method
        let knots = averaging_knot_vector(&params, p);

        // Step 3: Solve for control points using global interpolation
        let control_points = solve_interpolation_system(points, &params, &knots, p)?;

        Self::new(p, control_points, knots, None)
    }

    /// Closed curve interpolation (periodic B-spline).
    fn interpolate_closed(points: &[Point3], degree: usize) -> Result<Self, String> {
        let n = points.len();
        let p = degree;

        // For closed curves, we need to wrap the points and solve a cyclic system.
        // We add `degree` copies of points at the end to make it periodic.

        // Wrap points by appending first `degree` points at the end
        let mut wrapped_points = points.to_vec();
        for i in 0..p.min(n) {
            wrapped_points.push(points[i]);
        }

        // Compute parameters for wrapped points
        let wrapped_params = chord_length_parameters(&wrapped_points, false);

        // Compute knot vector
        let knots = averaging_knot_vector(&wrapped_params, p);

        // Solve the interpolation system
        let control_points = solve_interpolation_system(&wrapped_points, &wrapped_params, &knots, p)?;

        Self::new(p, control_points, knots, None)
    }
}

/// Computes chord-length parameterization for a set of points.
///
/// Returns parameter values in [0, 1] where each parameter is proportional to
/// the cumulative chord length from the first point.
fn chord_length_parameters(points: &[Point3], closed: bool) -> Vec<f64> {
    if points.len() < 2 {
        return if points.is_empty() { vec![] } else { vec![0.0] };
    }

    let mut lengths = Vec::with_capacity(points.len());
    lengths.push(0.0);

    let mut total = 0.0;
    for window in points.windows(2) {
        let dist = window[1].sub_point(window[0]).length();
        total += dist;
        lengths.push(total);
    }

    // For closed curves, add the closing segment
    if closed && points.len() > 2 {
        let closing_dist = points[0].sub_point(points[points.len() - 1]).length();
        total += closing_dist;
    }

    // Normalize to [0, 1]
    if total > 0.0 {
        for length in &mut lengths {
            *length /= total;
        }
    }

    lengths
}

/// Computes knot vector using the averaging method.
///
/// For a degree-p B-spline interpolating n points with parameters t_0 ... t_{n-1},
/// the interior knots are computed as:
///   u_{j+p} = (t_j + t_{j+1} + ... + t_{j+p-1}) / p  for j = 1, ..., n-p-1
fn averaging_knot_vector(params: &[f64], degree: usize) -> Vec<f64> {
    let n = params.len();
    let p = degree;

    // Total knots: n + p + 1
    let knot_count = n + p + 1;
    let mut knots = Vec::with_capacity(knot_count);

    // Clamped start: p+1 zeros
    for _ in 0..=p {
        knots.push(0.0);
    }

    // Interior knots computed by averaging
    // j goes from 1 to n-p-1 (inclusive)
    // For each interior knot index i (where i = p+j for j = 1..n-p-1),
    // we average params[j..j+p]
    let interior_count = if n > p + 1 { n - p - 1 } else { 0 };
    for j in 1..=interior_count {
        let mut sum = 0.0;
        for i in j..(j + p) {
            if i < params.len() {
                sum += params[i];
            }
        }
        knots.push(sum / p as f64);
    }

    // Clamped end: p+1 ones
    for _ in 0..=p {
        knots.push(1.0);
    }

    knots
}

/// Solves the global curve interpolation system to find control points.
///
/// Given n data points Q_0 ... Q_{n-1} at parameters t_0 ... t_{n-1},
/// finds control points P_0 ... P_{n-1} such that C(t_i) = Q_i.
///
/// For open clamped B-splines, the coefficient matrix is banded and we can
/// solve efficiently using LU decomposition of the band matrix.
fn solve_interpolation_system(
    data_points: &[Point3],
    params: &[f64],
    knots: &[f64],
    degree: usize,
) -> Result<Vec<Point3>, String> {
    let n = data_points.len();
    let p = degree;

    if n < 2 {
        return Err("need at least 2 points for interpolation".to_string());
    }

    // For clamped B-splines, the first and last data points are exactly the
    // first and last control points
    if n == 2 {
        return Ok(data_points.to_vec());
    }

    // Build the coefficient matrix N where N[i][j] = N_{j,p}(t_i)
    // This is a banded matrix with bandwidth p+1
    let mut matrix = vec![vec![0.0; n]; n];
    for i in 0..n {
        let t = params[i];
        for j in 0..n {
            matrix[i][j] = basis_function(j, p, t, knots);
        }
    }

    // Solve the linear system for each coordinate (x, y, z)
    let mut control_x = vec![0.0; n];
    let mut control_y = vec![0.0; n];
    let mut control_z = vec![0.0; n];

    let rhs_x: Vec<f64> = data_points.iter().map(|pt| pt.x).collect();
    let rhs_y: Vec<f64> = data_points.iter().map(|pt| pt.y).collect();
    let rhs_z: Vec<f64> = data_points.iter().map(|pt| pt.z).collect();

    solve_linear_system(&matrix, &rhs_x, &mut control_x)?;
    solve_linear_system(&matrix, &rhs_y, &mut control_y)?;
    solve_linear_system(&matrix, &rhs_z, &mut control_z)?;

    let control_points: Vec<Point3> = (0..n)
        .map(|i| Point3::new(control_x[i], control_y[i], control_z[i]))
        .collect();

    Ok(control_points)
}

/// Computes the B-spline basis function N_{i,p}(t) using the Cox-de Boor recursion.
fn basis_function(i: usize, p: usize, t: f64, knots: &[f64]) -> f64 {
    // Base case: degree 0
    if p == 0 {
        if i + 1 < knots.len() && t >= knots[i] && t < knots[i + 1] {
            return 1.0;
        }
        // Handle the right endpoint (t == last knot)
        if i + 1 < knots.len() && (t - knots[i + 1]).abs() < 1e-14 && t >= knots[i] {
            return 1.0;
        }
        return 0.0;
    }

    // Recursive case
    let mut result = 0.0;

    // Left term: (t - t_i) / (t_{i+p} - t_i) * N_{i,p-1}(t)
    if i + p < knots.len() {
        let denom1 = knots[i + p] - knots[i];
        if denom1.abs() > 1e-14 {
            result += (t - knots[i]) / denom1 * basis_function(i, p - 1, t, knots);
        }
    }

    // Right term: (t_{i+p+1} - t) / (t_{i+p+1} - t_{i+1}) * N_{i+1,p-1}(t)
    if i + p + 1 < knots.len() && i + 1 < knots.len() {
        let denom2 = knots[i + p + 1] - knots[i + 1];
        if denom2.abs() > 1e-14 {
            result += (knots[i + p + 1] - t) / denom2 * basis_function(i + 1, p - 1, t, knots);
        }
    }

    result
}

/// Solves a dense linear system Ax = b using Gaussian elimination with partial pivoting.
fn solve_linear_system(matrix: &[Vec<f64>], rhs: &[f64], result: &mut [f64]) -> Result<(), String> {
    let n = matrix.len();
    if n == 0 || rhs.len() != n || result.len() != n {
        return Err("invalid matrix dimensions".to_string());
    }

    // Create augmented matrix
    let mut aug: Vec<Vec<f64>> = matrix
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let mut r = row.clone();
            r.push(rhs[i]);
            r
        })
        .collect();

    // Forward elimination with partial pivoting
    for k in 0..n {
        // Find pivot
        let mut max_row = k;
        let mut max_val = aug[k][k].abs();
        for i in (k + 1)..n {
            if aug[i][k].abs() > max_val {
                max_val = aug[i][k].abs();
                max_row = i;
            }
        }

        if max_val < 1e-14 {
            return Err("singular matrix in interpolation system".to_string());
        }

        // Swap rows
        if max_row != k {
            aug.swap(k, max_row);
        }

        // Eliminate column k
        let pivot = aug[k][k];
        for i in (k + 1)..n {
            let factor = aug[i][k] / pivot;
            for j in k..=n {
                aug[i][j] -= factor * aug[k][j];
            }
        }
    }

    // Back substitution
    for i in (0..n).rev() {
        let mut sum = aug[i][n];
        for j in (i + 1)..n {
            sum -= aug[i][j] * result[j];
        }
        if aug[i][i].abs() < 1e-14 {
            return Err("singular matrix during back substitution".to_string());
        }
        result[i] = sum / aug[i][i];
    }

    Ok(())
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
    let include_end = !curve.is_closed();
    let params = curve_parameters_by_count(curve, steps, include_end);
    params.into_iter().map(|t| curve.point_at(t)).collect()
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

// ============================================================================
// Curve Division and Sampling Utilities
// ============================================================================

/// Result of sampling a curve at a parameter.
#[derive(Debug, Clone, Copy)]
pub struct CurveSample {
    /// The point on the curve at the given parameter.
    pub point: Point3,
    /// The tangent vector at the given parameter (unit length if derivable, else zero).
    pub tangent: Vec3,
    /// The parameter value at which the sample was taken.
    pub parameter: f64,
}

/// Result of dividing a curve.
#[derive(Debug, Clone)]
pub struct CurveDivisionResult {
    /// Points at each division.
    pub points: Vec<Point3>,
    /// Tangent vectors at each division.
    pub tangents: Vec<Vec3>,
    /// Parameter values at each division.
    pub parameters: Vec<f64>,
}

/// A subcurve extracted from a parent curve.
#[derive(Debug, Clone)]
pub struct SubCurve {
    /// Points defining the subcurve (tessellated).
    pub points: Vec<Point3>,
    /// Start parameter on the parent curve.
    pub start_param: f64,
    /// End parameter on the parent curve.
    pub end_param: f64,
}

/// Frame data representing an oriented coordinate system at a point on a curve.
#[derive(Debug, Clone, Copy)]
pub struct CurveFrame {
    /// Origin of the frame (point on curve).
    pub origin: Point3,
    /// X-axis (tangent direction).
    pub x_axis: Vec3,
    /// Y-axis (normal direction).
    pub y_axis: Vec3,
    /// Z-axis (binormal direction).
    pub z_axis: Vec3,
}

/// Samples a curve at a given parameter value.
///
/// # Arguments
/// * `curve` - The curve to sample.
/// * `t` - Parameter value (will be clamped to curve domain).
///
/// # Returns
/// A [`CurveSample`] containing the point, tangent, and parameter.
#[must_use]
pub fn sample_curve_at<C: Curve3>(curve: &C, t: f64) -> CurveSample {
    let (t0, t1) = curve.domain();
    let param = t.clamp(t0, t1);
    let point = curve.point_at(param);
    let tangent = curve.tangent_at(param).unwrap_or_else(|| Vec3::new(0.0, 0.0, 0.0));
    CurveSample {
        point,
        tangent,
        parameter: param,
    }
}

/// Computes the approximate arc length of a curve by sampling.
///
/// # Arguments
/// * `curve` - The curve to measure.
/// * `samples` - Number of sample segments (more = more accurate).
///
/// # Returns
/// The approximate arc length.
#[must_use]
pub fn curve_arc_length<C: Curve3>(curve: &C, samples: usize) -> f64 {
    let samples = samples.max(1);
    let (t0, t1) = curve.domain();
    let span = t1 - t0;
    if !span.is_finite() || span == 0.0 {
        return 0.0;
    }

    let mut length = 0.0;
    let mut prev = curve.point_at(t0);
    for i in 1..=samples {
        let t = t0 + span * (i as f64 / samples as f64);
        let curr = curve.point_at(t);
        length += curr.sub_point(prev).length();
        prev = curr;
    }
    length
}

/// Divides a curve into a specified number of equal arc-length segments.
///
/// # Arguments
/// * `curve` - The curve to divide.
/// * `count` - Number of segments (output will have `count + 1` division points).
///
/// # Returns
/// A [`CurveDivisionResult`] containing points, tangents, and parameters.
#[must_use]
pub fn divide_curve_by_count<C: Curve3>(curve: &C, count: usize) -> CurveDivisionResult {
    let count = count.max(1);
    let mut points = Vec::with_capacity(count + 1);
    let mut tangents = Vec::with_capacity(count + 1);
    let mut parameters = Vec::with_capacity(count + 1);

    let sample_params = curve_parameters_by_count(curve, count, true);
    for t in sample_params {
        let sample = sample_curve_at(curve, t);
        points.push(sample.point);
        tangents.push(sample.tangent);
        parameters.push(sample.parameter);
    }

    CurveDivisionResult {
        points,
        tangents,
        parameters,
    }
}

/// Divides a curve by a target arc-length distance between points.
///
/// # Arguments
/// * `curve` - The curve to divide.
/// * `distance` - Target distance between consecutive division points.
/// * `samples_per_segment` - Samples used for arc-length estimation per segment.
///
/// # Returns
/// A [`CurveDivisionResult`] containing points, tangents, and parameters.
/// The last point is always the curve endpoint.
#[must_use]
pub fn divide_curve_by_distance<C: Curve3>(
    curve: &C,
    distance: f64,
    samples_per_segment: usize,
) -> CurveDivisionResult {
    if distance <= 0.0 || !distance.is_finite() {
        return divide_curve_by_count(curve, 1);
    }

    let (t0, t1) = curve.domain();
    let span = t1 - t0;
    if !span.is_finite() || span == 0.0 {
        let pt = curve.point_at(t0);
        let tan = curve.tangent_at(t0).unwrap_or_else(|| Vec3::new(0.0, 0.0, 0.0));
        return CurveDivisionResult {
            points: vec![pt],
            tangents: vec![tan],
            parameters: vec![t0],
        };
    }

    // Estimate total arc length
    let total_length = curve_arc_length(curve, samples_per_segment.max(32));
    if total_length < distance * 0.5 {
        // Curve is shorter than half the target distance
        let start = sample_curve_at(curve, t0);
        let end = sample_curve_at(curve, t1);
        return CurveDivisionResult {
            points: vec![start.point, end.point],
            tangents: vec![start.tangent, end.tangent],
            parameters: vec![start.parameter, end.parameter],
        };
    }

    let num_segments = (total_length / distance).ceil() as usize;
    let num_segments = num_segments.max(1);

    // Build arc-length table
    let table_size = (samples_per_segment.max(8) * num_segments).min(1024);
    let arc_length_table = build_arc_length_table(curve, table_size);

    let mut points = Vec::with_capacity(num_segments + 1);
    let mut tangents = Vec::with_capacity(num_segments + 1);
    let mut parameters = Vec::with_capacity(num_segments + 1);

    let start = sample_curve_at(curve, t0);
    points.push(start.point);
    tangents.push(start.tangent);
    parameters.push(start.parameter);

    let mut next_target = distance;
    while next_target < total_length - distance * 0.25 {
        let t = parameter_at_arc_length(&arc_length_table, t0, t1, next_target);
        let sample = sample_curve_at(curve, t);
        points.push(sample.point);
        tangents.push(sample.tangent);
        parameters.push(sample.parameter);
        next_target += distance;
    }

    // Always include end point
    let end = sample_curve_at(curve, t1);
    if parameters.last().map_or(true, |&last| (last - t1).abs() > 1e-9) {
        points.push(end.point);
        tangents.push(end.tangent);
        parameters.push(end.parameter);
    }

    CurveDivisionResult {
        points,
        tangents,
        parameters,
    }
}

/// Extracts a subcurve between two parameter values.
///
/// # Arguments
/// * `curve` - The parent curve.
/// * `start` - Start parameter (will be clamped to domain).
/// * `end` - End parameter (will be clamped to domain).
/// * `samples` - Number of samples for the subcurve tessellation.
///
/// # Returns
/// A [`SubCurve`] containing the tessellated points and parameter range.
#[must_use]
pub fn extract_subcurve<C: Curve3>(
    curve: &C,
    start: f64,
    end: f64,
    samples: usize,
) -> SubCurve {
    let (t0, t1) = curve.domain();
    let start_clamped = start.clamp(t0, t1);
    let end_clamped = end.clamp(t0, t1);

    let (start_param, end_param) = if start_clamped <= end_clamped {
        (start_clamped, end_clamped)
    } else {
        (end_clamped, start_clamped)
    };

    let samples = samples.max(1);
    let span = end_param - start_param;

    let mut points = Vec::with_capacity(samples + 1);
    if span.abs() < 1e-12 {
        points.push(curve.point_at(start_param));
    } else {
        for i in 0..=samples {
            let u = i as f64 / samples as f64;
            let t = start_param + span * u;
            points.push(curve.point_at(t));
        }
    }

    SubCurve {
        points,
        start_param,
        end_param,
    }
}

/// Shatters a curve at a list of parameters into subcurves.
///
/// # Arguments
/// * `curve` - The curve to shatter.
/// * `parameters` - List of parameter values at which to shatter.
/// * `samples_per_segment` - Samples per resulting subcurve segment.
///
/// # Returns
/// A vector of [`SubCurve`] segments.
#[must_use]
pub fn shatter_curve<C: Curve3>(
    curve: &C,
    parameters: &[f64],
    samples_per_segment: usize,
) -> Vec<SubCurve> {
    let (t0, t1) = curve.domain();

    // Collect and sort unique parameters
    let mut params: Vec<f64> = parameters
        .iter()
        .copied()
        .filter(|t| t.is_finite())
        .map(|t| t.clamp(t0, t1))
        .collect();
    params.push(t0);
    params.push(t1);
    params.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    params.dedup_by(|a, b| (*a - *b).abs() < 1e-12);

    let mut subcurves = Vec::with_capacity(params.len().saturating_sub(1));
    for window in params.windows(2) {
        let start = window[0];
        let end = window[1];
        if (end - start).abs() < 1e-12 {
            continue;
        }
        subcurves.push(extract_subcurve(curve, start, end, samples_per_segment));
    }

    subcurves
}

/// Computes a Frenet frame at a point on a curve.
///
/// The Frenet frame consists of:
/// - X-axis: tangent direction
/// - Y-axis: normal direction (toward center of curvature)
/// - Z-axis: binormal direction (tangent × normal)
///
/// # Arguments
/// * `curve` - The curve to evaluate.
/// * `t` - Parameter value.
///
/// # Returns
/// A [`CurveFrame`] at the specified parameter.
#[must_use]
pub fn frenet_frame_at<C: Curve3>(curve: &C, t: f64) -> CurveFrame {
    let point = curve.point_at(t);
    let d1 = curve.derivative_at(t);
    let d2 = curve.second_derivative_at(t);

    let tangent = d1.normalized().unwrap_or_else(|| Vec3::new(1.0, 0.0, 0.0));

    // Normal is in the direction of curvature (d2 - (d2·T)T)
    let d2_proj = d2.sub(tangent.mul_scalar(d2.dot(tangent)));
    let normal = d2_proj
        .normalized()
        .unwrap_or_else(|| orthogonal_unit_vector(tangent));

    let binormal = tangent.cross(normal).normalized().unwrap_or_else(|| {
        tangent.cross(Vec3::new(0.0, 0.0, 1.0)).normalized().unwrap_or_else(|| Vec3::new(0.0, 1.0, 0.0))
    });

    // Recompute normal to ensure orthogonality
    let normal = binormal.cross(tangent).normalized().unwrap_or(normal);

    CurveFrame {
        origin: point,
        x_axis: tangent,
        y_axis: normal,
        z_axis: binormal,
    }
}

/// Computes a parallel-transport frame at a point on a curve.
///
/// Unlike the Frenet frame, the parallel transport frame minimizes rotation
/// around the tangent axis, making it more stable for curves with low curvature.
///
/// # Arguments
/// * `curve` - The curve to evaluate.
/// * `t` - Parameter value.
/// * `reference_up` - Reference "up" direction for initial frame orientation.
///
/// # Returns
/// A [`CurveFrame`] at the specified parameter.
#[must_use]
pub fn parallel_frame_at<C: Curve3>(curve: &C, t: f64, reference_up: Vec3) -> CurveFrame {
    let point = curve.point_at(t);
    let d1 = curve.derivative_at(t);
    let tangent = d1.normalized().unwrap_or_else(|| Vec3::new(1.0, 0.0, 0.0));

    // Try to use reference_up to form the frame
    let cross1 = reference_up.cross(tangent);
    let binormal = if cross1.length() > 1e-9 {
        cross1.normalized().unwrap_or_else(|| Vec3::new(0.0, 0.0, 1.0))
    } else {
        // reference_up is parallel to tangent, pick an orthogonal vector
        orthogonal_unit_vector(tangent)
    };

    let normal = tangent.cross(binormal).normalized().unwrap_or_else(|| Vec3::new(0.0, 1.0, 0.0));
    let binormal = tangent.cross(normal).normalized().unwrap_or(binormal);

    CurveFrame {
        origin: point,
        x_axis: tangent,
        y_axis: normal,
        z_axis: binormal,
    }
}

/// Computes a horizontal frame at a point on a curve.
///
/// The frame is oriented so that the Z-axis (binormal) points upward (world Z)
/// when possible, providing stable orientation for horizontal curves.
///
/// # Arguments
/// * `curve` - The curve to evaluate.
/// * `t` - Parameter value.
///
/// # Returns
/// A [`CurveFrame`] at the specified parameter.
#[must_use]
pub fn horizontal_frame_at<C: Curve3>(curve: &C, t: f64) -> CurveFrame {
    parallel_frame_at(curve, t, Vec3::new(0.0, 0.0, 1.0))
}

/// Generates frames along a curve at regular arc-length intervals.
///
/// # Arguments
/// * `curve` - The curve to evaluate.
/// * `count` - Number of segments (output will have `count + 1` frames).
/// * `frame_fn` - Function to compute the frame at each parameter.
///
/// # Returns
/// A tuple of `(frames, parameters)`.
#[must_use]
pub fn curve_frames<C: Curve3, F>(
    curve: &C,
    count: usize,
    frame_fn: F,
) -> (Vec<CurveFrame>, Vec<f64>)
where
    F: Fn(&C, f64) -> CurveFrame,
{
    let count = count.max(1);
    let parameters = curve_parameters_by_count(curve, count, true);
    let mut frames = Vec::with_capacity(count + 1);
    let mut parameters_out = Vec::with_capacity(count + 1);

    for t in parameters {
        frames.push(frame_fn(curve, t));
        parameters_out.push(t);
    }

    (frames, parameters_out)
}

/// Generates Frenet frames along a curve.
#[must_use]
pub fn frenet_frames<C: Curve3>(curve: &C, count: usize) -> (Vec<CurveFrame>, Vec<f64>) {
    curve_frames(curve, count, frenet_frame_at)
}

/// Generates horizontal frames along a curve.
#[must_use]
pub fn horizontal_frames<C: Curve3>(curve: &C, count: usize) -> (Vec<CurveFrame>, Vec<f64>) {
    curve_frames(curve, count, horizontal_frame_at)
}

/// Generates perpendicular (parallel transport) frames along a curve.
///
/// # Arguments
/// * `curve` - The curve to evaluate.
/// * `count` - Number of segments.
/// * `align` - If true, align successive frames to minimize rotation.
///
/// # Returns
/// A tuple of `(frames, parameters)`.
#[must_use]
pub fn perp_frames<C: Curve3>(
    curve: &C,
    count: usize,
    align: bool,
) -> (Vec<CurveFrame>, Vec<f64>) {
    let count = count.max(1);
    let parameters = curve_parameters_by_count(curve, count, true);
    let mut frames = Vec::with_capacity(count + 1);
    let mut parameters_out = Vec::with_capacity(count + 1);

    let mut prev_y: Option<Vec3> = None;
    let mut prev_z: Option<Vec3> = None;

    for t in parameters {
        let mut frame = horizontal_frame_at(curve, t);

        if align {
            if let (Some(py), Some(pz)) = (prev_y, prev_z) {
                if frame.y_axis.dot(py) < 0.0 {
                    frame.y_axis = frame.y_axis.mul_scalar(-1.0);
                }
                if frame.z_axis.dot(pz) < 0.0 {
                    frame.z_axis = frame.z_axis.mul_scalar(-1.0);
                }
            }
            prev_y = Some(frame.y_axis);
            prev_z = Some(frame.z_axis);
        }

        frames.push(frame);
        parameters_out.push(t);
    }

    (frames, parameters_out)
}

// ============================================================================
// Arc-Length Table Helpers
// ============================================================================

/// An entry in the arc-length lookup table.
struct ArcLengthEntry {
    parameter: f64,
    arc_length: f64,
}

/// Builds an arc-length lookup table for a curve.
fn build_arc_length_table<C: Curve3>(curve: &C, samples: usize) -> Vec<ArcLengthEntry> {
    let samples = samples.max(2);
    let (t0, t1) = curve.domain();
    let span = t1 - t0;

    let mut table = Vec::with_capacity(samples);
    let mut prev = curve.point_at(t0);
    let mut cumulative = 0.0;

    table.push(ArcLengthEntry {
        parameter: t0,
        arc_length: 0.0,
    });

    for i in 1..samples {
        let u = i as f64 / (samples - 1) as f64;
        let t = t0 + span * u;
        let curr = curve.point_at(t);
        cumulative += curr.sub_point(prev).length();
        table.push(ArcLengthEntry {
            parameter: t,
            arc_length: cumulative,
        });
        prev = curr;
    }

    table
}

/// Computes parameter values for evenly spaced arc-length segments.
fn curve_parameters_by_count<C: Curve3>(curve: &C, count: usize, include_end: bool) -> Vec<f64> {
    let count = count.max(1);
    let (t0, t1) = curve.domain();
    let span = t1 - t0;
    if !span.is_finite() || span == 0.0 {
        return vec![t0];
    }

    let sample_count = (count.saturating_mul(16)).clamp(32, 4096);
    let table = build_arc_length_table(curve, sample_count);
    let total = table.last().map(|e| e.arc_length).unwrap_or(0.0);
    if !total.is_finite() || total <= 0.0 {
        let denom = count as f64;
        if include_end {
            return (0..=count)
                .map(|i| t0 + span * (i as f64 / denom))
                .collect();
        }
        return (0..count)
            .map(|i| t0 + span * (i as f64 / denom))
            .collect();
    }

    let point_count = if include_end { count + 1 } else { count };
    let denom = count as f64;
    let mut params = Vec::with_capacity(point_count);
    for i in 0..point_count {
        let ratio = i as f64 / denom;
        let target = total * ratio;
        params.push(parameter_at_arc_length(&table, t0, t1, target));
    }
    params
}

/// Finds the parameter value corresponding to a target arc length.
fn parameter_at_arc_length(
    table: &[ArcLengthEntry],
    t0: f64,
    t1: f64,
    target_length: f64,
) -> f64 {
    if table.is_empty() {
        return t0;
    }
    if table.len() == 1 {
        return table[0].parameter;
    }

    let total = table.last().map(|e| e.arc_length).unwrap_or(0.0);
    if target_length <= 0.0 {
        return t0;
    }
    if target_length >= total {
        return t1;
    }

    // Binary search for the segment containing target_length
    let idx = table
        .binary_search_by(|entry| {
            entry
                .arc_length
                .partial_cmp(&target_length)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or_else(|i| i.saturating_sub(1));

    let idx = idx.min(table.len() - 2);
    let e0 = &table[idx];
    let e1 = &table[idx + 1];

    let segment_length = e1.arc_length - e0.arc_length;
    if segment_length.abs() < 1e-14 {
        return e0.parameter;
    }

    let ratio = (target_length - e0.arc_length) / segment_length;
    e0.parameter + (e1.parameter - e0.parameter) * ratio.clamp(0.0, 1.0)
}

/// Finds plane intersection points with a curve.
///
/// # Arguments
/// * `curve` - The curve to intersect.
/// * `plane_origin` - A point on the plane.
/// * `plane_normal` - The plane normal (will be normalized).
/// * `samples` - Number of samples to search for intersections.
///
/// # Returns
/// A vector of `(point, parameter)` tuples for each intersection.
#[must_use]
pub fn curve_plane_intersections<C: Curve3>(
    curve: &C,
    plane_origin: Point3,
    plane_normal: Vec3,
    samples: usize,
) -> Vec<(Point3, f64)> {
    let normal = plane_normal.normalized().unwrap_or_else(|| Vec3::new(0.0, 0.0, 1.0));
    let samples = samples.max(2);
    let (t0, t1) = curve.domain();
    let span = t1 - t0;

    if !span.is_finite() || span == 0.0 {
        return Vec::new();
    }

    let mut results = Vec::new();

    // Sample the curve and find sign changes in the signed distance
    let mut prev_t = t0;
    let mut prev_pt = curve.point_at(t0);
    let mut prev_dist = prev_pt.sub_point(plane_origin).dot(normal);

    for i in 1..=samples {
        let u = i as f64 / samples as f64;
        let t = t0 + span * u;
        let pt = curve.point_at(t);
        let dist = pt.sub_point(plane_origin).dot(normal);

        // Check for zero crossing
        if prev_dist.abs() < 1e-12 {
            results.push((prev_pt, prev_t));
        } else if prev_dist.signum() != dist.signum() {
            // Linear interpolation to find approximate intersection
            let denom = prev_dist - dist;
            if denom.abs() > 1e-14 {
                let ratio = prev_dist / denom;
                let t_intersect = prev_t + (t - prev_t) * ratio.clamp(0.0, 1.0);
                let pt_intersect = curve.point_at(t_intersect);
                results.push((pt_intersect, t_intersect));
            }
        }

        prev_t = t;
        prev_pt = pt;
        prev_dist = dist;
    }

    // Check last point
    if prev_dist.abs() < 1e-12 && results.last().map_or(true, |(_, t)| (*t - prev_t).abs() > 1e-9) {
        results.push((prev_pt, prev_t));
    }

    results
}

// ============================================================================
// Advanced Curve Analysis Functions
// ============================================================================

/// Result of curvature analysis at a point on a curve.
#[derive(Debug, Clone, Copy)]
pub struct CurvatureAnalysis {
    /// The point on the curve.
    pub point: Point3,
    /// The curvature vector (perpendicular to tangent, toward center of curvature).
    pub curvature_vector: Vec3,
    /// The curvature magnitude (1/radius).
    pub curvature: f64,
    /// The center of curvature (point + normal / curvature).
    pub center: Point3,
    /// The radius of curvature (1/curvature), or infinity if curvature is zero.
    pub radius: f64,
}

/// Computes the curvature vector at a parameter on a curve.
///
/// The curvature vector points toward the center of curvature and has
/// magnitude equal to the curvature (1/radius).
///
/// # Arguments
/// * `curve` - The curve to analyze.
/// * `t` - Parameter value.
///
/// # Returns
/// The curvature vector, or zero if the curve is straight at this point.
#[must_use]
pub fn curve_curvature_vector_at<C: Curve3>(curve: &C, t: f64) -> Vec3 {
    let d1 = curve.derivative_at(t);
    let d2 = curve.second_derivative_at(t);

    // Curvature vector = d2 - (d2·T)T where T is the unit tangent
    let tangent = match d1.normalized() {
        Some(t) => t,
        None => return Vec3::ZERO,
    };

    let d2_tangent_component = tangent.mul_scalar(d2.dot(tangent));
    d2.sub(d2_tangent_component)
}

/// Computes the center of curvature at a parameter on a curve.
///
/// The center of curvature is the center of the osculating circle at that point.
///
/// # Arguments
/// * `curve` - The curve to analyze.
/// * `t` - Parameter value.
///
/// # Returns
/// The center of curvature, or the point itself if the curve is straight.
#[must_use]
pub fn curve_curvature_center_at<C: Curve3>(curve: &C, t: f64) -> Point3 {
    let point = curve.point_at(t);
    let curvature_vec = curve_curvature_vector_at(curve, t);
    let curvature_magnitude = curvature_vec.length();

    if curvature_magnitude < 1e-12 {
        return point;
    }

    let normal = curvature_vec.normalized().unwrap_or(Vec3::ZERO);
    point.add_vec(normal.mul_scalar(1.0 / curvature_magnitude))
}

/// Performs full curvature analysis at a parameter on a curve.
///
/// # Arguments
/// * `curve` - The curve to analyze.
/// * `t` - Parameter value.
///
/// # Returns
/// A [`CurvatureAnalysis`] containing point, curvature vector, magnitude, center, and radius.
#[must_use]
pub fn analyze_curvature_at<C: Curve3>(curve: &C, t: f64) -> CurvatureAnalysis {
    let point = curve.point_at(t);
    let curvature_vector = curve_curvature_vector_at(curve, t);
    let curvature = curvature_vector.length();

    let (center, radius) = if curvature < 1e-12 {
        (point, f64::INFINITY)
    } else {
        let normal = curvature_vector.normalized().unwrap_or(Vec3::ZERO);
        let r = 1.0 / curvature;
        (point.add_vec(normal.mul_scalar(r)), r)
    };

    CurvatureAnalysis {
        point,
        curvature_vector,
        curvature,
        center,
        radius,
    }
}

/// Computes the third derivative of a curve at a parameter using finite differences.
///
/// # Arguments
/// * `curve` - The curve to analyze.
/// * `t` - Parameter value.
///
/// # Returns
/// The third derivative vector.
#[must_use]
pub fn curve_third_derivative_at<C: Curve3>(curve: &C, t: f64) -> Vec3 {
    let (t0, t1) = curve.domain();
    let span = t1 - t0;
    if !span.is_finite() || span == 0.0 {
        return Vec3::ZERO;
    }

    // Use a small step size relative to the domain
    let h = span / 256.0;
    let t_plus = (t + h).min(t1);
    let t_minus = (t - h).max(t0);

    // Third derivative ≈ (d2(t+h) - d2(t-h)) / (2h)
    let d2_plus = curve.second_derivative_at(t_plus);
    let d2_minus = curve.second_derivative_at(t_minus);

    let dt = t_plus - t_minus;
    if dt.abs() < 1e-12 {
        return Vec3::ZERO;
    }

    d2_plus.sub(d2_minus).mul_scalar(1.0 / dt)
}

/// Computes the torsion of a curve at a parameter.
///
/// Torsion measures how much the curve twists out of its osculating plane.
/// A planar curve has zero torsion. The sign indicates the direction of twist.
///
/// Formula: τ = (d1 × d2) · d3 / |d1 × d2|²
///
/// # Arguments
/// * `curve` - The curve to analyze.
/// * `t` - Parameter value.
///
/// # Returns
/// The torsion value, or 0 if undefined (at inflection points or straight sections).
#[must_use]
pub fn curve_torsion_at<C: Curve3>(curve: &C, t: f64) -> f64 {
    let d1 = curve.derivative_at(t);
    let d2 = curve.second_derivative_at(t);
    let d3 = curve_third_derivative_at(curve, t);

    let cross12 = d1.cross(d2);
    let denominator = cross12.length_squared();

    if denominator < 1e-18 {
        return 0.0;
    }

    cross12.dot(d3) / denominator
}

/// Computes the angle (rate of tangent direction change) at a parameter on a curve.
///
/// Uses central difference on tangent vectors to measure the angle between
/// tangent directions slightly before and after the parameter.
///
/// # Arguments
/// * `curve` - The curve to analyze.
/// * `t` - Parameter value.
///
/// # Returns
/// The angle in radians between tangent directions before and after the parameter.
#[must_use]
pub fn curve_angle_at<C: Curve3>(curve: &C, t: f64) -> f64 {
    let (t0, t1) = curve.domain();
    let span = t1 - t0;
    if !span.is_finite() || span == 0.0 {
        return 0.0;
    }

    let dt = span / 128.0;
    let t_before = (t - dt).max(t0);
    let t_after = (t + dt).min(t1);

    let tan_before = curve.tangent_at(t_before).unwrap_or(Vec3::X);
    let tan_after = curve.tangent_at(t_after).unwrap_or(Vec3::X);

    let dot_val = tan_before.dot(tan_after).clamp(-1.0, 1.0);
    dot_val.acos()
}

/// Computes the arc length from the start of the curve to a given parameter.
///
/// # Arguments
/// * `curve` - The curve to measure.
/// * `t` - Parameter value (will be clamped to curve domain).
/// * `samples` - Number of samples for numerical integration.
///
/// # Returns
/// The arc length from the curve start to parameter `t`.
#[must_use]
pub fn curve_length_at<C: Curve3>(curve: &C, t: f64, samples: usize) -> f64 {
    let (t0, t1) = curve.domain();
    let t_clamped = t.clamp(t0, t1);

    if (t_clamped - t0).abs() < 1e-12 {
        return 0.0;
    }

    let samples = samples.max(1);
    let span = t_clamped - t0;
    let mut length = 0.0;
    let mut prev = curve.point_at(t0);

    for i in 1..=samples {
        let u = t0 + span * (i as f64 / samples as f64);
        let curr = curve.point_at(u);
        length += curr.sub_point(prev).length();
        prev = curr;
    }

    length
}

/// Finds the parameter value corresponding to a target arc length from the start.
///
/// # Arguments
/// * `curve` - The curve to search.
/// * `target_length` - The target arc length from the start.
/// * `samples` - Number of samples for the arc-length table.
///
/// # Returns
/// The parameter value at which the arc length from start equals `target_length`,
/// or the domain end if `target_length` exceeds the total curve length.
#[must_use]
pub fn curve_parameter_at_length<C: Curve3>(curve: &C, target_length: f64, samples: usize) -> f64 {
    let (t0, t1) = curve.domain();

    if target_length <= 0.0 {
        return t0;
    }

    let table = build_arc_length_table(curve, samples.max(32));
    parameter_at_arc_length(&table, t0, t1, target_length)
}

/// Result of computing curve segment lengths.
#[derive(Debug, Clone)]
pub struct SegmentLengthAnalysis {
    /// Length of each segment.
    pub lengths: Vec<f64>,
    /// Total length of all segments.
    pub total_length: f64,
    /// Index and length of the shortest segment.
    pub shortest: Option<(usize, f64)>,
    /// Index and length of the longest segment.
    pub longest: Option<(usize, f64)>,
}

/// Analyzes the segment lengths of a polyline curve.
///
/// # Arguments
/// * `polyline` - The polyline to analyze.
///
/// # Returns
/// A [`SegmentLengthAnalysis`] containing lengths and statistics.
#[must_use]
pub fn analyze_polyline_segments(polyline: &Polyline3) -> SegmentLengthAnalysis {
    let points = polyline.points();
    if points.len() < 2 {
        return SegmentLengthAnalysis {
            lengths: Vec::new(),
            total_length: 0.0,
            shortest: None,
            longest: None,
        };
    }

    let mut lengths = Vec::with_capacity(points.len() - 1);
    let mut total = 0.0;
    let mut shortest: Option<(usize, f64)> = None;
    let mut longest: Option<(usize, f64)> = None;

    for (i, window) in points.windows(2).enumerate() {
        let len = window[1].sub_point(window[0]).length();
        lengths.push(len);
        total += len;

        match shortest {
            None => shortest = Some((i, len)),
            Some((_, min_len)) if len < min_len => shortest = Some((i, len)),
            _ => {}
        }

        match longest {
            None => longest = Some((i, len)),
            Some((_, max_len)) if len > max_len => longest = Some((i, len)),
            _ => {}
        }
    }

    // Handle closed polyline closing segment
    if polyline.is_closed() && points.len() > 2 {
        let last = points.last().copied().unwrap_or(Point3::ORIGIN);
        let first = points.first().copied().unwrap_or(Point3::ORIGIN);
        let closing_len = first.sub_point(last).length();
        let closing_idx = lengths.len();
        lengths.push(closing_len);
        total += closing_len;

        match shortest {
            None => shortest = Some((closing_idx, closing_len)),
            Some((_, min_len)) if closing_len < min_len => shortest = Some((closing_idx, closing_len)),
            _ => {}
        }

        match longest {
            None => longest = Some((closing_idx, closing_len)),
            Some((_, max_len)) if closing_len > max_len => longest = Some((closing_idx, closing_len)),
            _ => {}
        }
    }

    SegmentLengthAnalysis {
        lengths,
        total_length: total,
        shortest,
        longest,
    }
}
