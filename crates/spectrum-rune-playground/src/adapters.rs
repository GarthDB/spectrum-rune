//! Per-component adapters for the Playground: what each previewed component adds to the catalog (free-form
//! presets such as a label or an icon), how the chosen values build it, and what it passes to the oracle
//! besides its catalog options. Nothing here reads the gallery's state: an adapter sees a [`Choice`], so it
//! can move into a shared playground crate (Stage 3, a14.44) unchanged.

use crate::model::OptionValue;
use rune::El;
use spectrum_rune::badge::{self, Badge};
use spectrum_rune::button::{self, Button};
use spectrum_rune::constraints as c;
use spectrum_rune::icons::CHECKMARK_100;
use spectrum_rune::interaction::State;
use spectrum_rune::picker::Picker;
use spectrum_rune::status_light::{self, StatusLight};
use spectrum_rune::tokens::Tokens;
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

pub type Opts = Vec<(&'static str, OptionValue<'static>)>;
/// Option name and its presets as (id, shown); the first preset is the default.
pub type Extras = Vec<(&'static str, Vec<(&'static str, &'static str)>)>;

pub const WRAPPED: &str = "Wrapped label text goes here";

/// The chosen values of one component: its catalog options (defaults filled in) and the preset id chosen for
/// each of its extras.
pub struct Choice {
    pub options: Opts,
    pub extras: Vec<(&'static str, String)>,
}

impl Choice {
    /// A catalog option's value as a string (`""` when unset).
    pub fn v(&self, opt: &str) -> String {
        match self.options.iter().find(|(n, _)| *n == opt) {
            Some((_, OptionValue::Str(x))) => (*x).to_owned(),
            Some((_, OptionValue::Bool(b))) => b.to_string(),
            Some((_, OptionValue::Num(n))) => n.to_string(),
            _ => String::new(),
        }
    }
    fn on(&self, opt: &str) -> bool {
        self.v(opt) == "true"
    }
    /// The chosen preset id of an extra (`""` when the component has no such extra).
    pub fn extra(&self, opt: &str) -> &str {
        self.extras
            .iter()
            .find(|(n, _)| *n == opt)
            .map_or("", |(_, v)| v.as_str())
    }
    /// The label preset as text: `Label`, the wrapping one, or empty for `none`.
    pub fn label(&self) -> &'static str {
        match self.extra("label") {
            "short" => "Label",
            "long" => WRAPPED,
            _ => "",
        }
    }
    pub fn width(&self) -> Option<f32> {
        self.extra("width").parse().ok()
    }
    fn size(&self) -> Size {
        match self.v("size").as_str() {
            "s" => Size::S,
            "l" => Size::L,
            "xl" => Size::XL,
            "m" => Size::M,
            other => panic!("catalog size `{other}` is not a Size"),
        }
    }
}

/// One previewed component.
pub trait Adapter: Sync {
    /// The catalog name, e.g. `"button"`.
    fn id(&self) -> &'static str;
    /// The name shown in the playground.
    fn name(&self) -> &'static str;
    /// Free-form options and the request width: they have no catalog values, so a few presets each.
    fn extras(&self) -> Extras;
    /// The built component under `key` (its parts are `<key>/container`, ...) and the builder's own `options()`.
    fn build(&self, c: &Choice, t: &Tokens, key: &str) -> (Opts, El);
    /// `design-data render` arguments beyond the catalog options, `--size`, scheme, scale and width (which the
    /// playground adds itself): the label, the icon, a state.
    fn oracle_args(&self, c: &Choice) -> Vec<String>;
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
    fn extras(&self) -> Extras {
        vec![
            ("label", [label_presets(), vec![("none", "none")]].concat()),
            ("icon", none_or_check()),
            ("width", width_presets()),
        ]
    }
    fn build(&self, c: &Choice, t: &Tokens, key: &str) -> (Opts, El) {
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
            .size(c.size());
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
    fn build(&self, c: &Choice, t: &Tokens, key: &str) -> (Opts, El) {
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
            .size(c.size())
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
    fn extras(&self) -> Extras {
        vec![
            ("label", [label_presets(), vec![("none", "none")]].concat()),
            ("value", vec![("none", "none"), ("canada", "Canada")]),
            ("placeholder", vec![("select", "Select"), ("none", "none")]),
            ("description", vec![("none", "none"), ("help", "Help text")]),
            ("width", width_presets()),
        ]
    }
    fn build(&self, c: &Choice, t: &Tokens, key: &str) -> (Opts, El) {
        let mut p = Picker::new(key)
            .label(c.label())
            .size(c.size())
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
    fn extras(&self) -> Extras {
        vec![("label", label_presets()), ("width", width_presets())]
    }
    fn build(&self, c: &Choice, t: &Tokens, key: &str) -> (Opts, El) {
        let mut b = StatusLight::new(key, c.label())
            .variant(
                *status_light::Variant::ALL
                    .iter()
                    .find(|x| x.as_str() == c.v("variant"))
                    .expect("catalog variant missing from status_light::Variant::ALL"),
            )
            .size(c.size());
        if let Some(w) = c.width() {
            b = b.width(w);
        }
        (conv(b.options()), b.build(t))
    }
    fn oracle_args(&self, c: &Choice) -> Vec<String> {
        vec!["--label".to_owned(), c.label().to_owned()]
    }
}
