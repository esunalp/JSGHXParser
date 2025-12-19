use std::ops::{Add, Div, Mul, Neg, Sub};

// ─────────────────────────────────────────────────────────────────────────────
// Vec3
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    /// Zero vector.
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);
    /// Unit vector along the X axis.
    pub const X: Self = Self::new(1.0, 0.0, 0.0);
    /// Unit vector along the Y axis.
    pub const Y: Self = Self::new(0.0, 1.0, 0.0);
    /// Unit vector along the Z axis.
    pub const Z: Self = Self::new(0.0, 0.0, 1.0);

    #[must_use]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Create a Vec3 from an array.
    #[must_use]
    pub const fn from_array(arr: [f64; 3]) -> Self {
        Self::new(arr[0], arr[1], arr[2])
    }

    /// Convert to an array.
    #[must_use]
    pub const fn to_array(self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }

    #[must_use]
    pub fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    #[must_use]
    pub const fn length_squared(self) -> f64 {
        self.dot(self)
    }

    #[must_use]
    pub const fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    #[must_use]
    pub const fn cross(self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    #[must_use]
    pub fn normalized(self) -> Option<Self> {
        let len = self.length();
        if len.is_finite() && len > 0.0 {
            Some(Self::new(self.x / len, self.y / len, self.z / len))
        } else {
            None
        }
    }

    /// Linear interpolation between two vectors.
    /// Returns `self * (1 - t) + rhs * t`.
    #[must_use]
    pub fn lerp(self, rhs: Self, t: f64) -> Self {
        Self::new(
            self.x + (rhs.x - self.x) * t,
            self.y + (rhs.y - self.y) * t,
            self.z + (rhs.z - self.z) * t,
        )
    }

    #[must_use]
    pub const fn mul_scalar(self, s: f64) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }

    #[must_use]
    pub const fn div_scalar(self, s: f64) -> Self {
        Self::new(self.x / s, self.y / s, self.z / s)
    }

    #[must_use]
    pub const fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }

    #[must_use]
    pub const fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }

    #[must_use]
    pub const fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }

    /// Component-wise minimum.
    #[must_use]
    pub fn min(self, rhs: Self) -> Self {
        Self::new(self.x.min(rhs.x), self.y.min(rhs.y), self.z.min(rhs.z))
    }

    /// Component-wise maximum.
    #[must_use]
    pub fn max(self, rhs: Self) -> Self {
        Self::new(self.x.max(rhs.x), self.y.max(rhs.y), self.z.max(rhs.z))
    }

    /// Component-wise absolute value.
    #[must_use]
    pub fn abs(self) -> Self {
        Self::new(self.x.abs(), self.y.abs(), self.z.abs())
    }
}

impl Default for Vec3 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<[f64; 3]> for Vec3 {
    fn from(arr: [f64; 3]) -> Self {
        Self::from_array(arr)
    }
}

