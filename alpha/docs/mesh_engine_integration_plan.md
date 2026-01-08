Mesh Engine Integration Plan (ghx-engine)
==========================================

Context Snapshot
----------------
- Codebase: `alpha/ghx-engine` (Rust) with Grasshopper-style components (`src/components`) and graph evaluation (`src/graph`).
- Geometry today: curve/surface modules (e.g., `surface_freeform.rs`, `surface_primitive.rs`), mesh helpers (`mesh_triangulation.rs`, `mesh_primitive.rs`), math utilities, transforms, and extrusion/loft/sweep components.
- Current mesh representation piggybacks on `Value::Surface { vertices, faces }` with limited stitching/robustness; boolean and advanced mesh repair are absent.

Goals
-----
- Single, uniform meshing pipeline that ingests CAD primitives (curves, surfaces, solids, NURBS, existing meshes) and feature ops (loft/sweep/extrude/revolve/fillet/chamfer/morph/twist/boolean) and outputs watertight, manifold triangle meshes with controllable quality.
- Robustness: tolerance-aware predicates, welding, degeneracy handling, normal consistency, attribute propagation.
- Performance: adaptive tessellation, multithreading, BVH-backed intersection for booleans, streaming-friendly buffers.
- Ergonomics: clear API inside `ghx-engine` components, diagnostics surfaced via `ComponentError`, WASM-friendly memory layout.

AI Instructions Block
---------------------
- When modifying or adding features, update the documentation wiki with a short API description and at least one code snippet per geom module and its component wrapper.
- Keep components thin: always route geometry through `geom::*` modules; avoid duplicating math in `components/*`.
- Maintain backward compatibility: continue emitting `Value::Surface` adapters where needed while preferring `Value::Mesh`.
- Preserve pin order and GUIDs for Grasshopper components; add new pins only as optional/append-only.
- For three.js integration, ensure BufferGeometry creation remains compatible: positions/indices mandatory, normals/uvs optional with legacy fallback.
- Clarify requirements: Restate the task in your own words and list all assumptions before writing code.
- Think before coding: Outline the solution in small steps (algorithm, data structures, edge cases) before generating code.
- Keep it minimal: Write the simplest code that solves the problem; avoid unnecessary abstractions and dependencies.
- Be explicit: Use clear naming, small functions, and avoid “magic numbers” or hidden behavior.
- Handle errors: Add basic validation, error handling, and clear messages for failure cases.
- Target environment: Adapt syntax and libraries to the specified language, framework, version, and runtime.
- Show examples: Provide usage examples and small test cases that demonstrate correct behavior and edge cases.
- Explain briefly: After the code, give a short explanation of how it works and why it’s implemented this way.
- Self-check: Re-read the code to catch obvious bugs, missing imports, typos, and mismatched types before finalizing.
- Iterate safely: When updating code, show only the changed parts or the full final file, and keep it consistent with previous decisions.

Target Architecture
-------------------
- Core geometry kernel (new module): `src/geom/` with `Point3`, `Vec3`, `Transform`, `BBox`, `Tolerance`, `Curve3`, `Surface`, `Solid`, `Mesh` structs/traits; adaptive evaluators for lines/arcs/Bezier/B-spline/NURBS; surface evaluators (plane/cylinder/cone/sphere/torus/NURBS) with trimming.
- Meshing pipeline: curve tessellation -> surface grid generation -> face triangulation (constrained Delaunay/earcut) -> global stitch/weld -> quality/repair passes. Quality knobs: target edge length, max deviation, angle threshold, adaptivity budget. Generate UVs/tangents when parametrization exists; propagate per-face groups/material ids.
- Boolean kernel: triangle/surface intersection with filtered predicates, intersection band remeshing, classification (inside/outside), optional voxel fallback for degenerates.
- Feature operators implemented on geometry (not ad-hoc meshes): extrude/sweep/loft/revolve with caps; fillet/chamfer via offset/trim; morph/twist/bend via deformation fields then re-weld; offset/thickening and displacement pre-mesh with re-weld and normal repair.
- Optional simplification/LOD: edge-collapse decimation with watertightness guard for previews/exports.
- Subdivision/quad support: bridge subdivision inputs and optionally emit quad-friendly meshes where valid.
- Scene graph/instances: reuse existing graph evaluator; support lazy transforms; shared mesh buffers for instances; normalize units/handedness at ingestion.
- Caching and instrumentation: cache tessellation/triangulation results when inputs unchanged; timing/profiling hooks for regression/perf tracking.

Integration Strategy (Incremental Milestones)
---------------------------------------------
1) Geometry Core & Types
   - Add `src/geom/` with math/tolerance primitives and traits (`Curve3`, `Surface`, `MeshBuilder`, `Mesh`). Reuse/bridge `maths_*`, `vector_*`, `transform_*` to avoid duplication.
   - Define `MeshQuality` config (edge length, deviation, angle) and `MeshDiagnostics`.
   - Extend `graph::value::Value` with a dedicated `Value::Mesh { vertices, faces, normals, uvs, diagnostics }` while keeping `Value::Surface` for backward compatibility; add coercion utilities.

2) Curve & Surface Evaluators
   - Implement adaptive tessellation for lines/arcs/Bezier/B-spline/NURBS (arc-length parameterization, curvature-based subdivision) and land outputs as polylines with tangents/normals.
   - Implement surface evaluators (plane/cylinder/cone/sphere/torus/NURBS) with trimming loops; adaptive (u,v) subdivision driven by curvature/error; seam handling.
   - Create bridge functions under `components/curve_util.rs` and `components/surface_util.rs` to wrap new evaluators.

