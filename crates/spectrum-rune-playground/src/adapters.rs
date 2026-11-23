//! The previewed Spectrum components: what each adds to the catalog (free-form presets), how the chosen values
//! build it, and what it passes to the oracle besides its catalog options.

use crate::tokens_of;
use rune::{col, El};
use rune_options_playground::adapters::{Adapter, Choice, ExtraKind, Extras, Opts};
use rune_options_playground::model::{Env, OptionValue};
use spectrum_rune::badge::{self, Badge};
use spectrum_rune::button::{self, Button};
use spectrum_rune::constraints as c;
use spectrum_rune::icons::CHECKMARK_100;
use spectrum_rune::interaction::State;
use spectrum_rune::picker::Picker;
use spectrum_rune::status_light::{self, StatusLight};
use spectrum_rune::Size;

/// A Spectrum builder's `options()` as the playground's values.
fn conv(o: Vec<(&'static str, c::OptionValue<'static>)>) -> Opts {
    o.into_iter()
        .map(|(n, v)| {
            (
                n,
                match v {
                    c::OptionValue::Str(s) => OptionValue::Str(s),
                    c::OptionValue::Bool(b) => OptionValue::Bool(b),
                    c::OptionValue::Num(n) => OptionValue::Num(n),
                    c::OptionValue::Present => OptionValue::Present,
                },
            )
        })
        .collect()
}

const WRAPPED: &str = "Wrapped label text goes here";

/// The label and width extras every Spectrum adapter shares.
trait SpectrumChoice {
    /// The label preset as text: `Label`, the wrapping one, or empty for `none`.
    fn label(&self) -> &'static str;
    fn width(&self) -> Option<f32>;
}

impl SpectrumChoice for Choice {
    fn label(&self) -> &'static str {
        match self.extra("label") {
            "short" => "Label",
            "long" => WRAPPED,
            _ => "",
        }
    }
    fn width(&self) -> Option<f32> {
        self.extra("width").parse().ok()
    }
}

/// `label` and `icon` are options the rules can name; `state` is an oracle state.
fn kind(opt: &str) -> ExtraKind {
    match opt {
        "label" | "icon" => ExtraKind::FreeForm,
        "state" => ExtraKind::Oracle,
        _ => ExtraKind::Request,
    }
}

/// The catalog `size` as a [`Size`].
fn size(c: &Choice) -> Size {
    match c.v("size").as_str() {
        "s" => Size::S,
        "l" => Size::L,
        "xl" => Size::XL,
        "m" => Size::M,
        other => panic!("catalog size `{other}` is not a Size"),
    }
}

/// The previewed components, in the order the playground lists them.
pub static ADAPTERS: [&dyn Adapter; 4] = [&BadgeA, &ButtonA, &PickerA, &StatusLightA];

fn label_presets() -> Vec<(&'static str, &'static str)> {
    vec![("short", "Label"), ("long", WRAPPED)]
}
fn width_presets() -> Vec<(&'static str, &'static str)> {
    vec![("auto", "auto"), ("120", "120"), ("200", "200")]
}
fn none_or_check() -> Vec<(&'static str, &'static str)> {
    vec![("none", "none"), ("check", "checkmark")]
}
fn option(kv: String) -> [String; 2] {
    ["--option".to_owned(), kv]
}

