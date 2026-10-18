//! Build script: embeds the program icon and version metadata into the
//! Windows executable. On other targets it does nothing.
//!
//! The icon is rendered by the same code that draws the runtime window icon
//! (`src/icon_pixels.rs`), so there is no binary image asset in the repo.

#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::PathBuf;

include!("src/icon_pixels.rs");

const ICON_SIZES: [u32; 6] = [16, 32, 48, 64, 128, 256];

fn main() {
    println!("cargo:rerun-if-changed=src/icon_pixels.rs");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let Ok(out_dir) = env::var("OUT_DIR") else {
        fail("OUT_DIR is not set");
    };
    let icon_path = PathBuf::from(out_dir).join("rustcalc.ico");
    if let Err(error) = fs::write(&icon_path, render_ico()) {
        fail(&format!("failed to write {icon_path:?}: {error}"));
    }

    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_owned());
    let mut compiler = winresource::WindowsResource::new();
    compiler
        .set_icon(&icon_path.to_string_lossy())
        .set("ProductName", "RustCalc")
        .set("FileDescription", "RustCalc — a portable calculator")
        .set("FileVersion", &format!("{version}.0"))
        .set("ProductVersion", &format!("{version}.0"))
        .set("OriginalFilename", "rustcalc.exe")
        .set("LegalCopyright", "MIT-licensed open source");
    if let Err(error) = compiler.compile() {
        fail(&format!("failed to embed Windows resources: {error}"));
    }
}

/// Report a build failure without panicking.
fn fail(message: &str) -> ! {
    eprintln!("error: rustcalc-app build script: {message}");
    std::process::exit(1);
}

/// Render a multi-size `.ico` file with 32-bit BMP entries.
fn render_ico() -> Vec<u8> {
    let images: Vec<Vec<u8>> = ICON_SIZES.iter().map(|&s| render_ico_image(s)).collect();

    let mut ico = Vec::new();
    ico.extend_from_slice(&0_u16.to_le_bytes()); // reserved
    ico.extend_from_slice(&1_u16.to_le_bytes()); // resource type: icon
    ico.extend_from_slice(&(ICON_SIZES.len() as u16).to_le_bytes());

    let mut offset = 6 + 16 * ICON_SIZES.len() as u32;
    for (&size, image) in ICON_SIZES.iter().zip(&images) {
        ico.push(if size >= 256 { 0 } else { size as u8 });
        ico.push(if size >= 256 { 0 } else { size as u8 });
        ico.push(0); // palette colors
        ico.push(0); // reserved
        ico.extend_from_slice(&1_u16.to_le_bytes()); // color planes
        ico.extend_from_slice(&32_u16.to_le_bytes()); // bits per pixel
        ico.extend_from_slice(&(image.len() as u32).to_le_bytes());
        ico.extend_from_slice(&offset.to_le_bytes());
        offset += image.len() as u32;
    }
    for image in &images {
        ico.extend_from_slice(image);
    }
    ico
}

/// One `.ico` entry: BITMAPINFOHEADER + bottom-up BGRA pixels + AND mask.
fn render_ico_image(size: u32) -> Vec<u8> {
    let rgba = render_rgba(size);
    let mut image = Vec::new();

    image.extend_from_slice(&40_u32.to_le_bytes()); // header size
    image.extend_from_slice(&(size as i32).to_le_bytes()); // width
    image.extend_from_slice(&(2 * size as i32).to_le_bytes()); // height, doubled for mask
    image.extend_from_slice(&1_u16.to_le_bytes()); // planes
    image.extend_from_slice(&32_u16.to_le_bytes()); // bits per pixel
    image.extend_from_slice(&0_u32.to_le_bytes()); // compression: BI_RGB
    image.extend_from_slice(&(size * size * 4).to_le_bytes()); // image size
    for _ in 0..4 {
        image.extend_from_slice(&0_u32.to_le_bytes()); // pels/metre, palette sizes
    }

    for y in (0..size).rev() {
        for x in 0..size {
            let index = ((y * size + x) * 4) as usize;
            image.push(rgba[index + 2]); // blue
            image.push(rgba[index + 1]); // green
            image.push(rgba[index]); // red
            image.push(rgba[index + 3]); // alpha
        }
    }

    // AND mask: alpha channel does the work, so every bit is zero.
    let mask_row_bytes = size.div_ceil(32) * 4;
    image.resize(image.len() + (mask_row_bytes * size) as usize, 0);
    image
}
