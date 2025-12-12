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

Todo (Detailed, Feature-by-Feature) (IMPORTANT: please mark as completed when finished with the todo-task)
-----------------------------------
Foundations
- [ ] `alpha/ghx-engine/src/graph/value.rs`: Define `Value::Mesh { vertices, faces, normals, uvs, diagnostics }`.
- [ ] `alpha/ghx-engine/src/graph/value.rs`: Introduce the `MeshQuality` struct with sensible defaults and metadata parsing helpers.
- [ ] `alpha/ghx-engine/src/graph/value.rs`: Update serialization/deserialization helpers to understand the new mesh variant.
- [ ] `alpha/ghx-engine/src/components/coerce.rs`: Emit and accept `Value::Mesh` while keeping the `Value::Surface` shim for legacy consumers.
- [ ] `alpha/ghx-engine/src/geom/mod.rs`: Declare the `geom` module tree and re-export its submodules.
- [ ] `alpha/ghx-engine/src/geom/core.rs`: Scaffold the core geometry module with placeholder implementations.
- [ ] `alpha/ghx-engine/src/geom/curve.rs`: Scaffold the curve module with placeholder evaluators/traits.
- [ ] `alpha/ghx-engine/src/geom/surface.rs`: Scaffold the surface module with placeholder evaluators/traits.
- [ ] `alpha/ghx-engine/src/geom/solid.rs`: Scaffold the solid module with placeholder traits.
- [ ] `alpha/ghx-engine/src/geom/mesh.rs`: Scaffold the mesh module with placeholder builder traits.
- [ ] `alpha/ghx-engine/src/geom/core.rs`: Implement `Point3`, `Vec3`, `BBox`, `Transform`, and `Tolerance` on top of existing math utilities.
- [ ] `alpha/ghx-engine/src/geom/tolerance.rs`: Centralize absolute/relative epsilons and expose helpers for tolerance-safe comparisons.
- [ ] `alpha/ghx-engine/src/components/surface_freeform.rs`: Replace inline epsilon math with calls into `geom::tolerance`.
- [ ] `alpha/ghx-engine/src/components/surface_primitive.rs`: Replace inline epsilon math with calls into `geom::tolerance`.
- [ ] `alpha/ghx-engine/src/components/mesh_triangulation.rs`: Replace inline epsilon math with calls into `geom::tolerance`.
- [ ] `alpha/docs/geom_tolerances.md`: Document the shared tolerance strategy and expectations for new modules.
- [ ] `alpha/ghx-engine/src/components/coerce.rs`: Normalize units/handedness at ingestion before data enters the geom layer.
- [ ] `alpha/ghx-engine/src/geom/normalization.rs`: Implement helpers that convert incoming points/frames while respecting tolerance settings.
- [ ] `alpha/ghx-engine/tests/normalization.rs`: Add regression tests covering the normalization helpers.
- [ ] `alpha/ghx-engine/src/geom/cache.rs`: Define cache-key structures that cover inputs plus quality/tolerance controls.
- [ ] `alpha/ghx-engine/src/geom/cache.rs`: Implement tessellation/triangulation caches and expose instrumentation hooks.
- [ ] `alpha/ghx-engine/src/graph/evaluator.rs`: Wire cache invalidation hooks into the graph evaluator.
- [ ] `alpha/ghx-engine/src/geom/mesh.rs`: Design a shared vertex/index buffer representation for instanced meshes.
- [ ] `alpha/ghx-engine/src/graph/instancing.rs`: Implement transform-stack helpers so instancing stays deterministic.
- [ ] `alpha/ghx-engine/tests/instancing.rs`: Validate shared buffer emission and instancing transforms via tests.

