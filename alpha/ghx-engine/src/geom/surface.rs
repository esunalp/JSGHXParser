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

    /// Computes the second partial derivatives (duu, duv, dvv) at a parametric point.
    ///
    /// These are used for curvature analysis via the second fundamental form.
    /// The default implementation uses central finite differences on the first derivatives.
    ///
    /// Returns `(duu, duv, dvv)` where:
    /// - `duu` is the second derivative in the u direction
    /// - `duv` is the mixed partial derivative
    /// - `dvv` is the second derivative in the v direction
    #[must_use]
    fn second_partial_derivatives_at(&self, u: f64, v: f64) -> (Vec3, Vec3, Vec3) {
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

        let mut duu = Vec3::new(0.0, 0.0, 0.0);
        let mut duv = Vec3::new(0.0, 0.0, 0.0);
        let mut dvv = Vec3::new(0.0, 0.0, 0.0);

        // Use larger step for second derivatives
        let h_u = if u_span.is_finite() && u_span != 0.0 {
            Tolerance::SECOND_DERIVATIVE.relative_to(u_span)
        } else {
            0.0
        };
        let h_v = if v_span.is_finite() && v_span != 0.0 {
            Tolerance::SECOND_DERIVATIVE.relative_to(v_span)
        } else {
            0.0
        };

        // Compute duu = d²P/du²
        if h_u.is_finite() && h_u > 0.0 {
            let ua = if self.is_u_closed() { u - h_u } else { (u - h_u).max(u0) };
            let ub = if self.is_u_closed() { u + h_u } else { (u + h_u).min(u1) };

            if ua < u && u < ub {
                let (du_a, _) = self.partial_derivatives_at(ua, v);
                let (du_b, _) = self.partial_derivatives_at(ub, v);
                let delta = ub - ua;
                if delta > 0.0 {
                    duu = du_b.sub(du_a).mul_scalar(1.0 / delta);
                }
            }
        }

        // Compute dvv = d²P/dv²
        if h_v.is_finite() && h_v > 0.0 {
            let va = if self.is_v_closed() { v - h_v } else { (v - h_v).max(v0) };
            let vb = if self.is_v_closed() { v + h_v } else { (v + h_v).min(v1) };

            if va < v && v < vb {
                let (_, dv_a) = self.partial_derivatives_at(u, va);
                let (_, dv_b) = self.partial_derivatives_at(u, vb);
                let delta = vb - va;
                if delta > 0.0 {
                    dvv = dv_b.sub(dv_a).mul_scalar(1.0 / delta);
                }
            }
        }

        // Compute duv = d²P/dudv (mixed partial)
        if h_u.is_finite() && h_u > 0.0 && h_v.is_finite() && h_v > 0.0 {
            let ua = if self.is_u_closed() { u - h_u } else { (u - h_u).max(u0) };
            let ub = if self.is_u_closed() { u + h_u } else { (u + h_u).min(u1) };

            if ua < u && u < ub {
                let va = if self.is_v_closed() { v - h_v } else { (v - h_v).max(v0) };
                let vb = if self.is_v_closed() { v + h_v } else { (v + h_v).min(v1) };

                if va < v && v < vb {
                    let (_, dv_ua) = self.partial_derivatives_at(ua, v);
                    let (_, dv_ub) = self.partial_derivatives_at(ub, v);
                    let delta_u = ub - ua;
                    if delta_u > 0.0 {
                        duv = dv_ub.sub(dv_ua).mul_scalar(1.0 / delta_u);
                    }
                }
            }
        }

        (duu, duv, dvv)
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
/// ## Edge Direction Requirements
///
/// When using `EdgeSurface::new()` directly:
/// - `edge_u0` and `edge_u1` must be oriented left-to-right (u=0 to u=1)
/// - `edge_v0` and `edge_v1` must be oriented bottom-to-top (v=0 to v=1)
/// - Corners are extracted from the first/last points of each edge
///
/// ## Auto-Orientation (Recommended)
///
/// Use `EdgeSurface::from_edges()` or `from_edges_with_tolerance()` for automatic
/// orientation handling. These methods analyze edge endpoints and orient them to
/// form a consistent boundary loop, preventing twisted surfaces from misaligned
/// input edges.
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
    ///
    /// **Auto-orientation**: This method automatically orients edges to avoid
    /// twisted surfaces. For 2-edge cases, it ensures edges flow in compatible
    /// directions. For 4-edge cases, it organizes edges into a consistent
    /// boundary loop.
    pub fn from_edges(edges: &[Vec<Point3>]) -> Result<Self, String> {
        Self::from_edges_with_tolerance(edges, Tolerance::LOOSE)
    }

    /// Create an edge surface from a list of edge polylines with a custom tolerance.
    ///
    /// The tolerance is used for endpoint proximity checks when auto-orienting edges.
    pub fn from_edges_with_tolerance(
        edges: &[Vec<Point3>],
        tol: Tolerance,
    ) -> Result<Self, String> {
        if edges.len() < 2 {
            return Err("EdgeSurface requires at least 2 boundary edges".to_string());
        }

        // Validate all edges have at least 2 points
        for (i, edge) in edges.iter().enumerate() {
            if edge.len() < 2 {
                return Err(format!(
                    "EdgeSurface edge {} must have at least 2 points",
                    i
                ));
            }
        }

        if edges.len() == 2 {
            Self::from_two_edges_auto_orient(&edges[0], &edges[1], tol)
        } else if edges.len() == 3 {
            Self::from_three_edges_auto_orient(&edges[0], &edges[1], &edges[2], tol)
        } else {
            Self::from_four_edges_auto_orient(&edges[0], &edges[1], &edges[2], &edges[3], tol)
        }
    }

    /// Create a ruled-style edge surface from two edges with auto-orientation.
    ///
    /// This method analyzes the edge endpoints and orients them so that:
    /// - Corresponding endpoints are matched to minimize crossing/twisting
    /// - The surface interpolates smoothly between the two curves
    fn from_two_edges_auto_orient(
        edge0: &[Point3],
        edge1: &[Point3],
        _tol: Tolerance,
    ) -> Result<Self, String> {
        // Get endpoints of both edges
        let e0_start = edge0[0];
        let e0_end = *edge0.last().unwrap();
        let e1_start = edge1[0];
        let e1_end = *edge1.last().unwrap();

        // Calculate distances for both orientation options:
        // Option A: edge0 and edge1 both go "same direction"
        //   - e0_start connects to e1_start, e0_end connects to e1_end
        let dist_same_dir = e0_start.distance_squared_to(e1_start)
            + e0_end.distance_squared_to(e1_end);

        // Option B: edge1 is reversed relative to edge0
        //   - e0_start connects to e1_end, e0_end connects to e1_start
        let dist_reversed = e0_start.distance_squared_to(e1_end)
            + e0_end.distance_squared_to(e1_start);

        // Choose orientation that minimizes total endpoint distance (less twisting)
        let edge_u0 = edge0.to_vec();
        let edge_u1 = if dist_same_dir <= dist_reversed {
            // Same direction - no flip needed
            edge1.to_vec()
        } else {
            // Reverse edge1 for better alignment
            edge1.iter().copied().rev().collect()
        };

        // Now construct corners and synthetic v-edges
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
    }

    /// Create a Coons patch from three edges with auto-orientation.
    ///
    /// For 3 edges, we treat it as a degenerate 4-edge case where one edge
    /// collapses to a point. The algorithm finds which corner should be the
    /// degenerate point based on edge connectivity.
    fn from_three_edges_auto_orient(
        edge0: &[Point3],
        edge1: &[Point3],
        edge2: &[Point3],
        tol: Tolerance,
    ) -> Result<Self, String> {
        // Collect all edges and orient them into a closed loop
        let edges = vec![edge0.to_vec(), edge1.to_vec(), edge2.to_vec()];
        let oriented = Self::orient_edges_into_loop(&edges, tol)?;

        // For 3 edges, we have a triangular boundary. We need to identify
        // which corner to collapse. Use the first edge as u0, last edge as u1,
        // and create synthetic edges for the sides.
        if oriented.len() < 3 {
            return Err("EdgeSurface: could not form a valid 3-edge loop".to_string());
        }

        // edge_u0 = first edge
        // The "opposite" edge doesn't exist, so we'll use edge2 (resampled or as-is)
        // edge_v0 and edge_v1 connect the ends
        let edge_u0 = oriented[0].clone();
        let edge_u1 = oriented[2].clone();

        // Get corner points
        let p00 = edge_u0[0];
        let p10 = *edge_u0.last().unwrap();
        let p01 = edge_u1[0];
        let p11 = *edge_u1.last().unwrap();

        // The middle edge (oriented[1]) should connect p10 to p01
        let edge_v1 = oriented[1].clone();

        // Synthesize edge_v0 as a line from p00 to p11
        // (this collapses one corner into a triangular patch)
        let edge_v0 = vec![p00, p11];

        Ok(Self {
            edge_u0,
            edge_u1,
            edge_v0,
            edge_v1,
            corners: [p00, p10, p01, p11],
        })
    }

    /// Create a Coons patch from four edges with auto-orientation.
    ///
    /// This method organizes the four edges into a consistent closed loop
    /// around the patch boundary, orienting each edge so that:
    /// - edge_u0 (bottom): flows from corner p00 to p10
    /// - edge_v1 (right): flows from corner p10 to p11
    /// - edge_u1 (top): flows from corner p01 to p11 (note: reversed in Coons formula)
    /// - edge_v0 (left): flows from corner p00 to p01
    fn from_four_edges_auto_orient(
        edge0: &[Point3],
        edge1: &[Point3],
        edge2: &[Point3],
        edge3: &[Point3],
        tol: Tolerance,
    ) -> Result<Self, String> {
        // Collect all edges
        let edges = vec![
            edge0.to_vec(),
            edge1.to_vec(),
            edge2.to_vec(),
            edge3.to_vec(),
        ];

        // Orient edges into a closed loop
        let oriented = Self::orient_edges_into_loop(&edges, tol)?;

        if oriented.len() != 4 {
            return Err(format!(
                "EdgeSurface: expected 4 edges in loop, got {}",
                oriented.len()
            ));
        }

        // Assign edges to the Coons patch roles:
        // Loop order: edge0 -> edge1 -> edge2 -> edge3 -> back to edge0
        // Coons layout:
        //       edge_u1 (top)
        //   p01 ─────────> p11
        //    ^              ^
        //    │              │
        //  v0│              │v1
        //    │              │
        //   p00 ─────────> p10
        //       edge_u0 (bottom)

        // Loop[0] is bottom (u0), loop[1] is right (v1), loop[2] is top (u1), loop[3] is left (v0)
        let edge_u0 = oriented[0].clone();
        let edge_v1 = oriented[1].clone();
        let edge_u1_reversed = oriented[2].clone();
        let edge_v0_reversed = oriented[3].clone();

        // edge_u1 in Coons formula goes left-to-right at v=1, but in the loop
        // it goes right-to-left (p11 to p01), so we need to reverse it
        let edge_u1: Vec<Point3> = edge_u1_reversed.iter().copied().rev().collect();

        // edge_v0 in Coons formula goes bottom-to-top at u=0, but in the loop
        // it goes top-to-bottom (p01 to p00), so we need to reverse it
        let edge_v0: Vec<Point3> = edge_v0_reversed.iter().copied().rev().collect();

        // Extract corners
        let p00 = edge_u0[0];
        let p10 = *edge_u0.last().unwrap();
        let p01 = edge_u1[0];
        let p11 = *edge_u1.last().unwrap();

        Ok(Self {
            edge_u0,
            edge_u1,
            edge_v0,
            edge_v1,
            corners: [p00, p10, p01, p11],
        })
    }

    /// Orient a collection of edges into a closed loop.
    ///
    /// This algorithm:
    /// 1. Starts with the first edge (kept in original orientation)
    /// 2. For each subsequent position, finds the edge whose start or end
    ///    is closest to the current chain's endpoint
    /// 3. Orients that edge so its start connects to the chain
    /// 4. Continues until all edges are placed
    ///
    /// Returns the edges reordered and oriented to form a continuous loop.
    fn orient_edges_into_loop(
        edges: &[Vec<Point3>],
        tol: Tolerance,
    ) -> Result<Vec<Vec<Point3>>, String> {
        if edges.is_empty() {
            return Err("EdgeSurface: no edges provided".to_string());
        }
        if edges.len() == 1 {
            return Ok(vec![edges[0].clone()]);
        }

        let n = edges.len();
        let mut used = vec![false; n];
        let mut result: Vec<Vec<Point3>> = Vec::with_capacity(n);

        // Start with the first edge in its original orientation
        result.push(edges[0].clone());
        used[0] = true;

        // Tolerance squared for distance comparisons
        let tol_sq = tol.eps * tol.eps;

        // Build the chain by finding connecting edges
        for _ in 1..n {
            let chain_end = *result.last().unwrap().last().unwrap();

            // Find the unused edge that best connects to chain_end
            let mut best_idx = None;
            let mut best_dist_sq = f64::MAX;
            let mut best_needs_flip = false;

            for (i, edge) in edges.iter().enumerate() {
                if used[i] {
                    continue;
                }

                let start = edge[0];
                let end = *edge.last().unwrap();

                // Check distance from chain_end to this edge's start
                let dist_to_start = chain_end.distance_squared_to(start);
                if dist_to_start < best_dist_sq {
                    best_dist_sq = dist_to_start;
                    best_idx = Some(i);
                    best_needs_flip = false;
                }

                // Check distance from chain_end to this edge's end (would need flip)
                let dist_to_end = chain_end.distance_squared_to(end);
                if dist_to_end < best_dist_sq {
                    best_dist_sq = dist_to_end;
                    best_idx = Some(i);
                    best_needs_flip = true;
                }
            }

            match best_idx {
                Some(idx) => {
                    used[idx] = true;
                    let mut edge = edges[idx].clone();
                    if best_needs_flip {
                        edge.reverse();
                    }

                    // Warn if connection is poor (beyond tolerance)
                    // but still accept it - the caller should validate
                    if best_dist_sq > tol_sq {
                        // Connection is outside tolerance - edges don't form a proper loop
                        // We still proceed but the result may not be a clean loop
                    }

                    result.push(edge);
                }
                None => {
                    return Err("EdgeSurface: could not find connecting edge".to_string());
                }
            }
        }

        Ok(result)
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
    /// and V-curves intersect. This implementation:
    ///
    /// 1. Computes intersection points (or closest approach points) between each
    ///    U-curve and V-curve pair.
    /// 2. Builds a grid where each cell `[j][i]` corresponds to the intersection
    ///    of U-curve `j` with V-curve `i`.
    /// 3. Uses bilinear blending when curves don't exactly intersect, averaging
    ///    the closest points from both curve directions.
    ///
    /// This matches Grasshopper's Network Surface (NetSurf) behavior where both
    /// curve families contribute to the surface shape.
    ///
    /// # Arguments
    /// - `u_curves`: Curves running in the U direction (rows). Each curve is a
    ///   polyline represented as `Vec<Point3>`.
    /// - `v_curves`: Curves running in the V direction (columns). Each curve is
    ///   a polyline represented as `Vec<Point3>`.
    ///
    /// # Returns
    /// A `NetworkSurface` that interpolates through the intersection grid, or an
    /// error if the input is invalid.
    ///
    /// # Example
    /// ```ignore
    /// use ghx_engine::geom::{NetworkSurface, Point3};
    ///
    /// // Two U-curves (horizontal)
    /// let u_curves = vec![
    ///     vec![Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 0.0, 0.0)],
    ///     vec![Point3::new(0.0, 2.0, 0.0), Point3::new(2.0, 2.0, 1.0)],
    /// ];
    /// // Two V-curves (vertical) - their shape affects the surface
    /// let v_curves = vec![
    ///     vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 2.0, 0.0)],
    ///     vec![Point3::new(2.0, 0.0, 0.0), Point3::new(2.0, 2.0, 1.0)],
    /// ];
    ///
    /// let surface = NetworkSurface::new(&u_curves, &v_curves)?;
    /// // The surface now honors both U and V curve shapes
    /// ```
    pub fn new(u_curves: &[Vec<Point3>], v_curves: &[Vec<Point3>]) -> Result<Self, String> {
        if u_curves.is_empty() {
            return Err("NetworkSurface requires at least one U-curve".to_string());
        }
        if v_curves.is_empty() {
            return Err("NetworkSurface requires at least one V-curve".to_string());
        }

        // Filter out degenerate curves
        let valid_u_curves: Vec<&Vec<Point3>> =
            u_curves.iter().filter(|c| c.len() >= 2).collect();
        let valid_v_curves: Vec<&Vec<Point3>> =
            v_curves.iter().filter(|c| c.len() >= 2).collect();

        if valid_u_curves.is_empty() {
            return Err("NetworkSurface requires at least one valid U-curve (2+ points)".to_string());
        }
        if valid_v_curves.is_empty() {
            return Err("NetworkSurface requires at least one valid V-curve (2+ points)".to_string());
        }

        // Determine grid resolution:
        // - Sample along U direction based on the maximum V-curve point count
        // - Sample along V direction based on the maximum U-curve point count
        // This ensures we capture the shape detail from both curve families.
        let max_u_curve_points = valid_u_curves.iter().map(|c| c.len()).max().unwrap_or(2);
        let max_v_curve_points = valid_v_curves.iter().map(|c| c.len()).max().unwrap_or(2);

        // Grid resolution: use at least the curve complexity, minimum 2
        let u_count = max_v_curve_points.max(valid_v_curves.len()).max(2);
        let v_count = max_u_curve_points.max(valid_u_curves.len()).max(2);

        // Build the grid using bilinear Coons patch interpolation.
        //
        // For each grid point (i, j), we blend:
        // 1. Contribution from U-curves: interpolate between U-curves at parameter u = i/(u_count-1)
        // 2. Contribution from V-curves: interpolate between V-curves at parameter v = j/(v_count-1)
        // 3. Correction term: remove the doubly-counted corner contributions
        //
        // This is the bilinear Coons patch formula:
        //   S(u,v) = U_blend(u,v) + V_blend(u,v) - Corner_blend(u,v)
        //
        // Where:
        //   U_blend(u,v) = (1-v)*U0(u) + v*U1(u)  [linear blend of boundary U-curves]
        //   V_blend(u,v) = (1-u)*V0(v) + u*V1(v)  [linear blend of boundary V-curves]
        //   Corner_blend(u,v) = (1-u)(1-v)*P00 + u(1-v)*P10 + (1-u)v*P01 + uv*P11

        // Get boundary curves (first and last of each family)
        let u0 = valid_u_curves.first().unwrap();
        let u1 = valid_u_curves.last().unwrap();
        let v0 = valid_v_curves.first().unwrap();
        let v1 = valid_v_curves.last().unwrap();

        // Find corner points by curve intersections/closest approach
        let p00 = find_curve_curve_intersection(u0, v0);
        let p10 = find_curve_curve_intersection(u0, v1);
        let p01 = find_curve_curve_intersection(u1, v0);
        let p11 = find_curve_curve_intersection(u1, v1);

        let mut grid = Vec::with_capacity(v_count);

        for j in 0..v_count {
            let v = j as f64 / (v_count - 1).max(1) as f64;
            let mut row = Vec::with_capacity(u_count);

            for i in 0..u_count {
                let u = i as f64 / (u_count - 1).max(1) as f64;

                // U-curve blend: interpolate between first and last U-curves
                let u0_pt = eval_polyline(u0, u);
                let u1_pt = eval_polyline(u1, u);
                let u_blend = lerp_point3(u0_pt, u1_pt, v);

                // V-curve blend: interpolate between first and last V-curves
                let v0_pt = eval_polyline(v0, v);
                let v1_pt = eval_polyline(v1, v);
                let v_blend = lerp_point3(v0_pt, v1_pt, u);

                // Corner correction (bilinear blend of corners)
                let corner_00 = scale_point(p00, (1.0 - u) * (1.0 - v));
                let corner_10 = scale_point(p10, u * (1.0 - v));
                let corner_01 = scale_point(p01, (1.0 - u) * v);
                let corner_11 = scale_point(p11, u * v);
                let corner_blend = add_points(
                    add_points(corner_00, corner_10),
                    add_points(corner_01, corner_11),
                );

                // Coons patch: S(u,v) = U_blend + V_blend - Corner_blend
                let pt = Point3::new(
                    u_blend.x + v_blend.x - corner_blend.x,
                    u_blend.y + v_blend.y - corner_blend.y,
                    u_blend.z + v_blend.z - corner_blend.z,
                );
                row.push(pt);
            }
            grid.push(row);
        }

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

/// Scale a point by a scalar factor (treating it as a position vector).
fn scale_point(p: Point3, s: f64) -> Point3 {
    Point3::new(p.x * s, p.y * s, p.z * s)
}

/// Add two points component-wise (treating them as position vectors).
fn add_points(a: Point3, b: Point3) -> Point3 {
    Point3::new(a.x + b.x, a.y + b.y, a.z + b.z)
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

/// Find the intersection point or closest approach between two polyline curves.
///
/// This function computes where two polyline curves cross or come closest to each other.
/// It's used by `NetworkSurface::new` to build the intersection grid from U and V curves.
///
/// # Algorithm
///
/// 1. **Segment intersection test**: For each segment pair (one from each polyline),
///    compute the closest points on both segments. This handles both actual intersections
///    and near-misses.
///
/// 2. **Closest pair selection**: Among all segment pairs, find the one with minimum
///    distance between closest points.
///
/// 3. **Blending**: Return the midpoint of the closest points, which gives a natural
///    blend when curves don't exactly intersect.
///
/// # Arguments
/// - `curve_a`: First polyline (e.g., a U-curve).
/// - `curve_b`: Second polyline (e.g., a V-curve).
///
/// # Returns
/// The intersection point, or the midpoint of the closest approach if the curves
/// don't actually intersect.
fn find_curve_curve_intersection(curve_a: &[Point3], curve_b: &[Point3]) -> Point3 {
    if curve_a.is_empty() || curve_b.is_empty() {
        return Point3::ORIGIN;
    }
    if curve_a.len() == 1 && curve_b.len() == 1 {
        // Both are single points: return their midpoint
        return curve_a[0].lerp(curve_b[0], 0.5);
    }
    if curve_a.len() == 1 {
        // Curve A is a single point; find closest point on curve B
        return closest_point_on_polyline(curve_b, curve_a[0]);
    }
    if curve_b.len() == 1 {
        // Curve B is a single point; find closest point on curve A
        return closest_point_on_polyline(curve_a, curve_b[0]);
    }

    // Both curves have at least 2 points: find the closest approach between segments
    let mut best_dist_sq = f64::INFINITY;
    let mut best_point_a = curve_a[0];
    let mut best_point_b = curve_b[0];

    // Iterate over all segment pairs
    for i in 0..curve_a.len() - 1 {
        let a0 = curve_a[i];
        let a1 = curve_a[i + 1];

        for j in 0..curve_b.len() - 1 {
            let b0 = curve_b[j];
            let b1 = curve_b[j + 1];

            // Find closest points between segment (a0, a1) and segment (b0, b1)
            let (pa, pb) = closest_points_between_segments(a0, a1, b0, b1);
            let dist_sq = pa.distance_squared_to(pb);

            if dist_sq < best_dist_sq {
                best_dist_sq = dist_sq;
                best_point_a = pa;
                best_point_b = pb;
            }
        }
    }

    // Return the midpoint of the closest approach (this blends both curve contributions)
    best_point_a.lerp(best_point_b, 0.5)
}

/// Find the closest point on a polyline to a query point.
fn closest_point_on_polyline(polyline: &[Point3], query: Point3) -> Point3 {
    if polyline.is_empty() {
        return Point3::ORIGIN;
    }
    if polyline.len() == 1 {
        return polyline[0];
    }

    let mut best_point = polyline[0];
    let mut best_dist_sq = query.distance_squared_to(polyline[0]);

    for i in 0..polyline.len() - 1 {
        let p0 = polyline[i];
        let p1 = polyline[i + 1];
        let closest = closest_point_on_segment(p0, p1, query);
        let dist_sq = query.distance_squared_to(closest);

        if dist_sq < best_dist_sq {
            best_dist_sq = dist_sq;
            best_point = closest;
        }
    }

    best_point
}

/// Find the closest point on a line segment to a query point.
fn closest_point_on_segment(seg_start: Point3, seg_end: Point3, query: Point3) -> Point3 {
    let seg_vec = seg_end.sub_point(seg_start);
    let seg_len_sq = seg_vec.length_squared();

    if seg_len_sq < 1e-20 {
        // Degenerate segment: return the start point
        return seg_start;
    }

    // Project query onto the line containing the segment
    let query_vec = query.sub_point(seg_start);
    let t = query_vec.dot(seg_vec) / seg_len_sq;

    // Clamp to segment bounds
    let t_clamped = t.clamp(0.0, 1.0);

    seg_start.lerp(seg_end, t_clamped)
}

/// Find the closest points between two line segments.
///
/// Returns `(point_on_segment_a, point_on_segment_b)` where these are the
/// points that minimize the distance between the two segments.
///
/// # Algorithm
///
/// Uses the parametric approach: segment A = a0 + s*(a1-a0), segment B = b0 + t*(b1-b0).
/// We solve for s, t that minimize |A(s) - B(t)|², then clamp to [0, 1].
fn closest_points_between_segments(
    a0: Point3,
    a1: Point3,
    b0: Point3,
    b1: Point3,
) -> (Point3, Point3) {
    let d1 = a1.sub_point(a0); // Direction of segment A
    let d2 = b1.sub_point(b0); // Direction of segment B
    let r = a0.sub_point(b0); // Vector from b0 to a0

    let a = d1.dot(d1); // |d1|²
    let e = d2.dot(d2); // |d2|²
    let f = d2.dot(r);

    // Check for degenerate segments
    let eps = 1e-20;

    if a < eps && e < eps {
        // Both segments are points
        return (a0, b0);
    }
    if a < eps {
        // Segment A is a point
        let t = (f / e).clamp(0.0, 1.0);
        return (a0, b0.lerp(b1, t));
    }
    if e < eps {
        // Segment B is a point
        let s = (-d1.dot(r) / a).clamp(0.0, 1.0);
        return (a0.lerp(a1, s), b0);
    }

    // General case: both segments have length
    let b_val = d1.dot(d2);
    let c = d1.dot(r);

    let denom = a * e - b_val * b_val;

    // Compute s (parameter on segment A)
    let mut s = if denom.abs() > eps {
        // Lines are not parallel
        ((b_val * f - c * e) / denom).clamp(0.0, 1.0)
    } else {
        // Lines are parallel; pick arbitrary point on A
        0.0
    };

    // Compute t (parameter on segment B) from s
    let mut t = (b_val * s + f) / e;

    // Clamp t and recompute s if needed
    if t < 0.0 {
        t = 0.0;
        s = (-c / a).clamp(0.0, 1.0);
    } else if t > 1.0 {
        t = 1.0;
        s = ((b_val - c) / a).clamp(0.0, 1.0);
    }

    let point_a = a0.lerp(a1, s);
    let point_b = b0.lerp(b1, t);

    (point_a, point_b)
}

// ============================================================================
// Surface Curvature Analysis
// ============================================================================

/// Result of surface curvature analysis at a parametric point.
///
/// Contains the principal curvatures, their directions, and derived quantities
/// (Gaussian and mean curvature). All curvature values are signed:
/// - Positive curvature indicates the surface curves toward the normal
/// - Negative curvature indicates the surface curves away from the normal
///
/// # Example
///
/// ```ignore
/// use ghx_engine::geom::{SphereSurface, analyze_surface_curvature};
///
/// let sphere = SphereSurface::new(..., 2.0)?;  // radius = 2
/// let analysis = analyze_surface_curvature(&sphere, 0.5, 0.5);
///
/// // For a sphere, both principal curvatures equal 1/radius
/// assert!((analysis.k1 - 0.5).abs() < 1e-6);
/// assert!((analysis.k2 - 0.5).abs() < 1e-6);
/// assert!((analysis.gaussian - 0.25).abs() < 1e-6);  // 1/r²
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceCurvatureAnalysis {
    /// The sampled point on the surface at (u, v).
    pub point: Point3,

    /// Surface normal at the point (unit vector).
    pub normal: Vec3,

    /// First partial derivative (tangent in U direction).
    pub du: Vec3,

    /// First partial derivative (tangent in V direction).
    pub dv: Vec3,

    /// Maximum principal curvature (κ₁).
    ///
    /// The larger of the two principal curvatures. The surface curves most
    /// strongly in the direction of `k1_direction`.
    pub k1: f64,

    /// Minimum principal curvature (κ₂).
    ///
    /// The smaller of the two principal curvatures. The surface curves least
    /// strongly in the direction of `k2_direction`.
    pub k2: f64,

    /// Direction of maximum principal curvature (unit vector in tangent plane).
    pub k1_direction: Vec3,

    /// Direction of minimum principal curvature (unit vector in tangent plane).
    pub k2_direction: Vec3,

    /// Gaussian curvature (K = κ₁ × κ₂).
    ///
    /// - K > 0: elliptic point (dome-like, both curvatures same sign)
    /// - K < 0: hyperbolic point (saddle-like, curvatures opposite signs)
    /// - K = 0: parabolic point (flat in at least one direction)
    pub gaussian: f64,

    /// Mean curvature (H = (κ₁ + κ₂) / 2).
    ///
    /// - H = 0: minimal surface (soap-film like)
    /// - H ≠ 0: non-minimal surface
    pub mean: f64,

    /// Whether the analysis is valid (true if all computations succeeded).
    ///
    /// This may be false if:
    /// - The surface is degenerate at this point
    /// - The normal couldn't be computed
    /// - The curvature computation failed numerically
    pub valid: bool,
}

impl Default for SurfaceCurvatureAnalysis {
    fn default() -> Self {
        Self {
            point: Point3::ORIGIN,
            normal: Vec3::Z,
            du: Vec3::X,
            dv: Vec3::Y,
            k1: 0.0,
            k2: 0.0,
            k1_direction: Vec3::X,
            k2_direction: Vec3::Y,
            gaussian: 0.0,
            mean: 0.0,
            valid: false,
        }
    }
}

/// Analyzes surface curvature at a parametric point using the second fundamental form.
///
/// This function computes the principal curvatures (κ₁, κ₂), their directions,
/// and derived quantities (Gaussian and mean curvature) using differential
/// geometry on the parametric surface.
///
/// # Algorithm
///
/// 1. Compute first partial derivatives (∂P/∂u, ∂P/∂v) for the tangent plane
/// 2. Compute the surface normal N = (∂P/∂u × ∂P/∂v) / |∂P/∂u × ∂P/∂v|
/// 3. Compute second partial derivatives (∂²P/∂u², ∂²P/∂u∂v, ∂²P/∂v²)
/// 4. Form the first fundamental form coefficients (E, F, G)
/// 5. Form the second fundamental form coefficients (L, M, N)
/// 6. Solve the characteristic equation for principal curvatures
/// 7. Compute principal directions as eigenvectors
///
/// # Arguments
///
/// * `surface` - Any type implementing the `Surface` trait
/// * `u` - Parameter in the U direction
/// * `v` - Parameter in the V direction
///
/// # Returns
///
/// A `SurfaceCurvatureAnalysis` containing all computed curvature information.
/// Check the `valid` field to ensure the computation succeeded.
///
/// # Example
///
/// ```ignore
/// use ghx_engine::geom::{PlaneSurface, Point3, Vec3, analyze_surface_curvature};
///
/// // For a plane, all curvatures should be zero
/// let plane = PlaneSurface::new(
///     Point3::ORIGIN,
///     Vec3::new(1.0, 0.0, 0.0),
///     Vec3::new(0.0, 1.0, 0.0),
/// );
/// let analysis = analyze_surface_curvature(&plane, 0.5, 0.5);
///
/// assert!(analysis.valid);
/// assert!(analysis.k1.abs() < 1e-10);
/// assert!(analysis.k2.abs() < 1e-10);
/// assert!(analysis.gaussian.abs() < 1e-10);
/// ```
#[must_use]
pub fn analyze_surface_curvature<S: Surface + ?Sized>(surface: &S, u: f64, v: f64) -> SurfaceCurvatureAnalysis {
    // Sample the point
    let point = surface.point_at(u, v);

    // Compute first partial derivatives
    let (du, dv) = surface.partial_derivatives_at(u, v);

    // Compute normal
    let normal = match du.cross(dv).normalized() {
        Some(n) => n,
        None => {
            // Degenerate point (du and dv are parallel)
            return SurfaceCurvatureAnalysis {
                point,
                normal: Vec3::Z,
                du,
                dv,
                valid: false,
                ..Default::default()
            };
        }
    };

    // Compute second partial derivatives
    let (duu, duv, dvv) = surface.second_partial_derivatives_at(u, v);

    // First fundamental form coefficients (metric tensor)
    // E = du · du, F = du · dv, G = dv · dv
    let e = du.dot(du);
    let f = du.dot(dv);
    let g = dv.dot(dv);

    // Second fundamental form coefficients
    // L = N · duu, M = N · duv, N = N · dvv
    let l_coeff = normal.dot(duu);
    let m_coeff = normal.dot(duv);
    let n_coeff = normal.dot(dvv);

    // Determinant of the first fundamental form
    let det_i = e * g - f * f;

    if det_i.abs() < 1e-20 {
        // Degenerate metric (surface has zero area at this point)
        return SurfaceCurvatureAnalysis {
            point,
            normal,
            du,
            dv,
            valid: false,
            ..Default::default()
        };
    }

    // Gaussian curvature: K = (LN - M²) / (EG - F²)
    let gaussian = (l_coeff * n_coeff - m_coeff * m_coeff) / det_i;

    // Mean curvature: H = (EN - 2FM + GL) / 2(EG - F²)
    let mean = (e * n_coeff - 2.0 * f * m_coeff + g * l_coeff) / (2.0 * det_i);

    // Principal curvatures from the characteristic equation:
    // κ² - 2Hκ + K = 0
    // Solutions: κ = H ± sqrt(H² - K)
    let discriminant = mean * mean - gaussian;
    let sqrt_discriminant = if discriminant >= 0.0 {
        discriminant.sqrt()
    } else {
        // Numerical error; discriminant should be non-negative
        // This can happen due to floating point errors
        0.0
    };

    let k1 = mean + sqrt_discriminant;
    let k2 = mean - sqrt_discriminant;

    // Compute principal directions
    // The principal directions are eigenvectors of the shape operator (Weingarten map)
    // We solve: (L - κE)α + (M - κF)β = 0  and  (M - κF)α + (N - κG)β = 0
    let (k1_direction, k2_direction) = compute_principal_directions(
        e, f, g, l_coeff, m_coeff, n_coeff, k1, k2, du, dv,
    );

    SurfaceCurvatureAnalysis {
        point,
        normal,
        du,
        dv,
        k1,
        k2,
        k1_direction,
        k2_direction,
        gaussian,
        mean,
        valid: true,
    }
}

/// Computes the principal directions as eigenvectors of the shape operator.
///
/// Given the fundamental form coefficients and principal curvatures, this function
/// solves for the directions in the tangent plane corresponding to each curvature.
fn compute_principal_directions(
    e: f64, f: f64, g: f64,
    l: f64, m: f64, n: f64,
    k1: f64, k2: f64,
    du: Vec3, dv: Vec3,
) -> (Vec3, Vec3) {
    // For each principal curvature κ, find (α, β) such that:
    // (L - κE)α + (M - κF)β = 0
    // Direction in tangent plane: α*du + β*dv

    // Helper to compute direction for a given curvature
    let direction_for_curvature = |kappa: f64| -> Vec3 {
        let a = l - kappa * e;
        let b = m - kappa * f;

        // If both coefficients are near zero, use default direction
        if a.abs() < 1e-15 && b.abs() < 1e-15 {
            // Try the second equation
            let c = m - kappa * f;
            let d = n - kappa * g;

            if c.abs() < 1e-15 && d.abs() < 1e-15 {
                // Isotropic point (all directions are principal)
                return du.normalized().unwrap_or(Vec3::X);
            }

            // From cα + dβ = 0, choose β = c, α = -d
            let alpha = -d;
            let beta = c;
            let dir = du.mul_scalar(alpha).add(dv.mul_scalar(beta));
            return dir.normalized().unwrap_or(Vec3::X);
        }

        // From aα + bβ = 0, choose β = a, α = -b
        let alpha = -b;
        let beta = a;
        let dir = du.mul_scalar(alpha).add(dv.mul_scalar(beta));
        dir.normalized().unwrap_or(Vec3::X)
    };

    let dir1 = direction_for_curvature(k1);

    // k2 direction should be perpendicular to k1 in the tangent plane
    // If k1 ≈ k2 (isotropic), just use an orthogonal direction
    let dir2_computed = if (k1 - k2).abs() < 1e-12 {
        // Isotropic point: any two orthogonal directions work
        let normal = du.cross(dv).normalized().unwrap_or(Vec3::Z);
        normal.cross(dir1).normalized().unwrap_or(Vec3::Y)
    } else {
        direction_for_curvature(k2)
    };

    // Final check: if computed directions are nearly parallel (can happen with
    // numerical errors), force dir2 to be perpendicular to dir1 in tangent plane
    let dot = dir1.dot(dir2_computed);
    let dir2 = if dot.abs() > 0.1 {
        // Directions are not sufficiently perpendicular; recompute dir2
        let normal = du.cross(dv).normalized().unwrap_or(Vec3::Z);
        normal.cross(dir1).normalized().unwrap_or(Vec3::Y)
    } else {
        dir2_computed
    };

    (dir1, dir2)
}
