//! Opt-in timing/profiling hooks for the geometry engine.
//!
//! This module provides zero-cost timing instrumentation that is only active when
//! the `mesh_engine_metrics` feature is enabled and the target is not WASM.
//!
//! # Feature Gating
//!
//! - **`mesh_engine_metrics`**: Enables timing collection (disabled by default).
//! - **WASM**: Timing is always disabled on `wasm32` targets since `std::time::Instant`
//!   is not available.
//!
//! When disabled, all timing calls compile to no-ops with zero runtime overhead.
//!
//! # Usage
//!
//! ```ignore
//! use ghx_engine::geom::{GeomMetrics, TimingBucket};
//!
//! let mut metrics = GeomMetrics::default();
//! metrics.begin();
//!
//! // Time a tessellation operation
//! let result = metrics.time(TimingBucket::SurfaceTessellation, || {
//!     expensive_tessellation_work()
//! });
//!
//! // Time a triangulation operation
//! let triangles = metrics.time(TimingBucket::Triangulation, || {
//!     triangulate_grid(points)
//! });
//!
//! // Retrieve the timing report (None if feature disabled or on WASM)
//! if let Some(report) = metrics.end() {
//!     println!("Tessellation: {} ns", report.surface_tessellation_ns);
//!     println!("Triangulation: {} ns", report.triangulation_ns);
//! }
//! ```

/// Categories for timing different geometry operations.
///
/// Each bucket accumulates time across multiple calls, allowing profiling of
/// the overall time spent in each phase of mesh generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimingBucket {
    /// Curve tessellation (adaptive subdivision, point sampling).
    CurveTessellation,
    /// Surface grid generation and adaptive tessellation.
    SurfaceTessellation,
    /// Grid/polygon triangulation (Delaunay, ear-clipping, etc.).
    Triangulation,
    /// Vertex welding and mesh repair passes.
    Welding,
    /// Diagnostics computation (manifold checks, open edges, etc.).
    Diagnostics,
    /// Cache lookups and insertions.
    Cache,
    /// Loft surface/mesh generation.
    Loft,
    /// Sweep (Sweep1/Sweep2) operations.
    Sweep,
    /// Pipe and variable-radius pipe generation.
    Pipe,
    /// Revolution and rail-revolution operations.
    Revolve,
    /// Extrusion variants (linear, angled, to-point, along-curve).
    Extrusion,
    /// Patch and boundary surface filling.
    Patch,
    /// Offset and thickening/shelling operations.
    Offset,
    /// Displacement and heightfield deformation.
    Displacement,
    /// Twist/bend/taper/morph deformations.
    Deformation,
    /// Boolean/CSG operations (intersection, union, subtract).
    Boolean,
    /// Mesh simplification/LOD.
    Simplify,
    /// Subdivision surface operations.
    Subdivision,
}

/// Timing report with nanosecond precision for each operation category.
///
/// All fields are cumulative—multiple calls to the same bucket add to the total.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GeomTimingReport {
    /// Time spent in curve tessellation.
    pub curve_tessellation_ns: u64,
    /// Time spent in surface tessellation.
    pub surface_tessellation_ns: u64,
    /// Time spent in triangulation.
    pub triangulation_ns: u64,
    /// Time spent in welding/repair.
    pub welding_ns: u64,
    /// Time spent computing diagnostics.
    pub diagnostics_ns: u64,
    /// Time spent in cache operations.
    pub cache_ns: u64,
    /// Time spent in loft operations.
    pub loft_ns: u64,
    /// Time spent in sweep operations.
    pub sweep_ns: u64,
    /// Time spent in pipe operations.
    pub pipe_ns: u64,
    /// Time spent in revolve operations.
    pub revolve_ns: u64,
    /// Time spent in extrusion operations.
    pub extrusion_ns: u64,
    /// Time spent in patch operations.
    pub patch_ns: u64,
    /// Time spent in offset/thickening operations.
    pub offset_ns: u64,
    /// Time spent in displacement operations.
    pub displacement_ns: u64,
    /// Time spent in deformation operations.
    pub deformation_ns: u64,
    /// Time spent in boolean/CSG operations.
    pub boolean_ns: u64,
    /// Time spent in simplification operations.
    pub simplify_ns: u64,
    /// Time spent in subdivision operations.
    pub subdivision_ns: u64,
}

