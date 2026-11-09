//! The one font setup every component shares: the Spectrum family with fallbacks, and a
//! [`TextStyle`] built from a generated [`Typography`] token.
//!
//! The fonts are licensed and never bundled. Rune loads installed fonts by name on first use;
//! a family list falls through name by name and ends at Rune's default sans.

use crate::tokens::Typography;
use rune::{Family, TextStyle};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// What the tokens name (`default_font_family`, every `Typography::family`).
pub const SPECTRUM_VF: &str = "Adobe Clean Spectrum VF";

/// Tried in order: the variable font, the static Adobe Clean Spectrum styles, then Source Sans 3.
/// Source Sans 3 is only a width-compatible stand-in (the design-data CLI measures text with it);
/// it is not Spectrum's look.
pub const FAMILY_LIST: &str = "Adobe Clean Spectrum VF, Adobe Clean Spectrum, Source Sans 3";

const FALLBACKS: [&str; 2] = ["Adobe Clean Spectrum", "Source Sans 3"];

/// The token's family followed by the fallbacks. For today's token this is [`FAMILY_LIST`]; a
/// different token family is put first and the list built once (leaked, as `Family::Named`
/// needs a `'static` str).
pub fn family(token_family: &'static str) -> &'static str {
    if token_family == SPECTRUM_VF {
        return FAMILY_LIST;
    }
    static CACHE: OnceLock<Mutex<HashMap<&'static str, &'static str>>> = OnceLock::new();
    let mut cache = CACHE.get_or_init(Default::default).lock().unwrap();
    cache.entry(token_family).or_insert_with(|| {
        let mut names = vec![token_family];
        names.extend(FALLBACKS.iter().filter(|f| **f != token_family));
        Box::leak(names.join(", ").into_boxed_str())
    })
}

/// A text style from a typography token. Tracking is in points: the token's em times the size.
pub fn text_style(t: &Typography) -> TextStyle {
    TextStyle::new(t.size, f32::from(t.weight))
        .family(Family::Named(family(t.family)))
        .lh(t.line_height)
        .tracking(t.letter_spacing_em * t.size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune::text::measure;

    const SAMPLE: &str = "Spectrum Switch label 0123";

    fn typo(family: &'static str) -> Typography {
        Typography {
            family,
            size: 14.0,
            weight: 400,
            line_height: 18.0,
            letter_spacing_em: 0.05,
        }
    }

    fn width(family: Family) -> f32 {
        let style = text_style(&typo(SPECTRUM_VF)).family(family);
        measure(SAMPLE, &[], style, None).0
    }

    #[test]
    fn token_family_leads_the_list() {
        assert_eq!(
            family("Other Face"),
            "Other Face, Adobe Clean Spectrum, Source Sans 3"
        );
        assert_eq!(
            family("Adobe Clean Spectrum"),
            "Adobe Clean Spectrum, Source Sans 3"
        );
    }

    #[test]
    fn style_copies_the_token() {
        let s = text_style(&typo(SPECTRUM_VF));
        assert_eq!((s.size, s.weight, s.line_height), (14.0, 400.0, 18.0));
        assert!((s.tracking - 0.7).abs() < 1e-6, "em converted to points");
        assert_eq!(s.family, Family::Named(FAMILY_LIST));
    }

    /// Runs anywhere: names that are not installed fall back to Rune's sans.
    #[test]
    fn missing_families_fall_back_to_sans() {
        let missing = Family::Named("No Such Face A, No Such Face B");
        assert_eq!(width(missing), width(Family::Sans));
    }

    /// Needs the licensed fonts installed; CI has none. Run: `cargo test -p spectrum-rune -- --ignored`
    #[test]
    #[ignore = "needs Adobe Clean Spectrum VF installed"]
    fn vf_is_used_when_installed() {
        let listed = width(Family::Named(FAMILY_LIST));
        let vf_only = width(Family::Named(SPECTRUM_VF));
        assert_eq!(listed, vf_only, "the list resolves to the VF");
        assert_ne!(
            listed,
            width(Family::Sans),
            "and it is not the fallback sans"
        );
    }

    /// Needs the static Adobe Clean Spectrum installed. Run with `--ignored`.
    #[test]
    #[ignore = "needs Adobe Clean Spectrum installed"]
    fn list_works_when_first_name_is_missing() {
        let list = Family::Named("No Such Face, Adobe Clean Spectrum, Source Sans 3");
        let w = width(list);
        assert_eq!(w, width(Family::Named("Adobe Clean Spectrum")));
        assert_ne!(w, width(Family::Sans));
    }
}
