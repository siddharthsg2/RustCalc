# RustCalc

A fast, beautiful, cross-platform calculator built in Rust with egui.

Portable by design:

- **Windows** — single `rustcalc.exe`, no installer, no runtime dependencies
- **macOS** — universal `.dmg` (`x86_64` + `aarch64`)
- **Linux** — `.AppImage` and a plain binary tarball

## Status

Under construction.

## Layout

```
crates/
├── rustcalc-core/   pure calculator logic — zero GUI dependencies, fully tested
└── rustcalc-app/    egui GUI shell (thin)
```

## Development

```sh
cargo run -p rustcalc-app
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## License

MIT
