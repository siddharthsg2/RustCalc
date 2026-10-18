# AGENTS.md — RustCalc

Rules for AI agents and contributors working in this repository.

## Hard rules

- `#![forbid(unsafe_code)]` stays in every crate. Never remove it.
  Documented exception: `crates/rustcalc-android` cannot carry the attribute
  because Android looks up the exported `android_main` symbol, which Rust only
  produces via `#[unsafe(no_mangle)]`. No other unsafe code is allowed there.
- All business logic lives in `crates/rustcalc-core`. The GUI crates
  (`crates/rustcalc-app` and `crates/rustcalc-android`) are thin shells: they
  render state and forward input — no calculator logic there.
- No `.unwrap()`, `.expect()` or panics in production code (tests only).
  Return `Result`/`Option` and handle errors explicitly.
- Never invent dependency versions. Add dependencies with `cargo add <crate>`
  only, and keep the dependency count minimal.
- Do not add a dependency for something the standard library already does.

## Verification gates (run before any commit)

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

## Workflow

- Small, single-purpose commits with green tests.
- Every logic change ships with tests in the same commit.
- Core logic is verified with `cargo-mutants`: tests must actually kill
  mutants, not merely execute lines.
