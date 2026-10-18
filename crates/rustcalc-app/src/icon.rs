//! The window icon, drawn in code so there is no image asset to ship.

use eframe::egui::IconData;

use crate::icon_pixels;

const SIZE: u32 = 64;

/// The RustCalc window icon: an amber equals sign on a dark rounded square.
pub fn window_icon() -> IconData {
    IconData {
        rgba: icon_pixels::render_rgba(SIZE),
        width: SIZE,
        height: SIZE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme;

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

    #[test]
    fn scales_to_small_sizes() {
        let icon = icon_pixels::render_rgba(16);
        assert_eq!(icon.len(), 16 * 16 * 4);
        let center = ((8 * 16 + 8) * 4) as usize;
        assert_eq!(icon[center + 3], 255);
    }
}
