//! Tessellation and triangulation caching for the geom mesh engine.
//!
//! This module provides caching infrastructure to avoid redundant computation
//! when the same surface/grid is tessellated or triangulated multiple times.
//!
//! # Features
//! - Surface grid point caching (keyed by surface identity + resolution)
//! - Grid triangulation caching (keyed by grid dimensions + wrap flags)
//! - Curve tessellation caching (keyed by curve identity + segment count)
//! - Cache hit/miss statistics for diagnostics
//! - Memory estimation for monitoring
//! - Cache invalidation/clearing
//!
//! # Example
//! ```ignore
//! let mut cache = GeomCache::default();
//! let points = cache.get_or_insert_surface_grid_points(&plane, 10, 10, || {
//!     tessellate_surface_grid_points(&plane, 10, 10)
//! });
//! let stats = cache.stats();
//! println!("Cache entries: {}, hits: {}", stats.surface_grid_entries, stats.surface_grid_hits);
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use super::core::Point3;
use super::surface::{Surface, SurfaceCacheKey};

/// Cache key for surface grid tessellation results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SurfaceGridKey {
    surface_key: SurfaceCacheKey,
    u_count: usize,
    v_count: usize,
}

/// Cache key for grid triangulation index buffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GridTriangulationKey {
    u_count: usize,
    v_count: usize,
    wrap_u: bool,
    wrap_v: bool,
}

/// Cache key for curve tessellation results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CurveTessellationKey {
    /// A hash or identifier for the curve (computed externally).
    curve_hash: u64,
    /// Number of segments requested.
    segment_count: usize,
    /// Whether adaptive tessellation was used.
    adaptive: bool,
}

/// Shared buffer for cached point data (avoids cloning on every access).
type SharedPoints = Arc<Vec<Point3>>;

/// Shared buffer for cached index data (avoids cloning on every access).
type SharedIndices = Arc<Vec<u32>>;

/// Geometry tessellation and triangulation cache.
///
/// Caches expensive tessellation and triangulation results to avoid redundant
/// computation when the same geometry is processed multiple times with
/// identical parameters.
#[derive(Debug, Default)]
pub struct GeomCache {
    // Surface grid tessellation cache
    surface_grid_points: HashMap<SurfaceGridKey, SharedPoints>,
    
    // Grid triangulation index cache
    grid_triangulation: HashMap<GridTriangulationKey, SharedIndices>,
    
    // Curve tessellation cache
    curve_tessellation: HashMap<CurveTessellationKey, SharedPoints>,
    
    // Hit/miss counters for diagnostics
    surface_grid_hits: usize,
    surface_grid_misses: usize,
    grid_triangulation_hits: usize,
    grid_triangulation_misses: usize,
    curve_tessellation_hits: usize,
    curve_tessellation_misses: usize,
}

/// Cache statistics for diagnostics and monitoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GeomCacheStats {
    /// Number of cached surface grid entries.
    pub surface_grid_entries: usize,
    /// Number of cached grid triangulation entries.
    pub grid_triangulation_entries: usize,
    /// Number of cached curve tessellation entries.
    pub curve_tessellation_entries: usize,
    /// Total cache hits for surface grids.
    pub surface_grid_hits: usize,
    /// Total cache misses for surface grids.
    pub surface_grid_misses: usize,
    /// Total cache hits for grid triangulation.
    pub grid_triangulation_hits: usize,
    /// Total cache misses for grid triangulation.
    pub grid_triangulation_misses: usize,
    /// Total cache hits for curve tessellation.
    pub curve_tessellation_hits: usize,
    /// Total cache misses for curve tessellation.
    pub curve_tessellation_misses: usize,
    /// Estimated memory usage in bytes.
    pub estimated_memory_bytes: usize,
}

impl GeomCacheStats {
    /// Returns the total number of cache entries across all caches.
    #[must_use]
    pub const fn total_entries(&self) -> usize {
        self.surface_grid_entries + self.grid_triangulation_entries + self.curve_tessellation_entries
    }

    /// Returns the total number of cache hits across all caches.
    #[must_use]
    pub const fn total_hits(&self) -> usize {
        self.surface_grid_hits + self.grid_triangulation_hits + self.curve_tessellation_hits
    }

    /// Returns the total number of cache misses across all caches.
    #[must_use]
    pub const fn total_misses(&self) -> usize {
        self.surface_grid_misses + self.grid_triangulation_misses + self.curve_tessellation_misses
    }