3) Face Triangulation & Stitching
   - Introduce constrained triangulation utility (new module or extend `mesh_triangulation.rs`) supporting holes and trimming curves; produce indexed triangle meshes.
   - Add welding/vertex-merge with tolerance; detect/correct inverted normals; thin-triangle culling.
   - Add manifold checks and diagnostics helpers (`mesh_analysis.rs`).
   - Generate UVs/tangents where parametrization exists; carry per-face groups/material ids into outputs; provide legacy adapters for consumers expecting only positions/indices.

4) Feature Operators on Geometry
   - Refactor `extrude.rs`, `surface_freeform.rs`, `surface_primitive.rs`, and `surface_subd.rs` to build geometry via the new kernel instead of manual point math.
   - Implement loft/sweep/revolve/extrude pipelines that output intermediate surfaces, then mesh via shared mesher; ensure caps/closures and normal orientation.
   - Add fillet/chamfer/blend: offset surfaces/curves, intersect offsets, mesh transitional surfaces with G1/G2 targets where possible.
   - Add morph/twist/bend/displacement: deformation fields applied pre-mesh; post-pass weld + normal recompute.
   - Add offset/thickening (shelling) for surfaces/meshes with inside/outside options; bridge subdivision/quads where applicable.

5) Boolean/CSG
   - New module `src/geom/boolean.rs`: triangle-surface intersection, plane splitting, face classification; filtered predicates with exact fallback (rational/robust epsilon).
   - Integrate with components to expose boolean union/intersect/subtract nodes; emit diagnostics if repair/fallback applied.

6) API/Component Layer
   - Update component registry to expose new mesh outputs (`Value::Mesh`) while preserving legacy pins; add migration helpers in `coerce.rs`.
   - Provide mesh-quality inputs on relevant components (loft/sweep/extrude/boolean) with sane defaults and presets.
   - Add display helpers in `display_preview.rs` to visualize diagnostics (open edges, self-intersections).

7) Performance & WASM
   - Add BVH acceleration for intersection/tessellation queries; use rayon for CPU path with cfg gate for WASM single-thread.
   - Optimize memory layout (SoA for vertices/normals/uvs) and use arenas/pools in hot paths.
   - Add caching/invalidations for tessellation/triangulation; reuse buffers where possible.
   - Add optional simplification/LOD (edge-collapse) with watertightness guard for previews/exports.
   - Add timing/profiling hooks around tessellation/boolean/repair to track regressions (native and WASM).

8) Validation & Tooling
   - Unit tests per geom primitive and mesher; property tests for manifoldness/watertightness; fuzz booleans with random transforms.
   - Golden models under `tests/` for key ops (loft, sweep1/2, pipe, fillet, boolean) with tolerance budgets.
   - Add debug logging/feature flag for WASM (`console_error_panic_hook`) to surface diagnostics.
   - Author and maintain a documentation wiki (see Docs section) with API guides and code snippets for geom modules and components.

UI Integration (three.js)
-------------------------
- Preserve the existing mesh shape: continue emitting indexed triangle meshes with `vertices: Vec<[f64;3]>` and `faces: Vec<[u32]>`; adapters convert to three.js `BufferGeometry` (Float32Array positions, Uint32Array indices).
- Add normals/uvs in `Value::Mesh`; if absent, the web layer should compute flat/smooth normals as it does today-keep this path for backward compatibility.
- Keep the same scene graph hooks: materials/colors remain in `Value` as today; only the mesh payload changes.
- WASM bridge: expose a function to fetch mesh buffers (positions/indices/normals/uvs) without reallocating; avoid copies by using views.
- Diagnostics overlay (optional): provide open-edge/self-intersection info to the viewer; gate by a flag so default UX is unchanged.
- Regression check: reuse existing three.js render tests/examples; add a few golden snapshots for loft/sweep/pipe to ensure geometry parity after the new kernel lands.

Compatibility with Existing Math/Sets and Grasshopper Semantics
--------------------------------------------------------------
- Reuse shared utilities: keep `maths_*`, `transform_*`, `vector_*`, and `sets_*` as the single source; `geom` should call these instead of re-implementing transforms, domains, polylines, or set/list logic.
- Boundary adapters: add conversion helpers between Grasshopper `Value` types and `geom` types (points, vectors, domains, lists) in `components/coerce.rs` and/or a `geom::bridge` module.
- Preserve tolerances/units: use the same tolerance constants and unit conventions; normalize units/handedness on ingestion so downstream math behaves identically.
- Pins/GUIDs: keep existing pin order and GUIDs; add new mesh/tolerance options only as optional appended pins.
- Legacy surfaces: keep `Value::Surface` adapters so components expecting surfaces continue to work while new outputs prefer `Value::Mesh`.
- Data trees and ordering: respect Grasshopper data-tree alignment/flattening semantics; avoid reordering vertices/faces/lists unless necessary—if welding changes ordering, keep stable indices and diagnostics.
- Transform parity: ensure transforms applied through `geom` match existing component behavior; add round-trip tests.
- Compatibility tests: add regression cases that run math/sets-heavy chains (curve division/domains/list ops -> extrude/loft/sweep) before/after the mesher; include data-tree alignment checks.
- Diagnostics/docs: document any tolerance snapping/repairs and surface warnings when geometry is altered.

Landing Plan (Deliverables)
---------------------------
- New modules under `src/geom/` (core, surfaces, curves, mesh_builder, boolean, loft/sweep/revolve/pipe/patch/extrusion, offset/displacement, subdivision, simplify/LOD, cache/instrumentation, diagnostics).
- Extended value type & coercions (`graph/value.rs`, `components/coerce.rs`) plus updated components to emit `Value::Mesh`.
- Refactored feature components to use the unified mesher (`components/surface_freeform.rs`, `surface_primitive.rs`, `extrude.rs`, `mesh_triangulation.rs`).
- Test suite additions under `tests/` covering primitives, features, booleans, and robustness cases.
- Documentation: this plan + developer guide for meshing API and migration notes for component authors.
  - Documentation wiki: per-module pages (geom core/tessellation/triangulation/boolean/feature builders/offset/displacement/subdivision/LOD/cache) with code snippets, sample inputs/outputs, and integration recipes for components and three.js.

