//! Every Spectrum component in Rune, one story per component, with live colour scheme and scale.
//!
//! ```sh
//! cargo run -p spectrum-rune --example gallery
//! cargo run -p spectrum-rune --example gallery -- --scene button --theme dark --snap /tmp/button-dark.png
//! cargo run -p spectrum-rune --example gallery -- --list-scenes
//! ```
//!
//! The page is Spectrum's `background_base_color`. The licensed Adobe Clean Spectrum font is read
//! from the system, never bundled; the header and the startup output say whether it was found.

mod stories;

use rune::text::measure;
use rune::{col, row, text, AppSpec, Cx, El, Family, ScrollSpec};
use spectrum_rune::action_button::ActionButton;
use spectrum_rune::font;
use spectrum_rune::switch::Switch;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontStatus {
    Vf,
    Static,
    Fallback,
}

impl FontStatus {
    fn describe(self) -> &'static str {
        match self {
            FontStatus::Vf => "Adobe Clean Spectrum VF found",
            FontStatus::Static => "Adobe Clean Spectrum (static) found",
            FontStatus::Fallback => {
                "Adobe Clean Spectrum NOT found: showing the fallback (Source Sans 3 or Rune's sans), not Spectrum's look"
            }
        }
    }
}

/// A named family that is not installed measures like Rune's default sans, so a width that
/// differs from it means the face resolved (the approach of the ignored tests in `font.rs`).
fn detect_font() -> FontStatus {
    let t = tokens(ColorScheme::Light, Scale::Desktop);
    let width = |f: Family| {
        let style = font::text_style(&t.component_m_regular).family(f);
        measure("Spectrum Switch label 0123", &[], style, None).0
    };
    let sans = width(Family::Sans);
    if width(Family::Named(font::SPECTRUM_VF)) != sans {
        FontStatus::Vf
    } else if width(Family::Named("Adobe Clean Spectrum")) != sans {
        FontStatus::Static
    } else {
        FontStatus::Fallback
    }
}

fn font_status() -> FontStatus {
    static S: OnceLock<FontStatus> = OnceLock::new();
    *S.get_or_init(detect_font)
}

pub struct G {
    pub story: &'static str,
    /// `None` follows the system (`--theme`) until the header switch is used.
    pub scheme: Option<ColorScheme>,
    pub scale: Scale,
    /// Keys whose state differs from the story's default (toggles).
    pub flipped: HashSet<String>,
    /// The chosen radio per group.
    pub picked: HashMap<String, String>,
    /// The open items per accordion group.
    pub open: HashMap<String, Vec<String>>,
}

impl G {
    /// A toggle's state: its default, unless flipped.
    pub fn on(&self, key: &str, default: bool) -> bool {
        default ^ self.flipped.contains(key)
    }
    pub fn flip(&mut self, key: &str) {
        if !self.flipped.remove(key) {
            self.flipped.insert(key.to_owned());
        }
    }
}

fn init() -> G {
    G {
        story: stories::STORIES[0].0,
        scheme: None,
        scale: Scale::Desktop,
        flipped: HashSet::new(),
        picked: HashMap::new(),
        open: HashMap::new(),
    }
}

fn view(s: &G, cx: &mut Cx<G>) -> El {
    let scheme = s.scheme.unwrap_or(if cx.system_dark() {
        ColorScheme::Dark
    } else {
        ColorScheme::Light
    });
    build(s, scheme)
}

