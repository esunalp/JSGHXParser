# Repository Guidelines

## Project Structure & Module Organization
- `alpha/ghx-engine/`: Rust 2024 crate that parses and evaluates GHX, exported to WASM; internal modules live under `src/`, with integration tests in `tests/`.
- `alpha/web/`: Three.js viewer that consumes the WASM `pkg/` output; GHX fixtures and shaders sit here for quick manual runs.
- `poc-ghx-three/`: JavaScript prototype of the worker pipeline plus GHX samples and rendering helpers that mirror the alpha viewer.
- `tests/workers/`: Node test suite covering the worker protocol (`*.test.mjs`).
- `nodelist/`, `tools/ghx-samples/`, and `alpha/docs/`: shared metadata, minimal GHX fixtures, and design notes/plan documents.

## Build, Test, and Development Commands
- `npm test` (repo root): runs `node --test` for the worker protocol.
- `cd alpha/web && npm install && npm run build` (or `npm run dev`): builds the WASM package and serves the viewer on `http://localhost:8080/` via `python -m http.server`.
- `cd alpha/ghx-engine && cargo fmt && cargo clippy --all-targets && cargo test`: format, lint, and validate the Rust core before exporting to WASM.
- When changing WASM bindings, rerun `npm run build:wasm` in `alpha/web` to refresh `pkg/`.

## Coding Style & Naming Conventions
- JavaScript is ESM with 2-space indentation; prefer named exports, PascalCase classes, and camelCase helpers. Freeze constant maps/enums (e.g., `WorkerMessageType`) before export.
- Rust follows `cargo fmt` output, explicit error types via `thiserror`, and feature flags (`debug_logs`, `parallel`) for optional behavior. Keep module names snake_case and functions descriptive.
- GHX metadata and sample names favor lowercase with hyphens/underscores (`minimal_extrude.ghx`); mirror existing patterns when adding fixtures.

## Testing Guidelines
- Add or extend `tests/workers/*.test.mjs` when adjusting payload validation, slider normalization, or worker message shapes; cover both valid and rejected inputs using `assert`.
- Place Rust unit tests near the code and heavier scenarios in `alpha/ghx-engine/tests/`, reusing the smallest GHX fixtures that trigger the behavior.
- Run `npm test` plus `cargo test` before pushing to keep the JS protocol and WASM core aligned; include new fixtures in version control so tests stay deterministic.

## Commit & Pull Request Guidelines
- Recent history uses short, imperative Conventional Commit subjects with scopes (`feat(components): ...`, `fix(sets): ...`). Match that style and avoid multi-topic commits.
- In pull requests, describe the issue, the approach, and commands run; link related issues and attach viewer screenshots or GHX diffs when rendering output changes.
- Keep changes reviewable: prefer smaller PRs, note any follow-ups, and document any new feature flags or environment expectations.