Suggested Execution Order
-------------------------
1) Boot geom core (`src/geom/*`), `Value::Mesh`, and mesh quality config.
2) Curve/surface evaluators + adaptive tessellation bridging into existing util modules.
3) Triangulation + welding/repair + diagnostics; hook into `mesh_primitive.rs`.
4) Refactor feature components (extrude/loft/sweep/revolve/pipe) to the new pipeline; add morph/twist hooks.
5) Boolean kernel and boolean components.
6) Fillet/chamfer/blend surfaces; deformation ops; mesh smoothing.
7) Performance/parallelization, WASM gating, and memory optimizations.
8) Harden with tests, fuzzing, golden models; ship docs/examples.

Notes for ghx-engine Fit
------------------------
- Keep `Value::Surface` as shim but steer UI/components to `Value::Mesh`; supply adapters for downstream consumers expecting surfaces.
- Preserve GUID/name registrations in components; expose quality/tolerance inputs without breaking pin order (append optional pins).
- Align math with existing `maths_*` utilities to avoid drift; add tolerance constants in one place to reduce epsilon scatter.
- For large freeform ops (e.g., `surface_freeform.rs` loft/sweep), encapsulate geometry construction into helper structs to avoid duplication and improve testability.

Build Guide: Integrating the Mesh Engine with Grasshopper Components
--------------------------------------------------------------------
1) Types and wiring
   - Add `Value::Mesh { vertices, faces, normals, uvs, diagnostics }` and `MeshQuality` to `graph/value.rs`.
   - Extend `components/coerce.rs` to accept/emit `Value::Mesh`; keep `Value::Surface` adapters.
   - Expose `pub mod geom;` in `lib.rs` and register feature modules (loft/sweep/extrusion/etc.).

2) Geometry modules
   - Implement geometry in `src/geom/` (loft, sweep, revolve, pipe, patch, extrusion, boolean).
   - Keep components thin: they parse inputs, build a `MeshQuality`, call geom, and return `Value::Mesh`.

3) Component refactors (example: loft)
   - Map existing pins to typed inputs (curves, options, quality).
   - Call `geom::loft::loft_mesh(profiles, options, quality, tolerances)`.
   - Return `Value::Mesh` on the main output; optionally also `Value::Surface` via adapter for legacy consumers.

4) Diagnostics and preview
   - Thread `MeshDiagnostics` through outputs; update `display_preview.rs` to overlay open edges/self-intersections.
   - Log tolerance snaps and repairs behind a feature flag for WASM friendliness.

5) Testing
   - Add golden meshes for each component in `tests/`; property tests for manifoldness and watertightness.

Code sketch: component calling the geom loft mesher
```rust
use crate::components::{Component, ComponentError, ComponentResult};
use crate::graph::value::{MeshQuality, Value};
use crate::geom::{loft, mesh::MeshDiagnostics};

#[derive(Debug, Default, Clone, Copy)]
pub struct LoftComponent;

impl Component for LoftComponent {
    fn evaluate(&self, inputs: &[Value], _meta: &MetaMap) -> ComponentResult {
        // 1) Coerce inputs
        let profiles = coerce::coerce_curve_list(&inputs[0])?;
        let options = coerce::coerce_loft_options(&inputs.get(1))?;
        let quality = MeshQuality::from_meta_or_default(_meta);

        // 2) Call geometry kernel
        let (mesh, diagnostics): (Value, MeshDiagnostics) = loft::loft_mesh(profiles, options, quality)
            .map_err(|e| ComponentError::new(format!("Loft failed: {e}")))?;

        // 3) Return mesh (and optionally diagnostics)
        let mut out = BTreeMap::new();
        out.insert("M".to_string(), mesh);            // primary mesh output
        out.insert("D".to_string(), Value::from(diagnostics)); // optional diagnostics pin
        Ok(out)
    }
}
```
Apply the same pattern for sweep/pipe/revolve/patch: coerce inputs -> build quality/tolerances -> call `geom::<feature>::...` -> return `Value::Mesh` (+ diagnostics).