impl GeomTimingReport {
    /// Returns the total time across all buckets in nanoseconds.
    #[must_use]
    pub fn total_ns(&self) -> u64 {
        self.curve_tessellation_ns
            .saturating_add(self.surface_tessellation_ns)
            .saturating_add(self.triangulation_ns)
            .saturating_add(self.welding_ns)
            .saturating_add(self.diagnostics_ns)
            .saturating_add(self.cache_ns)
            .saturating_add(self.loft_ns)
            .saturating_add(self.sweep_ns)
            .saturating_add(self.pipe_ns)
            .saturating_add(self.revolve_ns)
            .saturating_add(self.extrusion_ns)
            .saturating_add(self.patch_ns)
            .saturating_add(self.offset_ns)
            .saturating_add(self.displacement_ns)
            .saturating_add(self.deformation_ns)
            .saturating_add(self.boolean_ns)
            .saturating_add(self.simplify_ns)
            .saturating_add(self.subdivision_ns)
    }

    /// Returns the total time in milliseconds (for display purposes).
    #[must_use]
    pub fn total_ms(&self) -> f64 {
        self.total_ns() as f64 / 1_000_000.0
    }
}

/// Accumulator for timing geometry operations.
///
/// Create an instance, call [`begin`](Self::begin) to reset, wrap operations
/// with [`time`](Self::time), and call [`end`](Self::end) to retrieve the report.
///
/// When the `mesh_engine_metrics` feature is disabled (or on WASM), all methods
/// are no-ops and [`end`](Self::end) returns `None`.
#[derive(Debug, Default)]
pub struct GeomMetrics {
    #[cfg(all(feature = "mesh_engine_metrics", not(target_arch = "wasm32")))]
    report: GeomTimingReport,
}

impl GeomMetrics {
    /// Resets all timing counters to zero.
    ///
    /// Call this at the start of an operation you want to profile.
    pub fn begin(&mut self) {
        #[cfg(all(feature = "mesh_engine_metrics", not(target_arch = "wasm32")))]
        {
            self.report = GeomTimingReport::default();
        }
    }

    /// Returns the accumulated timing report, or `None` if metrics are disabled.
    ///
    /// Returns `None` when:
    /// - The `mesh_engine_metrics` feature is not enabled, or
    /// - The target architecture is `wasm32`.
    #[must_use]
    pub fn end(&self) -> Option<GeomTimingReport> {
        #[cfg(all(feature = "mesh_engine_metrics", not(target_arch = "wasm32")))]
        {
            Some(self.report.clone())
        }
        #[cfg(not(all(feature = "mesh_engine_metrics", not(target_arch = "wasm32"))))]
        {
            None
        }
    }

    /// Times the execution of `f` and accumulates the elapsed time in `bucket`.
    ///
    /// When metrics are disabled, this simply calls `f()` with no overhead.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let result = metrics.time(TimingBucket::Triangulation, || {
    ///     triangulate_polygon(points)
    /// });
    /// ```
    pub fn time<R>(&mut self, bucket: TimingBucket, f: impl FnOnce() -> R) -> R {
        #[cfg(all(feature = "mesh_engine_metrics", not(target_arch = "wasm32")))]
        {
            let start = std::time::Instant::now();
            let result = f();
            let elapsed = start.elapsed();
            // Cap at u64::MAX to prevent overflow
            let nanos_u64 = elapsed.as_nanos().min(u128::from(u64::MAX)) as u64;
            self.add_to_bucket(bucket, nanos_u64);
            result
        }

        #[cfg(not(all(feature = "mesh_engine_metrics", not(target_arch = "wasm32"))))]
        {
            let _ = bucket;
            f()
        }
    }

