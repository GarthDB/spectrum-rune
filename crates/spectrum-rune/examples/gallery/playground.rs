//! The Playground story: a control per catalog option, a live preview, the options the builder reports,
//! and the catalog's constraint rules. Values come from the generated catalog (`constraints::component`);
//! a value whose choice would break an `error` rule is greyed out and the reason is listed. Each component
//! needs only an adapter (`preview`) from the chosen values to its builder. Dev-only: no token panel yet.

use crate::G;
use rune::{col, el, row, text, El};
use spectrum_rune::action_button::ActionButton;
use spectrum_rune::badge::{self, Badge};
use spectrum_rune::button::{self, Button};
use spectrum_rune::constraints::{check, component, Lit, OptionDef, OptionValue, Severity};
use spectrum_rune::font;
use spectrum_rune::icons::CHECKMARK_100;
use spectrum_rune::status_light::{self, StatusLight};
use spectrum_rune::tokens::Tokens;
use spectrum_rune::Size;

type Opts = Vec<(&'static str, OptionValue<'static>)>;

const COMPONENTS: [(&str, &str); 3] = [
    ("badge", "Badge"),
    ("button", "Button"),
    ("status-light", "Status light"),
];

const WRAPPED: &str = "Wrapped label text goes here";

/// Free-form options and the request width have no catalog values: a few presets each, the first is the default.
fn extras(comp: &str) -> Vec<(&'static str, Vec<(&'static str, &'static str)>)> {
    let label = vec![("short", "Label"), ("long", WRAPPED)];
    let width = vec![("auto", "auto"), ("120", "120"), ("200", "200")];
    match comp {
        "badge" => vec![
            ("label", [label, vec![("none", "none")]].concat()),
            ("icon", vec![("none", "none"), ("check", "checkmark")]),
            ("width", width),
        ],
        "button" => vec![
            ("label", label),
            ("icon", vec![("none", "none"), ("check", "checkmark")]),
            ("width", width),
        ],
        _ => vec![("label", label), ("width", width)],
    }
}

fn lit_str(l: &Lit) -> String {
    match l {
        Lit::Str(s) => (*s).to_owned(),
        Lit::Bool(b) => b.to_string(),
    }
}

/// An option the catalog gives no default is unset until chosen: the choice `none`.
fn default_of(def: &OptionDef) -> String {
    def.default.as_ref().map_or("none".to_owned(), lit_str)
}

fn pick<'a>(s: &'a G, comp: &str, opt: &str, default: &'a str) -> &'a str {
    s.play
        .get(&format!("{comp}/{opt}"))
        .map_or(default, String::as_str)
}

fn extra(s: &G, comp: &str, opt: &str) -> String {
    let presets = extras(comp);
    let (_, list) = presets.iter().find(|(o, _)| *o == opt).unwrap();
    pick(s, comp, opt, list[0].0).to_owned()
}

/// The chosen values under the catalog's option names, with `over` (option, value) replacing one choice.
fn options(s: &G, comp: &str, over: Option<(&str, &str)>) -> Opts {
    let mut o: Opts = Vec::new();
    for def in component(comp)
        .unwrap()
        .options
        .iter()
        .filter(|d| !d.values.is_empty())
    {
        let chosen = match over {
            Some((n, v)) if n == def.name => v.to_owned(),
            _ => pick(s, comp, def.name, &default_of(def)).to_owned(),
        };
        if let Some(l) = def.values.iter().find(|l| lit_str(l) == chosen) {
            o.push((def.name, l.into()));
        }
    }
    for opt in ["label", "icon"] {
        let chosen = match over {
            Some((n, v)) if n == opt => v.to_owned(),
            _ if extras(comp).iter().any(|(e, _)| *e == opt) => extra(s, comp, opt),
            _ => continue,
        };
        if chosen != "none" {
            o.push((opt, OptionValue::Present));
        }
    }
    o
}

fn errors(comp: &str, o: &Opts) -> Vec<&'static str> {
    check(comp, o)
        .into_iter()
        .filter(|v| v.severity == Severity::Error)
        .map(|v| v.description.unwrap_or("excluded option combination"))
        .collect()
}

fn size_of(v: &str) -> Size {
    match v {
        "s" => Size::S,
        "l" => Size::L,
        "xl" => Size::XL,
        _ => Size::M,
    }
}