Scaffolding File Structure (proposed)
-------------------------------------
```
alpha/ghx-engine/
+- src/
|  +- geom/                       # New geometry kernel
|  |  +- mod.rs
|  |  +- core.rs                  # Point3, Vec3, Transform, BBox, Tolerance
|  |  +- curve.rs                 # Trait Curve3 + evaluators (line/arc/bez/b-spline/nurbs)
|  |  +- surface.rs               # Trait Surface + evaluators (plane/cylinder/cone/sphere/torus/nurbs)
|  |  +- solid.rs                 # Solid/B-rep wrapper
|  |  +- mesh.rs                  # Mesh struct, MeshBuilder, welding, diagnostics
|  |  +- tessellation.rs          # Adaptive curve/surface tessellators
|  |  +- triangulation.rs         # Constrained triangulation utilities (holes/trim loops)
|  |  +- boolean.rs               # CSG operations and intersection band remeshing
|  |  +- extrusion.rs             # Geometry construction for extrude variants
|  |  +- loft.rs                  # Loft implementations (Fit/ControlPoint/Network options)
|  |  +- sweep.rs                 # Sweep1/2 and pipe/rail handling logic
|  |  +- revolve.rs               # Revolve/RailRevolution helpers
|  |  +- pipe.rs                  # Pipe/PipeVariable geometry helpers
|  |  +- patch.rs                 # Patch/Boundary/Fragment patch filling
|  |  +- offset.rs                # Shell/thickening/offset solids for surfaces/meshes
|  |  +- displacement.rs          # Displacement/height field application pre-mesh
|  |  +- subdivision.rs           # Optional subdivision/quad-friendly mesh support
|  |  +- simplify.rs              # LOD/simplification (edge-collapse) with watertight guard
|  |  +- deformation.rs           # Twist/bend/taper/morph fields
|  |  +- fillet_chamfer.rs        # Offset/intersections for fillet/chamfer/blend helpers
|  |  +- cache.rs                 # Tessellation/triangulation caching and instrumentation
|  |  +- bvh.rs                   # Acceleration structures for intersection queries
|  +- components/
|  |  +- ... existing files ...
|  |  +- mesh_triangulation.rs    # Extended/bridged to geom::triangulation
|  |  +- mesh_analysis.rs         # Uses geom diagnostics
|  |  +- surface_freeform.rs      # Refactored to call unified mesher
|  |  +- surface_primitive.rs     # Refactored to use geom surfaces/mesher
|  |  +- surface_subd.rs          # Bridges to new mesher
|  |  +- ... (Grasshopper component wrappers only; geometry lives in geom/)
|  +- graph/
|  |  +- value.rs                 # Adds Value::Mesh, MeshQuality, MeshDiagnostics
|  |  +- ... unchanged ...
|  +- lib.rs                      # Exposes geom and updated components
+- docs/
|  +- mesh_engine_integration_plan.md
|  +- ... other docs ...
+- tests/
|  +- mesh_primitives.rs          # Golden outputs for mesh construction
|  +- features_extrude.rs
|  +- features_loft_sweep.rs
|  +- features_pipe_revolve.rs
|  +- booleans.rs
|  +- fillet_chamfer.rs
|  +- deformation.rs
|  +- fuzz.rs                     # Property/fuzz tests for manifoldness/robustness
+- Cargo.toml                     # Add feature flags for rayon, wasm, debug logging
```

 Todo (Isolated Build First, Component Integration After WASM Success)
------------------------------------
Phase 1 - Isolated mesh engine in `geom/` (no component or graph changes)
Goal: the entire new pipeline lives in `alpha/ghx-engine/src/geom/*` behind a feature flag; the existing `components/*` + `graph/*` compile unchanged.

Phase 1 rules (non-invasive)
----------------------------
- Only touch `alpha/ghx-engine/src/geom/*`, `alpha/ghx-engine/Cargo.toml`, and `alpha/ghx-engine/src/lib.rs`.
- Do not edit `alpha/ghx-engine/src/components/*` or `alpha/ghx-engine/src/graph/*` until Phase 3.
- WASM build must pass before starting component integration.

Feature gating (hard isolation)
- [x] `alpha/ghx-engine/Cargo.toml`: Add a `[features] mesh_engine_next = []` feature that is **not** included in `default`.
- [x] `alpha/ghx-engine/src/lib.rs`: Only `pub mod geom;` (or re-export the new geom API) when `cfg(feature = "mesh_engine_next")` is enabled.
- [ ] Repo rule (Phase 1): do **not** edit `alpha/ghx-engine/src/components/*` or `alpha/ghx-engine/src/graph/*` in this phase (the point is to keep the current engine stable).
- [x] Build check (native, default features): `cargo build -p ghx-engine` succeeds with `mesh_engine_next` disabled.

Scaffold the new `geom` module tree (compile-first skeleton)
- [x] `alpha/ghx-engine/src/geom/mod.rs`: Declare submodules and re-exports (keep the public surface minimal; prefer `pub(crate)` until integration).
- [x] `alpha/ghx-engine/src/geom/core.rs`: Add core types and tolerances used everywhere in geom (even if initially minimal).
- [x] `alpha/ghx-engine/src/geom/curve.rs`: Define minimal curve API + at least one evaluator (line/polyline) returning sampled points.
- [x] `alpha/ghx-engine/src/geom/surface.rs`: Define minimal surface API + at least one evaluator (plane) returning a grid.
- [x] `alpha/ghx-engine/src/geom/mesh.rs`: Define the internal mesh representation (positions + indices) and a builder.
- [x] `alpha/ghx-engine/src/geom/tessellation.rs`: Implement the "glue" functions that call curve/surface evaluators and produce polylines/grids.
- [x] `alpha/ghx-engine/src/geom/triangulation.rs`: Add a first triangulation path for simple grids (no trimming/holes yet).
- [x] `alpha/ghx-engine/src/geom/diagnostics.rs`: Define lightweight diagnostics structs (counts + warnings) for Phase 1.
- [x] `alpha/ghx-engine/src/geom/boolean.rs`: Create a stub module that compiles (real implementation can come later in Phase 2+).
- [x] Build check (native, feature on): `cargo build -p ghx-engine --features mesh_engine_next` succeeds.

Temporary payloads (no `Value::Mesh` yet)
- [x] `alpha/ghx-engine/src/geom/mesh.rs`: Add a temporary payload type (e.g., `GeomMesh`) that can be returned from geom APIs without touching `graph/value.rs`.
- [x] `alpha/ghx-engine/src/geom/mesh.rs`: Add a temporary diagnostics payload (e.g., `GeomMeshDiagnostics`) with room for: open edges, degenerate triangles, weld stats, timing.
- [x] `alpha/ghx-engine/src/geom/mod.rs`: Expose only these temporary types/functions from `geom` (avoid committing to final integration API yet).

