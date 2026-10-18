//! The window icon, drawn in code so there is no image asset to ship.

use eframe::egui::{Color32, IconData};

use crate::theme;

const SIZE: u32 = 64;

/// The RustCalc window icon: an amber equals sign on a dark rounded square.
pub fn window_icon() -> IconData {
    let mut rgba = vec![0_u8; (SIZE * SIZE * 4) as usize];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let background = coverage(sd_rounded_rect(px, py, 32.0, 32.0, 30.0, 30.0, 13.0));
            if background <= 0.0 {
                continue;
            }

            let top_bar = coverage(sd_rounded_rect(px, py, 32.0, 24.0, 15.0, 3.0, 3.0));
            let bottom_bar = coverage(sd_rounded_rect(px, py, 32.0, 40.0, 15.0, 3.0, 3.0));
            let color = mix(theme::ACCENT, theme::ACCENT_TEXT, top_bar.max(bottom_bar));

            let index = ((y * SIZE + x) * 4) as usize;
            rgba[index] = color.r();
            rgba[index + 1] = color.g();
            rgba[index + 2] = color.b();
            rgba[index + 3] = (background * 255.0).round() as u8;
        }
    }
    IconData {
        rgba,
        width: SIZE,
        height: SIZE,
    }
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

/// Linear blend between two colors; `t = 0` is `from`, `t = 1` is `to`.
fn mix(from: Color32, to: Color32, t: f32) -> Color32 {
    let lerp = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
    Color32::from_rgb(
        lerp(from.r(), to.r()),
        lerp(from.g(), to.g()),
        lerp(from.b(), to.b()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(icon: &IconData, x: u32, y: u32) -> [u8; 4] {
        let index = ((y * icon.width + x) * 4) as usize;
        [
            icon.rgba[index],
            icon.rgba[index + 1],
            icon.rgba[index + 2],
            icon.rgba[index + 3],
        ]
    }

    #[test]
    fn has_expected_dimensions() {
        let icon = window_icon();
        assert_eq!(icon.width, SIZE);
        assert_eq!(icon.height, SIZE);
        assert_eq!(icon.rgba.len(), (SIZE * SIZE * 4) as usize);
    }

    #[test]
    fn corners_are_transparent() {
        let icon = window_icon();
        assert_eq!(pixel(&icon, 0, 0)[3], 0);
        assert_eq!(pixel(&icon, SIZE - 1, SIZE - 1)[3], 0);
    }

    #[test]
    fn center_is_accent_between_the_bars() {
        let icon = window_icon();
        let [r, g, b, a] = pixel(&icon, 32, 32);
        assert_eq!(a, 255);
        assert_eq!(
            (r, g, b),
            (theme::ACCENT.r(), theme::ACCENT.g(), theme::ACCENT.b())
        );
    }

    #[test]
    fn equals_bars_are_dark() {
        let icon = window_icon();
        for y in [24, 40] {
            let [r, g, b, a] = pixel(&icon, 32, y);
            assert_eq!(a, 255, "bar at y={y} must be opaque");
            assert_eq!(
                (r, g, b),
                (
                    theme::ACCENT_TEXT.r(),
                    theme::ACCENT_TEXT.g(),
                    theme::ACCENT_TEXT.b()
                ),
                "bar at y={y} must use the dark text color"
            );
        }
    }

    #[test]
    fn body_is_opaque() {
        let icon = window_icon();
        assert_eq!(pixel(&icon, 4, 32)[3], 255);
        assert_eq!(pixel(&icon, 32, 4)[3], 255);
    }
}
