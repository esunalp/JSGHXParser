# Geom tolerances (Phase 1)

This document describes the tolerance strategy for the new `geom` module in `alpha/ghx-engine`.

Status / scope
--------------
- Phase 1: tolerances are internal-only and used inside `geom/*`.
- Until Phase 3 integration, `geom` APIs may change; do not expose tolerance knobs via `components/*` or `graph/*` yet.

Current model (Phase 1)
-----------------------
`geom::Tolerance` currently carries a single `eps: f64` value.

- Meaning: "values within `eps` are considered equal" for low-level geometric predicates.
- Default: `Tolerance::default_geom()` returns `1e-9`.

Usage example
-------------
```rust
use ghx_engine::geom::Tolerance;

let tol = Tolerance::default_geom();
assert!(tol.eps > 0.0);
```

Rules
-----
- Prefer passing a `Tolerance` through helpers instead of sprinkling ad-hoc epsilons.
- When adding welding/repair/predicates in Phase 2, route all thresholds through `Tolerance` (or `Tolerance` + per-op overrides).

Phase 2 usage (triangulation + welding)
---------------------------------------
- These APIs live behind `--features mesh_engine_next` until Phase 3 integration.
- `geom::triangulate_trim_region` uses `Tolerance` for degenerate filtering and loop cleanup.
- `geom::mesh_surface_*` performs a tolerance-aware weld + degenerate cull + winding consistency pass before emitting vertex normals/UVs.

Example (trim region triangulation)
-----------------------------------
```rust
use ghx_engine::geom::{Tolerance, TrimLoop, TrimRegion, UvPoint, triangulate_trim_region};

let tol = Tolerance::default_geom();
let outer = TrimLoop::new(
    vec![
        UvPoint::new(0.0, 0.0),
        UvPoint::new(1.0, 0.0),
        UvPoint::new(1.0, 1.0),
        UvPoint::new(0.0, 1.0),
    ],
    tol,
)?;

let region = TrimRegion::from_loops(vec![outer], tol)?;
let tri = triangulate_trim_region(&region, tol)?;
assert!(!tri.indices.is_empty());
# Ok::<(), String>(())
```