struct BadgeA;
impl Adapter for BadgeA {
    fn id(&self) -> &'static str {
        "badge"
    }
    fn name(&self) -> &'static str {
        "Badge"
    }
    fn extra_kind(&self, opt: &str) -> ExtraKind {
        kind(opt)
    }
    fn extras(&self) -> Extras {
        vec![
            ("label", [label_presets(), vec![("none", "none")]].concat()),
            ("icon", none_or_check()),
            ("width", width_presets()),
        ]
    }
    fn build(&self, c: &Choice, env: Env, key: &str) -> (Opts, El) {
        let t = tokens_of(env);
        let mut b = Badge::new(key)
            .label(c.label())
            .variant(
                *badge::Variant::ALL
                    .iter()
                    .find(|x| x.as_str() == c.v("variant"))
                    .expect("catalog variant missing from badge::Variant::ALL"),
            )
            .style(
                *badge::Style::ALL
                    .iter()
                    .find(|x| x.as_str() == c.v("style"))
                    .expect("catalog style missing from badge::Style::ALL"),
            )
            .size(size(c));
        if c.extra("icon") == "check" {
            b = b.icon(&CHECKMARK_100);
        }
        if let Some(w) = c.width() {
            b = b.width(w);
        }
        (conv(b.options()), b.build(t))
    }
    fn oracle_args(&self, c: &Choice) -> Vec<String> {
        let mut a = Vec::new();
        if c.extra("icon") == "check" {
            a.extend(option("icon=true".into()));
        }
        // A badge without a label is a catalog combination; the label is a request field otherwise.
        if c.extra("label") == "none" {
            a.extend(option("label=false".into()));
        } else {
            a.extend(["--label".to_owned(), c.label().to_owned()]);
        }
        a
    }
}

struct ButtonA;
impl Adapter for ButtonA {
    fn id(&self) -> &'static str {
        "button"
    }
    fn name(&self) -> &'static str {
        "Button"
    }
    /// A static white or black Button is invisible on the stage's own surface, so it sits on a contrasting one:
    /// the darker of the two tokens behind white, the lighter behind black, in either scheme.
    /// shortcut: Spectrum has no backdrop token for static colours; these are stand-ins, see a14.56.
    fn stage(&self, c: &Choice, env: Env, built: El) -> El {
        let on = match c.v("staticColor").as_str() {
            "white" => true,
            "black" => false,
            _ => return built,
        };
        let t = tokens_of(env);
        let (dark_surface, light_surface) = if env.dark {
            (t.background_layer_2_color, t.neutral_content_color_default)
        } else {
            (t.neutral_content_color_default, t.background_layer_2_color)
        };
        col()
            .key("playground/stage/backdrop")
            .padding(t.spacing_300)
            .fill(if on { dark_surface } else { light_surface })
            .child(built)
    }
    fn extra_kind(&self, opt: &str) -> ExtraKind {
        kind(opt)
    }
    fn extras(&self) -> Extras {
        vec![
            ("label", label_presets()),
            ("icon", none_or_check()),
            ("width", width_presets()),
            // Rest, hover and down also drive the preview; keyboard focus is oracle only.
            (
                "state",
                vec![
                    ("rest", "rest"),
                    ("hover", "hover"),
                    ("down", "down"),
                    ("keyboard-focus", "keyboard focus"),
                ],
            ),
        ]
    }
    fn build(&self, c: &Choice, env: Env, key: &str) -> (Opts, El) {
        let t = tokens_of(env);
        let mut b = Button::new(key)
            .label(c.label())
            .variant(match c.v("variant").as_str() {
                "accent" => button::Variant::Accent,
                "negative" => button::Variant::Negative,
                "secondary" => button::Variant::Secondary,
                _ => button::Variant::Primary,
            })
            .style(match c.v("style").as_str() {
                "outline" => button::Style::Outline,
                _ => button::Style::Fill,
            })
            .size(size(c))
            .label_hidden(c.on("isLabelHidden"))
            .disabled(c.on("isDisabled"))
            .pending(c.on("isPending"));
        match c.v("staticColor").as_str() {
            "white" => b = b.static_color(button::StaticColor::White),
            "black" => b = b.static_color(button::StaticColor::Black),
            _ => {}
        }
        match c.extra("state") {
            "hover" => b = b.forced_state(State::Hover),
            "down" => b = b.forced_state(State::Down),
            _ => {}
        }
        if c.extra("icon") == "check" {
            b = b.icon(&CHECKMARK_100);
        }
        if let Some(w) = c.width() {
            b = b.width(w);
        }
        (conv(b.options()), b.build(t))
    }
    fn oracle_args(&self, c: &Choice) -> Vec<String> {
        let mut a = Vec::new();
        // The Button's icon is not an option: the request's `--icon` field hides it.
        if c.extra("icon") == "check" {
            a.extend(option("icon=true".into()));
        } else {
            a.extend(["--icon".to_owned(), "none".to_owned()]);
        }
        a.extend(["--label".to_owned(), c.label().to_owned()]);
        let state = c.extra("state");
        if state != "rest" {
            a.extend(["--state".to_owned(), state.to_owned()]);
        }
        a
    }
}

