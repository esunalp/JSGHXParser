# Geom extrusion (Phase 2)

This document describes the `geom::extrusion` helpers in `alpha/ghx-engine`.

Status / scope
--------------
- Phase 2: the API is still internal and may change until Phase 3 component integration.
- Outputs are `geom::GeomMesh` plus `geom::GeomMeshDiagnostics`.

API
---
- `extrude_polyline(profile, direction, caps)`: linear translation extrusion; set `caps` to `ExtrusionCaps::BOTH` for a closed prism mesh.
- `extrude_to_point(profile, tip, cap_base)`: connects a polyline to a tip point (pyramid/cone); optionally caps the base.
- `extrude_angled_polyline(polyline, base_height, top_height, angles)`: "draft" extrusion in world Z (vertical portion + per-edge angles).

Example (linear extrusion)
-------------------------
```rust
use ghx_engine::geom::{ExtrusionCaps, Point3, Vec3, extrude_polyline};

let profile = [
    Point3::new(0.0, 0.0, 0.0),
    Point3::new(1.0, 0.0, 0.0),
    Point3::new(1.0, 1.0, 0.0),
    Point3::new(0.0, 1.0, 0.0),
];

let (mesh, diag) = extrude_polyline(&profile, Vec3::new(0.0, 0.0, 2.0), ExtrusionCaps::BOTH)?;
assert_eq!(diag.open_edge_count, 0);
assert_eq!(diag.non_manifold_edge_count, 0);
assert!(mesh.indices.len() % 3 == 0);
# Ok::<(), ghx_engine::geom::ExtrusionError>(())
```