/// The adapter: chosen values to the builder. Returns the builder's own `options()` and the built element.
fn preview(s: &G, comp: &str, t: &Tokens) -> (Opts, El) {
    let width = match extra(s, comp, "width").as_str() {
        "auto" => None,
        w => w.parse::<f32>().ok(),
    };
    let label = match extra(s, comp, "label").as_str() {
        "short" => "Label",
        "long" => WRAPPED,
        _ => "",
    };
    let v = |opt, default: &str| pick(s, comp, opt, default).to_owned();
    match comp {
        "badge" => {
            let mut b = Badge::new("playground/preview")
                .label(label)
                .variant(
                    *badge::Variant::ALL
                        .iter()
                        .find(|x| x.as_str() == v("variant", "neutral"))
                        .unwrap(),
                )
                .style(
                    *badge::Style::ALL
                        .iter()
                        .find(|x| x.as_str() == v("style", "bold"))
                        .unwrap(),
                )
                .size(size_of(&v("size", "s")));
            if extra(s, comp, "icon") == "check" {
                b = b.icon(&CHECKMARK_100);
            }
            if let Some(w) = width {
                b = b.width(w);
            }
            (b.options(), b.build(t))
        }
        "button" => {
            let on = |opt: &'static str| v(opt, "false") == "true";
            let mut b = Button::new("playground/preview")
                .label(label)
                .variant(match v("variant", "primary").as_str() {
                    "accent" => button::Variant::Accent,
                    "negative" => button::Variant::Negative,
                    "secondary" => button::Variant::Secondary,
                    _ => button::Variant::Primary,
                })
                .style(match v("style", "fill").as_str() {
                    "outline" => button::Style::Outline,
                    _ => button::Style::Fill,
                })
                .size(size_of(&v("size", "m")))
                .label_hidden(on("isLabelHidden"))
                .disabled(on("isDisabled"))
                .pending(on("isPending"));
            match v("staticColor", "none").as_str() {
                "white" => b = b.static_color(button::StaticColor::White),
                "black" => b = b.static_color(button::StaticColor::Black),
                _ => {}
            }
            if extra(s, comp, "icon") == "check" {
                b = b.icon(&CHECKMARK_100);
            }
            if let Some(w) = width {
                b = b.width(w);
            }
            (b.options(), b.build(t))
        }
        _ => {
            let mut b = StatusLight::new("playground/preview", label)
                .variant(
                    *status_light::Variant::ALL
                        .iter()
                        .find(|x| x.as_str() == v("variant", "informative"))
                        .unwrap(),
                )
                .size(size_of(&v("size", "m")));
            if let Some(w) = width {
                b = b.width(w);
            }
            (b.options(), b.build(t))
        }
    }
}

fn fmt(v: &OptionValue) -> String {
    match v {
        OptionValue::Str(s) => (*s).to_owned(),
        OptionValue::Bool(b) => b.to_string(),
        OptionValue::Present => "(set)".to_owned(),
    }
}

fn caption(t: &Tokens, s: impl Into<String>) -> El {
    text(s.into())
        .font(font::text_style(&t.component_s_regular))
        .color(t.neutral_subdued_content_color_default)
}

/// One option: its values as buttons, the chosen one selected, an excluded one disabled.
fn control(
    t: &'static Tokens,
    comp: &'static str,
    opt: &'static str,
    values: Vec<(String, String)>,
    current: &str,
    excluded: impl Fn(&str) -> Vec<&'static str>,
) -> El {
    let mut notes = Vec::new();
    let buttons = values.into_iter().map(|(id, shown)| {
        let why = excluded(&id);
        if !why.is_empty() {
            notes.push(format!("{shown}: {}", why.join("; ")));
        }
        let key = format!("{comp}/{opt}");
        let id2 = id.clone();
        ActionButton::new(format!("playground/{comp}/{opt}/{id}"))
            .label(shown)
            .quiet(true)
            .selected(id == current)
            .disabled(!why.is_empty())
            .on_press(move |g: &mut G| {
                g.play.insert(key.clone(), id2.clone());
            })
            .build(t)
    });
    let row_ = row().gap(4.0).flex_wrap().children(buttons);
    let mut c = col().gap(4.0).child(caption(t, opt)).child(row_);
    for n in notes {
        c = c.child(caption(t, format!("greyed out, {n}")));
    }
    c
}