    /// Returns the cache hit rate as a value between 0.0 and 1.0.
    /// Returns 0.0 if no cache accesses have been made.
    #[must_use]
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_hits() + self.total_misses();
        if total == 0 {
            0.0
        } else {
            self.total_hits() as f64 / total as f64
        }
    }
}

impl GeomCache {
    /// Creates a new empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns cache statistics including entry counts, hit/miss rates, and memory estimate.
    #[must_use]
    pub fn stats(&self) -> GeomCacheStats {
        GeomCacheStats {
            surface_grid_entries: self.surface_grid_points.len(),
            grid_triangulation_entries: self.grid_triangulation.len(),
            curve_tessellation_entries: self.curve_tessellation.len(),
            surface_grid_hits: self.surface_grid_hits,
            surface_grid_misses: self.surface_grid_misses,
            grid_triangulation_hits: self.grid_triangulation_hits,
            grid_triangulation_misses: self.grid_triangulation_misses,
            curve_tessellation_hits: self.curve_tessellation_hits,
            curve_tessellation_misses: self.curve_tessellation_misses,
            estimated_memory_bytes: self.estimate_memory_usage(),
        }
    }

    /// Clears all cached data and resets hit/miss counters.
    pub fn clear(&mut self) {
        self.surface_grid_points.clear();
        self.grid_triangulation.clear();
        self.curve_tessellation.clear();
        self.surface_grid_hits = 0;
        self.surface_grid_misses = 0;
        self.grid_triangulation_hits = 0;
        self.grid_triangulation_misses = 0;
        self.curve_tessellation_hits = 0;
        self.curve_tessellation_misses = 0;
    }

    /// Clears only the surface grid cache.
    pub fn clear_surface_grids(&mut self) {
        self.surface_grid_points.clear();
        self.surface_grid_hits = 0;
        self.surface_grid_misses = 0;
    }

    /// Clears only the triangulation cache.
    pub fn clear_triangulations(&mut self) {
        self.grid_triangulation.clear();
        self.grid_triangulation_hits = 0;
        self.grid_triangulation_misses = 0;
    }

    /// Clears only the curve tessellation cache.
    pub fn clear_curve_tessellations(&mut self) {
        self.curve_tessellation.clear();
        self.curve_tessellation_hits = 0;
        self.curve_tessellation_misses = 0;
    }

    /// Resets hit/miss counters without clearing cached data.
    pub fn reset_counters(&mut self) {
        self.surface_grid_hits = 0;
        self.surface_grid_misses = 0;
        self.grid_triangulation_hits = 0;
        self.grid_triangulation_misses = 0;
        self.curve_tessellation_hits = 0;
        self.curve_tessellation_misses = 0;
    }

    /// Estimates the memory usage of all cached data in bytes.
    #[must_use]
    pub fn estimate_memory_usage(&self) -> usize {
        let mut total = 0usize;

        // Surface grid points: each Point3 is 3 × f64 = 24 bytes
        for points in self.surface_grid_points.values() {
            total += points.len() * 24;
        }

        // Grid triangulation: each u32 index is 4 bytes
        for indices in self.grid_triangulation.values() {
            total += indices.len() * 4;
        }

        // Curve tessellation: each Point3 is 24 bytes
        for points in self.curve_tessellation.values() {
            total += points.len() * 24;
        }

        // Add overhead for HashMap entries (approximate)
        let entry_overhead = std::mem::size_of::<(SurfaceGridKey, SharedPoints)>();
        total += self.surface_grid_points.len() * entry_overhead;
        total += self.grid_triangulation.len() * entry_overhead;
        total += self.curve_tessellation.len() * entry_overhead;

        total
    }

    /// Gets or computes and caches surface grid points.
    ///
    /// If the surface grid for the given parameters is already cached, returns
    /// a clone of the cached data. Otherwise, calls `make` to compute the points,
    /// caches the result, and returns it.
    #[must_use]
    pub fn get_or_insert_surface_grid_points(
        &mut self,
        surface: &impl Surface,
        u_count: usize,
        v_count: usize,
        make: impl FnOnce() -> Vec<Point3>,
    ) -> Vec<Point3> {
        let key = SurfaceGridKey {
            surface_key: surface.cache_key(),
            u_count,
            v_count,
        };
        if let Some(cached) = self.surface_grid_points.get(&key) {
            self.surface_grid_hits += 1;
            return (**cached).clone();
        }
        self.surface_grid_misses += 1;
        let points = Arc::new(make());
        self.surface_grid_points.insert(key, Arc::clone(&points));
        Arc::try_unwrap(points).unwrap_or_else(|arc| (*arc).clone())
    }