End-to-end "hello mesher" API (prove the pipeline works in isolation)
- [x] `alpha/ghx-engine/src/geom/mesh.rs`: Provide a single entry-point function (e.g., `geom::mesh::mesh_surface(...) -> (GeomMesh, GeomMeshDiagnostics)`).
- [x] `alpha/ghx-engine/src/geom/curve.rs`: Provide a single entry-point tessellator (e.g., `geom::curve::tessellate_curve(...)`).
- [x] `alpha/ghx-engine/src/geom/surface.rs`: Provide a single entry-point tessellator (e.g., `geom::surface::tessellate_surface(...)`).
- [x] Build check: `cargo check -p ghx-engine --features mesh_engine_next` stays green as APIs evolve.

Tests (stay inside `geom/tests` so components are untouched)
- [x] `alpha/ghx-engine/src/geom/tests/mod.rs`: Add a test module root (if not present) that is only compiled when `mesh_engine_next` is enabled.
- [x] `alpha/ghx-engine/src/geom/tests/test_curve_basic.rs`: Add tests for curve tessellation invariants (monotonic parameter steps, endpoints preserved, open vs closed behavior).
- [x] `alpha/ghx-engine/src/geom/tests/test_surface_basic.rs`: Add tests for plane grid tessellation (grid dims, normals consistency).
- [x] `alpha/ghx-engine/src/geom/tests/test_triangulation_basic.rs`: Add tests for grid triangulation (index bounds, triangle count, winding consistency).
- [x] `alpha/ghx-engine/src/geom/tests/test_mesh_sanity.rs`: Add mesh sanity checks (no NaNs, no out-of-range indices, optional degenerate triangle detection).
- [x] Test run: `cargo test -p ghx-engine --features mesh_engine_next` passes.

WASM build proof (still isolated)
- [ ] Ensure target exists: `rustup target add wasm32-unknown-unknown`.
- [ ] WASM build (feature on): `cargo build -p ghx-engine --no-default-features --features mesh_engine_next --target wasm32-unknown-unknown` passes.
- [ ] If any dependency is not wasm-friendly, gate it inside `cfg(not(target_arch = \"wasm32\"))` or behind an additional feature (but keep Phase 1 changes inside `geom` + `Cargo.toml`/`lib.rs` only).

Docs (keep the rules explicit so Phase 1 stays "non-invasive")
- [x] `alpha/docs/mesh_engine_integration_plan.md`: Add a short "Phase 1 rules" paragraph: *no edits to components/graph; feature-flag only; wasm build required to pass before Phase 3*.
- [x] `alpha/docs/geom_tolerances.md`: Describe the tolerance strategy used by `geom` and note it is internal until Phase 3 integration.

Phase 2 - Stabilize isolated pipeline
Goal: implement all CAD/feature functionality listed in this document inside `alpha/ghx-engine/src/geom/*` while keeping `components/*` and `graph/*` untouched.

Quality/tolerance/caching/diagnostics (still internal-only)
- [x] `alpha/ghx-engine/src/geom/core.rs`: Finalize `Tolerance` + helpers used consistently across all geom modules (no epsilon scatter).
- [x] `alpha/ghx-engine/src/geom/diagnostics.rs`: Expand diagnostics to cover: open edges, non-manifold edges, degenerate tris, weld stats, boolean fallbacks, timing buckets.
- [x] `alpha/ghx-engine/src/geom/cache.rs`: Implement cache keys and caches for tessellation + triangulation + weld/repair passes.
- [x] `alpha/ghx-engine/src/geom/metrics.rs`: Add opt-in timing/profiling hooks (feature-gated; wasm-safe).

Curve evaluators + adaptive tessellation (implement the full curve set mentioned)
- [x] `alpha/ghx-engine/src/geom/curve.rs`: Implement line/polyline (if not done in Phase 1).
- [x] `alpha/ghx-engine/src/geom/curve.rs`: Implement arc/circle/ellipse with stable frames and seam handling.
- [x] `alpha/ghx-engine/src/geom/curve.rs`: Implement quadratic/cubic Bezier with derivatives/curvature estimates.
- [x] `alpha/ghx-engine/src/geom/curve.rs`: Implement B-spline + rational B-spline/NURBS (knot multiplicity, weights, tangent continuity reporting).
- [x] `alpha/ghx-engine/src/geom/tessellation.rs`: Adaptive curve subdivision (curvature/flatness + max deviation + segment caps).
- [x] `alpha/ghx-engine/src/geom/tests/test_curve_basic.rs`: Add regression coverage for extreme scales, open/closed, seams, and parameterization stability.

Surface evaluators + trimming + adaptive tessellation (implement the full surface set mentioned)
- [x] `alpha/ghx-engine/src/geom/surface.rs`: Implement plane/cylinder/cone/sphere/torus evaluators with seam/pole handling.
- [x] `alpha/ghx-engine/src/geom/surface.rs`: Implement NURBS surface evaluation (including partial derivatives for normals).
- [x] `alpha/ghx-engine/src/geom/trim.rs`: Convert trimming curves into parameter-space loops (outer + holes), including orientation normalization.
- [x] `alpha/ghx-engine/src/geom/tessellation.rs`: Adaptive (u,v) refinement driven by curvature/error budgets + seam stitching rules.
- [x] `alpha/ghx-engine/src/geom/tests/test_surface_basic.rs`: Add trimmed-patch tests (holes, seams, singularities/poles).

Triangulation + welding + repair (produce watertight/manifold triangle meshes)
- [x] `alpha/ghx-engine/src/geom/triangulation.rs`: Constrained triangulation supporting holes + trim loops.
- [x] `alpha/ghx-engine/src/geom/triangulation.rs`: Triangle quality metrics + skinny-triangle culling (with diagnostics).
- [x] `alpha/ghx-engine/src/geom/mesh.rs`: Tolerance-aware vertex welding + stable index remap.
- [x] `alpha/ghx-engine/src/geom/mesh.rs`: Normal consistency fixes + recompute (smooth normals; flat policy TBD if needed).
- [x] `alpha/ghx-engine/src/geom/mesh.rs`: UV generation where parametrization exists; placeholder tangents if needed for consumers later.
- [x] `alpha/ghx-engine/src/geom/tests/test_triangulation_basic.rs`: Cover holes, trims, normal consistency, weld accuracy.