Curve Tessellation
- [ ] `alpha/ghx-engine/src/geom/curve.rs`: Implement the straight-line evaluator that outputs points and tangents.
- [ ] `alpha/ghx-engine/src/geom/curve.rs`: Implement the polyline evaluator that respects existing domain sampling data.
- [ ] `alpha/ghx-engine/src/geom/curve.rs`: Implement arc evaluators with consistent frames/seam handling.
- [ ] `alpha/ghx-engine/src/geom/curve.rs`: Implement circle evaluators with consistent frames/seam handling.
- [ ] `alpha/ghx-engine/src/geom/curve.rs`: Implement an ellipse evaluator aligned to principal axes.
- [ ] `alpha/ghx-engine/src/geom/curve.rs`: Implement quadratic/cubic Bezier evaluators that emit curvature and derivatives.
- [ ] `alpha/ghx-engine/src/geom/curve.rs`: Implement B-spline evaluators that handle knot multiplicity and tangents.
- [ ] `alpha/ghx-engine/src/geom/curve.rs`: Implement rational B-spline/NURBS evaluators with weight normalization/error reporting.
- [ ] `alpha/ghx-engine/src/geom/tessellation.rs`: Build curvature/flatness-driven adaptive subdivision for curves.
- [ ] `alpha/ghx-engine/src/geom/tessellation.rs`: Add arc-length parameterization utilities and expose them to curve evaluators.
- [ ] `alpha/ghx-engine/src/geom/diagnostics.rs`: Emit per-curve diagnostics (segment count, deviations, curvature stats).
- [ ] `alpha/ghx-engine/src/components/curve_util.rs`: Route component helpers through the new curve evaluators.
- [ ] `alpha/ghx-engine/src/components/curve_analysis.rs`: Consume the richer tessellation outputs and diagnostics.
- [ ] `alpha/ghx-engine/tests/curve_tessellation.rs`: Cover curvature-driven subdivision, extreme scales, and open/closed curves.

Surface Tessellation
- [ ] `alpha/ghx-engine/src/geom/surface.rs`: Implement plane surface evaluators that output grids and normals.
- [ ] `alpha/ghx-engine/src/geom/surface.rs`: Implement cylinder evaluators with seam/origin handling.
- [ ] `alpha/ghx-engine/src/geom/surface.rs`: Implement cone evaluators with seam/origin handling.
- [ ] `alpha/ghx-engine/src/geom/surface.rs`: Implement sphere evaluators that remain stable at the poles.
- [ ] `alpha/ghx-engine/src/geom/surface.rs`: Implement torus evaluators with seam handling.
- [ ] `alpha/ghx-engine/src/geom/surface.rs`: Implement NURBS surface evaluators that accept trimming data.
- [ ] `alpha/ghx-engine/src/geom/trim.rs`: Convert trimming curves into parameter-space loops ready for meshing.
- [ ] `alpha/ghx-engine/src/geom/tessellation.rs`: Add adaptive (u,v) grid refinement driven by curvature/error budgets.
- [ ] `alpha/ghx-engine/src/geom/surface.rs`: Add seam/orientation correction before meshing.
- [ ] `alpha/ghx-engine/src/components/surface_util.rs`: Wire component helpers to the new surface evaluators.
- [ ] `alpha/ghx-engine/src/components/surface_analysis.rs`: Read diagnostics produced by the surface tessellator.
- [ ] `alpha/ghx-engine/tests/surface_tessellation.rs`: Cover trimmed patches, seams, singularities, and tolerance adherence.

Triangulation, Welding, and Repair
- [ ] `alpha/ghx-engine/src/components/mesh_triangulation.rs`: Refactor the entry point to call the new constrained triangulator.
- [ ] `alpha/ghx-engine/src/geom/triangulation.rs`: Add trimming-boundary and interior-hole support.
- [ ] `alpha/ghx-engine/src/geom/triangulation.rs`: Implement triangle quality metrics plus skinny-triangle culling.
- [ ] `alpha/ghx-engine/src/geom/mesh.rs`: Implement tolerance-aware vertex welding/merge routines.
- [ ] `alpha/ghx-engine/src/geom/mesh.rs`: Detect and correct inverted normals after welding.
- [ ] `alpha/ghx-engine/src/components/mesh_analysis.rs`: Emit manifold/watertight diagnostics through `MeshDiagnostics`.
- [ ] `alpha/ghx-engine/src/geom/mesh.rs`: Generate UV buffers when parametrization data exists.
- [ ] `alpha/ghx-engine/src/geom/mesh.rs`: Generate tangents/bitangents alongside UVs for shading consumers.
- [ ] `alpha/ghx-engine/src/geom/mesh.rs`: Propagate per-face groups/material ids during triangulation.
- [ ] `alpha/ghx-engine/src/components/coerce.rs`: Build adapters that coerce `Value::Surface`/legacy meshes into `Value::Mesh` with weld/repair hooks.
- [ ] `alpha/ghx-engine/src/geom/diagnostics.rs`: Capture ingestion diagnostics (weld counts, open edges fixed).
- [ ] `alpha/ghx-engine/tests/triangulation.rs`: Add coverage for holes, trimming boundaries, open edges, normal consistency, weld accuracy, and UV/material propagation.

