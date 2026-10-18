// Dependency-free icon renderer shared between the runtime window icon
// (`src/icon.rs`) and the build script that embeds the Windows `.exe` icon.
//
// This file is included by `build.rs`, so it must not depend on any crate
// and must not use inner doc comments.

/// Accent color: amber (must match `theme::ACCENT`).
pub const ACCENT: [u8; 3] = [0xF2, 0xA9, 0x3B];
/// Text color drawn on top of the accent color (must match `theme::ACCENT_TEXT`).
pub const ACCENT_TEXT: [u8; 3] = [0x1B, 0x1E, 0x24];

/// Render the RustCalc icon ("=" on a rounded square) as RGBA bytes.
///
/// The reference design is 64×64; every size scales it proportionally.
pub fn render_rgba(size: u32) -> Vec<u8> {
    let scale = size as f32 / 64.0;
    let half = size as f32 / 2.0;
    let mut rgba = vec![0_u8; (size * size * 4) as usize];

    for y in 0..size {
        for x in 0..size {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let radius = 13.0 * scale;
            let background = coverage(sd_rounded_rect(
                px,
                py,
                half,
                half,
                half - 2.0 * scale,
                half - 2.0 * scale,
                radius,
            ));
            if background <= 0.0 {
                continue;
            }

            let top_bar = coverage(sd_rounded_rect(
                px,
                py,
                half,
                24.0 * scale,
                15.0 * scale,
                3.0 * scale,
                3.0 * scale,
            ));
            let bottom_bar = coverage(sd_rounded_rect(
                px,
                py,
                half,
                40.0 * scale,
                15.0 * scale,
                3.0 * scale,
                3.0 * scale,
            ));
            let color = mix(ACCENT, ACCENT_TEXT, top_bar.max(bottom_bar));

            let index = ((y * size + x) * 4) as usize;
            rgba[index] = color[0];
            rgba[index + 1] = color[1];
            rgba[index + 2] = color[2];
            rgba[index + 3] = (background * 255.0).round() as u8;
        }
    }
    rgba
}

/// Signed distance from a point to a rounded rectangle.
fn sd_rounded_rect(
    px: f32,
    py: f32,
    cx: f32,
    cy: f32,
    half_w: f32,
    half_h: f32,
    radius: f32,
) -> f32 {
    let qx = (px - cx).abs() - (half_w - radius);
    let qy = (py - cy).abs() - (half_h - radius);
    let outside_x = qx.max(0.0);
    let outside_y = qy.max(0.0);
    (outside_x * outside_x + outside_y * outside_y).sqrt() + qx.max(qy).min(0.0) - radius
}

/// Antialiased coverage in `[0, 1]` for a pixel at `distance` from the shape.
fn coverage(distance: f32) -> f32 {
    (0.5 - distance).clamp(0.0, 1.0)
}

/// Linear blend between two RGB colors; `t = 0` is `from`, `t = 1` is `to`.
fn mix(from: [u8; 3], to: [u8; 3], t: f32) -> [u8; 3] {
    let lerp = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
    [
        lerp(from[0], to[0]),
        lerp(from[1], to[1]),
        lerp(from[2], to[2]),
    ]
}