Feature builders (implement CAD ops inside `geom/` only)
- [x] `alpha/ghx-engine/src/geom/extrusion.rs`: Extrude variants (linear + along/angled/point), caps, and orientation rules.
- [x] `alpha/ghx-engine/src/geom/revolve.rs`: Revolution + RailRevolution with seam handling, caps, and normal orientation.
- [x] `alpha/ghx-engine/src/geom/loft.rs`: Loft covering component variants (FitLoft + ControlPointLoft + standard Loft) and LoftOptions (closed/seam adjust/rebuild/refit/type) + diagnostics for twists/self-intersections.
- [x] `alpha/ghx-engine/src/geom/sweep.rs`: Sweep1/2 + stable frame computation + twist control + cap meshing + rail continuity diagnostics.
- [x] `alpha/ghx-engine/src/geom/pipe.rs`: Pipe + PipeVariable (parameter/radius lists), junction handling, and self-intersection guards.
- [x] `alpha/ghx-engine/src/geom/patch.rs`: Patch + FragmentPatch + BoundarySurfaces that respect trim loops and use constrained triangulation.
- [x] `alpha/ghx-engine/src/geom/surface_fit.rs`: SurfaceFromPoints / surface-grid fitting with diagnostics (this backs the "Surface From Points" component).
- [x] `alpha/ghx-engine/src/geom/surface.rs`: Surface builders used by existing components: FourPointSurface, EdgeSurface, RuledSurface, NetworkSurface, SumSurface (construct surface + then mesh through the shared pipeline).
- [x] `alpha/ghx-engine/src/geom/offset.rs`: Offset/thickening/shelling for surfaces/meshes with inside/outside options + remesh triggers.
- [x] `alpha/ghx-engine/src/geom/displacement.rs`: Displacement/heightfield pre-mesh deformation + post weld/normal repair.
- [x] `alpha/ghx-engine/src/geom/deformation.rs`: Twist/bend/taper/morph deformation fields (deterministic frames; post weld/normal repair).
- [x] `alpha/ghx-engine/src/geom/subdivision.rs`: SubD support mirroring current components (Box, Fuse, MultiPipe, FromMesh, MeshFromSubd, control polygon/vertex+edge tags) + (optional) quad-friendly path (still output triangles in Phase 2).
- [x] `alpha/ghx-engine/src/geom/simplify.rs`: Optional LOD simplification (edge-collapse) with watertightness guard.

Surface/Brep utilities currently implemented in `src/components/surface_util.rs` (move logic into `geom/`, keep wrappers for Phase 3)
- [x] `alpha/ghx-engine/src/geom/solid.rs`: BrepJoin (join surfaces into a closed/sane shell where possible) with diagnostics.
- [x] `alpha/ghx-engine/src/geom/solid.rs`: CapHoles + CapHolesEx (cap planar-ish holes, report failures explicitly).
- [x] `alpha/ghx-engine/src/geom/solid.rs`: MergeFaces (merge coplanar/continuous faces with tolerance guards).
- [x] `alpha/ghx-engine/src/geom/trim.rs`: CopyTrim / Retrim / Untrim support (trim loop manipulation and surface parameter-space consistency).
- [x] `alpha/ghx-engine/src/geom/surface.rs`: Isotrim (subsurface extraction) and DivideSurface (grid sampling policies).
- [x] `alpha/ghx-engine/src/geom/surface.rs`: Flip surface orientation (normals + parameter directions) with diagnostics.
- [x] `alpha/ghx-engine/src/geom/analysis.rs`: SurfaceFrames + edge extraction helpers used by ClosedEdges / EdgesFrom* components (needed for trimming + diagnostics parity later).
- [x] `alpha/ghx-engine/src/geom/fillet_chamfer.rs`: FilletEdge implementation (even a first "polyline/mesh-edge fillet" version), with clear limitations/diagnostics.

Booleans/CSG (geom-only implementation)
- [x] `alpha/ghx-engine/src/geom/boolean.rs`: Triangle/triangle and triangle/surface intersection primitives (filtered predicates; robust eps fallback).
- [x] `alpha/ghx-engine/src/geom/boolean.rs`: Classification (inside/outside) + tagging.
- [x] `alpha/ghx-engine/src/geom/boolean.rs`: Intersection-band remeshing + stitching + repair with diagnostics.
- [x] `alpha/ghx-engine/src/geom/boolean.rs`: Fallback strategy (tolerance relax and/or voxel fallback) with explicit diagnostics flags.

Acceleration + performance (still isolated; wasm-safe)
- [x] `alpha/ghx-engine/src/geom/bvh.rs`: BVH acceleration for intersection and proximity queries (feature-gated parallelism off on wasm).
- [x] `alpha/ghx-engine/src/geom/mesh.rs`: Move toward SoA buffers internally if needed; keep API stable for Phase 3 adapters.

Tests + fixtures + tooling for Phase 2 (prove feature coverage before integration)
- [x] `alpha/ghx-engine/tests/`: Add golden fixtures for extrude/loft/sweep/pipe/revolve/boolean/patch/offset/deform outputs (still using geom temporary payloads).
- [x] `alpha/ghx-engine/src/bin/mesh_cli.rs`: CLI that runs selected `geom` ops (golden scenarios) and exports `.obj` meshes + `.snap` diagnostics for review.
  - `cargo run -p ghx-engine --bin mesh_cli --features mesh_engine_next -- list`
  - `cargo run -p ghx-engine --bin mesh_cli --features mesh_engine_next -- run all --out-dir /tmp/mesh_cli_out`