Feature Builders (one-by-one)
- [ ] `alpha/ghx-engine/src/geom/loft.rs`: Scaffold the loft geometry module.
- [ ] `alpha/ghx-engine/src/geom/sweep.rs`: Scaffold the sweep geometry module.
- [ ] `alpha/ghx-engine/src/geom/revolve.rs`: Scaffold the revolve geometry module.
- [ ] `alpha/ghx-engine/src/geom/pipe.rs`: Scaffold the pipe geometry module.
- [ ] `alpha/ghx-engine/src/geom/patch.rs`: Scaffold the patch geometry module.
- [ ] `alpha/ghx-engine/src/geom/extrusion.rs`: Scaffold the extrusion geometry module.
- [ ] `alpha/ghx-engine/src/geom/extrusion.rs`: Implement linear extrude surface generation.
- [ ] `alpha/ghx-engine/src/geom/extrusion.rs`: Mesh caps/sides for linear extrudes while honoring `MeshQuality`.
- [ ] `alpha/ghx-engine/src/components/extrude.rs`: Refactor the linear extrude component to call the geom module.
- [ ] `alpha/ghx-engine/tests/features_extrude.rs`: Add regression tests/diagnostics for linear extrudes.
- [ ] `alpha/ghx-engine/src/geom/extrusion.rs`: Implement frame/taper handling for along/angled/point extrudes.
- [ ] `alpha/ghx-engine/src/components/extrude.rs`: Wire along/angled/point extrude inputs to the geom API.
- [ ] `alpha/ghx-engine/tests/features_extrude.rs`: Cover along/angled/point extrude variants.
- [ ] `alpha/ghx-engine/src/geom/revolve.rs`: Implement revolve geometry with robust seam handling.
- [ ] `alpha/ghx-engine/src/geom/revolve.rs`: Add cap closing and normal orientation for revolved solids.
- [ ] `alpha/ghx-engine/src/components/revolve.rs`: Refactor revolve components to call the geom module.
- [ ] `alpha/ghx-engine/tests/features_revolve.rs`: Add revolve regression coverage.
- [ ] `alpha/ghx-engine/src/geom/loft.rs`: Implement adaptive section lofting.
- [ ] `alpha/ghx-engine/src/geom/loft.rs`: Add tight/loose/rebuild option handling plus diagnostics.
- [ ] `alpha/ghx-engine/src/components/loft.rs`: Refactor loft components to call the geom module.
- [ ] `alpha/ghx-engine/tests/features_loft.rs`: Add loft regression coverage.
- [ ] `alpha/ghx-engine/src/geom/sweep.rs`: Implement stable frame computation and twist control.
- [ ] `alpha/ghx-engine/src/geom/sweep.rs`: Add cap meshing and rail continuity diagnostics.
- [ ] `alpha/ghx-engine/src/components/sweep.rs`: Refactor sweep components to call the geom module.
- [ ] `alpha/ghx-engine/tests/features_sweep.rs`: Add sweep regression coverage.
- [ ] `alpha/ghx-engine/src/geom/pipe.rs`: Implement variable-radius sampling and junction handling.
- [ ] `alpha/ghx-engine/src/geom/pipe.rs`: Add tolerance-aware self-intersection guards and diagnostics.
- [ ] `alpha/ghx-engine/src/components/pipe.rs`: Refactor pipe components to call the geom module.
- [ ] `alpha/ghx-engine/tests/features_pipe.rs`: Add pipe regression coverage.
- [ ] `alpha/ghx-engine/src/components/surface_primitive.rs`: Refactor FourPointSurface to use surface evaluators then mesh.
- [ ] `alpha/ghx-engine/src/components/surface_primitive.rs`: Refactor EdgeSurface to use surface evaluators then mesh.
- [ ] `alpha/ghx-engine/src/components/surface_primitive.rs`: Refactor RuledSurface to use surface evaluators then mesh.
- [ ] `alpha/ghx-engine/src/components/surface_primitive.rs`: Refactor NetworkSurface/SumSurface to use surface evaluators then mesh.
- [ ] `alpha/ghx-engine/src/geom/patch.rs`: Implement trimming plus constrained triangulation for Patch/FragmentPatch/BoundarySurfaces.
- [ ] `alpha/ghx-engine/src/components/surface_freeform.rs`: Wire patch-style components to the geom module with G0/G1 options.
- [ ] `alpha/ghx-engine/tests/features_patch.rs`: Add regression coverage for patch, fragment, and boundary surfaces.
- [ ] `alpha/ghx-engine/src/geom/surface_fit.rs`: Implement SurfaceFromPoints fitting plus diagnostics.
- [ ] `alpha/ghx-engine/src/components/surface_from_points.rs`: Refactor the component to call `geom::surface_fit`.
- [ ] `alpha/ghx-engine/tests/surface_from_points.rs`: Cover fit-quality diagnostics.
- [ ] `alpha/ghx-engine/src/components/loft_options.rs`: Add mesh-quality/tolerance pins and forward them to geom APIs.
- [ ] `alpha/ghx-engine/src/geom/offset.rs`: Implement shelling logic for surfaces/meshes and trigger remeshing.
- [ ] `alpha/ghx-engine/src/components/offset.rs`: Expose inside/outside options and diagnostics for offset/thicken.
- [ ] `alpha/ghx-engine/tests/offset.rs`: Cover varying thickness/tolerance scenarios.
- [ ] `alpha/ghx-engine/src/geom/displacement.rs`: Implement height/scalar-field deformation plus weld/normal recompute passes.
- [ ] `alpha/ghx-engine/src/components/displacement.rs`: Refactor the displacement component to call the geom module and emit diagnostics.
- [ ] `alpha/ghx-engine/tests/displacement.rs`: Cover displacement workflows and diagnostics.
- [ ] `alpha/ghx-engine/src/geom/subdivision.rs`: Bridge `surface_subd` outputs into the new mesher with optional quad-friendly paths.
- [ ] `alpha/ghx-engine/src/components/surface_subd.rs`: Wire subdivision components to the geom module.
- [ ] `alpha/ghx-engine/tests/subdivision.rs`: Verify subdivision-to-mesh consistency.

