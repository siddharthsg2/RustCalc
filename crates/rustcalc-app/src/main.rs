#![forbid(unsafe_code)]
//! RustCalc desktop application entry point.

fn main() {
    println!("{} v{}", rustcalc_core::APP_NAME, env!("CARGO_PKG_VERSION"));
}