- [x] `cargo test -p ghx-engine --features mesh_engine_next`: Passes on native.
- [x] `cargo build -p ghx-engine --no-default-features --features mesh_engine_next --target wasm32-unknown-unknown`: Passes for wasm.

Gate (must be true before touching components/graph)
- [x] `mesh_engine_next` passes native tests.
- [x] `mesh_engine_next` passes wasm build (`wasm32-unknown-unknown`) and smoke tests.
- [x] Feature coverage parity checklist approved (loft/sweep/extrude/revolve/pipe/boolean/patch/offset/deform/simplify).

Phase 3 - Integration into `graph` and `components` (after gate passes)
Goal: switch the live component layer to call the new `geom` engine (now proven in Phase 2) while keeping all existing graphs stable (pin order + GUIDs + legacy surface outputs).

Graph/value layer (introduce the stable public mesh type)
- [x] `alpha/ghx-engine/src/graph/value.rs`: Add `Value::Mesh { vertices, faces, normals, uvs, diagnostics }` (or equivalent) and keep `Value::Surface { vertices, faces }` as a legacy shim.
- [x] `alpha/ghx-engine/src/graph/value.rs`: Define `MeshQuality` + parsing from `MetaMap` (defaults + presets); keep API wasm-friendly.
- [x] `alpha/ghx-engine/src/graph/value.rs`: Define `MeshDiagnostics` (match what Phase 2 produced internally) + conversion from `geom` diagnostics.
- [x] `alpha/ghx-engine/src/graph/value.rs`: Update (de)serialization and any `expect_*` helpers to support `Value::Mesh`.
- [x] `alpha/ghx-engine/src/graph/value.rs`: Add explicit adapters: `mesh_to_surface_legacy()` and `surface_legacy_to_mesh()` (document lossy parts).

Cross-cutting component helpers (must be updated before feature components)
- [x] `alpha/ghx-engine/src/components/coerce.rs`: Accept `Value::Mesh` anywhere a mesh/surface is expected; keep accepting `Value::Surface` for backward compatibility.
- [x] `alpha/ghx-engine/src/components/coerce.rs`: Add conversions between `Value::{Surface,Mesh}` and `geom` payloads (including tolerance/quality/meta plumbing).
- [x] `alpha/ghx-engine/src/components/params_geometry.rs`: Treat `Value::Mesh` as a first-class geometry type anywhere `Value::Surface` was accepted; keep legacy behavior for lists/trees.
- [x] `alpha/ghx-engine/src/components/display_preview.rs`: Render `Value::Mesh` directly; for `Value::Surface` keep the current path (or adapt through the legacy adapter).
- [x] `alpha/ghx-engine/src/components/transform_affine.rs`: Apply transforms to `Value::Mesh` (positions + normals) and keep existing `Value::Surface` logic.
- [x] `alpha/ghx-engine/src/components/transform_euclidean.rs`: Apply transforms to `Value::Mesh` and keep existing `Value::Surface` logic.
- [x] `alpha/ghx-engine/src/components/transform_array.rs`: Array-copy/instance transforms must handle `Value::Mesh` while preserving deterministic ordering.
- [x] `alpha/ghx-engine/src/components/transform_util.rs`: Any generic transform helpers must handle `Value::Mesh` and preserve diagnostics where possible.
- [x] `alpha/ghx-engine/src/components/mesh_analysis.rs`: Accept `Value::Mesh` inputs everywhere it currently accepts `Value::Surface` meshes (DeconstructMesh, FaceNormals, MeshEdges, ClosestPoint, etc.).
- [x] `alpha/ghx-engine/src/components/mesh_primitive.rs`: Switch mesh construction outputs to `Value::Mesh` as primary; keep emitting `Value::Surface` legacy outputs where existing pins expect it.
- [x] `alpha/ghx-engine/src/components/mesh_triangulation.rs`: Switch algorithmic mesh outputs (FacetDome, Voronoi, DelaunayMesh, etc.) to `Value::Mesh` as primary; keep `Value::Surface` legacy adapters if consumers rely on them.

Curve components (switch to geom curves)
- [x] `alpha/ghx-engine/src/components/curve_primitive.rs`: Circle/Arc/Line/Polygon/Rectangle/Ellipse -> build `geom::curve` primitives and tessellate via `geom::tessellation`.
- [x] `alpha/ghx-engine/src/components/curve_spline.rs`: Nurbs/Bezier/Interpolate/Polyline -> build `geom::curve` splines and tessellate via `geom::tessellation`.
- [x] `alpha/ghx-engine/src/components/curve_division.rs`: Divide/Shatter/Contour -> use `geom::curve` splitting and sampling methods.
- [x] `alpha/ghx-engine/src/components/curve_analysis.rs`: Evaluate/Length/Curvature/Frames -> use `geom::curve` analysis methods.
- [x] `alpha/ghx-engine/src/components/curve_util.rs`: Offset/Fillet/Join/Flip/Extend -> use `geom` operations.