Deformations and Morphs
- [ ] `alpha/ghx-engine/src/geom/deformation.rs`: Implement the twist deformation field plus weld/normal recompute stages.
- [ ] `alpha/ghx-engine/src/geom/deformation.rs`: Implement the bend deformation field with axis/strength controls.
- [ ] `alpha/ghx-engine/src/geom/deformation.rs`: Implement the taper deformation field for uniform and non-uniform modes.
- [ ] `alpha/ghx-engine/src/geom/deformation.rs`: Implement the morph/blend deformation field between reference meshes.
- [ ] `alpha/ghx-engine/src/geom/deformation.rs`: Add deterministic frame helpers to keep deformations repeatable.
- [ ] `alpha/ghx-engine/src/components/deformation.rs`: Plumb deformation parameters through component inputs and metadata.
- [ ] `alpha/ghx-engine/tests/deformation.rs`: Cover deformation extremes while verifying manifoldness.

Booleans/CSG
- [ ] `alpha/ghx-engine/src/geom/boolean.rs`: Scaffold the boolean geometry module.
- [ ] `alpha/ghx-engine/src/geom/boolean.rs`: Implement triangle/surface intersection routines with filtered predicates.
- [ ] `alpha/ghx-engine/src/geom/boolean.rs`: Implement classification and inside/outside tagging.
- [ ] `alpha/ghx-engine/src/geom/boolean.rs`: Implement intersection-band remeshing and stitching.
- [ ] `alpha/ghx-engine/src/geom/boolean.rs`: Add tolerance-relax or voxel fallbacks plus diagnostics.
- [ ] `alpha/ghx-engine/src/components/boolean.rs`: Expose the union component wired to the boolean kernel.
- [ ] `alpha/ghx-engine/src/components/boolean.rs`: Expose the intersect component wired to the boolean kernel.
- [ ] `alpha/ghx-engine/src/components/boolean.rs`: Expose the difference component wired to the boolean kernel.
- [ ] `alpha/ghx-engine/src/components/boolean.rs`: Add diagnostics pins reporting fallbacks/repairs.
- [ ] `alpha/ghx-engine/tests/boolean.rs`: Cover coplanar overlaps, near-coincident faces, nested solids, and tiny feature handling.