    /// Gets or computes and caches grid triangulation indices.
    ///
    /// Grid triangulation only depends on grid dimensions and wrap flags,
    /// not on the actual point positions, so it can be shared across
    /// different surfaces with the same grid structure.
    #[must_use]
    pub fn get_or_insert_triangulated_grid(
        &mut self,
        u_count: usize,
        v_count: usize,
        wrap_u: bool,
        wrap_v: bool,
        make: impl FnOnce() -> Vec<u32>,
    ) -> Vec<u32> {
        let key = GridTriangulationKey {
            u_count,
            v_count,
            wrap_u,
            wrap_v,
        };
        if let Some(cached) = self.grid_triangulation.get(&key) {
            self.grid_triangulation_hits += 1;
            return (**cached).clone();
        }
        self.grid_triangulation_misses += 1;
        let indices = Arc::new(make());
        self.grid_triangulation.insert(key, Arc::clone(&indices));
        Arc::try_unwrap(indices).unwrap_or_else(|arc| (*arc).clone())
    }

    /// Gets or computes and caches curve tessellation points.
    ///
    /// The `curve_hash` should uniquely identify the curve geometry. This is
    /// typically computed by the caller (e.g., hashing control points and knots).
    #[must_use]
    pub fn get_or_insert_curve_tessellation(
        &mut self,
        curve_hash: u64,
        segment_count: usize,
        adaptive: bool,
        make: impl FnOnce() -> Vec<Point3>,
    ) -> Vec<Point3> {
        let key = CurveTessellationKey {
            curve_hash,
            segment_count,
            adaptive,
        };
        if let Some(cached) = self.curve_tessellation.get(&key) {
            self.curve_tessellation_hits += 1;
            return (**cached).clone();
        }
        self.curve_tessellation_misses += 1;
        let points = Arc::new(make());
        self.curve_tessellation.insert(key, Arc::clone(&points));
        Arc::try_unwrap(points).unwrap_or_else(|arc| (*arc).clone())
    }

    /// Checks if a surface grid is cached without computing it.
    #[must_use]
    pub fn has_surface_grid(&self, surface: &impl Surface, u_count: usize, v_count: usize) -> bool {
        let key = SurfaceGridKey {
            surface_key: surface.cache_key(),
            u_count,
            v_count,
        };
        self.surface_grid_points.contains_key(&key)
    }

    /// Checks if a grid triangulation is cached without computing it.
    #[must_use]
    pub fn has_triangulated_grid(
        &self,
        u_count: usize,
        v_count: usize,
        wrap_u: bool,
        wrap_v: bool,
    ) -> bool {
        let key = GridTriangulationKey {
            u_count,
            v_count,
            wrap_u,
            wrap_v,
        };
        self.grid_triangulation.contains_key(&key)
    }

