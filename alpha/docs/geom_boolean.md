# Geom boolean / CSG (Phase 2)

This document describes the initial `geom::boolean` helpers in `alpha/ghx-engine`.

Status / scope
--------------
- Phase 2: internal API behind `mesh_engine_next`; subject to change before Phase 3 component integration.
- Current implementation targets **triangle-mesh booleans** and prioritizes deterministic behavior with explicit diagnostics.

API
---
- `boolean_meshes(a, b, op, tol) -> Result<BooleanResult, BooleanError>`
  - `op`: `BooleanOp::{Union, Difference, Intersection}`
  - Output: `BooleanResult { mesh, mesh_diagnostics, diagnostics }`
  - Includes a fallback strategy (tolerance relaxation and voxel fallback) with explicit diagnostic flags.

Example (union)
--------------
```rust
use ghx_engine::geom::{BooleanOp, ExtrusionCaps, Point3, Tolerance, Vec3, boolean_meshes, extrude_polyline};

let square = [
    Point3::new(0.0, 0.0, 0.0),
    Point3::new(1.0, 0.0, 0.0),
    Point3::new(1.0, 1.0, 0.0),
    Point3::new(0.0, 1.0, 0.0),
];

let (a, _) = extrude_polyline(&square, Vec3::new(0.0, 0.0, 1.0), ExtrusionCaps::BOTH)?;
let (b, _) = extrude_polyline(&square, Vec3::new(0.5, 0.0, 0.2), ExtrusionCaps::BOTH)?;

let tol = Tolerance::default_geom();
let result = boolean_meshes(&a, &b, BooleanOp::Union, tol)?;

assert!(result.mesh.indices.len() % 3 == 0);
assert!(result.mesh.positions.iter().all(|p| p[0].is_finite() && p[1].is_finite() && p[2].is_finite()));
# Ok::<(), ghx_engine::geom::BooleanError>(())
```

