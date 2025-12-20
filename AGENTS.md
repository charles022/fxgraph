# Repository Guidelines

## Project Structure & Module Organization
- `server/` is the Rust Axum WebSocket server; entry point is `server/src/main.rs`.
- `client/` is the Rust/WASM client built with `wasm-bindgen`; entry point is `client/src/lib.rs`.
- `www/` is the Vite dev shell that loads the WASM bundle and HTML/CSS (`www/index.html`, `www/vite.config.js`).
- Root docs and notes live alongside `README.md`, `GEMINI.md`, `notes_implementation.md`, and `DataTransfer*.md`. The file `manual_arrowrs_examples.rs` contains standalone examples.

## Build, Test, and Development Commands
- `./run_dev.sh` builds the WASM client, installs `www/` dependencies, starts the server, then runs the Vite dev server.
- `cargo run -p fxgraph-server` runs the server only (binds `127.0.0.1:3000`).
- `wasm-pack build --target web` builds the WASM package in `client/`.
- `npm install` then `npm run dev` inside `www/` starts the frontend locally.
- `npm run build` inside `www/` produces a production bundle.
- `cargo build` builds the workspace; `cargo test` runs Rust tests (none exist yet).

## Coding Style & Naming Conventions
- Rust uses standard `rustfmt` style (`cargo fmt`) with `snake_case` for functions/modules and `PascalCase` for types.
- Keep the WASM boundary small in `client/src/lib.rs`; prefer Rust-side logic over JavaScript.
- HTML/CSS in `www/index.html` uses 2-space indentation and `kebab-case` DOM IDs like `fxgraph-canvas`.

## Testing Guidelines
- There is no automated test suite yet. If you add tests, use Rust unit tests in `mod tests` or integration tests under `server/tests` and `client/tests`.
- Run tests from the repo root with `cargo test` and document any manual verification in your PR.

## Commit & Pull Request Guidelines
- Commit messages in history are short, sentence-style descriptions without conventional prefixes; keep them specific and action-oriented.
- PRs should include: a brief summary, commands run (if any), and manual verification steps. Include screenshots or gifs for UI changes in `www/`.
