# Geom fillet + chamfer (Phase 2)

This document describes the initial `geom::fillet_chamfer` helpers in `alpha/ghx-engine`.

Status / scope
--------------
- Phase 2: internal API behind `mesh_engine_next`; subject to change before Phase 3 component integration.
- Focus is on **clear diagnostics** and **minimal deterministic behavior**.

API
---
- `fillet_polyline_points(points, radius, segments, closed, tol)`:
  - Inserts circular arc segments at polyline corners.
  - `segments == 1` behaves like a chamfer (single segment between tangent points).
- `fillet_triangle_mesh_edges(mesh, edges, options, tol)`:
  - Experimental triangle-mesh edge fillet.
  - **Only supports “hinge” edges** where both endpoints are used by exactly two triangles total.
  - Skips unsupported edges and reports them in diagnostics.

Example (polyline fillet)
------------------------
```rust
use ghx_engine::geom::{Point3, Tolerance, fillet_polyline_points};

let polyline = [
    Point3::new(0.0, 0.0, 0.0),
    Point3::new(1.0, 0.0, 0.0),
    Point3::new(1.0, 1.0, 0.0),
];

let tol = Tolerance::default_geom();
let (rounded, diag) = fillet_polyline_points(&polyline, 0.2, 4, false, tol)?;
assert_eq!(diag.filleted_corner_count, 1);
assert!(rounded.len() > polyline.len());
# Ok::<(), ghx_engine::geom::FilletChamferError>(())
```

Tolerances and diagnostics
--------------------------
- `Tolerance` is used to detect degenerate segments/angles and to avoid duplicate points.
- If a radius is too large for a corner/segment, the implementation clamps locally and
  increments `clamped_corner_count` (polyline).
- For triangle meshes, unsupported/unsafe edges are skipped and recorded in
  `FilletMeshEdgeDiagnostics.errors`.