Surface “Freeform” components (wire each feature to `geom`, keep GUIDs/pins)
- [ ] `alpha/ghx-engine/src/components/surface_freeform.rs`: Extrude / ExtrudeLinear / ExtrudeAngled / ExtrudePoint / ExtrudeAlong -> call `geom::extrusion::*`; output `Value::Mesh` and (where required) a `Value::Surface` legacy adapter on existing surface pins.
- [ ] `alpha/ghx-engine/src/components/surface_freeform.rs`: Loft / FitLoft / ControlPointLoft + LoftOptions -> call `geom::loft::*`; preserve LoftOptions pin parsing and forward to `MeshQuality`/loft options.
- [ ] `alpha/ghx-engine/src/components/surface_freeform.rs`: Sweep1 / Sweep2 -> call `geom::sweep::*`; preserve twist/frames semantics; surface/mesh outputs remain compatible.
- [ ] `alpha/ghx-engine/src/components/surface_freeform.rs`: Pipe / PipeVariable -> call `geom::pipe::*`; preserve parameter/radius list handling and error messages.
- [ ] `alpha/ghx-engine/src/components/surface_freeform.rs`: Patch / FragmentPatch / BoundarySurfaces -> call `geom::patch::*` (trim loops + constrained triangulation); preserve input coercion rules.
- [ ] `alpha/ghx-engine/src/components/surface_freeform.rs`: Revolution / RailRevolution -> call `geom::revolve::*`; preserve seam defaults and cap behavior.
- [ ] `alpha/ghx-engine/src/components/surface_freeform.rs`: FourPointSurface / EdgeSurface / RuledSurface / NetworkSurface / SumSurface -> construct through `geom::surface::*` builders then mesh via the shared pipeline (no more ad-hoc vertex math).
- [ ] `alpha/ghx-engine/src/components/surface_freeform.rs`: SurfaceFromPoints -> call `geom::surface_fit::*`; preserve grid sizing/ordering behavior.

Surface “Primitive” components (switch to geom surfaces + mesher)
- [ ] `alpha/ghx-engine/src/components/surface_primitive.rs`: Cylinder/Cone/Sphere/PlaneSurface/QuadSphere -> build `geom::surface` primitives and mesh via shared pipeline; keep existing outputs (including “tip” outputs) stable.
- [ ] `alpha/ghx-engine/src/components/surface_primitive.rs`: Any component that currently emits a mesh as `Value::Surface` should additionally expose `Value::Mesh` (append-only pin) or keep emitting legacy as an adapter.

Surface “Util” components (move geometry logic to geom; keep wrappers thin)
- [ ] `alpha/ghx-engine/src/components/surface_util.rs`: DivideSurface / SurfaceFrames / Isotrim -> call `geom::surface_ops::*` or `geom::analysis::*` and return the same shapes/trees as today.
- [ ] `alpha/ghx-engine/src/components/surface_util.rs`: CopyTrim / Retrim / Untrim -> call `geom::trim::*`; preserve trim-loop ordering semantics.
- [ ] `alpha/ghx-engine/src/components/surface_util.rs`: OffsetSurface / OffsetSurfaceLoose -> call `geom::offset::*` and return mesh/surface outputs consistent with today.
- [ ] `alpha/ghx-engine/src/components/surface_util.rs`: CapHoles / CapHolesEx -> call `geom::brep_ops::*` (or `geom::solid::*`) and preserve the “solid” boolean pin semantics.
- [ ] `alpha/ghx-engine/src/components/surface_util.rs`: BrepJoin / MergeFaces -> call `geom::brep_ops::*` and preserve diagnostic/boolean pins.
- [ ] `alpha/ghx-engine/src/components/surface_util.rs`: Flip -> call `geom::surface_ops::flip()` (or equivalent) and preserve result flags.
- [ ] `alpha/ghx-engine/src/components/surface_util.rs`: FilletEdge -> call `geom::fillet_chamfer::*` (document limitations; emit diagnostics instead of silent failures).

Surface “Analysis” components (add Mesh support where appropriate)
- [ ] `alpha/ghx-engine/src/components/surface_analysis.rs`: Accept `Value::Mesh` inputs where the component only needs a triangulated representation (e.g., normals/areas/bounds); otherwise keep surface-only behavior explicit and error clearly.

Surface “SubD” components (bridge to geom SubD)
- [ ] `alpha/ghx-engine/src/components/surface_subd.rs`: Box/Fuse/MultiPipe/FromMesh/MeshFromSubd/Tags/ControlPolygon/Vertices -> move data model into `geom::subdivision` and keep component as a thin wrapper; keep existing pins stable.
- [ ] `alpha/ghx-engine/src/components/surface_subd.rs`: Ensure `MeshFromSubd` emits `Value::Mesh` (and optionally `Value::Surface` legacy adapter).

Other component files that must become `Mesh`-aware (because they mention `Value::Surface` today)
- [ ] `alpha/ghx-engine/src/components/curve_spline.rs`: Where a surface’s vertex set is accepted as an input, accept `Value::Mesh` too (or hard-error with a clear message if true surface parameterization is required).
- [ ] `alpha/ghx-engine/src/components/vector_grid.rs`: Accept `Value::Mesh` anywhere `Value::Surface` meshes are currently used for grid/triangulation logic.
- [ ] `alpha/ghx-engine/src/components/vector_point.rs`: Update any type-dispatch logic that currently treats `Value::Surface` as “mesh-like” to include `Value::Mesh`.
- [ ] `alpha/ghx-engine/src/components/maths_script.rs`: Update any “allowed input types” lists to include `Value::Mesh` where `Value::Surface` is currently allowed.

Integration verification (native + wasm)
- [ ] `cargo test -p ghx-engine`: passes with default features (no `mesh_engine_next` required anymore for runtime behavior).
- [ ] `cargo test -p ghx-engine --features mesh_engine_next`: still passes (keeps the isolated engine healthy).
- [ ] `cargo build -p ghx-engine --target wasm32-unknown-unknown`: passes (default feature set used by wasm).
- [ ] Add/extend regression tests that compare a handful of key component outputs before/after (vertex/face counts, watertightness diagnostics, stable pin outputs) without requiring perfect geometric identity.