    /// Adds nanoseconds to the specified bucket.
    #[cfg(all(feature = "mesh_engine_metrics", not(target_arch = "wasm32")))]
    fn add_to_bucket(&mut self, bucket: TimingBucket, nanos: u64) {
        match bucket {
            TimingBucket::CurveTessellation => {
                self.report.curve_tessellation_ns =
                    self.report.curve_tessellation_ns.saturating_add(nanos);
            }
            TimingBucket::SurfaceTessellation => {
                self.report.surface_tessellation_ns =
                    self.report.surface_tessellation_ns.saturating_add(nanos);
            }
            TimingBucket::Triangulation => {
                self.report.triangulation_ns = self.report.triangulation_ns.saturating_add(nanos);
            }
            TimingBucket::Welding => {
                self.report.welding_ns = self.report.welding_ns.saturating_add(nanos);
            }
            TimingBucket::Diagnostics => {
                self.report.diagnostics_ns = self.report.diagnostics_ns.saturating_add(nanos);
            }
            TimingBucket::Cache => {
                self.report.cache_ns = self.report.cache_ns.saturating_add(nanos);
            }
            TimingBucket::Loft => {
                self.report.loft_ns = self.report.loft_ns.saturating_add(nanos);
            }
            TimingBucket::Sweep => {
                self.report.sweep_ns = self.report.sweep_ns.saturating_add(nanos);
            }
            TimingBucket::Pipe => {
                self.report.pipe_ns = self.report.pipe_ns.saturating_add(nanos);
            }
            TimingBucket::Revolve => {
                self.report.revolve_ns = self.report.revolve_ns.saturating_add(nanos);
            }
            TimingBucket::Extrusion => {
                self.report.extrusion_ns = self.report.extrusion_ns.saturating_add(nanos);
            }
            TimingBucket::Patch => {
                self.report.patch_ns = self.report.patch_ns.saturating_add(nanos);
            }
            TimingBucket::Offset => {
                self.report.offset_ns = self.report.offset_ns.saturating_add(nanos);
            }
            TimingBucket::Displacement => {
                self.report.displacement_ns = self.report.displacement_ns.saturating_add(nanos);
            }
            TimingBucket::Deformation => {
                self.report.deformation_ns = self.report.deformation_ns.saturating_add(nanos);
            }
            TimingBucket::Boolean => {
                self.report.boolean_ns = self.report.boolean_ns.saturating_add(nanos);
            }
            TimingBucket::Simplify => {
                self.report.simplify_ns = self.report.simplify_ns.saturating_add(nanos);
            }
            TimingBucket::Subdivision => {
                self.report.subdivision_ns = self.report.subdivision_ns.saturating_add(nanos);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timing_report_total() {
        let mut report = GeomTimingReport::default();
        report.loft_ns = 1000;
        report.triangulation_ns = 2000;
        report.surface_tessellation_ns = 3000;
        assert_eq!(report.total_ns(), 6000);
        assert!((report.total_ms() - 0.006).abs() < 1e-9);
    }

    #[test]
    fn test_metrics_begin_resets() {
        let mut metrics = GeomMetrics::default();
        // On non-metrics builds, this just ensures no panics
        metrics.begin();
        let _ = metrics.end();
    }

    #[test]
    fn test_time_returns_closure_result() {
        let mut metrics = GeomMetrics::default();
        metrics.begin();
        let result = metrics.time(TimingBucket::Loft, || 42);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_all_buckets_exist() {
        // Ensure all bucket variants are covered (compile-time check)
        let buckets = [
            TimingBucket::CurveTessellation,
            TimingBucket::SurfaceTessellation,
            TimingBucket::Triangulation,
            TimingBucket::Welding,
            TimingBucket::Diagnostics,
            TimingBucket::Cache,
            TimingBucket::Loft,
            TimingBucket::Sweep,
            TimingBucket::Pipe,
            TimingBucket::Revolve,
            TimingBucket::Extrusion,
            TimingBucket::Patch,
            TimingBucket::Offset,
            TimingBucket::Displacement,
            TimingBucket::Deformation,
            TimingBucket::Boolean,
            TimingBucket::Simplify,
            TimingBucket::Subdivision,
        ];
        assert_eq!(buckets.len(), 18);
    }
}

