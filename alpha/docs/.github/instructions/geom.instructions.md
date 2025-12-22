---
applyTo: "alpha/ghx-engine/**"
---

# Geom Mesh Engine (ghx-engine) — Agent Instructions

These instructions guide AI-agent development for the `geom` mesh engine work described in [JSGHXParser/alpha/docs/mesh_engine_integration_plan.md](../../JSGHXParser/alpha/docs/mesh_engine_integration_plan.md).

## Mission
Build a single, uniform meshing pipeline under `alpha/ghx-engine/src/geom/` that ingests CAD primitives + feature operations and produces robust, watertight, manifold triangle meshes with controllable quality, good diagnostics, and WASM-friendly data/layout.

## Non‑negotiables
- Keep components thin: **components only coerce inputs + call `geom::*` + return Values**. No geometry algorithms in `src/components/*`.
- Preserve backward compatibility: keep `Value::Surface` adapters where needed while preferring `Value::Mesh` once Phase 3 begins.
- Preserve Grasshopper semantics: pin order + GUIDs must not change; new pins are **optional and append-only**.
- Respect existing math utilities: reuse `maths_*`, `vector_*`, `transform_*`, `sets_*` instead of duplicating.
- Keep it minimal: simplest correct implementation; avoid unnecessary abstractions/dependencies.
- Don’t refactor/reformat unrelated code.

## Phase rules (from the integration plan)
- **Phase 1 and Phase 2 are “geom-only”**:
  - Allowed edits: `alpha/ghx-engine/src/geom/**`, `alpha/ghx-engine/Cargo.toml`, `alpha/ghx-engine/src/lib.rs`, and docs/tests explicitly under `geom` as outlined.
  - Not allowed: edits to `alpha/ghx-engine/src/components/*` or `alpha/ghx-engine/src/graph/*`.
  - Feature gating: everything new stays behind `mesh_engine_next` (not in default features).
  - Gate to proceed: WASM build passes with `--target wasm32-unknown-unknown`.
- **Phase 3 integrates into `graph` and `components`** only after the gate passes.

## Architectural expectations
- New kernel lives in `alpha/ghx-engine/src/geom/` with clear module boundaries:
  - Core types: `Point3`, `Vec3`, `Transform`, `BBox`, `Tolerance`.
  - Evaluators: `Curve3`, `Surface` + adaptive tessellation.
  - Meshing: triangulation (incl. trims/holes), weld/repair, UV/normal generation, diagnostics.
  - Features implemented on geometry (not ad-hoc meshes): extrude/loft/sweep/revolve/pipe, later offset/deform/booleans.
- Mesh quality is explicit and configurable (edge length / deviation / angle thresholds).
- Diagnostics are first-class: open edges, non-manifold edges, degenerate tris, weld stats, boolean fallbacks, and timing buckets.

## API design guidance
- Prefer small, explicit entry points with predictable outputs:
  - Example shape (Phase 1/2): `geom::mesh::mesh_surface(...) -> (GeomMesh, GeomMeshDiagnostics)`
  - Example shape (Phase 3): return `Value::Mesh { vertices, faces, normals, uvs, diagnostics }` + legacy adapters.
- Favor WASM-friendly buffers:
  - indexed triangles (`positions` + `indices` mandatory)
  - `normals`/`uvs` optional
  - avoid reallocations; expose views/slices when bridging to web.
- Stable ordering matters: do not reorder unless necessary; if welding changes indices, provide stable remap and record it in diagnostics.

## Error handling
- Validate inputs early (empty lists, invalid domains, NaNs, self-intersection detection where applicable).
- Use clear, actionable error messages (what failed, why, how to adjust tolerance/quality).
- Avoid silent “fix-ups”: any snapping/repair/fallback must be recorded in diagnostics.

## Performance + WASM constraints
- Keep `rayon`/parallelism feature-gated and disabled on wasm32.
- Use BVH for intersection-heavy operations (booleans) when introduced; keep memory layout stream-friendly (SoA when helpful).
- Add caching/instrumentation hooks where the plan specifies; ensure caches are invalidation-safe.

## three.js / web compatibility
- Maintain compatibility with existing BufferGeometry creation:
  - positions + indices always emitted
  - normals/uvs optional; if absent, web layer may compute as today
- Don’t break the viewer contract; add diagnostics overlays only behind explicit flags.

## Testing + verification
- Phase 1/2: tests should live under `alpha/ghx-engine/src/geom/tests/**` and compile only with `mesh_engine_next`.
- Add tests for:
  - index bounds
  - winding consistency
  - watertightness/manifold checks (or strong proxies)
  - no NaNs/Infs
  - seam/pole handling
  - trims/holes triangulation
- Keep fixtures minimal and deterministic.
- The cargo test run can be done with this command in wsl: cd /mnt/c/Users/Erol/Documents/JSGHXParser/JSGHXParser/alpha/ghx-engine && cargo test --features mesh_engine_next

## Documentation expectations
- When you add/modify a `geom` module, also add:
  - a short API description
  - at least one usage snippet
  - a note on tolerances and diagnostics
- Prefer updating plan-adjacent docs under `JSGHXParser/alpha/docs/`.

## Working style
- Clarify requirements: restate task + assumptions before code changes.
- Think before coding: outline algorithm/data structures/edge cases.
- Show small examples and focused tests.
- Keep edits surgical and consistent with existing style.

---

## Memory (reusable facts/decisions)

Use this section as a lightweight, repo-local “memory bank” for future agent work. Keep entries short, stable, and actionable.

### Memory entries
- **Feature gate**: `mesh_engine_next` must remain non-default; Phase 1/2 edits stay in `src/geom/**` (+ `Cargo.toml`, `lib.rs`).
- **WASM gate**: must build with `--target wasm32-unknown-unknown` before touching `src/components/*` or `src/graph/*`.

(Add new entries below as decisions solidify.)

---

## Knowledge (snippets, patterns, gotchas)

Store small, copy-pastable snippets and “gotchas” here. Prefer durable guidance over transient debugging notes.

### Templates
- **Diagnostics reporting** (pattern):
  - Record *what changed* (weld count, dropped degenerate tris, relaxed tolerance, boolean fallback)
  - Record *why* (threshold exceeded, predicate instability, self-intersection)
  - Provide a *next action* (tighten/loosen `MeshQuality`, check inputs, enable debug flag)

### Snippets
- *(Add Rust snippets here as they stabilize: mesh payload structs, wasm-safe buffer views, welding helpers, etc.)*
