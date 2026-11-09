//! Spectrum tokens as plain Rust data. `generated.rs` comes from `cargo xtask gen`.

mod generated;

pub use generated::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorScheme {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scale {
    Desktop,
    Mobile,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Typography {
    pub family: &'static str,
    pub size: f32,
    pub weight: u16,
    pub line_height: f32,
    pub letter_spacing_em: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub x: f32,
    pub y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: rune::Color,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: rune::Color, b: rune::Color) -> bool {
        [(a.r, b.r), (a.g, b.g), (a.b, b.b), (a.a, b.a)]
            .iter()
            .all(|(x, y)| (x - y).abs() < 1e-5)
    }

    #[test]
    fn colors_match_rune_hex() {
        let light = tokens(ColorScheme::Light, Scale::Desktop);
        let dark = tokens(ColorScheme::Dark, Scale::Desktop);
        assert!(close(
            light.accent_background_color_default,
            rune::color::hex("#3b63fb").unwrap()
        ));
        assert!(close(
            dark.accent_background_color_default,
            rune::color::hex("#4069fd").unwrap()
        ));
    }

    #[test]
    fn scale_changes_sizes() {
        assert_eq!(
            tokens(ColorScheme::Light, Scale::Desktop).component_height_100,
            32.0
        );
        assert_ne!(
            tokens(ColorScheme::Light, Scale::Mobile).component_height_100,
            32.0
        );
    }

    #[test]
    fn recipe_scoped_values_are_present_per_scale() {
        assert_eq!(
            tokens(ColorScheme::Light, Scale::Desktop).switch_control_width_medium,
            26.0
        );
        assert_eq!(
            tokens(ColorScheme::Light, Scale::Mobile).switch_control_width_medium,
            34.0
        );
        assert_eq!(
            tokens(ColorScheme::Light, Scale::Mobile).chevron_size_100,
            14.0
        );
    }

    #[test]
    fn typography_and_lookup() {
        let t = tokens(ColorScheme::Dark, Scale::Mobile);
        assert_eq!(t.component_m_regular.family, "Adobe Clean Spectrum VF");
        assert!(std::ptr::eq(t, &DARK_MOBILE));
    }
}
