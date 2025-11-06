# Repository Guidelines

## Project Structure & Module Organization
`alpha/ghx-engine/` hosts the Rust WASM core, split into `components/`, `graph/`, and `parse/` modules. Its artifacts land in `alpha/web/`, a Three.js shell with shaders, fixtures, and the wasm-pack `pkg/` output. Prototypes and reference GHX flows remain in `poc-ghx-three/` and support the worker helpers exercised in `tests/workers/`. Shared node metadata sits in `nodelist/`, while plans and research are tracked in root Markdown files and `alpha/docs/`.

## Build, Test, and Development Commands
- `npm install && npm run build:wasm` (inside `alpha/web/`): builds the engine to WebAssembly with debug logging enabled.
- `npm run dev` (inside `alpha/web/`): rebuilds the WASM module and serves the static viewer on `http://localhost:8080/`.
- `npm test`: runs `node --test` against the worker-protocol suite under `tests/workers/`.
- `cargo fmt && cargo clippy --all-targets` and `cargo test -p ghx-engine`: format, lint, and test the Rust crate before exporting to WASM.

## Coding Style & Naming Conventions
Rust code targets Edition 2024 defaults—keep modules cohesive, favor explicit types, and ensure `cargo fmt` produces a clean diff. JavaScript is ESM with 2-space indentation; use PascalCase for classes, camelCase for helpers, and freeze enums like `WorkerMessageType` once exported. JSON assets in `nodelist/` use snake_case keys and string node identifiers; mirror that structure when introducing new component metadata.

## Testing Guidelines
Extend `*.test.mjs` files whenever worker payloads, slider handling, or GHX parsing contracts change, covering both validation and normalization paths with `assert` helpers. Rust additions need unit tests either alongside the module or in `alpha/ghx-engine/tests/`, ideally with minimal GHX fixtures that exercise edge cases. Ship only after `npm test` and `cargo test` both pass locally (or in CI) to ensure the Node workers and WASM core stay in sync.

## Commit & Pull Request Guidelines
Git history favors concise, imperative subjects with optional scopes (e.g., `Fix(components): Correct output values for Panel and Unit Z`, `feat(ui): Toon uitvoerwaarden in de nodelijst`). Follow that convention, reference issues when relevant, and keep each commit reviewable on its own. Always begin a task by running `git pull` to sync with the remote, and once the task is complete run `git push` so the latest changes are published. Pull requests should outline the problem, summarize the solution, list commands executed, and attach screenshots or GHX diffs when UI or geometry output changes.

## Security & Configuration Tips
GHX fixtures may contain proprietary geometry—scrub identifying metadata or use anonymized samples before committing. Keep wasm-pack and Cargo dependencies current via isolated `cargo update` PRs, and avoid committing credentials or machine-specific paths by relying on `.env` files that stay untracked.