fn build(s: &G, scheme: ColorScheme) -> El {
    let t = tokens(scheme, s.scale);
    let dark = scheme == ColorScheme::Dark;
    let mobile = s.scale == Scale::Mobile;

    let nav = row()
        .gap(4.0)
        .flex_wrap()
        .children(stories::STORIES.iter().map(|(id, title, _)| {
            let id: &'static str = id;
            ActionButton::new(format!("nav/{id}"))
                .label(*title)
                .quiet(true)
                .selected(s.story == id)
                .on_press(move |g: &mut G| g.story = id)
                .build(t)
        }));
    let controls = row()
        .gap(24.0)
        .center_y()
        .flex_wrap()
        .child(
            text("Spectrum for Rune")
                .font(font::text_style(&t.component_m_bold))
                .color(t.neutral_content_color_default),
        )
        .child(
            Switch::new("ctl/dark")
                .label("Dark")
                .selected(dark)
                .on_toggle(move |g: &mut G| {
                    g.scheme = Some(if dark {
                        ColorScheme::Light
                    } else {
                        ColorScheme::Dark
                    })
                })
                .build(t),
        )
        .child(
            Switch::new("ctl/mobile")
                .label("Mobile scale")
                .selected(mobile)
                .on_toggle(move |g: &mut G| {
                    g.scale = if mobile {
                        Scale::Desktop
                    } else {
                        Scale::Mobile
                    }
                })
                .build(t),
        )
        .child(
            text(font_status().describe())
                .font(font::text_style(&t.component_s_regular))
                .color(t.neutral_subdued_content_color_default),
        );
    let story = stories::STORIES
        .iter()
        .find(|(id, ..)| *id == s.story)
        .map(|(.., f)| *f);
    let body = story.map_or_else(rune::el, |f| f(s, t));

    col()
        .key("gallery")
        .w_full()
        .h_full()
        .fill(t.background_base_color)
        .child(
            col()
                .key("gallery/header")
                .w_full()
                .padding(16.0)
                .gap(12.0)
                .child(controls)
                .child(nav),
        )
        .child(
            col()
                .key("gallery/body")
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .padding(16.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(body),
        )
}

fn main() {
    eprintln!("spectrum-rune gallery: {}", font_status().describe());
    rune::run(AppSpec {
        scenes: stories::SCENES,
        ..AppSpec::new("Spectrum for Rune", (1100.0, 900.0), init, view)
    });
}

#[cfg(test)]
#[path = "../../tests/support/pixels.rs"]
mod pixels;

#[cfg(test)]
mod tests {
    use super::*;
    use rune::{Env, Runtime};

    /// Renders every scene with Rune's CPU reference renderer (no GPU) to `target/gallery/*.png`,
    /// which CI uploads. The font is whatever the machine has, so CI shows the fallback; Rune's
    /// GPU output differs slightly (see `support/pixels.rs`).
    #[test]
    #[ignore = "slow unoptimised; run with --release -- --ignored"]
    fn render_scenes_to_png() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/gallery");
        std::fs::create_dir_all(&dir).unwrap();
        let scene = |id: &str, h: f32| {
            let (_, setup) = stories::SCENES.iter().find(|(i, _)| *i == id).unwrap();
            let mut s = init();
            setup(&mut s);
            let mut rt = Runtime::new(s, view, (1100.0, h), Env::new(false, 1.0));
            rt.frame(0.05);
            rt
        };
        for (id, _) in stories::SCENES {
            // Lay out on a tall page, then render at the content's height (plus the body padding).
            let bottom = scene(id, 6000.0)
                .elements()
                .iter()
                .filter(|e| !matches!(e.key.as_deref(), Some("gallery" | "gallery/body")))
                .map(|e| e.rect[1] + e.rect[3])
                .fold(0.0_f32, f32::max);
            assert!(
                bottom < 5000.0,
                "{id}: measured the whole page, was a key renamed?"
            );
            let h = (bottom.ceil() + 16.0).max(300.0) as u32;
            let rt = scene(id, h as f32);
            crate::pixels::Pixels::of(&rt, (1100, h)).write_png(&dir.join(format!("{id}.png")));
        }
    }

    /// Every story builds on both schemes and scales (no GPU, no font). Components `debug_assert`
    /// their `constraints::check` errors, so an excluded combination fails here too.
    #[test]
    fn every_story_builds() {
        for scheme in [ColorScheme::Light, ColorScheme::Dark] {
            for scale in [Scale::Desktop, Scale::Mobile] {
                for (id, setup) in stories::SCENES {
                    let mut s = init();
                    setup(&mut s);
                    s.scale = scale;
                    let dark = scheme == ColorScheme::Dark;
                    s.scheme = Some(scheme);
                    let story = s.story;
                    let mut rt = Runtime::new(s, view, (1100.0, 900.0), Env::new(dark, 1.0));
                    rt.frame(0.05);
                    let prefix = format!("{story}/");
                    let keys: Vec<_> = rt
                        .elements()
                        .into_iter()
                        .filter(|e| !e.exiting)
                        .filter_map(|e| e.key)
                        .filter(|k| k.starts_with(&prefix))
                        .collect();
                    let n = keys.len();
                    let unique: std::collections::HashSet<_> = keys.iter().collect();
                    assert_eq!(unique.len(), n, "{id}: duplicate keys (shared Interaction)");
                    assert!(n > 0, "{id}: the story built no elements");
                }
            }
        }
    }

    #[test]
    fn scenes_cover_every_story() {
        assert_eq!(stories::SCENES.len(), stories::STORIES.len() * 3);
    }

    #[test]
    fn switch_toggles_on_click() {
        let mut s = init();
        s.story = "switch";
        let mut rt = Runtime::new(s, view, (1100.0, 900.0), Env::new(false, 1.0));
        rt.frame(0.05);
        let (x, y) = rt.find("switch/off/container").unwrap();
        rt.pointer_move(x, y);
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(0.05);
        assert!(rt.state.flipped.contains("switch/off"));
    }
}
