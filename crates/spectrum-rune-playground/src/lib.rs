//! The Spectrum library for [`rune_options_playground`]: the catalog constraints, adapters for the `spectrum-rune`
//! components, and the design-data oracle. `stories()` is what an app gives to `Playground::story`.

mod adapters;
mod library;
mod oracle;

use rune_options_playground::model::Env;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale, Tokens};

pub use library::Spectrum;

/// Spectrum's tokens for an environment.
pub fn tokens_of(env: Env) -> &'static Tokens {
    tokens(
        if env.dark {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        },
        if env.mobile {
            Scale::Mobile
        } else {
            Scale::Desktop
        },
    )
}

/// One story per Spectrum catalog component, for `rune::Playground::story`.
pub fn stories() -> Vec<rune::Story<rune_options_playground::Page>> {
    rune_options_playground::stories(&Spectrum)
}