impl From<Vec3> for [f64; 3] {
    fn from(v: Vec3) -> Self {
        v.to_array()
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3::new(self * rhs.x, self * rhs.y, self * rhs.z)
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Point3
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3 {
    /// The origin point (0, 0, 0).
    pub const ORIGIN: Self = Self::new(0.0, 0.0, 0.0);

    #[must_use]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Create a Point3 from an array.
    #[must_use]
    pub const fn from_array(arr: [f64; 3]) -> Self {
        Self::new(arr[0], arr[1], arr[2])
    }

    #[must_use]
    pub const fn to_array(self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }

    /// Convert point to a position vector from the origin.
    #[must_use]
    pub const fn to_vec3(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    #[must_use]
    pub const fn add_vec(self, v: Vec3) -> Self {
        Self::new(self.x + v.x, self.y + v.y, self.z + v.z)
    }

    #[must_use]
    pub const fn sub_vec(self, v: Vec3) -> Self {
        Self::new(self.x - v.x, self.y - v.y, self.z - v.z)
    }

    #[must_use]
    pub const fn sub_point(self, rhs: Self) -> Vec3 {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }

    /// Linear interpolation between two points.
    /// Returns `self * (1 - t) + rhs * t`.
    #[must_use]
    pub fn lerp(self, rhs: Self, t: f64) -> Self {
        Self::new(
            self.x + (rhs.x - self.x) * t,
            self.y + (rhs.y - self.y) * t,
            self.z + (rhs.z - self.z) * t,
        )
    }

    /// Euclidean distance to another point.
    #[must_use]
    pub fn distance_to(self, other: Self) -> f64 {
        self.sub_point(other).length()
    }

    /// Squared Euclidean distance to another point.
    #[must_use]
    pub fn distance_squared_to(self, other: Self) -> f64 {
        self.sub_point(other).length_squared()
    }
}

impl Default for Point3 {
    fn default() -> Self {
        Self::ORIGIN
    }
}

impl From<[f64; 3]> for Point3 {
    fn from(arr: [f64; 3]) -> Self {
        Self::from_array(arr)
    }
}

impl From<Point3> for [f64; 3] {
    fn from(p: Point3) -> Self {
        p.to_array()
    }
}

impl From<Vec3> for Point3 {
    fn from(v: Vec3) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}

impl From<Point3> for Vec3 {
    fn from(p: Point3) -> Self {
        p.to_vec3()
    }
}

impl Add<Vec3> for Point3 {
    type Output = Self;
    fn add(self, rhs: Vec3) -> Self::Output {
        self.add_vec(rhs)
    }
}

impl Sub<Vec3> for Point3 {
    type Output = Self;
    fn sub(self, rhs: Vec3) -> Self::Output {
        self.sub_vec(rhs)
    }
}

impl Sub for Point3 {
    type Output = Vec3;
    fn sub(self, rhs: Self) -> Self::Output {
        self.sub_point(rhs)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Transform
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    m: [[f64; 4]; 4],
}

impl Transform {
    #[must_use]
    pub const fn identity() -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// Construct a transform from origin and three orthonormal axes.
    /// The axes are expected to be unit vectors and mutually perpendicular.
    #[must_use]
    pub fn from_axes(origin: Point3, x_axis: Vec3, y_axis: Vec3, z_axis: Vec3) -> Self {
        Self {
            m: [
                [x_axis.x, y_axis.x, z_axis.x, origin.x],
                [x_axis.y, y_axis.y, z_axis.y, origin.y],
                [x_axis.z, y_axis.z, z_axis.z, origin.z],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// Construct a look-at transform (camera-style).
    /// `eye` is the camera position, `target` is what the camera looks at,
    /// `up` is the world up direction.
    #[must_use]
    pub fn look_at(eye: Point3, target: Point3, up: Vec3) -> Option<Self> {
        let forward = (target - eye).normalized()?;
        let right = forward.cross(up).normalized()?;
        let actual_up = right.cross(forward);
        Some(Self::from_axes(eye, right, actual_up, forward.neg()))
    }

    #[must_use]
    pub const fn translate(offset: Vec3) -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, offset.x],
                [0.0, 1.0, 0.0, offset.y],
                [0.0, 0.0, 1.0, offset.z],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    #[must_use]
    pub fn scale(sx: f64, sy: f64, sz: f64) -> Self {
        Self {
            m: [
                [sx, 0.0, 0.0, 0.0],
                [0.0, sy, 0.0, 0.0],
                [0.0, 0.0, sz, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    #[must_use]
    pub fn uniform_scale(s: f64) -> Self {
        Self::scale(s, s, s)
    }

    #[must_use]
    pub fn rotate_x(angle: f64) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, c, -s, 0.0],
                [0.0, s, c, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    #[must_use]
    pub fn rotate_y(angle: f64) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self {
            m: [
                [c, 0.0, s, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [-s, 0.0, c, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    #[must_use]
    pub fn rotate_z(angle: f64) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self {
            m: [
                [c, -s, 0.0, 0.0],
                [s, c, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    #[must_use]
    pub fn rotate_axis(axis: Vec3, angle: f64) -> Option<Self> {
        let axis = axis.normalized()?;
        let c = angle.cos();
        let s = angle.sin();
        let t = 1.0 - c;
        let x = axis.x;
        let y = axis.y;
        let z = axis.z;

        Some(Self {
            m: [
                [
                    t * x * x + c,
                    t * x * y - s * z,
                    t * x * z + s * y,
                    0.0,
                ],
                [
                    t * x * y + s * z,
                    t * y * y + c,
                    t * y * z - s * x,
                    0.0,
                ],
                [
                    t * x * z - s * y,
                    t * y * z + s * x,
                    t * z * z + c,
                    0.0,
                ],
                [0.0, 0.0, 0.0, 1.0],
            ],
        })
    }

    #[must_use]
    pub fn compose(self, other: Self) -> Self {
        let mut result = Self::identity();
        for i in 0..4 {
            for j in 0..4 {
                result.m[i][j] = self.m[i][0] * other.m[0][j]
                    + self.m[i][1] * other.m[1][j]
                    + self.m[i][2] * other.m[2][j]
                    + self.m[i][3] * other.m[3][j];
            }
        }
        result
    }

    /// Compute the inverse of this transform.
    /// Returns `None` if the matrix is singular (non-invertible).
    #[must_use]
    pub fn inverse(self) -> Option<Self> {
        // Use the adjugate method for 4x4 matrix inversion
        let m = &self.m;

        // Compute 2x2 determinants for the first two rows
        let s0 = m[0][0] * m[1][1] - m[1][0] * m[0][1];
        let s1 = m[0][0] * m[1][2] - m[1][0] * m[0][2];
        let s2 = m[0][0] * m[1][3] - m[1][0] * m[0][3];
        let s3 = m[0][1] * m[1][2] - m[1][1] * m[0][2];
        let s4 = m[0][1] * m[1][3] - m[1][1] * m[0][3];
        let s5 = m[0][2] * m[1][3] - m[1][2] * m[0][3];

        // Compute 2x2 determinants for the last two rows
        let c5 = m[2][2] * m[3][3] - m[3][2] * m[2][3];
        let c4 = m[2][1] * m[3][3] - m[3][1] * m[2][3];
        let c3 = m[2][1] * m[3][2] - m[3][1] * m[2][2];
        let c2 = m[2][0] * m[3][3] - m[3][0] * m[2][3];
        let c1 = m[2][0] * m[3][2] - m[3][0] * m[2][2];
        let c0 = m[2][0] * m[3][1] - m[3][0] * m[2][1];

        // Compute the determinant
        let det = s0 * c5 - s1 * c4 + s2 * c3 + s3 * c2 - s4 * c1 + s5 * c0;

        if !det.is_finite() || det.abs() < 1e-15 {
            return None;
        }

        let inv_det = 1.0 / det;

        // Compute adjugate matrix and multiply by 1/det
        Some(Self {
            m: [
                [
                    (m[1][1] * c5 - m[1][2] * c4 + m[1][3] * c3) * inv_det,
                    (-m[0][1] * c5 + m[0][2] * c4 - m[0][3] * c3) * inv_det,
                    (m[3][1] * s5 - m[3][2] * s4 + m[3][3] * s3) * inv_det,
                    (-m[2][1] * s5 + m[2][2] * s4 - m[2][3] * s3) * inv_det,
                ],
                [
                    (-m[1][0] * c5 + m[1][2] * c2 - m[1][3] * c1) * inv_det,
                    (m[0][0] * c5 - m[0][2] * c2 + m[0][3] * c1) * inv_det,
                    (-m[3][0] * s5 + m[3][2] * s2 - m[3][3] * s1) * inv_det,
                    (m[2][0] * s5 - m[2][2] * s2 + m[2][3] * s1) * inv_det,
                ],
                [
                    (m[1][0] * c4 - m[1][1] * c2 + m[1][3] * c0) * inv_det,
                    (-m[0][0] * c4 + m[0][1] * c2 - m[0][3] * c0) * inv_det,
                    (m[3][0] * s4 - m[3][1] * s2 + m[3][3] * s0) * inv_det,
                    (-m[2][0] * s4 + m[2][1] * s2 - m[2][3] * s0) * inv_det,
                ],
                [
                    (-m[1][0] * c3 + m[1][1] * c1 - m[1][2] * c0) * inv_det,
                    (m[0][0] * c3 - m[0][1] * c1 + m[0][2] * c0) * inv_det,
                    (-m[3][0] * s3 + m[3][1] * s1 - m[3][2] * s0) * inv_det,
                    (m[2][0] * s3 - m[2][1] * s1 + m[2][2] * s0) * inv_det,
                ],
            ],
        })
    }

    /// Compute the determinant of this transform matrix.
    #[must_use]
    pub fn determinant(self) -> f64 {
        let m = &self.m;
        let s0 = m[0][0] * m[1][1] - m[1][0] * m[0][1];
        let s1 = m[0][0] * m[1][2] - m[1][0] * m[0][2];
        let s2 = m[0][0] * m[1][3] - m[1][0] * m[0][3];
        let s3 = m[0][1] * m[1][2] - m[1][1] * m[0][2];
        let s4 = m[0][1] * m[1][3] - m[1][1] * m[0][3];
        let s5 = m[0][2] * m[1][3] - m[1][2] * m[0][3];

        let c5 = m[2][2] * m[3][3] - m[3][2] * m[2][3];
        let c4 = m[2][1] * m[3][3] - m[3][1] * m[2][3];
        let c3 = m[2][1] * m[3][2] - m[3][1] * m[2][2];
        let c2 = m[2][0] * m[3][3] - m[3][0] * m[2][3];
        let c1 = m[2][0] * m[3][2] - m[3][0] * m[2][2];
        let c0 = m[2][0] * m[3][1] - m[3][0] * m[2][1];

        s0 * c5 - s1 * c4 + s2 * c3 + s3 * c2 - s4 * c1 + s5 * c0
    }

    /// Get the translation component of this transform.
    #[must_use]
    pub fn translation(self) -> Vec3 {
        Vec3::new(self.m[0][3], self.m[1][3], self.m[2][3])
    }

    #[must_use]
    pub fn apply_point(self, p: Point3) -> Point3 {
        let x = self.m[0][0] * p.x + self.m[0][1] * p.y + self.m[0][2] * p.z + self.m[0][3];
        let y = self.m[1][0] * p.x + self.m[1][1] * p.y + self.m[1][2] * p.z + self.m[1][3];
        let z = self.m[2][0] * p.x + self.m[2][1] * p.y + self.m[2][2] * p.z + self.m[2][3];
        Point3::new(x, y, z)
    }

    #[must_use]
    pub fn apply_vec(self, v: Vec3) -> Vec3 {
        let x = self.m[0][0] * v.x + self.m[0][1] * v.y + self.m[0][2] * v.z;
        let y = self.m[1][0] * v.x + self.m[1][1] * v.y + self.m[1][2] * v.z;
        let z = self.m[2][0] * v.x + self.m[2][1] * v.y + self.m[2][2] * v.z;
        Vec3::new(x, y, z)
    }

    /// Access the raw 4x4 matrix data.
    #[must_use]
    pub const fn as_matrix(&self) -> &[[f64; 4]; 4] {
        &self.m
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl Mul for Transform {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        self.compose(rhs)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// BBox
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBox {
    pub min: Point3,
    pub max: Point3,
}

impl BBox {
    #[must_use]
    pub const fn new(min: Point3, max: Point3) -> Self {
        Self { min, max }
    }

    #[must_use]
    pub fn from_points(points: &[Point3]) -> Option<Self> {
        let mut iter = points.iter().copied();
        let first = iter.next()?;
        let mut min = first;
        let mut max = first;
        for p in iter {
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            min.z = min.z.min(p.z);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
            max.z = max.z.max(p.z);
        }
        Some(Self::new(min, max))
    }

    /// Create a bounding box centered at origin with the given half-extents.
    #[must_use]
    pub fn from_half_extents(half_extents: Vec3) -> Self {
        Self::new(
            Point3::new(-half_extents.x, -half_extents.y, -half_extents.z),
            Point3::new(half_extents.x, half_extents.y, half_extents.z),
        )
    }

    /// Center point of the bounding box.
    #[must_use]
    pub fn center(self) -> Point3 {
        Point3::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
            (self.min.z + self.max.z) * 0.5,
        )
    }

    /// Size (dimensions) of the bounding box.
    #[must_use]
    pub fn size(self) -> Vec3 {
        Vec3::new(
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }

    /// Half-extents of the bounding box (half the size).
    #[must_use]
    pub fn half_extents(self) -> Vec3 {
        self.size().mul_scalar(0.5)
    }

    /// Diagonal length of the bounding box.
    #[must_use]
    pub fn diagonal(self) -> f64 {
        self.size().length()
    }

    /// Volume of the bounding box.
    #[must_use]
    pub fn volume(self) -> f64 {
        let s = self.size();
        s.x * s.y * s.z
    }

    /// Surface area of the bounding box.
    #[must_use]
    pub fn surface_area(self) -> f64 {
        let s = self.size();
        2.0 * (s.x * s.y + s.y * s.z + s.z * s.x)
    }

    /// Check if a point is inside the bounding box (inclusive).
    #[must_use]
    pub fn contains_point(self, p: Point3) -> bool {
        p.x >= self.min.x
            && p.x <= self.max.x
            && p.y >= self.min.y
            && p.y <= self.max.y
            && p.z >= self.min.z
            && p.z <= self.max.z
    }

    /// Check if this bounding box fully contains another.
    #[must_use]
    pub fn contains_bbox(self, other: Self) -> bool {
        self.contains_point(other.min) && self.contains_point(other.max)
    }

    /// Check if this bounding box intersects (overlaps) with another.
    #[must_use]
    pub fn intersects(self, other: Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Compute the intersection of two bounding boxes.
    /// Returns `None` if they don't intersect.
    #[must_use]
    pub fn intersection(self, other: Self) -> Option<Self> {
        if !self.intersects(other) {
            return None;
        }
        Some(Self::new(
            Point3::new(
                self.min.x.max(other.min.x),
                self.min.y.max(other.min.y),
                self.min.z.max(other.min.z),
            ),
            Point3::new(
                self.max.x.min(other.max.x),
                self.max.y.min(other.max.y),
                self.max.z.min(other.max.z),
            ),
        ))
    }

    #[must_use]
    pub fn expand_point(self, p: Point3) -> Self {
        Self::new(
            Point3::new(
                self.min.x.min(p.x),
                self.min.y.min(p.y),
                self.min.z.min(p.z),
            ),
            Point3::new(
                self.max.x.max(p.x),
                self.max.y.max(p.y),
                self.max.z.max(p.z),
            ),
        )
    }

    /// Expand the bounding box by a scalar amount in all directions.
    #[must_use]
    pub fn expand_by(self, amount: f64) -> Self {
        Self::new(
            Point3::new(
                self.min.x - amount,
                self.min.y - amount,
                self.min.z - amount,
            ),
            Point3::new(
                self.max.x + amount,
                self.max.y + amount,
                self.max.z + amount,
            ),
        )
    }

    /// Expand the bounding box by a tolerance value.
    #[must_use]
    pub fn expand_tolerance(self, tol: Tolerance) -> Self {
        self.expand_by(tol.eps)
    }

    #[must_use]
    pub fn union(self, other: Self) -> Self {
        Self::new(
            Point3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            Point3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        )
    }

    /// Apply a transform to this bounding box.
    /// Note: this computes the axis-aligned bounding box of the transformed corners.
    #[must_use]
    pub fn transform(self, t: Transform) -> Self {
        let corners = [
            Point3::new(self.min.x, self.min.y, self.min.z),
            Point3::new(self.max.x, self.min.y, self.min.z),
            Point3::new(self.min.x, self.max.y, self.min.z),
            Point3::new(self.max.x, self.max.y, self.min.z),
            Point3::new(self.min.x, self.min.y, self.max.z),
            Point3::new(self.max.x, self.min.y, self.max.z),
            Point3::new(self.min.x, self.max.y, self.max.z),
            Point3::new(self.max.x, self.max.y, self.max.z),
        ];
        let transformed: Vec<Point3> = corners.iter().map(|&c| t.apply_point(c)).collect();
        Self::from_points(&transformed).unwrap_or(*&self)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tolerance
// ─────────────────────────────────────────────────────────────────────────────

/// Tolerance configuration for geometric operations.
///
/// Use the named constants for specific use cases to avoid epsilon scatter:
/// - `Tolerance::default_geom()` - General geometry comparisons (1e-9)
/// - `Tolerance::ZERO_LENGTH` - Detecting degenerate/zero-length vectors (1e-12)
/// - `Tolerance::DERIVATIVE` - First derivative numerical step size (1e-6)
/// - `Tolerance::SECOND_DERIVATIVE` - Second derivative numerical step size (1e-4)
/// - `Tolerance::ANGLE` - Angular comparisons in radians (1e-9)
/// - `Tolerance::WELD` - Vertex welding operations (1e-9)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tolerance {
    pub eps: f64,
}

impl Tolerance {
    /// Default geometric tolerance (1e-9).
    pub const DEFAULT: Self = Self { eps: 1e-9 };

    /// Tolerance for detecting zero-length/degenerate vectors and edges (1e-12).
    /// Use this when checking if a vector length is essentially zero.
    pub const ZERO_LENGTH: Self = Self { eps: 1e-12 };

    /// Step size for numerical differentiation (1e-6).
    /// Use this as a multiplier for domain span when computing first derivatives.
    pub const DERIVATIVE: Self = Self { eps: 1e-6 };

    /// Step size for second derivative numerical differentiation (1e-4).
    /// Larger than DERIVATIVE because second derivatives need more separation.
    pub const SECOND_DERIVATIVE: Self = Self { eps: 1e-4 };

    /// Tolerance for angular comparisons in radians (1e-9).
    pub const ANGLE: Self = Self { eps: 1e-9 };

    /// Tolerance for vertex welding operations (1e-9).
    pub const WELD: Self = Self { eps: 1e-9 };

    /// Loose tolerance for coarse comparisons (1e-6).
    pub const LOOSE: Self = Self { eps: 1e-6 };

    /// Tight tolerance for precise comparisons (1e-12).
    pub const TIGHT: Self = Self { eps: 1e-12 };

    #[must_use]
    pub const fn new(eps: f64) -> Self {
        Self { eps }
    }

    #[must_use]
    pub const fn default_geom() -> Self {
        Self::DEFAULT
    }

    #[must_use]
    pub const fn eps_squared(self) -> f64 {
        self.eps * self.eps
    }

    /// Create a scaled tolerance (e.g., for relative comparisons).
    #[must_use]
    pub fn scaled(self, scale: f64) -> Self {
        Self::new(self.eps * scale.abs())
    }

    /// Create tolerance relative to a span/domain size.
    /// Useful for numerical differentiation: `tol.relative_to(span)`.
    #[must_use]
    pub fn relative_to(self, span: f64) -> f64 {
        self.eps * span.abs()
    }

    #[must_use]
    pub fn approx_eq_f64(self, a: f64, b: f64) -> bool {
        (a - b).abs() <= self.eps
    }

    #[must_use]
    pub fn approx_zero_f64(self, a: f64) -> bool {
        a.abs() <= self.eps
    }

    #[must_use]
    pub fn approx_eq_point3(self, a: Point3, b: Point3) -> bool {
        a.sub_point(b).length_squared() <= self.eps_squared()
    }

    #[must_use]
    pub fn approx_eq_vec3(self, a: Vec3, b: Vec3) -> bool {
        a.sub(b).length_squared() <= self.eps_squared()
    }

    /// Check if a vector is approximately zero (degenerate).
    #[must_use]
    pub fn is_zero_vec3(self, v: Vec3) -> bool {
        v.length_squared() <= self.eps_squared()
    }

    /// Check if a length/distance is approximately zero.
    #[must_use]
    pub fn is_zero_length(self, len: f64) -> bool {
        len.abs() <= self.eps
    }

    /// Return the stricter (smaller) of two tolerances.
    #[must_use]
    pub fn min(self, other: Self) -> Self {
        if self.eps < other.eps {
            self
        } else {
            other
        }
    }

    /// Return the looser (larger) of two tolerances.
    #[must_use]
    pub fn max(self, other: Self) -> Self {
        if self.eps > other.eps {
            self
        } else {
            other
        }
    }
}

impl Default for Tolerance {
    fn default() -> Self {
        Self::DEFAULT
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_constants() {
        assert_eq!(Vec3::ZERO, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(Vec3::X, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(Vec3::Y, Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(Vec3::Z, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_vec3_operators() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);

        assert_eq!(a + b, Vec3::new(5.0, 7.0, 9.0));
        assert_eq!(b - a, Vec3::new(3.0, 3.0, 3.0));
        assert_eq!(a * 2.0, Vec3::new(2.0, 4.0, 6.0));
        assert_eq!(2.0 * a, Vec3::new(2.0, 4.0, 6.0));
        assert_eq!(a / 2.0, Vec3::new(0.5, 1.0, 1.5));
        assert_eq!(-a, Vec3::new(-1.0, -2.0, -3.0));
    }

    #[test]
    fn test_vec3_lerp() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(10.0, 20.0, 30.0);

        assert_eq!(a.lerp(b, 0.0), a);
        assert_eq!(a.lerp(b, 1.0), b);
        assert_eq!(a.lerp(b, 0.5), Vec3::new(5.0, 10.0, 15.0));
    }

    #[test]
    fn test_point3_operators() {
        let p = Point3::new(1.0, 2.0, 3.0);
        let v = Vec3::new(1.0, 1.0, 1.0);

        assert_eq!(p + v, Point3::new(2.0, 3.0, 4.0));
        assert_eq!(p - v, Point3::new(0.0, 1.0, 2.0));

        let q = Point3::new(4.0, 5.0, 6.0);
        assert_eq!(q - p, Vec3::new(3.0, 3.0, 3.0));
    }

    #[test]
    fn test_point3_lerp() {
        let a = Point3::new(0.0, 0.0, 0.0);
        let b = Point3::new(10.0, 20.0, 30.0);

        assert_eq!(a.lerp(b, 0.5), Point3::new(5.0, 10.0, 15.0));
    }

    #[test]
    fn test_transform_inverse() {
        let t = Transform::translate(Vec3::new(1.0, 2.0, 3.0));
        let inv = t.inverse().unwrap();
        let composed = t.compose(inv);

        // Should be approximately identity
        let identity = Transform::identity();
        for i in 0..4 {
            for j in 0..4 {
                assert!(
                    (composed.as_matrix()[i][j] - identity.as_matrix()[i][j]).abs() < 1e-10
                );
            }
        }
    }

    #[test]
    fn test_transform_compose_mul() {
        let a = Transform::rotate_x(0.5);
        let b = Transform::translate(Vec3::new(1.0, 0.0, 0.0));

        assert_eq!(a.compose(b), a * b);
    }

    #[test]
    fn test_bbox_methods() {
        let bbox = BBox::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(2.0, 4.0, 6.0),
        );

        assert_eq!(bbox.center(), Point3::new(1.0, 2.0, 3.0));
        assert_eq!(bbox.size(), Vec3::new(2.0, 4.0, 6.0));
        assert_eq!(bbox.half_extents(), Vec3::new(1.0, 2.0, 3.0));
        assert!((bbox.volume() - 48.0).abs() < 1e-10);

        assert!(bbox.contains_point(Point3::new(1.0, 2.0, 3.0)));
        assert!(!bbox.contains_point(Point3::new(-1.0, 2.0, 3.0)));
    }

    #[test]
    fn test_bbox_intersects() {
        let a = BBox::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(2.0, 2.0, 2.0),
        );
        let b = BBox::new(
            Point3::new(1.0, 1.0, 1.0),
            Point3::new(3.0, 3.0, 3.0),
        );
        let c = BBox::new(
            Point3::new(5.0, 5.0, 5.0),
            Point3::new(6.0, 6.0, 6.0),
        );

        assert!(a.intersects(b));
        assert!(!a.intersects(c));

        let intersection = a.intersection(b).unwrap();
        assert_eq!(intersection.min, Point3::new(1.0, 1.0, 1.0));
        assert_eq!(intersection.max, Point3::new(2.0, 2.0, 2.0));
    }

    #[test]
    fn test_tolerance_constants() {
        assert!(Tolerance::ZERO_LENGTH.eps < Tolerance::DEFAULT.eps);
        assert!(Tolerance::DERIVATIVE.eps > Tolerance::DEFAULT.eps);
    }

    #[test]
    fn test_tolerance_vec3_comparison() {
        let tol = Tolerance::new(1e-9);
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(1.0 + 1e-10, 2.0, 3.0);
        let c = Vec3::new(1.0 + 1e-8, 2.0, 3.0);

        assert!(tol.approx_eq_vec3(a, b));
        assert!(!tol.approx_eq_vec3(a, c));
    }

    #[test]
    fn test_from_into_conversions() {
        let arr: [f64; 3] = [1.0, 2.0, 3.0];
        let v: Vec3 = arr.into();
        let back: [f64; 3] = v.into();
        assert_eq!(arr, back);

        let p: Point3 = arr.into();
        let back_p: [f64; 3] = p.into();
        assert_eq!(arr, back_p);

        // Point3 <-> Vec3 conversion
        let v2: Vec3 = p.into();
        assert_eq!(v2, Vec3::new(1.0, 2.0, 3.0));

        let p2: Point3 = v.into();
        assert_eq!(p2, Point3::new(1.0, 2.0, 3.0));
    }
}