pub fn playground(s: &G, t: &'static Tokens) -> El {
    let comp = COMPONENTS
        .iter()
        .find(|(id, _)| *id == pick(s, "", "component", "badge"))
        .map_or("badge", |c| c.0);
    let picker = row().gap(4.0).children(COMPONENTS.iter().map(|(id, name)| {
        let id: &'static str = id;
        ActionButton::new(format!("playground/pick/{id}"))
            .label(*name)
            .selected(id == comp)
            .on_press(move |g: &mut G| {
                g.play.insert("/component".into(), id.into());
            })
            .build(t)
    }));
    let mut controls = col().gap(12.0);
    for def in component(comp)
        .unwrap()
        .options
        .iter()
        .filter(|d| !d.values.is_empty())
    {
        let current = pick(s, comp, def.name, &default_of(def)).to_owned();
        let unset = def
            .default
            .is_none()
            .then(|| ("none".to_owned(), "none".to_owned()));
        let values = unset
            .into_iter()
            .chain(def.values.iter().map(|l| (lit_str(l), lit_str(l))))
            .collect();
        controls = controls.child(control(t, comp, def.name, values, &current, |v| {
            errors(comp, &options(s, comp, Some((def.name, v))))
        }));
    }
    for (opt, presets) in extras(comp) {
        let current = extra(s, comp, opt);
        let values = presets
            .iter()
            .map(|(id, shown)| (id.to_string(), shown.to_string()))
            .collect();
        controls = controls.child(control(t, comp, opt, values, &current, |v| {
            if opt == "width" {
                Vec::new()
            } else {
                errors(comp, &options(s, comp, Some((opt, v))))
            }
        }));
    }

    let now = options(s, comp, None);
    let broken = errors(comp, &now);
    let (reported, shown) = if broken.is_empty() {
        let (o, e) = preview(s, comp, t);
        (o, e)
    } else {
        (
            Vec::new(),
            text(format!("Not defined: {}", broken.join("; ")))
                .color(t.negative_content_color_default),
        )
    };
    let mut panel = col()
        .gap(2.0)
        .child(caption(t, "options() reported by the builder"));
    for (n, v) in &reported {
        panel = panel.child(caption(t, format!("{n} = {}", fmt(v))));
    }
    let mut rules = col().gap(2.0).child(caption(t, "catalog constraints"));
    let c = component(comp).unwrap();
    if c.rules.is_empty() {
        rules = rules.child(caption(
            t,
            "none: the catalog has no rules for this component",
        ));
    }
    for r in c.rules {
        rules = rules.child(caption(
            t,
            format!("{:?}: {}", r.severity, r.description.unwrap_or("")),
        ));
    }
    col().w_full().gap(24.0).child(picker).child(
        row().gap(32.0).flex_wrap().child(controls.w(520.0)).child(
            col()
                .gap(24.0)
                .child(el().padding(16.0).child(shown))
                .child(panel)
                .child(rules),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectrum_rune::tokens::{tokens, ColorScheme, Scale};

    /// The options derived from the catalog (what the controls grey out by) equal what each builder reports,
    /// for every value of every enumerated option, so the adapter and the catalog cannot drift apart.
    #[test]
    fn catalog_options_match_builder_options() {
        let t = tokens(ColorScheme::Light, Scale::Desktop);
        for (comp, _) in COMPONENTS {
            for def in component(comp)
                .unwrap()
                .options
                .iter()
                .filter(|d| !d.values.is_empty())
            {
                for l in def.values {
                    let mut s = crate::init();
                    s.play.insert(format!("{comp}/{}", def.name), lit_str(l));
                    let sorted = |mut o: Opts| {
                        o.sort_by_key(|(n, _)| *n);
                        o
                    };
                    let mut derived = sorted(options(&s, comp, None));
                    if errors(comp, &derived).is_empty() {
                        let reported = sorted(preview(&s, comp, t).0);
                        // A builder may leave out options with no visual effect (Button's size, isDisabled).
                        derived.retain(|(n, _)| reported.iter().any(|(r, _)| r == n));
                        assert_eq!(derived, reported, "{comp} {} = {}", def.name, lit_str(l));
                    }
                }
            }
        }
    }

    #[test]
    fn outline_is_greyed_out_for_accent_button() {
        let mut s = crate::init();
        s.play.insert("button/variant".into(), "accent".into());
        let why = errors("button", &options(&s, "button", Some(("style", "outline"))));
        assert_eq!(
            why,
            ["Outline style is only defined for primary and secondary."]
        );
        assert!(errors("button", &options(&s, "button", Some(("style", "fill")))).is_empty());
        // An icon-only button needs an icon: hiding the label is excluded until one is chosen.
        let why = errors(
            "button",
            &options(&s, "button", Some(("isLabelHidden", "true"))),
        );
        assert_eq!(why, ["An icon-only button needs an icon."]);
    }

    #[test]
    fn pressing_a_value_selects_it() {
        use rune::{Env, Runtime};
        let mut s = crate::init();
        s.story = "playground";
        let mut rt = Runtime::new(s, crate::view, (1100.0, 900.0), Env::new(false, 1.0));
        rt.frame(0.05);
        let (x, y) = rt.find("playground/badge/style/subtle/container").unwrap();
        rt.pointer_move(x, y);
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(0.05);
        assert_eq!(rt.state.play["badge/style"], "subtle");
    }
}
