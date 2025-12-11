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

Todo (Detailed, Feature-by-Feature) (please mark as completed when finished with the todo-task)
-----------------------------------
Foundations
- [ ] Add `Value::Mesh` and `MeshQuality` to `graph/value.rs`; update `components/coerce.rs` for mesh inputs/outputs; keep `Value::Surface` shim.
- [ ] Scaffold `src/geom/` with core types (`Point3`, `Vec3`, `BBox`, `Tolerance`, `Transform`) and traits (`Curve3`, `Surface`, `Solid`, `MeshBuilder`).
- [ ] Centralize tolerances (absolute/relative) and epsilon-safe comparisons used across geometry modules.
- [ ] Normalize units/handedness at ingestion; ensure tolerances are unit-aware.
- [ ] Add caching layer for tessellation/triangulation results and perf instrumentation hooks (timers/counters).
- [ ] Ensure instancing/hierarchy support (shared mesh buffers + transform stacks).

Curve Tessellation
- [ ] Implement evaluators for line, polyline, arc, circle, ellipse, Bezier, B-spline, NURBS curves with tangent/normal access.
- [ ] Add adaptive subdivision by curvature/flatness and arc-length parameterization; expose as polyline plus diagnostics.
- [ ] Bridge to existing `curve_util.rs` and `curve_analysis.rs` so components reuse the new tessellator.
- [ ] Tests: curvature-driven subdivision limits, small/large scale, closed/open curves.

Surface Tessellation
- [ ] Implement surface evaluators (plane, cylinder, cone, sphere, torus, NURBS) with trimming loops.
- [ ] Add adaptive (u,v) grid refinement driven by curvature/error; handle seams and orientation.
- [ ] Integrate with `surface_util.rs` and `surface_analysis.rs` for downstream components.
- [ ] Tests: trimmed patches, seams, singularities (poles), tolerance adherence.

Triangulation, Welding, and Repair
- [ ] Extend/replace `mesh_triangulation.rs` with constrained triangulation (support holes, trimming boundaries).
- [ ] Add welding/vertex merge with tolerance, inverted normal detection/correction, skinny triangle culling.
- [ ] Add manifold/watertight checks in `mesh_analysis.rs` with diagnostics struct.
- [ ] Generate UVs/tangents where parametrization exists; propagate per-face groups/material ids to downstream renderers.
- [ ] Add mesh ingest/repair adapters from legacy `Value::Surface`/meshes into `Value::Mesh` with weld/repair hooks.
- [ ] Tests: hole handling, open-edge detection, normal consistency, weld accuracy, UV/group propagation.

Feature Builders (one-by-one)
- [ ] Add geom feature modules: `geom/loft.rs`, `geom/sweep.rs`, `geom/revolve.rs`, `geom/pipe.rs`, `geom/patch.rs`, `geom/extrusion.rs` to hold geometry logic (components stay thin).
- [ ] Extrude Linear: refactor `components/extrude.rs` to call `geom::extrusion` for surfaces then mesh caps/sides, respect quality knobs.
- [ ] Extrude Along/Angled/Point: reuse curve normals/frames; handle taper and orientation; emit `Value::Mesh`.
- [ ] Revolve/RailRevolution: build revolved surfaces with seam handling; cap closing; mesh via unified path.
- [ ] Loft/FitLoft/ControlPointLoft: use `geom::loft` with adaptive sections; options (tight/loose, rebuild); mesh after surface build.
- [ ] Sweep1/Sweep2: use `geom::sweep` for stable frames, twist control, rail continuity; mesh with caps.
- [ ] Pipe/PipeVariable: use `geom::pipe` for variable radius sampling, junction handling, end caps; tolerance-aware self-intersection guard.
- [ ] FourPointSurface/EdgeSurface/RuledSurface/NetworkSurface/SumSurface: refactor to use surface evaluators then mesh.
- [ ] Patch/FragmentPatch/BoundarySurfaces: use `geom::patch` trimming + constrained triangulation; ensure G0/G1 options where possible.
- [ ] SurfaceFromPoints: fit grid/poisson-like surface, then mesh; provide diagnostics on fit quality.
- [ ] Loft Options component: add mesh quality/tolerance pins and plumb to mesher.
- [ ] Offset/Thicken: add shelling for surfaces/meshes (inside/outside) via robust offsets and remesh.
- [ ] Displacement: apply height/scalar fields pre-mesh; re-weld and recompute normals.
- [ ] Subdivision/quad: bridge `surface_subd` to the new mesher; optional quad-friendly output where applicable.

Deformations and Morphs
- [ ] Implement deformation fields (twist, bend, taper, morph) applied pre-mesh; recompute normals and weld after deformation.
- [ ] Add parameter inputs to relevant components; ensure deterministic frame handling.
- [ ] Tests: deformation extremes, preservation of manifoldness.

Booleans/CSG
- [ ] Create `src/geom/boolean.rs` with intersection, classification, and remeshing of intersection bands.
- [ ] Add boolean components (union/intersect/difference) using mesh kernel; include diagnostics for fallback/repairs.
- [ ] Tests: coplanar overlaps, near-coincident faces, nested solids, tiny features.

Fillet/Chamfer/Blend
- [ ] Implement offset surfaces/curves, intersection of offsets, and transitional surface tessellation (G1/G2 where available).
- [ ] Add fillet/chamfer components with radius/limit options; fallback handling for tight radii.
- [ ] Tests: edge/vertex blends, small-radius failures, continuity checks.

Diagnostics and Visualization
- [ ] Extend `display_preview.rs` to visualize open edges, self-intersections, and degenerate triangles; add optional overlays.
- [ ] Standardize `MeshDiagnostics` struct returned alongside meshes for logging and UI.

Performance/WASM
- [ ] Add BVH acceleration for intersection queries; gate rayon for native, single-thread for WASM.
- [ ] Optimize mesh buffer layout (SoA) and pooling to reduce allocations in hot paths.
- [ ] Add caching/invalidations for tessellation/triangulation; reuse buffers where possible.
- [ ] Add optional simplification/LOD (edge-collapse) with watertightness guard for previews/exports.
- [ ] Add timing/profiling hooks around tessellation/boolean/repair to track regressions (native and WASM).

Testing and Tooling
- [ ] Build regression corpus in `tests/` with golden meshes for each feature (extrude, loft, sweep1/2, pipe, revolve, boolean, fillet).
- [ ] Add property tests for manifoldness/watertightness and fuzzing for booleans and trimming.
- [ ] Wire debug logging flags and panic hooks for WASM; add CLI/sample scripts to generate and validate meshes.
- [ ] Add render/golden snapshots (three.js) for UV/material group propagation, LOD/simplification, and offset/displacement cases.
- [ ] Create and maintain a documentation wiki: per geom module (core/tessellation/triangulation/boolean/loft/sweep/revolve/pipe/patch/extrusion/offset/displacement/subdivision/simplify/cache) with API docs, code snippets, and component integration examples (including three.js adapter snippets).



