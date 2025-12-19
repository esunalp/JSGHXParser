use super::core::{Point3, Tolerance, Vec3};

fn is_non_decreasing(values: &[f64]) -> bool {
    values.windows(2).all(|w| w[0] <= w[1])
}

fn wrap_param(value: f64, start: f64, end: f64) -> f64 {
    let span = end - start;
    if !span.is_finite() || span == 0.0 {
        return start;
    }
    let mut t = (value - start) % span;
    if t < 0.0 {
        t += span;
    }
    start + t
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

fn frame_axes_from_xaxis_normal(x_axis: Vec3, normal: Vec3) -> (Vec3, Vec3, Vec3) {
    let z = normal.normalized().unwrap_or_else(|| Vec3::new(0.0, 0.0, 1.0));
    let projected = x_axis.sub(z.mul_scalar(x_axis.dot(z)));
    let x = projected
        .normalized()
        .unwrap_or_else(|| orthogonal_unit_vector(z));
    let y = z.cross(x).normalized().unwrap_or_else(|| Vec3::new(0.0, 1.0, 0.0));
    (x, y, z)
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

    const fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z, self.w - rhs.w)
    }

    const fn mul_scalar(self, s: f64) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s, self.w * s)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SurfaceCacheKey {
    Plane {
        origin: [u64; 3],
        u_axis: [u64; 3],
        v_axis: [u64; 3],
    },
    Cylinder {
        base: [u64; 3],
        axis: [u64; 3],
        x_axis: [u64; 3],
        radius: u64,
    },
    Cone {
        base: [u64; 3],
        axis: [u64; 3],
        x_axis: [u64; 3],
        radius0: u64,
        radius1: u64,
    },
    Sphere {
        center: [u64; 3],
        x_axis: [u64; 3],
        z_axis: [u64; 3],
        radius: u64,
    },
    Torus {
        center: [u64; 3],
        x_axis: [u64; 3],
        z_axis: [u64; 3],
        major_radius: u64,
        minor_radius: u64,
    },
    Nurbs {
        hash: u64,
    },
}

pub trait Surface {
    fn point_at(&self, u: f64, v: f64) -> Point3;

    #[must_use]
    fn domain_u(&self) -> (f64, f64) {
        (0.0, 1.0)
    }

    #[must_use]
    fn domain_v(&self) -> (f64, f64) {
        (0.0, 1.0)
    }

    #[must_use]
    fn is_u_closed(&self) -> bool {
        false
    }

    #[must_use]
    fn is_v_closed(&self) -> bool {
        false
    }

    #[must_use]
    fn pole_v_start(&self) -> bool {
        false
    }

    #[must_use]
    fn pole_v_end(&self) -> bool {
        false
    }

    #[must_use]
    fn partial_derivatives_at(&self, u: f64, v: f64) -> (Vec3, Vec3) {
        let (u0, u1) = self.domain_u();
        let (v0, v1) = self.domain_v();

        let u_span = u1 - u0;
        let v_span = v1 - v0;

        let u = if self.is_u_closed() {
            wrap_param(u, u0, u1)
        } else {
            u.clamp(u0, u1)
        };

        let v = if self.is_v_closed() {
            wrap_param(v, v0, v1)
        } else {
            v.clamp(v0, v1)
        };

        let mut du = Vec3::new(0.0, 0.0, 0.0);
        let mut dv = Vec3::new(0.0, 0.0, 0.0);

        if u_span.is_finite() && u_span != 0.0 {
            let h = Tolerance::DERIVATIVE.relative_to(u_span);
            if h.is_finite() && h != 0.0 {
                let ua = if self.is_u_closed() { u - h } else { (u - h).max(u0) };
                let ub = if self.is_u_closed() { u + h } else { (u + h).min(u1) };

                if ua != ub {
                    let pa = self.point_at(ua, v);
                    let pb = self.point_at(ub, v);
                    du = pb.sub_point(pa).mul_scalar(1.0 / (ub - ua));
                }
            }
        }

        if v_span.is_finite() && v_span != 0.0 {
            let h = Tolerance::DERIVATIVE.relative_to(v_span);
            if h.is_finite() && h != 0.0 {
                let va = if self.is_v_closed() { v - h } else { (v - h).max(v0) };
                let vb = if self.is_v_closed() { v + h } else { (v + h).min(v1) };

                if va != vb {
                    let pa = self.point_at(u, va);
                    let pb = self.point_at(u, vb);
                    dv = pb.sub_point(pa).mul_scalar(1.0 / (vb - va));
                }
            }
        }

        (du, dv)
    }

    #[must_use]
    fn normal_at(&self, u: f64, v: f64) -> Option<Vec3> {
        let (du, dv) = self.partial_derivatives_at(u, v);
        du.cross(dv).normalized()
    }

    fn cache_key(&self) -> SurfaceCacheKey;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaneSurface {
    pub origin: Point3,
    pub u_axis: Vec3,
    pub v_axis: Vec3,
}

impl PlaneSurface {
    #[must_use]
    pub const fn new(origin: Point3, u_axis: Vec3, v_axis: Vec3) -> Self {
        Self {
            origin,
            u_axis,
            v_axis,
        }
    }
}

impl Surface for PlaneSurface {
    fn point_at(&self, u: f64, v: f64) -> Point3 {
        self.origin
            .add_vec(self.u_axis.mul_scalar(u))
            .add_vec(self.v_axis.mul_scalar(v))
    }

    fn normal_at(&self, _u: f64, _v: f64) -> Option<Vec3> {
        self.u_axis.cross(self.v_axis).normalized()
    }