    /// Checks if a curve tessellation is cached without computing it.
    #[must_use]
    pub fn has_curve_tessellation(
        &self,
        curve_hash: u64,
        segment_count: usize,
        adaptive: bool,
    ) -> bool {
        let key = CurveTessellationKey {
            curve_hash,
            segment_count,
            adaptive,
        };
        self.curve_tessellation.contains_key(&key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::surface::PlaneSurface;
    use crate::geom::Vec3;

    #[test]
    fn cache_surface_grid_hit() {
        let mut cache = GeomCache::new();
        let plane = PlaneSurface::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );

        // First call should be a miss
        let mut call_count = 0;
        let _points = cache.get_or_insert_surface_grid_points(&plane, 5, 5, || {
            call_count += 1;
            vec![Point3::new(0.0, 0.0, 0.0); 25]
        });
        assert_eq!(call_count, 1);

        // Second call should be a hit (make not called)
        let _points = cache.get_or_insert_surface_grid_points(&plane, 5, 5, || {
            call_count += 1;
            vec![Point3::new(0.0, 0.0, 0.0); 25]
        });
        assert_eq!(call_count, 1);

        let stats = cache.stats();
        assert_eq!(stats.surface_grid_entries, 1);
        assert_eq!(stats.surface_grid_hits, 1);
        assert_eq!(stats.surface_grid_misses, 1);
    }

    #[test]
    fn cache_triangulated_grid_hit() {
        let mut cache = GeomCache::new();

        // First call should be a miss
        let mut call_count = 0;
        let _indices = cache.get_or_insert_triangulated_grid(5, 5, false, false, || {
            call_count += 1;
            vec![0, 1, 2]
        });
        assert_eq!(call_count, 1);

        // Second call should be a hit
        let _indices = cache.get_or_insert_triangulated_grid(5, 5, false, false, || {
            call_count += 1;
            vec![0, 1, 2]
        });
        assert_eq!(call_count, 1);

        let stats = cache.stats();
        assert_eq!(stats.grid_triangulation_entries, 1);
        assert_eq!(stats.grid_triangulation_hits, 1);
        assert_eq!(stats.grid_triangulation_misses, 1);
    }

    #[test]
    fn cache_curve_tessellation_hit() {
        let mut cache = GeomCache::new();

        let mut call_count = 0;
        let _points = cache.get_or_insert_curve_tessellation(12345, 10, false, || {
            call_count += 1;
            vec![Point3::new(0.0, 0.0, 0.0); 11]
        });
        assert_eq!(call_count, 1);

        let _points = cache.get_or_insert_curve_tessellation(12345, 10, false, || {
            call_count += 1;
            vec![Point3::new(0.0, 0.0, 0.0); 11]
        });
        assert_eq!(call_count, 1);

        let stats = cache.stats();
        assert_eq!(stats.curve_tessellation_entries, 1);
        assert_eq!(stats.curve_tessellation_hits, 1);
        assert_eq!(stats.curve_tessellation_misses, 1);
    }

    #[test]
    fn cache_clear() {
        let mut cache = GeomCache::new();
        let plane = PlaneSurface::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );

        let _ = cache.get_or_insert_surface_grid_points(&plane, 5, 5, || {
            vec![Point3::new(0.0, 0.0, 0.0); 25]
        });
        let _ = cache.get_or_insert_triangulated_grid(5, 5, false, false, || vec![0, 1, 2]);
        let _ = cache.get_or_insert_curve_tessellation(12345, 10, false, || {
            vec![Point3::new(0.0, 0.0, 0.0); 11]
        });

        assert_eq!(cache.stats().total_entries(), 3);

        cache.clear();

        let stats = cache.stats();
        assert_eq!(stats.total_entries(), 0);
        assert_eq!(stats.total_hits(), 0);
        assert_eq!(stats.total_misses(), 0);
    }

    #[test]
    fn cache_memory_estimation() {
        let mut cache = GeomCache::new();
        let plane = PlaneSurface::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );

        let _ = cache.get_or_insert_surface_grid_points(&plane, 5, 5, || {
            vec![Point3::new(0.0, 0.0, 0.0); 25]
        });

        let mem = cache.estimate_memory_usage();
        // 25 points × 24 bytes = 600 bytes, plus overhead
        assert!(mem >= 600, "Expected at least 600 bytes, got {}", mem);
    }

    #[test]
    fn cache_hit_rate() {
        let mut cache = GeomCache::new();

        // 1 miss, 3 hits = 75% hit rate
        for i in 0..4 {
            let _ = cache.get_or_insert_triangulated_grid(5, 5, false, false, || {
                vec![0, 1, 2]
            });
            if i == 0 {
                assert_eq!(cache.stats().hit_rate(), 0.0);
            }
        }

        let stats = cache.stats();
        assert_eq!(stats.grid_triangulation_hits, 3);
        assert_eq!(stats.grid_triangulation_misses, 1);
        assert!((stats.hit_rate() - 0.75).abs() < 0.001);
    }

    #[test]
    fn has_cached_checks() {
        let mut cache = GeomCache::new();
        let plane = PlaneSurface::new(
            Point3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );

        assert!(!cache.has_surface_grid(&plane, 5, 5));
        assert!(!cache.has_triangulated_grid(5, 5, false, false));
        assert!(!cache.has_curve_tessellation(12345, 10, false));

        let _ = cache.get_or_insert_surface_grid_points(&plane, 5, 5, || {
            vec![Point3::new(0.0, 0.0, 0.0); 25]
        });
        let _ = cache.get_or_insert_triangulated_grid(5, 5, false, false, || vec![0, 1, 2]);
        let _ = cache.get_or_insert_curve_tessellation(12345, 10, false, || {
            vec![Point3::new(0.0, 0.0, 0.0); 11]
        });

        assert!(cache.has_surface_grid(&plane, 5, 5));
        assert!(cache.has_triangulated_grid(5, 5, false, false));
        assert!(cache.has_curve_tessellation(12345, 10, false));
    }
}