struct PickerA;
impl PickerA {
    /// The free-text presets: `None` for `none`.
    fn text(c: &Choice, opt: &str) -> Option<&'static str> {
        match (opt, c.extra(opt)) {
            ("value", "canada") => Some("Canada"),
            ("placeholder", "select") => Some("Select"),
            ("description", "help") => Some("Help text"),
            _ => None,
        }
    }
}
impl Adapter for PickerA {
    fn id(&self) -> &'static str {
        "picker"
    }
    fn name(&self) -> &'static str {
        "Picker"
    }
    fn extra_kind(&self, opt: &str) -> ExtraKind {
        kind(opt)
    }
    fn extras(&self) -> Extras {
        vec![
            ("label", [label_presets(), vec![("none", "none")]].concat()),
            ("value", vec![("none", "none"), ("canada", "Canada")]),
            ("placeholder", vec![("select", "Select"), ("none", "none")]),
            ("description", vec![("none", "none"), ("help", "Help text")]),
            ("width", width_presets()),
        ]
    }
    fn build(&self, c: &Choice, env: Env, key: &str) -> (Opts, El) {
        let t = tokens_of(env);
        let mut p = Picker::new(key)
            .label(c.label())
            .size(size(c))
            .quiet(c.on("isQuiet"))
            .disabled(c.on("isDisabled"))
            .error(c.on("isError"))
            .required(c.on("isRequired"));
        if let Some(x) = Self::text(c, "description") {
            p = p.description(x);
        }
        if let Some(x) = Self::text(c, "value") {
            p = p.value(x);
        }
        if let Some(x) = Self::text(c, "placeholder") {
            p = p.placeholder(x);
        }
        if let Some(w) = c.width() {
            p = p.width(w);
        }
        (conv(p.options()), p.build(t))
    }
    fn oracle_args(&self, c: &Choice) -> Vec<String> {
        let mut a = Vec::new();
        // The picker's label is an option of the recipe (its field-label), not `--label`.
        if c.extra("label") != "none" {
            a.extend(option(format!("label=\"{}\"", c.label())));
        }
        for opt in ["value", "placeholder", "description"] {
            if let Some(x) = Self::text(c, opt) {
                a.extend(option(format!("{opt}=\"{x}\"")));
            }
        }
        a
    }
}

struct StatusLightA;
impl Adapter for StatusLightA {
    fn id(&self) -> &'static str {
        "status-light"
    }
    fn name(&self) -> &'static str {
        "Status light"
    }
    fn extra_kind(&self, opt: &str) -> ExtraKind {
        kind(opt)
    }
    fn extras(&self) -> Extras {
        vec![("label", label_presets()), ("width", width_presets())]
    }
    fn build(&self, c: &Choice, env: Env, key: &str) -> (Opts, El) {
        let t = tokens_of(env);
        let mut b = StatusLight::new(key, c.label())
            .variant(
                *status_light::Variant::ALL
                    .iter()
                    .find(|x| x.as_str() == c.v("variant"))
                    .expect("catalog variant missing from status_light::Variant::ALL"),
            )
            .size(size(c));
        if let Some(w) = c.width() {
            b = b.width(w);
        }
        (conv(b.options()), b.build(t))
    }
    fn oracle_args(&self, c: &Choice) -> Vec<String> {
        vec!["--label".to_owned(), c.label().to_owned()]
    }
}