    fn cache_key(&self) -> SurfaceCacheKey {
        SurfaceCacheKey::Plane {
            origin: [
                self.origin.x.to_bits(),
                self.origin.y.to_bits(),
                self.origin.z.to_bits(),
            ],
            u_axis: [
                self.u_axis.x.to_bits(),
                self.u_axis.y.to_bits(),
                self.u_axis.z.to_bits(),
            ],
            v_axis: [
                self.v_axis.x.to_bits(),
                self.v_axis.y.to_bits(),
                self.v_axis.z.to_bits(),
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CylinderSurface {
    pub base: Point3,
    pub axis: Vec3,
    pub axis_dir: Vec3,
    pub x_axis: Vec3,
    pub y_axis: Vec3,
    pub radius: f64,
}

impl CylinderSurface {
    pub fn new(base: Point3, axis: Vec3, radius: f64) -> Result<Self, String> {
        if !radius.is_finite() || radius <= 0.0 {
            return Err("cylinder radius must be finite and > 0".to_string());
        }

        let axis_dir = axis
            .normalized()
            .ok_or_else(|| "cylinder axis must be non-zero".to_string())?;
        let x_axis = orthogonal_unit_vector(axis_dir);
        let y_axis = axis_dir
            .cross(x_axis)
            .normalized()
            .unwrap_or_else(|| Vec3::new(0.0, 1.0, 0.0));

        Ok(Self {
            base,
            axis,
            axis_dir,
            x_axis,
            y_axis,
            radius,
        })
    }

    pub fn from_base_axis_xaxis(
        base: Point3,
        axis: Vec3,
        x_axis: Vec3,
        radius: f64,
    ) -> Result<Self, String> {
        if !radius.is_finite() || radius <= 0.0 {
            return Err("cylinder radius must be finite and > 0".to_string());
        }

        let axis_dir = axis
            .normalized()
            .ok_or_else(|| "cylinder axis must be non-zero".to_string())?;
        let (x_axis, y_axis, _) = frame_axes_from_xaxis_normal(x_axis, axis_dir);

        Ok(Self {
            base,
            axis,
            axis_dir,
            x_axis,
            y_axis,
            radius,
        })
    }
}

impl Surface for CylinderSurface {
    fn point_at(&self, u: f64, v: f64) -> Point3 {
        let u = wrap_param(u, 0.0, 1.0);
        let angle = std::f64::consts::TAU * u;
        let radial = self
            .x_axis
            .mul_scalar(angle.cos())
            .add(self.y_axis.mul_scalar(angle.sin()))
            .mul_scalar(self.radius);

        self.base
            .add_vec(self.axis.mul_scalar(v))
            .add_vec(radial)
    }

    fn is_u_closed(&self) -> bool {
        true
    }

    fn cache_key(&self) -> SurfaceCacheKey {
        SurfaceCacheKey::Cylinder {
            base: [self.base.x.to_bits(), self.base.y.to_bits(), self.base.z.to_bits()],
            axis: [self.axis.x.to_bits(), self.axis.y.to_bits(), self.axis.z.to_bits()],
            x_axis: [
                self.x_axis.x.to_bits(),
                self.x_axis.y.to_bits(),
                self.x_axis.z.to_bits(),
            ],
            radius: self.radius.to_bits(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConeSurface {
    pub base: Point3,
    pub axis: Vec3,
    pub axis_dir: Vec3,
    pub x_axis: Vec3,
    pub y_axis: Vec3,
    pub radius0: f64,
    pub radius1: f64,
}

impl ConeSurface {
    pub fn new(base: Point3, axis: Vec3, radius0: f64, radius1: f64) -> Result<Self, String> {
        if !radius0.is_finite() || radius0 < 0.0 || !radius1.is_finite() || radius1 < 0.0 {
            return Err("cone radii must be finite and >= 0".to_string());
        }

        let axis_dir = axis
            .normalized()
            .ok_or_else(|| "cone axis must be non-zero".to_string())?;
        let x_axis = orthogonal_unit_vector(axis_dir);
        let y_axis = axis_dir
            .cross(x_axis)
            .normalized()
            .unwrap_or_else(|| Vec3::new(0.0, 1.0, 0.0));

        Ok(Self {
            base,
            axis,
            axis_dir,
            x_axis,
            y_axis,
            radius0,
            radius1,
        })
    }

    pub fn from_base_axis_xaxis(
        base: Point3,
        axis: Vec3,
        x_axis: Vec3,
        radius0: f64,
        radius1: f64,
    ) -> Result<Self, String> {
        if !radius0.is_finite() || radius0 < 0.0 || !radius1.is_finite() || radius1 < 0.0 {
            return Err("cone radii must be finite and >= 0".to_string());
        }

        let axis_dir = axis
            .normalized()
            .ok_or_else(|| "cone axis must be non-zero".to_string())?;
        let (x_axis, y_axis, _) = frame_axes_from_xaxis_normal(x_axis, axis_dir);

        Ok(Self {
            base,
            axis,
            axis_dir,
            x_axis,
            y_axis,
            radius0,
            radius1,
        })
    }
}

impl Surface for ConeSurface {
    fn point_at(&self, u: f64, v: f64) -> Point3 {
        let u = wrap_param(u, 0.0, 1.0);
        let angle = std::f64::consts::TAU * u;
        let radius = self.radius0 + (self.radius1 - self.radius0) * v;
        let radial = self
            .x_axis
            .mul_scalar(angle.cos())
            .add(self.y_axis.mul_scalar(angle.sin()))
            .mul_scalar(radius);

        self.base.add_vec(self.axis.mul_scalar(v)).add_vec(radial)
    }

    fn is_u_closed(&self) -> bool {
        true
    }

    fn pole_v_start(&self) -> bool {
        self.radius0 == 0.0
    }

    fn pole_v_end(&self) -> bool {
        self.radius1 == 0.0
    }

    fn cache_key(&self) -> SurfaceCacheKey {
        SurfaceCacheKey::Cone {
            base: [self.base.x.to_bits(), self.base.y.to_bits(), self.base.z.to_bits()],
            axis: [self.axis.x.to_bits(), self.axis.y.to_bits(), self.axis.z.to_bits()],
            x_axis: [
                self.x_axis.x.to_bits(),
                self.x_axis.y.to_bits(),
                self.x_axis.z.to_bits(),
            ],
            radius0: self.radius0.to_bits(),
            radius1: self.radius1.to_bits(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphereSurface {
    pub center: Point3,
    pub x_axis: Vec3,
    pub y_axis: Vec3,
    pub z_axis: Vec3,
    pub radius: f64,
}

impl SphereSurface {
    pub fn new(center: Point3, radius: f64) -> Result<Self, String> {
        if !radius.is_finite() || radius <= 0.0 {
            return Err("sphere radius must be finite and > 0".to_string());
        }

        Ok(Self {
            center,
            x_axis: Vec3::new(1.0, 0.0, 0.0),
            y_axis: Vec3::new(0.0, 1.0, 0.0),
            z_axis: Vec3::new(0.0, 0.0, 1.0),
            radius,
        })
    }

    pub fn from_center_xaxis_normal(
        center: Point3,
        x_axis: Vec3,
        normal: Vec3,
        radius: f64,
    ) -> Result<Self, String> {
        if !radius.is_finite() || radius <= 0.0 {
            return Err("sphere radius must be finite and > 0".to_string());
        }

        let (x_axis, y_axis, z_axis) = frame_axes_from_xaxis_normal(x_axis, normal);
        Ok(Self {
            center,
            x_axis,
            y_axis,
            z_axis,
            radius,
        })
    }
}

impl Surface for SphereSurface {
    fn point_at(&self, u: f64, v: f64) -> Point3 {
        let u = wrap_param(u, 0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        let theta = std::f64::consts::TAU * u;
        let phi = std::f64::consts::PI * (v - 0.5);

        let cos_phi = phi.cos();
        let sin_phi = phi.sin();

        let x = cos_phi * theta.cos();
        let y = cos_phi * theta.sin();
        let z = sin_phi;

        self.center.add_vec(
            self.x_axis
                .mul_scalar(x)
                .add(self.y_axis.mul_scalar(y))
                .add(self.z_axis.mul_scalar(z))
                .mul_scalar(self.radius),
        )
    }

    fn is_u_closed(&self) -> bool {
        true
    }

    fn pole_v_start(&self) -> bool {
        true
    }

    fn pole_v_end(&self) -> bool {
        true
    }

    fn cache_key(&self) -> SurfaceCacheKey {
        SurfaceCacheKey::Sphere {
            center: [
                self.center.x.to_bits(),
                self.center.y.to_bits(),
                self.center.z.to_bits(),
            ],
            x_axis: [
                self.x_axis.x.to_bits(),
                self.x_axis.y.to_bits(),
                self.x_axis.z.to_bits(),
            ],
            z_axis: [
                self.z_axis.x.to_bits(),
                self.z_axis.y.to_bits(),
                self.z_axis.z.to_bits(),
            ],
            radius: self.radius.to_bits(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TorusSurface {
    pub center: Point3,
    pub x_axis: Vec3,
    pub y_axis: Vec3,
    pub z_axis: Vec3,
    pub major_radius: f64,
    pub minor_radius: f64,
}

impl TorusSurface {
    pub fn from_center_xaxis_normal(
        center: Point3,
        x_axis: Vec3,
        normal: Vec3,
        major_radius: f64,
        minor_radius: f64,
    ) -> Result<Self, String> {
        if !major_radius.is_finite() || major_radius <= 0.0 {
            return Err("torus major radius must be finite and > 0".to_string());
        }
        if !minor_radius.is_finite() || minor_radius <= 0.0 {
            return Err("torus minor radius must be finite and > 0".to_string());
        }

        let (x_axis, y_axis, z_axis) = frame_axes_from_xaxis_normal(x_axis, normal);
        Ok(Self {
            center,
            x_axis,
            y_axis,
            z_axis,
            major_radius,
            minor_radius,
        })
    }
}

impl Surface for TorusSurface {
    fn point_at(&self, u: f64, v: f64) -> Point3 {
        let u = wrap_param(u, 0.0, 1.0);
        let v = wrap_param(v, 0.0, 1.0);

        let theta = std::f64::consts::TAU * u;
        let phi = std::f64::consts::TAU * v;

        let cos_theta = theta.cos();
        let sin_theta = theta.sin();
        let cos_phi = phi.cos();
        let sin_phi = phi.sin();

        let radial = self
            .x_axis
            .mul_scalar(cos_theta)
            .add(self.y_axis.mul_scalar(sin_theta));

        let tube = radial.mul_scalar(self.major_radius + self.minor_radius * cos_phi);
        let vertical = self.z_axis.mul_scalar(self.minor_radius * sin_phi);
        self.center.add_vec(tube.add(vertical))
    }

    fn is_u_closed(&self) -> bool {
        true
    }

    fn is_v_closed(&self) -> bool {
        true
    }

    fn cache_key(&self) -> SurfaceCacheKey {
        SurfaceCacheKey::Torus {
            center: [
                self.center.x.to_bits(),
                self.center.y.to_bits(),
                self.center.z.to_bits(),
            ],
            x_axis: [
                self.x_axis.x.to_bits(),
                self.x_axis.y.to_bits(),
                self.x_axis.z.to_bits(),
            ],
            z_axis: [
                self.z_axis.x.to_bits(),
                self.z_axis.y.to_bits(),
                self.z_axis.z.to_bits(),
            ],
            major_radius: self.major_radius.to_bits(),
            minor_radius: self.minor_radius.to_bits(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NurbsSurface {
    pub degree_u: usize,
    pub degree_v: usize,
    pub u_count: usize,
    pub v_count: usize,
    pub control_points: Vec<Point3>,
    pub knots_u: Vec<f64>,
    pub knots_v: Vec<f64>,
    pub weights: Option<Vec<f64>>,
    cache_hash: u64,
    u_closed: bool,
    v_closed: bool,
    pole_v_start: bool,
    pole_v_end: bool,
}

impl NurbsSurface {
    pub fn new(
        degree_u: usize,
        degree_v: usize,
        u_count: usize,
        v_count: usize,
        control_points: Vec<Point3>,
        knots_u: Vec<f64>,
        knots_v: Vec<f64>,
        weights: Option<Vec<f64>>,
    ) -> Result<Self, String> {
        if u_count < 2 || v_count < 2 {
            return Err("nurbs surface requires at least a 2x2 control net".to_string());
        }
        if degree_u == 0 || degree_v == 0 {
            return Err("nurbs surface degrees must be >= 1".to_string());
        }
        if degree_u >= u_count || degree_v >= v_count {
            return Err("nurbs surface degrees must be < control point counts".to_string());
        }
        if control_points.len() != u_count * v_count {
            return Err("nurbs surface control point count must match u_count*v_count".to_string());
        }

        let expected_u_knots = u_count + degree_u + 1;
        if knots_u.len() != expected_u_knots {
            return Err(format!(
                "nurbs surface u knot length must be {}, got {}",
                expected_u_knots,
                knots_u.len()
            ));
        }

        let expected_v_knots = v_count + degree_v + 1;
        if knots_v.len() != expected_v_knots {
            return Err(format!(
                "nurbs surface v knot length must be {}, got {}",
                expected_v_knots,
                knots_v.len()
            ));
        }

        if !is_non_decreasing(&knots_u) || !is_non_decreasing(&knots_v) {
            return Err("nurbs surface knots must be non-decreasing".to_string());
        }

        if let Some(ref weights) = weights {
            if weights.len() != control_points.len() {
                return Err(
                    "nurbs surface weights length must match control point count".to_string(),
                );
            }
            if weights.iter().any(|w| !w.is_finite() || *w <= 0.0) {
                return Err("nurbs surface weights must be finite and > 0".to_string());
            }
        }

        use std::hash::{Hash, Hasher};

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        degree_u.hash(&mut hasher);
        degree_v.hash(&mut hasher);
        u_count.hash(&mut hasher);
        v_count.hash(&mut hasher);

        for point in &control_points {
            point.x.to_bits().hash(&mut hasher);
            point.y.to_bits().hash(&mut hasher);
            point.z.to_bits().hash(&mut hasher);
        }

        for value in &knots_u {
            value.to_bits().hash(&mut hasher);
        }
        for value in &knots_v {
            value.to_bits().hash(&mut hasher);
        }

        if let Some(ref weights) = weights {
            for value in weights {
                value.to_bits().hash(&mut hasher);
            }
        }

        let cache_hash = hasher.finish();

        let mut surface = Self {
            degree_u,
            degree_v,
            u_count,
            v_count,
            control_points,
            knots_u,
            knots_v,
            weights,
            cache_hash,
            u_closed: false,
            v_closed: false,
            pole_v_start: false,
            pole_v_end: false,
        };

        let tol = Tolerance::default_geom();
        surface.u_closed = surface.compute_u_closed(tol);
        surface.v_closed = surface.compute_v_closed(tol);
        surface.pole_v_start = surface.compute_pole_v_start(tol);
        surface.pole_v_end = surface.compute_pole_v_end(tol);

        Ok(surface)
    }

    fn expected_knot_lengths(&self) -> (usize, usize) {
        (
            self.u_count + self.degree_u + 1,
            self.v_count + self.degree_v + 1,
        )
    }

    fn control_hpoint(&self, u_index: usize, v_index: usize) -> HPoint4 {
        let idx = v_index * self.u_count + u_index;
        let p = self.control_points[idx];
        let w = self
            .weights
            .as_ref()
            .and_then(|weights| weights.get(idx).copied())
            .unwrap_or(1.0);
        HPoint4::new(p.x * w, p.y * w, p.z * w, w)
    }

    fn point_at_clamped(&self, u: f64, v: f64) -> Point3 {
        if self.u_count < 2 || self.v_count < 2 || self.control_points.is_empty() {
            return Point3::new(0.0, 0.0, 0.0);
        }

        let p = self.degree_u;
        let q = self.degree_v;
        if p == 0 || q == 0 || p >= self.u_count || q >= self.v_count {
            return self.control_points[0];
        }

        let (expected_u_knots, expected_v_knots) = self.expected_knot_lengths();
        if self.knots_u.len() != expected_u_knots
            || self.knots_v.len() != expected_v_knots
            || !is_non_decreasing(&self.knots_u)
            || !is_non_decreasing(&self.knots_v)
        {
            return self.control_points[0];
        }

        let (u0, u1) = self.domain_u();
        let (v0, v1) = self.domain_v();
        let u = u.clamp(u0, u1);
        let v = v.clamp(v0, v1);

        let nu = self.u_count - 1;
        let nv = self.v_count - 1;
        let span_u = find_span(nu, p, u, &self.knots_u);
        let span_v = find_span(nv, q, v, &self.knots_v);

        let mut temp = vec![HPoint4::new(0.0, 0.0, 0.0, 0.0); q + 1];

        for l in 0..=q {
            let v_index = span_v - q + l;
            let mut d = Vec::with_capacity(p + 1);
            for j in 0..=p {
                let u_index = span_u - p + j;
                d.push(self.control_hpoint(u_index, v_index));
            }
            de_boor(&mut d, span_u, p, u, &self.knots_u);
            temp[l] = d[p];
        }

        de_boor(&mut temp, span_v, q, v, &self.knots_v);
        temp[q].to_point3().unwrap_or_else(|| self.control_points[0])
    }

    fn compute_u_closed(&self, tol: Tolerance) -> bool {
        let (u0, u1) = self.domain_u();
        let (v0, v1) = self.domain_v();
        let u_span = u1 - u0;
        let v_span = v1 - v0;
        if !u_span.is_finite() || u_span == 0.0 || !v_span.is_finite() {
            return false;
        }

        let v_mid = v0 + 0.5 * v_span;
        tol.approx_eq_point3(self.point_at_clamped(u0, v0), self.point_at_clamped(u1, v0))
            && tol.approx_eq_point3(
                self.point_at_clamped(u0, v_mid),
                self.point_at_clamped(u1, v_mid),
            )
            && tol.approx_eq_point3(self.point_at_clamped(u0, v1), self.point_at_clamped(u1, v1))
    }

    fn compute_v_closed(&self, tol: Tolerance) -> bool {
        let (u0, u1) = self.domain_u();
        let (v0, v1) = self.domain_v();
        let u_span = u1 - u0;
        let v_span = v1 - v0;
        if !v_span.is_finite() || v_span == 0.0 || !u_span.is_finite() {
            return false;
        }

        let u_mid = u0 + 0.5 * u_span;
        tol.approx_eq_point3(self.point_at_clamped(u0, v0), self.point_at_clamped(u0, v1))
            && tol.approx_eq_point3(
                self.point_at_clamped(u_mid, v0),
                self.point_at_clamped(u_mid, v1),
            )
            && tol.approx_eq_point3(self.point_at_clamped(u1, v0), self.point_at_clamped(u1, v1))
    }

    fn compute_pole_v_start(&self, tol: Tolerance) -> bool {
        if self.v_closed {
            return false;
        }

        let (u0, u1) = self.domain_u();
        let (v0, v1) = self.domain_v();
        let u_span = u1 - u0;
        let v_span = v1 - v0;
        if !u_span.is_finite() || u_span == 0.0 || !v_span.is_finite() {
            return false;
        }

        let u25 = u0 + 0.25 * u_span;
        let u50 = u0 + 0.50 * u_span;
        let u75 = u0 + 0.75 * u_span;

        let p0 = self.point_at_clamped(u0, v0);
        tol.approx_eq_point3(p0, self.point_at_clamped(u25, v0))
            && tol.approx_eq_point3(p0, self.point_at_clamped(u50, v0))
            && tol.approx_eq_point3(p0, self.point_at_clamped(u75, v0))
            && tol.approx_eq_point3(p0, self.point_at_clamped(u1, v0))
    }

    fn compute_pole_v_end(&self, tol: Tolerance) -> bool {
        if self.v_closed {
            return false;
        }

        let (u0, u1) = self.domain_u();
        let (v0, v1) = self.domain_v();
        let u_span = u1 - u0;
        let v_span = v1 - v0;
        if !u_span.is_finite() || u_span == 0.0 || !v_span.is_finite() {
            return false;
        }

        let u25 = u0 + 0.25 * u_span;
        let u50 = u0 + 0.50 * u_span;
        let u75 = u0 + 0.75 * u_span;

        let p0 = self.point_at_clamped(u0, v1);
        tol.approx_eq_point3(p0, self.point_at_clamped(u25, v1))
            && tol.approx_eq_point3(p0, self.point_at_clamped(u50, v1))
            && tol.approx_eq_point3(p0, self.point_at_clamped(u75, v1))
            && tol.approx_eq_point3(p0, self.point_at_clamped(u1, v1))
    }
}

impl Surface for NurbsSurface {
    fn point_at(&self, u: f64, v: f64) -> Point3 {
        let (u0, u1) = self.domain_u();
        let (v0, v1) = self.domain_v();
        let u = if self.u_closed {
            wrap_param(u, u0, u1)
        } else {
            u.clamp(u0, u1)
        };
        let v = if self.v_closed {
            wrap_param(v, v0, v1)
        } else {
            v.clamp(v0, v1)
        };
        self.point_at_clamped(u, v)
    }

    fn domain_u(&self) -> (f64, f64) {
        if self.u_count < 2 || self.control_points.is_empty() || self.knots_u.is_empty() {
            return (0.0, 0.0);
        }

        let p = self.degree_u;
        let expected = self.u_count + p + 1;
        if p == 0 || p >= self.u_count || self.knots_u.len() != expected || !is_non_decreasing(&self.knots_u) {
            return (0.0, 0.0);
        }

        let start = self.knots_u[p];
        let end = self.knots_u[self.u_count];
        (start, end)
    }

    fn domain_v(&self) -> (f64, f64) {
        if self.v_count < 2 || self.control_points.is_empty() || self.knots_v.is_empty() {
            return (0.0, 0.0);
        }

        let q = self.degree_v;
        let expected = self.v_count + q + 1;
        if q == 0 || q >= self.v_count || self.knots_v.len() != expected || !is_non_decreasing(&self.knots_v) {
            return (0.0, 0.0);
        }

        let start = self.knots_v[q];
        let end = self.knots_v[self.v_count];
        (start, end)
    }

    fn cache_key(&self) -> SurfaceCacheKey {
        SurfaceCacheKey::Nurbs { hash: self.cache_hash }
    }

    fn is_u_closed(&self) -> bool {
        self.u_closed
    }

    fn is_v_closed(&self) -> bool {
        self.v_closed
    }

    fn pole_v_start(&self) -> bool {
        self.pole_v_start
    }

    fn pole_v_end(&self) -> bool {
        self.pole_v_end
    }

    fn partial_derivatives_at(&self, u: f64, v: f64) -> (Vec3, Vec3) {
        if self.u_count < 2 || self.v_count < 2 || self.control_points.is_empty() {
            return (Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0));
        }

        let p = self.degree_u;
        let q = self.degree_v;
        if p == 0 || q == 0 || p >= self.u_count || q >= self.v_count {
            return (Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0));
        }

        let (expected_u_knots, expected_v_knots) = self.expected_knot_lengths();
        if self.knots_u.len() != expected_u_knots
            || self.knots_v.len() != expected_v_knots
            || !is_non_decreasing(&self.knots_u)
            || !is_non_decreasing(&self.knots_v)
        {
            return (Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0));
        }

        let (u0, u1) = self.domain_u();
        let (v0, v1) = self.domain_v();
        let u = if self.u_closed {
            wrap_param(u, u0, u1)
        } else {
            u.clamp(u0, u1)
        };
        let v = if self.v_closed {
            wrap_param(v, v0, v1)
        } else {
            v.clamp(v0, v1)
        };

        let nu = self.u_count - 1;
        let nv = self.v_count - 1;
        let span_u = find_span(nu, p, u, &self.knots_u);
        let span_v = find_span(nv, q, v, &self.knots_v);

        let mut temp: Vec<HPoint4> = Vec::with_capacity(q + 1);
        let mut temp_du: Vec<HPoint4> = Vec::with_capacity(q + 1);

        let knots_u_der = &self.knots_u[1..self.knots_u.len().saturating_sub(1)];
        let span_u_der = span_u.saturating_sub(1);
        let p_der = p.saturating_sub(1);

        for l in 0..=q {
            let v_index = span_v - q + l;

            let mut row_ctrl = Vec::with_capacity(p + 1);
            for j in 0..=p {
                let u_index = span_u - p + j;
                row_ctrl.push(self.control_hpoint(u_index, v_index));
            }

            let du = if p == 0 {
                HPoint4::new(0.0, 0.0, 0.0, 0.0)
            } else {
                let mut d_der = Vec::with_capacity(p);
                for k in 0..p {
                    let i = span_u - p + k;
                    let denom = self.knots_u[i + p + 1] - self.knots_u[i + 1];
                    let scale = if denom == 0.0 {
                        0.0
                    } else {
                        p as f64 / denom
                    };
                    d_der.push(row_ctrl[k + 1].sub(row_ctrl[k]).mul_scalar(scale));
                }

                de_boor(&mut d_der, span_u_der, p_der, u, knots_u_der);
                d_der[p_der]
            };

            let mut row_eval = row_ctrl;
            de_boor(&mut row_eval, span_u, p, u, &self.knots_u);
            temp.push(row_eval[p]);
            temp_du.push(du);
        }

        let hv = if q == 0 {
            HPoint4::new(0.0, 0.0, 0.0, 0.0)
        } else {
            let mut d_der = Vec::with_capacity(q);
            for k in 0..q {
                let i = span_v - q + k;
                let denom = self.knots_v[i + q + 1] - self.knots_v[i + 1];
                let scale = if denom == 0.0 {
                    0.0
                } else {
                    q as f64 / denom
                };
                d_der.push(temp[k + 1].sub(temp[k]).mul_scalar(scale));
            }

            let knots_v_der = &self.knots_v[1..self.knots_v.len().saturating_sub(1)];
            let span_v_der = span_v.saturating_sub(1);
            let q_der = q.saturating_sub(1);

            de_boor(&mut d_der, span_v_der, q_der, v, knots_v_der);
            d_der[q_der]
        };

        let mut d_v = temp;
        de_boor(&mut d_v, span_v, q, v, &self.knots_v);
        let h = d_v[q];

        let mut d_du = temp_du;
        de_boor(&mut d_du, span_v, q, v, &self.knots_v);
        let hu = d_du[q];

        if !h.w.is_finite() || h.w == 0.0 {
            return (Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0));
        }

        let w = h.w;
        let inv_w2 = 1.0 / (w * w);

        let du = Vec3::new(
            (hu.x * w - h.x * hu.w) * inv_w2,
            (hu.y * w - h.y * hu.w) * inv_w2,
            (hu.z * w - h.z * hu.w) * inv_w2,
        );

        let dv = Vec3::new(
            (hv.x * w - h.x * hv.w) * inv_w2,
            (hv.y * w - h.y * hv.w) * inv_w2,
            (hv.z * w - h.z * hv.w) * inv_w2,
        );

        (du, dv)
    }
}

#[must_use]
pub fn tessellate_surface_grid(
    surface: &impl Surface,
    u_count: usize,
    v_count: usize,
) -> Vec<Point3> {
    let (u0, u1) = surface.domain_u();
    let (v0, v1) = surface.domain_v();

    let u_span = u1 - u0;
    let v_span = v1 - v0;

    let u_closed = surface.is_u_closed();
    let v_closed = surface.is_v_closed();

    let u_count = if u_closed { u_count.max(3) } else { u_count.max(2) };
    let v_count = if v_closed { v_count.max(3) } else { v_count.max(2) };

    let u_denom = if u_closed {
        u_count as f64
    } else {
        (u_count - 1) as f64
    };
    let v_denom = if v_closed {
        v_count as f64
    } else {
        (v_count - 1) as f64
    };

    let mut points = Vec::with_capacity(u_count * v_count);
    for v in 0..v_count {
        let v_u = v as f64 / v_denom;
        let v_t = if v_span.is_finite() && v_span != 0.0 {
            v0 + v_span * v_u
        } else {
            v0
        };
        for u in 0..u_count {
            let u_u = u as f64 / u_denom;
            let u_t = if u_span.is_finite() && u_span != 0.0 {
                u0 + u_span * u_u
            } else {
                u0
            };
            points.push(surface.point_at(u_t, v_t));
        }
    }
    points
}

// ---------------------------------------------------------------------------
// Surface Ops (DivideSurface / Isotrim / Flip)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClosedSurfaceSampling {
    ExcludeSeam,
    IncludeSeam,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DivideSurfaceOptions {
    pub closed_u: ClosedSurfaceSampling,
    pub closed_v: ClosedSurfaceSampling,
}

impl Default for DivideSurfaceOptions {
    fn default() -> Self {
        Self {
            closed_u: ClosedSurfaceSampling::ExcludeSeam,
            closed_v: ClosedSurfaceSampling::ExcludeSeam,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DivideSurfaceResult {
    pub points: Vec<Point3>,
    pub normals: Vec<Vec3>,
    pub parameters: Vec<(f64, f64)>,
    pub u_count: usize,
    pub v_count: usize,
}

#[must_use]
pub fn divide_surface(
    surface: &impl Surface,
    u_segments: usize,
    v_segments: usize,
    options: DivideSurfaceOptions,
) -> DivideSurfaceResult {
    let u_segments = u_segments.max(1);
    let v_segments = v_segments.max(1);

    let (u0, u1) = surface.domain_u();
    let (v0, v1) = surface.domain_v();
    let u_span = u1 - u0;
    let v_span = v1 - v0;

    let u_closed = surface.is_u_closed();
    let v_closed = surface.is_v_closed();

    let u_exclude_seam = u_closed && options.closed_u == ClosedSurfaceSampling::ExcludeSeam;
    let v_exclude_seam = v_closed && options.closed_v == ClosedSurfaceSampling::ExcludeSeam;

    let u_count = if u_exclude_seam {
        u_segments.max(3)
    } else {
        u_segments + 1
    };
    let v_count = if v_exclude_seam {
        v_segments.max(3)
    } else {
        v_segments + 1
    };

    let u_denom = if u_exclude_seam {
        u_count as f64
    } else {
        u_segments as f64
    };
    let v_denom = if v_exclude_seam {
        v_count as f64
    } else {
        v_segments as f64
    };

    let mut points = Vec::with_capacity(u_count * v_count);
    let mut normals = Vec::with_capacity(u_count * v_count);
    let mut parameters = Vec::with_capacity(u_count * v_count);

    for v in 0..v_count {
        let fv = v as f64 / v_denom;
        let v_param = if v_span.is_finite() && v_span != 0.0 {
            v0 + v_span * fv
        } else {
            v0
        };

        for u in 0..u_count {
            let fu = u as f64 / u_denom;
            let u_param = if u_span.is_finite() && u_span != 0.0 {
                u0 + u_span * fu
            } else {
                u0
            };

            let p = surface.point_at(u_param, v_param);
            let n = surface
                .normal_at(u_param, v_param)
                .unwrap_or_else(|| Vec3::new(0.0, 0.0, 1.0));

            points.push(p);
            normals.push(n);
            parameters.push((u_param, v_param));
        }
    }

    DivideSurfaceResult {
        points,
        normals,
        parameters,
        u_count,
        v_count,
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct IsotrimDiagnostics {
    pub reverse_u: bool,
    pub reverse_v: bool,
    pub clamped_u: bool,
    pub clamped_v: bool,
    pub full_span_u: bool,
    pub full_span_v: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IsotrimSurface<'a, S: Surface + ?Sized> {
    surface: &'a S,
    u_min: f64,
    u_max: f64,
    v_min: f64,
    v_max: f64,
    reverse_u: bool,
    reverse_v: bool,
    u_closed: bool,
    v_closed: bool,
    pole_v_start: bool,
    pole_v_end: bool,
    cache_key: SurfaceCacheKey,
}

impl<'a, S: Surface + ?Sized> IsotrimSurface<'a, S> {
    #[must_use]
    pub const fn u_range(&self) -> (f64, f64) {
        (self.u_min, self.u_max)
    }

    #[must_use]
    pub const fn v_range(&self) -> (f64, f64) {
        (self.v_min, self.v_max)
    }
}

impl<S: Surface + ?Sized> Surface for IsotrimSurface<'_, S> {
    fn point_at(&self, u: f64, v: f64) -> Point3 {
        let u = if self.reverse_u {
            self.u_min + (self.u_max - u)
        } else {
            u
        };
        let v = if self.reverse_v {
            self.v_min + (self.v_max - v)
        } else {
            v
        };
        self.surface.point_at(u, v)
    }

    fn domain_u(&self) -> (f64, f64) {
        (self.u_min, self.u_max)
    }

    fn domain_v(&self) -> (f64, f64) {
        (self.v_min, self.v_max)
    }

    fn is_u_closed(&self) -> bool {
        self.u_closed
    }

    fn is_v_closed(&self) -> bool {
        self.v_closed
    }

    fn pole_v_start(&self) -> bool {
        self.pole_v_start
    }

    fn pole_v_end(&self) -> bool {
        self.pole_v_end
    }

    fn partial_derivatives_at(&self, u: f64, v: f64) -> (Vec3, Vec3) {
        let u = if self.reverse_u {
            self.u_min + (self.u_max - u)
        } else {
            u
        };
        let v = if self.reverse_v {
            self.v_min + (self.v_max - v)
        } else {
            v
        };

        let (du, dv) = self.surface.partial_derivatives_at(u, v);
        (
            if self.reverse_u { du.neg() } else { du },
            if self.reverse_v { dv.neg() } else { dv },
        )
    }

    fn cache_key(&self) -> SurfaceCacheKey {
        self.cache_key
    }
}

#[must_use]
pub fn isotrim_surface<'a, S: Surface + ?Sized>(
    surface: &'a S,
    u_range: (f64, f64),
    v_range: (f64, f64),
    tol: Tolerance,
) -> (IsotrimSurface<'a, S>, IsotrimDiagnostics) {
    let (u0, u1) = surface.domain_u();
    let (v0, v1) = surface.domain_v();

    let mut ua = if u_range.0.is_finite() { u_range.0 } else { u0 };
    let mut ub = if u_range.1.is_finite() { u_range.1 } else { u1 };
    let mut va = if v_range.0.is_finite() { v_range.0 } else { v0 };
    let mut vb = if v_range.1.is_finite() { v_range.1 } else { v1 };

    let ua_clamped = ua.clamp(u0, u1);
    let ub_clamped = ub.clamp(u0, u1);
    let va_clamped = va.clamp(v0, v1);
    let vb_clamped = vb.clamp(v0, v1);

    let mut diagnostics = IsotrimDiagnostics::default();
    diagnostics.clamped_u = ua_clamped != ua || ub_clamped != ub;
    diagnostics.clamped_v = va_clamped != va || vb_clamped != vb;

    ua = ua_clamped;
    ub = ub_clamped;
    va = va_clamped;
    vb = vb_clamped;

    diagnostics.reverse_u = ua > ub;
    diagnostics.reverse_v = va > vb;

    let (u_min, u_max) = if diagnostics.reverse_u { (ub, ua) } else { (ua, ub) };
    let (v_min, v_max) = if diagnostics.reverse_v { (vb, va) } else { (va, vb) };

    diagnostics.full_span_u = tol.approx_eq_f64(u_min, u0) && tol.approx_eq_f64(u_max, u1);
    diagnostics.full_span_v = tol.approx_eq_f64(v_min, v0) && tol.approx_eq_f64(v_max, v1);

    let u_closed = surface.is_u_closed() && diagnostics.full_span_u;
    let v_closed = surface.is_v_closed() && diagnostics.full_span_v;
    let pole_v_start = surface.pole_v_start() && tol.approx_eq_f64(v_min, v0);
    let pole_v_end = surface.pole_v_end() && tol.approx_eq_f64(v_max, v1);

    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    surface.cache_key().hash(&mut hasher);
    u_min.to_bits().hash(&mut hasher);
    u_max.to_bits().hash(&mut hasher);
    v_min.to_bits().hash(&mut hasher);
    v_max.to_bits().hash(&mut hasher);
    diagnostics.reverse_u.hash(&mut hasher);
    diagnostics.reverse_v.hash(&mut hasher);
    let cache_key = SurfaceCacheKey::Nurbs { hash: hasher.finish() };

    (
        IsotrimSurface {
            surface,
            u_min,
            u_max,
            v_min,
            v_max,
            reverse_u: diagnostics.reverse_u,
            reverse_v: diagnostics.reverse_v,
            u_closed,
            v_closed,
            pole_v_start,
            pole_v_end,
            cache_key,
        },
        diagnostics,
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SurfaceFlipGuide {
    Vector(Vec3),
    Point(Point3),
}

#[derive(Debug, Clone, Default)]
pub struct SurfaceFlipDiagnostics {
    pub flipped: bool,
    pub guide_used: bool,
    pub dot_before: Option<f64>,
    pub sample_u: f64,
    pub sample_v: f64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlippedSurface<'a, S: Surface + ?Sized> {
    surface: &'a S,
    flip_u: bool,
    cache_key: SurfaceCacheKey,
}

impl<S: Surface + ?Sized> Surface for FlippedSurface<'_, S> {
    fn point_at(&self, u: f64, v: f64) -> Point3 {
        if !self.flip_u {
            return self.surface.point_at(u, v);
        }
        let (u0, u1) = self.domain_u();
        self.surface.point_at(u0 + (u1 - u), v)
    }

    fn domain_u(&self) -> (f64, f64) {
        self.surface.domain_u()
    }

    fn domain_v(&self) -> (f64, f64) {
        self.surface.domain_v()
    }

    fn is_u_closed(&self) -> bool {
        self.surface.is_u_closed()
    }

    fn is_v_closed(&self) -> bool {
        self.surface.is_v_closed()
    }

    fn pole_v_start(&self) -> bool {
        self.surface.pole_v_start()
    }

    fn pole_v_end(&self) -> bool {
        self.surface.pole_v_end()
    }

    fn partial_derivatives_at(&self, u: f64, v: f64) -> (Vec3, Vec3) {
        if !self.flip_u {
            return self.surface.partial_derivatives_at(u, v);
        }

        let (u0, u1) = self.domain_u();
        let u_mapped = u0 + (u1 - u);
        let (du, dv) = self.surface.partial_derivatives_at(u_mapped, v);
        (du.neg(), dv)
    }

    fn cache_key(&self) -> SurfaceCacheKey {
        self.cache_key
    }
}

#[must_use]
pub fn flip_surface_orientation<'a, S: Surface + ?Sized>(
    surface: &'a S,
    guide: Option<SurfaceFlipGuide>,
) -> (FlippedSurface<'a, S>, SurfaceFlipDiagnostics) {
    let (u0, u1) = surface.domain_u();
    let (v0, v1) = surface.domain_v();

    let u_span = u1 - u0;
    let v_span = v1 - v0;

    let sample_u = if u_span.is_finite() && u_span != 0.0 {
        u0 + 0.5 * u_span
    } else {
        u0
    };
    let sample_v = if v_span.is_finite() && v_span != 0.0 {
        v0 + 0.5 * v_span
    } else {
        v0
    };

    let mut diagnostics = SurfaceFlipDiagnostics {
        sample_u,
        sample_v,
        ..Default::default()
    };

    let should_flip = match guide {
        None => true,
        Some(guide) => {
            diagnostics.guide_used = true;
            let normal = match surface.normal_at(sample_u, sample_v) {
                Some(normal) => normal,
                None => {
                    diagnostics
                        .warnings
                        .push("flip surface: could not compute normal".to_string());
                    return (
                        FlippedSurface {
                            surface,
                            flip_u: false,
                            cache_key: surface.cache_key(),
                        },
                        diagnostics,
                    );
                }
            };

            let desired = match guide {
                SurfaceFlipGuide::Vector(v) => v.normalized(),
                SurfaceFlipGuide::Point(p) => {
                    let origin = surface.point_at(sample_u, sample_v);
                    p.sub_point(origin).normalized()
                }
            };

            let desired = match desired {
                Some(desired) => desired,
                None => {
                    diagnostics
                        .warnings
                        .push("flip surface: guide direction is zero".to_string());
                    return (
                        FlippedSurface {
                            surface,
                            flip_u: false,
                            cache_key: surface.cache_key(),
                        },
                        diagnostics,
                    );
                }
            };

            let dot = normal.dot(desired);
            diagnostics.dot_before = Some(dot);
            dot < 0.0
        }
    };

    diagnostics.flipped = should_flip;

    let cache_key = if should_flip {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        surface.cache_key().hash(&mut hasher);
        true.hash(&mut hasher);
        SurfaceCacheKey::Nurbs { hash: hasher.finish() }
    } else {
        surface.cache_key()
    };

    (
        FlippedSurface {
            surface,
            flip_u: should_flip,
            cache_key,
        },
        diagnostics,
    )
}

// ---------------------------------------------------------------------------
// Surface Builders
// ---------------------------------------------------------------------------

/// A bilinear surface patch defined by four corner points.
///
/// The surface interpolates the four corners using bilinear blending:
/// - `p00` at (u=0, v=0)
/// - `p10` at (u=1, v=0)
/// - `p01` at (u=0, v=1)
/// - `p11` at (u=1, v=1)
///
/// This corresponds to Grasshopper's "4Point Surface" (Srf4Pt) component.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FourPointSurface {
    pub p00: Point3,
    pub p10: Point3,
    pub p01: Point3,
    pub p11: Point3,
}

impl FourPointSurface {
    /// Create a new four-point surface from corner points.
    ///
    /// # Arguments
    /// * `p00` - Corner at (u=0, v=0)
    /// * `p10` - Corner at (u=1, v=0)
    /// * `p01` - Corner at (u=0, v=1)
    /// * `p11` - Corner at (u=1, v=1)
    #[must_use]
    pub const fn new(p00: Point3, p10: Point3, p01: Point3, p11: Point3) -> Self {
        Self { p00, p10, p01, p11 }
    }

    /// Create a four-point surface from an array of points.
    ///
    /// Returns an error if fewer than 3 points are provided.
    /// If exactly 3 points are provided, the fourth is computed as p0 + (p2 - p1).
    pub fn from_points(points: &[Point3]) -> Result<Self, String> {
        if points.len() < 3 {
            return Err("FourPointSurface requires at least 3 corner points".to_string());
        }

        let p00 = points[0];
        let p10 = points[1];
        let p01 = points[2];
        let p11 = if points.len() >= 4 {
            points[3]
        } else {
            // Compute parallelogram completion: p11 = p00 + (p10 - p00) + (p01 - p00) = p10 + p01 - p00
            Point3::new(
                p10.x + p01.x - p00.x,
                p10.y + p01.y - p00.y,
                p10.z + p01.z - p00.z,
            )
        };

        Ok(Self { p00, p10, p01, p11 })
    }
}

impl Surface for FourPointSurface {
    fn point_at(&self, u: f64, v: f64) -> Point3 {
        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        // Bilinear interpolation
        let s = 1.0 - u;
        let t = 1.0 - v;

        Point3::new(
            s * t * self.p00.x + u * t * self.p10.x + s * v * self.p01.x + u * v * self.p11.x,
            s * t * self.p00.y + u * t * self.p10.y + s * v * self.p01.y + u * v * self.p11.y,
            s * t * self.p00.z + u * t * self.p10.z + s * v * self.p01.z + u * v * self.p11.z,
        )
    }

    fn cache_key(&self) -> SurfaceCacheKey {
        SurfaceCacheKey::Plane {
            origin: [
                self.p00.x.to_bits(),
                self.p00.y.to_bits(),
                self.p00.z.to_bits(),
            ],
            u_axis: [
                self.p10.x.to_bits(),
                self.p10.y.to_bits(),
                self.p10.z.to_bits(),
            ],
            v_axis: [
                self.p01.x.to_bits(),
                self.p01.y.to_bits(),
                self.p01.z.to_bits(),
            ],
        }
    }
}

/// A ruled surface created by linear interpolation between two boundary curves.
///
/// For each u parameter, the surface linearly blends between the point on
/// curve A at parameter u and the point on curve B at parameter u.
///
/// This corresponds to Grasshopper's "Ruled Surface" (RuleSrf) component.
#[derive(Debug, Clone, PartialEq)]
pub struct RuledSurface {
    /// Control points of the first boundary curve (at v=0)
    pub curve_a: Vec<Point3>,
    /// Control points of the second boundary curve (at v=1)
    pub curve_b: Vec<Point3>,
}

impl RuledSurface {
    /// Create a new ruled surface from two polylines.
    ///
    /// The polylines are resampled to have equal point counts for consistent interpolation.
    pub fn new(curve_a: Vec<Point3>, curve_b: Vec<Point3>) -> Result<Self, String> {
        if curve_a.len() < 2 {
            return Err("RuledSurface curve A must have at least 2 points".to_string());
        }
        if curve_b.len() < 2 {
            return Err("RuledSurface curve B must have at least 2 points".to_string());
        }

        // Resample to equal point counts
        let target_count = curve_a.len().max(curve_b.len());
        let curve_a = resample_polyline(&curve_a, target_count);
        let curve_b = resample_polyline(&curve_b, target_count);

        Ok(Self { curve_a, curve_b })
    }

    /// Create a ruled surface from raw polyline data without resampling.
    /// Both curves must have the same number of points.
    pub fn from_equal_polylines(curve_a: Vec<Point3>, curve_b: Vec<Point3>) -> Result<Self, String> {
        if curve_a.len() < 2 {
            return Err("RuledSurface curve A must have at least 2 points".to_string());
        }
        if curve_a.len() != curve_b.len() {
            return Err("RuledSurface curves must have equal point counts".to_string());
        }
        Ok(Self { curve_a, curve_b })
    }
}

impl Surface for RuledSurface {
    fn point_at(&self, u: f64, v: f64) -> Point3 {
        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        let n = self.curve_a.len();
        if n == 0 {
            return Point3::new(0.0, 0.0, 0.0);
        }

        // Find the two adjacent points on each curve
        let t = u * (n - 1) as f64;
        let i = (t.floor() as usize).min(n - 2);
        let frac = t - i as f64;

        // Interpolate along curve A
        let pa = lerp_point3(self.curve_a[i], self.curve_a[i + 1], frac);
        // Interpolate along curve B
        let pb = lerp_point3(self.curve_b[i], self.curve_b[i + 1], frac);

        // Linear blend between the two curves
        lerp_point3(pa, pb, v)
    }

    fn cache_key(&self) -> SurfaceCacheKey {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for p in &self.curve_a {
            p.x.to_bits().hash(&mut hasher);
            p.y.to_bits().hash(&mut hasher);
            p.z.to_bits().hash(&mut hasher);
        }
        for p in &self.curve_b {
            p.x.to_bits().hash(&mut hasher);
            p.y.to_bits().hash(&mut hasher);
            p.z.to_bits().hash(&mut hasher);
        }
        SurfaceCacheKey::Nurbs {
            hash: hasher.finish(),
        }
    }
}

/// A Coons patch surface defined by four boundary curves.
///
/// The surface blends between four boundary curves using bilinear Coons interpolation:
/// - `edge_u0`: curve at v=0 (bottom edge, parametrized along u)
/// - `edge_u1`: curve at v=1 (top edge, parametrized along u)
/// - `edge_v0`: curve at u=0 (left edge, parametrized along v)
/// - `edge_v1`: curve at u=1 (right edge, parametrized along v)
///
/// **Important**: Edge direction matters!
/// - `edge_u0` and `edge_u1` must be oriented left-to-right (u=0 to u=1)
/// - `edge_v0` and `edge_v1` must be oriented bottom-to-top (v=0 to v=1)
/// - Corners are extracted from the first/last points of each edge, so reversed
///   edges will produce incorrect surface geometry.
///
/// This corresponds to Grasshopper's "Edge Surface" (EdgeSrf) component.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeSurface {
    /// Bottom edge (v=0), points parametrized from u=0 to u=1
    pub edge_u0: Vec<Point3>,
    /// Top edge (v=1), points parametrized from u=0 to u=1
    pub edge_u1: Vec<Point3>,
    /// Left edge (u=0), points parametrized from v=0 to v=1
    pub edge_v0: Vec<Point3>,
    /// Right edge (u=1), points parametrized from v=0 to v=1
    pub edge_v1: Vec<Point3>,
    /// Corner points: [p00, p10, p01, p11]
    corners: [Point3; 4],
}

impl EdgeSurface {
    /// Create a new edge surface from four boundary curves.
    ///
    /// The curves are expected to form a closed quadrilateral boundary.
    /// If corners don't match exactly, the surface uses the edge curve endpoints.
    pub fn new(
        edge_u0: Vec<Point3>,
        edge_u1: Vec<Point3>,
        edge_v0: Vec<Point3>,
        edge_v1: Vec<Point3>,
    ) -> Result<Self, String> {
        if edge_u0.len() < 2 {
            return Err("EdgeSurface edge_u0 must have at least 2 points".to_string());
        }
        if edge_u1.len() < 2 {
            return Err("EdgeSurface edge_u1 must have at least 2 points".to_string());
        }
        if edge_v0.len() < 2 {
            return Err("EdgeSurface edge_v0 must have at least 2 points".to_string());
        }
        if edge_v1.len() < 2 {
            return Err("EdgeSurface edge_v1 must have at least 2 points".to_string());
        }

        // Extract corner points from edge endpoints
        let p00 = edge_u0[0];
        let p10 = *edge_u0.last().unwrap();
        let p01 = *edge_u1.first().unwrap();
        let p11 = *edge_u1.last().unwrap();

        Ok(Self {
            edge_u0,
            edge_u1,
            edge_v0,
            edge_v1,
            corners: [p00, p10, p01, p11],
        })
    }

    /// Create an edge surface from a list of edge polylines (2-4 edges).
    ///
    /// If 2 edges are provided, creates a ruled surface behavior.
    /// If 3-4 edges are provided, constructs a Coons patch.
    pub fn from_edges(edges: &[Vec<Point3>]) -> Result<Self, String> {
        if edges.len() < 2 {
            return Err("EdgeSurface requires at least 2 boundary edges".to_string());
        }

        // For simplicity, support 2 and 4 edge cases
        if edges.len() == 2 {
            // Treat as ruled surface: edge_u0 and edge_u1 are the two curves,
            // edge_v0 and edge_v1 are synthetic straight lines
            let edge_u0 = edges[0].clone();
            let edge_u1 = edges[1].clone();

            if edge_u0.len() < 2 || edge_u1.len() < 2 {
                return Err("EdgeSurface edges must have at least 2 points".to_string());
            }

            let p00 = edge_u0[0];
            let p10 = *edge_u0.last().unwrap();
            let p01 = edge_u1[0];
            let p11 = *edge_u1.last().unwrap();

            let edge_v0 = vec![p00, p01];
            let edge_v1 = vec![p10, p11];

            Ok(Self {
                edge_u0,
                edge_u1,
                edge_v0,
                edge_v1,
                corners: [p00, p10, p01, p11],
            })
        } else {
            // 4-edge case: use first 4 edges
            let edge_u0 = edges[0].clone();
            let edge_u1 = if edges.len() > 2 {
                edges[2].clone()
            } else {
                edges[1].clone()
            };
            let edge_v0 = if edges.len() > 1 {
                edges[1].clone()
            } else {
                vec![edge_u0[0], edge_u1[0]]
            };
            let edge_v1 = if edges.len() > 3 {
                edges[3].clone()
            } else {
                vec![*edge_u0.last().unwrap(), *edge_u1.last().unwrap()]
            };

            Self::new(edge_u0, edge_u1, edge_v0, edge_v1)
        }
    }

    /// Evaluate a polyline at parameter t in [0, 1]
    fn eval_curve(curve: &[Point3], t: f64) -> Point3 {
        let t = t.clamp(0.0, 1.0);
        let n = curve.len();
        if n == 0 {
            return Point3::new(0.0, 0.0, 0.0);
        }
        if n == 1 {
            return curve[0];
        }

        let param = t * (n - 1) as f64;
        let i = (param.floor() as usize).min(n - 2);
        let frac = param - i as f64;

        lerp_point3(curve[i], curve[i + 1], frac)
    }
}

impl Surface for EdgeSurface {
    fn point_at(&self, u: f64, v: f64) -> Point3 {
        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        // Bilinear Coons patch formula:
        // S(u,v) = (1-v)*C_u0(u) + v*C_u1(u) + (1-u)*C_v0(v) + u*C_v1(v)
        //        - [(1-u)*(1-v)*P00 + u*(1-v)*P10 + (1-u)*v*P01 + u*v*P11]

        let cu0 = Self::eval_curve(&self.edge_u0, u);
        let cu1 = Self::eval_curve(&self.edge_u1, u);
        let cv0 = Self::eval_curve(&self.edge_v0, v);
        let cv1 = Self::eval_curve(&self.edge_v1, v);

        let [p00, p10, p01, p11] = self.corners;

        let s = 1.0 - u;
        let t = 1.0 - v;

        // Ruled surface in u direction
        let ru = Point3::new(
            t * cu0.x + v * cu1.x,
            t * cu0.y + v * cu1.y,
            t * cu0.z + v * cu1.z,
        );

        // Ruled surface in v direction
        let rv = Point3::new(
            s * cv0.x + u * cv1.x,
            s * cv0.y + u * cv1.y,
            s * cv0.z + u * cv1.z,
        );

        // Bilinear correction term
        let bl = Point3::new(
            s * t * p00.x + u * t * p10.x + s * v * p01.x + u * v * p11.x,
            s * t * p00.y + u * t * p10.y + s * v * p01.y + u * v * p11.y,
            s * t * p00.z + u * t * p10.z + s * v * p01.z + u * v * p11.z,
        );

        // Coons patch: ru + rv - bl
        Point3::new(
            ru.x + rv.x - bl.x,
            ru.y + rv.y - bl.y,
            ru.z + rv.z - bl.z,
        )
    }

    fn cache_key(&self) -> SurfaceCacheKey {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for p in &self.edge_u0 {
            p.x.to_bits().hash(&mut hasher);
            p.y.to_bits().hash(&mut hasher);
            p.z.to_bits().hash(&mut hasher);
        }
        for p in &self.edge_u1 {
            p.x.to_bits().hash(&mut hasher);
            p.y.to_bits().hash(&mut hasher);
            p.z.to_bits().hash(&mut hasher);
        }
        for p in &self.edge_v0 {
            p.x.to_bits().hash(&mut hasher);
            p.y.to_bits().hash(&mut hasher);
            p.z.to_bits().hash(&mut hasher);
        }
        for p in &self.edge_v1 {
            p.x.to_bits().hash(&mut hasher);
            p.y.to_bits().hash(&mut hasher);
            p.z.to_bits().hash(&mut hasher);
        }
        SurfaceCacheKey::Nurbs {
            hash: hasher.finish(),
        }
    }
}

/// A sum surface (translational surface) created by translating one curve along another.
///
/// Given two profile curves A and B, the surface point at (u, v) is computed as:
/// `S(u, v) = A(u) + B(v) - origin`
///
/// where `origin` is typically the start point of one of the curves.
///
/// This corresponds to Grasshopper's "Sum Surface" (SumSrf) component.
#[derive(Debug, Clone, PartialEq)]
pub struct SumSurface {
    /// The U-direction profile curve
    pub curve_u: Vec<Point3>,
    /// The V-direction profile curve
    pub curve_v: Vec<Point3>,
    /// The origin point (typically curve_u[0] or curve_v[0])
    pub origin: Point3,
}

impl SumSurface {
    /// Create a new sum surface from two profile curves.
    ///
    /// The origin is set to the start point of `curve_u`.
    pub fn new(curve_u: Vec<Point3>, curve_v: Vec<Point3>) -> Result<Self, String> {
        if curve_u.len() < 2 {
            return Err("SumSurface curve_u must have at least 2 points".to_string());
        }
        if curve_v.len() < 2 {
            return Err("SumSurface curve_v must have at least 2 points".to_string());
        }

        let origin = curve_u[0];

        Ok(Self {
            curve_u,
            curve_v,
            origin,
        })
    }

    /// Create a sum surface with a custom origin point.
    pub fn with_origin(
        curve_u: Vec<Point3>,
        curve_v: Vec<Point3>,
        origin: Point3,
    ) -> Result<Self, String> {
        if curve_u.len() < 2 {
            return Err("SumSurface curve_u must have at least 2 points".to_string());
        }
        if curve_v.len() < 2 {
            return Err("SumSurface curve_v must have at least 2 points".to_string());
        }

        Ok(Self {
            curve_u,
            curve_v,
            origin,
        })
    }

    /// Evaluate a polyline at parameter t in [0, 1]
    fn eval_curve(curve: &[Point3], t: f64) -> Point3 {
        let t = t.clamp(0.0, 1.0);
        let n = curve.len();
        if n == 0 {
            return Point3::new(0.0, 0.0, 0.0);
        }
        if n == 1 {
            return curve[0];
        }

        let param = t * (n - 1) as f64;
        let i = (param.floor() as usize).min(n - 2);
        let frac = param - i as f64;

        lerp_point3(curve[i], curve[i + 1], frac)
    }
}

impl Surface for SumSurface {
    fn point_at(&self, u: f64, v: f64) -> Point3 {
        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        let pu = Self::eval_curve(&self.curve_u, u);
        let pv = Self::eval_curve(&self.curve_v, v);

        // S(u,v) = A(u) + B(v) - origin
        Point3::new(
            pu.x + pv.x - self.origin.x,
            pu.y + pv.y - self.origin.y,
            pu.z + pv.z - self.origin.z,
        )
    }

    fn cache_key(&self) -> SurfaceCacheKey {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for p in &self.curve_u {
            p.x.to_bits().hash(&mut hasher);
            p.y.to_bits().hash(&mut hasher);
            p.z.to_bits().hash(&mut hasher);
        }
        for p in &self.curve_v {
            p.x.to_bits().hash(&mut hasher);
            p.y.to_bits().hash(&mut hasher);
            p.z.to_bits().hash(&mut hasher);
        }
        self.origin.x.to_bits().hash(&mut hasher);
        self.origin.y.to_bits().hash(&mut hasher);
        self.origin.z.to_bits().hash(&mut hasher);
        SurfaceCacheKey::Nurbs {
            hash: hasher.finish(),
        }
    }
}

/// A network surface created from two sets of curves (U-curves and V-curves).
///
/// The surface interpolates through the intersection points of the U and V curve networks.
/// This is a simplified implementation that creates a NURBS-like surface from the
/// intersection grid.
///
/// This corresponds to Grasshopper's "Network Surface" (NetSurf) component.
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkSurface {
    /// Grid of control/interpolation points [v_count][u_count]
    pub grid: Vec<Vec<Point3>>,
    /// Number of points in U direction
    pub u_count: usize,
    /// Number of points in V direction
    pub v_count: usize,
}

impl NetworkSurface {
    /// Create a network surface from U-curves and V-curves.
    ///
    /// The curves are expected to form a proper network where U-curves
    /// and V-curves intersect. This implementation samples the curves
    /// uniformly and creates an interpolation grid.
    pub fn new(u_curves: &[Vec<Point3>], v_curves: &[Vec<Point3>]) -> Result<Self, String> {
        if u_curves.is_empty() {
            return Err("NetworkSurface requires at least one U-curve".to_string());
        }
        if v_curves.is_empty() {
            return Err("NetworkSurface requires at least one V-curve".to_string());
        }

        // Determine grid size
        let u_count = v_curves.len().max(2);
        let v_count = u_curves.len().max(2);

        // Sample each U-curve at u_count points
        let mut grid = Vec::with_capacity(v_count);
        for u_curve in u_curves.iter() {
            if u_curve.len() < 2 {
                continue;
            }
            let mut row = Vec::with_capacity(u_count);
            for i in 0..u_count {
                let t = i as f64 / (u_count - 1).max(1) as f64;
                row.push(eval_polyline(u_curve, t));
            }
            grid.push(row);
        }

        // If we have fewer rows than needed, pad with the last row
        while grid.len() < v_count {
            if let Some(last) = grid.last().cloned() {
                grid.push(last);
            } else {
                grid.push(vec![Point3::new(0.0, 0.0, 0.0); u_count]);
            }
        }

        // Optionally blend with V-curves for better accuracy
        // For simplicity, this implementation just uses the U-curve samples
        // A full implementation would solve a network interpolation problem

        Ok(Self {
            grid,
            u_count,
            v_count,
        })
    }

    /// Create a network surface from a pre-computed grid of points.
    pub fn from_grid(grid: Vec<Vec<Point3>>) -> Result<Self, String> {
        if grid.is_empty() {
            return Err("NetworkSurface grid cannot be empty".to_string());
        }
        let v_count = grid.len();
        let u_count = grid[0].len();
        if u_count < 2 || v_count < 2 {
            return Err("NetworkSurface grid must be at least 2x2".to_string());
        }
        for row in &grid {
            if row.len() != u_count {
                return Err("NetworkSurface grid rows must have equal length".to_string());
            }
        }
        Ok(Self {
            grid,
            u_count,
            v_count,
        })
    }
}

impl Surface for NetworkSurface {
    fn point_at(&self, u: f64, v: f64) -> Point3 {
        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        if self.grid.is_empty() || self.grid[0].is_empty() {
            return Point3::new(0.0, 0.0, 0.0);
        }

        // Bilinear interpolation over the grid
        let u_param = u * (self.u_count - 1) as f64;
        let v_param = v * (self.v_count - 1) as f64;

        let i = (u_param.floor() as usize).min(self.u_count - 2);
        let j = (v_param.floor() as usize).min(self.v_count - 2);

        let u_frac = u_param - i as f64;
        let v_frac = v_param - j as f64;

        // Get the four corner points of the cell
        let p00 = self.grid[j][i];
        let p10 = self.grid[j][i + 1];
        let p01 = self.grid[j + 1][i];
        let p11 = self.grid[j + 1][i + 1];

        // Bilinear interpolation
        let s = 1.0 - u_frac;
        let t = 1.0 - v_frac;

        Point3::new(
            s * t * p00.x + u_frac * t * p10.x + s * v_frac * p01.x + u_frac * v_frac * p11.x,
            s * t * p00.y + u_frac * t * p10.y + s * v_frac * p01.y + u_frac * v_frac * p11.y,
            s * t * p00.z + u_frac * t * p10.z + s * v_frac * p01.z + u_frac * v_frac * p11.z,
        )
    }

    fn cache_key(&self) -> SurfaceCacheKey {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.u_count.hash(&mut hasher);
        self.v_count.hash(&mut hasher);
        for row in &self.grid {
            for p in row {
                p.x.to_bits().hash(&mut hasher);
                p.y.to_bits().hash(&mut hasher);
                p.z.to_bits().hash(&mut hasher);
            }
        }
        SurfaceCacheKey::Nurbs {
            hash: hasher.finish(),
        }
    }
}

// ---------------------------------------------------------------------------
// Helper functions for surface builders
// ---------------------------------------------------------------------------

/// Linear interpolation between two points.
fn lerp_point3(a: Point3, b: Point3, t: f64) -> Point3 {
    let s = 1.0 - t;
    Point3::new(s * a.x + t * b.x, s * a.y + t * b.y, s * a.z + t * b.z)
}

/// Evaluate a polyline at parameter t in [0, 1].
///
/// Returns the interpolated point along the polyline. Handles edge cases:
/// - Empty polyline: returns origin (0, 0, 0)
/// - Single point: returns that point
/// - Multiple points: linear interpolation between segments
fn eval_polyline(polyline: &[Point3], t: f64) -> Point3 {
    let t = t.clamp(0.0, 1.0);
    let n = polyline.len();
    if n == 0 {
        return Point3::new(0.0, 0.0, 0.0);
    }
    if n == 1 {
        return polyline[0];
    }

    // Safety: n >= 2 here, so (n - 1) >= 1 and division is safe
    let param = t * (n - 1) as f64;
    let i = (param.floor() as usize).min(n - 2);
    let frac = param - i as f64;

    lerp_point3(polyline[i], polyline[i + 1], frac)
}

/// Resample a polyline to have exactly `target_count` points.
fn resample_polyline(polyline: &[Point3], target_count: usize) -> Vec<Point3> {
    if polyline.is_empty() || target_count == 0 {
        return Vec::new();
    }
    if target_count == 1 {
        return vec![polyline[0]];
    }

    let mut result = Vec::with_capacity(target_count);
    for i in 0..target_count {
        let t = i as f64 / (target_count - 1) as f64;
        result.push(eval_polyline(polyline, t));
    }
    result
}