Fillet/Chamfer/Blend
- [ ] `alpha/ghx-engine/src/geom/fillet_chamfer.rs`: Implement curve offsetting helpers for fillet/chamfer workflows.
- [ ] `alpha/ghx-engine/src/geom/fillet_chamfer.rs`: Implement surface offsetting and intersection solving.
- [ ] `alpha/ghx-engine/src/geom/fillet_chamfer.rs`: Implement transitional surface tessellation targeting G1/G2 continuity.
- [ ] `alpha/ghx-engine/src/components/fillet.rs`: Build fillet/chamfer component APIs with radius/limit options and tolerance fallbacks.
- [ ] `alpha/ghx-engine/tests/fillet.rs`: Cover edge/vertex blends, small-radius failures, and continuity checks.

Diagnostics and Visualization
- [ ] `alpha/ghx-engine/src/graph/value.rs`: Define the `MeshDiagnostics` struct fields (open edges, inverted faces, weld counts, timing, etc.).
- [ ] `alpha/ghx-engine/src/geom/diagnostics.rs`: Ensure every geom/feature module populates `MeshDiagnostics`.
- [ ] `alpha/ghx-engine/src/components/display_preview.rs`: Render overlays for open edges, self-intersections, and degenerate triangles with toggles.
- [ ] `alpha/docs/mesh_diagnostics.md`: Document diagnostic fields and how components expose them.

Performance/WASM
- [ ] `alpha/ghx-engine/src/geom/bvh.rs`: Implement BVH acceleration structures for intersection queries.
- [ ] `alpha/ghx-engine/Cargo.toml`: Gate rayon-based parallel loops for native builds and single-thread WASM fallbacks.
- [ ] `alpha/ghx-engine/src/geom/mesh.rs`: Reorganize mesh buffers into a structure-of-arrays layout and add pooling.
- [ ] `alpha/ghx-engine/src/geom/cache.rs`: Reuse tessellation/triangulation buffers between evaluations when inputs match.
- [ ] `alpha/ghx-engine/src/geom/simplify.rs`: Implement optional simplification/LOD (edge-collapse) with watertightness guards.
- [ ] `alpha/ghx-engine/src/geom/metrics.rs`: Add timing/profiling hooks around tessellation, booleans, and repair passes (native + WASM).
- [ ] `alpha/ghx-engine/src/geom/diagnostics.rs`: Surface performance counters through diagnostics/logging.

Testing and Tooling
- [ ] `alpha/ghx-engine/tests/golden_meshes.rs`: Build golden-mesh fixtures for extrude, loft, sweep1/2, pipe, revolve, boolean, and fillet outputs.
- [ ] `alpha/ghx-engine/tests/deformation.rs`: Add property tests for manifoldness/watertightness.
- [ ] `alpha/ghx-engine/tests/fuzz_booleans.rs`: Add fuzzing harnesses for boolean operations and trimming workflows.
- [ ] `alpha/ghx-engine/src/lib.rs`: Wire debug logging flags and panic hooks for WASM builds.
- [ ] `tools/mesh_cli.rs`: Provide CLI/sample scripts to generate meshes and validate diagnostics.
- [ ] `alpha/ghx-engine/tests/render_snapshots.rs`: Capture render/golden snapshots (three.js) for UV/material propagation, LOD, and offset/displacement cases.
- [ ] `docs/wiki/geom/index.md`: Build and maintain the documentation wiki with per-module API docs, snippets, and integration recipes.
