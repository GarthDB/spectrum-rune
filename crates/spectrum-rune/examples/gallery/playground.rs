//! The Playground story: a control per catalog option, a live preview, the options the builder reports,
//! and the catalog's constraint rules. Values come from the generated catalog (`constraints::component`);
//! a value whose choice would break an `error` rule is greyed out and the reason is listed. Each component
//! needs only an adapter (`preview`) from the chosen values to its builder. Dev-only: the token panel runs
//! the oracle CLI (`oracle.rs`) on demand.

use crate::G;
use rune::{col, el, row, text, El};
use spectrum_rune::action_button::ActionButton;
use spectrum_rune::badge::{self, Badge};
use spectrum_rune::button::{self, Button};
use spectrum_rune::constraints::{check, component, Lit, OptionDef, OptionValue, Severity};
use spectrum_rune::font;
use spectrum_rune::icons::CHECKMARK_100;
use spectrum_rune::status_light::{self, StatusLight};
use spectrum_rune::tokens::{tokens, ColorScheme, Scale, Tokens};
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
            // Oracle only: the preview itself answers to the real pointer and keyboard.
            (
                "state",
                vec![
                    ("rest", "rest"),
                    ("hover", "hover"),
                    ("down", "down"),
                    ("keyboard-focus", "keyboard focus"),
                ],
            ),
        ],
        _ => vec![("label", label), ("width", width)],
    }
}

fn lit_str(l: &Lit) -> String {
    match l {
        Lit::Str(s) => (*s).to_owned(),
        Lit::Bool(b) => b.to_string(),
        Lit::Num(n) => n.to_string(),
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
        "m" => Size::M,
        other => panic!("catalog size `{other}` is not a Size"),
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
    // One source of truth: the catalog values (and defaults) `options` derives, not defaults repeated here.
    let derived = options(s, comp, None);
    let v = |opt: &str| match derived.iter().find(|(n, _)| *n == opt) {
        Some((_, OptionValue::Str(x))) => (*x).to_owned(),
        Some((_, OptionValue::Bool(b))) => b.to_string(),
        Some((_, OptionValue::Num(n))) => n.to_string(),
        _ => String::new(),
    };
    match comp {
        "badge" => {
            let mut b = Badge::new("playground/preview")
                .label(label)
                .variant(
                    *badge::Variant::ALL
                        .iter()
                        .find(|x| x.as_str() == v("variant"))
                        .expect("catalog variant missing from badge::Variant::ALL"),
                )
                .style(
                    *badge::Style::ALL
                        .iter()
                        .find(|x| x.as_str() == v("style"))
                        .expect("catalog style missing from badge::Style::ALL"),
                )
                .size(size_of(&v("size")));
            if extra(s, comp, "icon") == "check" {
                b = b.icon(&CHECKMARK_100);
            }
            if let Some(w) = width {
                b = b.width(w);
            }
            (b.options(), b.build(t))
        }
        "button" => {
            let on = |opt: &str| v(opt) == "true";
            let mut b = Button::new("playground/preview")
                .label(label)
                .variant(match v("variant").as_str() {
                    "accent" => button::Variant::Accent,
                    "negative" => button::Variant::Negative,
                    "secondary" => button::Variant::Secondary,
                    _ => button::Variant::Primary,
                })
                .style(match v("style").as_str() {
                    "outline" => button::Style::Outline,
                    _ => button::Style::Fill,
                })
                .size(size_of(&v("size")))
                .label_hidden(on("isLabelHidden"))
                .disabled(on("isDisabled"))
                .pending(on("isPending"));
            match v("staticColor").as_str() {
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
                        .find(|x| x.as_str() == v("variant"))
                        .expect("catalog variant missing from status_light::Variant::ALL"),
                )
                .size(size_of(&v("size")));
            if let Some(w) = width {
                b = b.width(w);
            }
            (b.options(), b.build(t))
        }
    }
}

/// The `design-data render` arguments for the chosen values (the catalog option names, `--size`, scheme, scale).
fn oracle_args(s: &G, comp: &str, dark: bool) -> Vec<String> {
    let mut a: Vec<String> = vec![
        "--color-scheme".into(),
        if dark { "dark" } else { "light" }.into(),
        "--scale".into(),
        if s.scale == Scale::Mobile {
            "mobile"
        } else {
            "desktop"
        }
        .into(),
    ];
    let mut opt = |kv: String| a.extend(["--option".to_owned(), kv]);
    let mut size = None;
    let mut icon = false;
    let mut label = false;
    for (n, v) in options(s, comp, None) {
        match (n, v) {
            ("size", OptionValue::Str(x)) => size = Some(x),
            ("icon", _) => icon = true,
            ("label", _) => label = true,
            (n, OptionValue::Str(x)) => opt(format!("{n}=\"{x}\"")),
            (n, OptionValue::Bool(b)) => opt(format!("{n}={b}")),
            (n, OptionValue::Num(x)) => opt(format!("{n}={x}")),
            (_, OptionValue::Present) => {}
        }
    }
    if icon {
        opt("icon=true".into());
    }
    if !label && comp == "badge" {
        opt("label=false".into());
    }
    // The Button's icon is not an option: the request's `--icon` field hides it.
    if comp == "button" && !icon {
        a.extend(["--icon".to_owned(), "none".to_owned()]);
    }
    if let Some(x) = size {
        a.extend(["--size".to_owned(), x.to_owned()]);
    }
    if label || comp != "badge" {
        let text = if extra(s, comp, "label") == "long" {
            WRAPPED
        } else {
            "Label"
        };
        a.extend(["--label".to_owned(), text.to_owned()]);
    }
    if comp == "button" {
        let state = extra(s, comp, "state");
        if state != "rest" {
            a.extend(["--state".to_owned(), state]);
        }
    }
    if let Ok(w) = extra(s, comp, "width").parse::<f32>() {
        a.extend(["--width".to_owned(), w.to_string()]);
    }
    a
}

struct Preview {
    s: G,
    comp: &'static str,
    dark: bool,
}

fn preview_view(p: &Preview, _: &mut rune::Cx<Preview>) -> El {
    col().child(preview(&p.s, p.comp, tokens(scheme(p.dark), p.s.scale)).1)
}

fn scheme(dark: bool) -> ColorScheme {
    if dark {
        ColorScheme::Dark
    } else {
        ColorScheme::Light
    }
}

/// Rune's own geometry for the current selection: the preview built in a throwaway runtime, each part's
/// box relative to the container (the view of the real page cannot read element rects).
fn rune_parts(s: &G, comp: &'static str, dark: bool) -> Vec<(String, [f32; 4])> {
    use rune::{Env, Runtime};
    let state = Preview {
        s: s.clone(),
        comp,
        dark,
    };
    let mut rt = Runtime::new(state, preview_view, (800.0, 400.0), Env::new(dark, 1.0));
    for _ in 0..40 {
        rt.frame(0.05);
    }
    let prefix = "playground/preview/";
    let all: Vec<_> = rt
        .elements()
        .into_iter()
        .filter_map(|e| Some((e.key?.strip_prefix(prefix)?.to_owned(), e.rect)))
        .collect();
    let c = all
        .iter()
        .find(|(n, _)| n == "container")
        .map_or([0.0; 4], |(_, r)| *r);
    all.into_iter()
        .map(|(n, r)| (n, [r[0] - c[0], r[1] - c[1], r[2], r[3]]))
        .collect()
}

/// Oracle vs Rune for the current selection, as panel lines: each oracle part's box in both, flagged
/// when they differ by more than 1px, then the tokens the part reads.
fn compare(s: &G, comp: &'static str, dark: bool, args: &[String]) -> Result<Vec<String>, String> {
    if let Some(why) = errors(comp, &options(s, comp, None)).first() {
        return Err(format!("not defined: {why}"));
    }
    let (v, parts) = crate::oracle::run(comp, args)?;
    let rune = rune_parts(s, comp, dark);
    let c = parts
        .iter()
        .find(|p| p.name == "container")
        .map_or([0.0; 4], |p| p.rect);
    let r1 = |x: f32| (x * 10.0).round() / 10.0;
    let mut lines = vec![format!(
        "oracle {v} vs Rune, box x y w h relative to the container"
    )];
    for p in &parts {
        let o = [p.rect[0] - c[0], p.rect[1] - c[1], p.rect[2], p.rect[3]];
        let got = rune.iter().find(|(n, _)| *n == p.name).map(|(_, r)| *r);
        let verdict = match got {
            Some(r) if r.iter().zip(o).all(|(a, b)| (a - b).abs() <= 1.0) => "same".to_owned(),
            Some(r) => format!("DIFFERS, Rune {:?}", r.map(r1)),
            None => "no such part in Rune".to_owned(),
        };
        lines.push(format!("{}: oracle {:?}, {verdict}", p.name, o.map(r1)));
        lines.extend(p.tokens.iter().map(|t| format!("    {t}")));
    }
    Ok(lines)
}

fn fmt(v: &OptionValue) -> String {
    match v {
        OptionValue::Str(s) => (*s).to_owned(),
        OptionValue::Bool(b) => b.to_string(),
        OptionValue::Num(n) => n.to_string(),
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
            if opt == "width" || opt == "state" {
                Vec::new()
            } else {
                errors(comp, &options(s, comp, Some((opt, v))))
            }
        }));
    }

    let now = options(s, comp, None);
    let broken = errors(comp, &now);
    let (reported, shown) = if broken.is_empty() {
        preview(s, comp, t)
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
    let dark = std::ptr::eq(t, tokens(ColorScheme::Dark, s.scale));
    let args = oracle_args(s, comp, dark);
    let mut oracle = col().gap(2.0).child(
        ActionButton::new("playground/oracle/run")
            .label("Run oracle (design-data CLI)")
            .on_press({
                let args = args.clone();
                move |g: &mut G| {
                    let r = compare(g, comp, dark, &args);
                    g.oracle = Some((args.clone(), r));
                }
            })
            .build(t),
    );
    oracle = match &s.oracle {
        None => oracle.child(caption(
            t,
            "tokens each part reads, from the oracle: not run yet",
        )),
        Some((ran, _)) if *ran != args => oracle.child(caption(
            t,
            "the options changed since the last run: run it again",
        )),
        Some((_, Ok(lines))) => lines
            .iter()
            .fold(oracle, |c, l| c.child(caption(t, l.clone()))),
        Some((_, Err(e))) => oracle.child(caption(t, format!("oracle failed: {e}"))),
    };
    col().w_full().gap(24.0).child(picker).child(
        row().gap(32.0).flex_wrap().child(controls.w(520.0)).child(
            col()
                .gap(24.0)
                .child(el().padding(16.0).child(shown))
                .child(panel)
                .child(rules)
                .child(oracle),
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

    /// The label and icon presets (not catalog values) must reach the builder too.
    #[test]
    fn extras_match_builder_options() {
        let t = tokens(ColorScheme::Light, Scale::Desktop);
        for (comp, _) in COMPONENTS {
            for (opt, presets) in extras(comp) {
                for (id, _) in presets {
                    let mut s = crate::init();
                    s.play.insert(format!("{comp}/{opt}"), id.to_owned());
                    let derived = options(&s, comp, None);
                    if !errors(comp, &derived).is_empty() {
                        continue;
                    }
                    let reported = preview(&s, comp, t).0;
                    for name in ["label", "icon"] {
                        let has = |o: &Opts| o.iter().any(|(n, _)| *n == name);
                        // Button's builder does not report its label (no visual option of its own).
                        if comp == "button" && name == "label" {
                            continue;
                        }
                        assert_eq!(has(&derived), has(&reported), "{comp} {opt} = {id}: {name}");
                    }
                }
            }
        }
    }

    /// Every combination the controls can reach builds without a panic: the catalog options and the
    /// presets, skipping what `errors()` excludes (the UI shows a message for those instead).
    #[test]
    fn every_reachable_combination_builds() {
        let t = tokens(ColorScheme::Light, Scale::Desktop);
        for (comp, _) in COMPONENTS {
            let mut controls: Vec<(String, Vec<String>)> = Vec::new();
            for def in component(comp)
                .unwrap()
                .options
                .iter()
                .filter(|d| !d.values.is_empty())
            {
                let unset = def.default.is_none().then(|| "none".to_owned());
                let all = unset.into_iter().chain(def.values.iter().map(lit_str));
                controls.push((def.name.to_owned(), all.collect()));
            }
            for (opt, presets) in extras(comp) {
                controls.push((
                    opt.to_owned(),
                    presets.iter().map(|(id, _)| id.to_string()).collect(),
                ));
            }
            let total: usize = controls.iter().map(|(_, v)| v.len()).product();
            let mut built = 0;
            for mut n in 0..total {
                let mut s = crate::init();
                for (opt, values) in &controls {
                    s.play
                        .insert(format!("{comp}/{opt}"), values[n % values.len()].clone());
                    n /= values.len();
                }
                if errors(comp, &options(&s, comp, None)).is_empty() {
                    preview(&s, comp, t);
                    built += 1;
                }
            }
            assert!(built > 0, "{comp}: no combination is valid");
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

    /// A value that would break an error rule is rendered disabled, pressing it changes nothing, and
    /// the reason is listed in the UI (the test above checks `errors()`, this checks what is rendered).
    #[test]
    fn excluded_value_is_rendered_disabled_with_its_reason() {
        use rune::{Env, Runtime};
        let mut s = crate::init();
        s.story = "playground";
        s.play.insert("/component".into(), "button".into());
        s.play.insert("button/variant".into(), "accent".into());
        let mut rt = Runtime::new(s, crate::view, (1100.0, 1400.0), Env::new(false, 1.0));
        rt.frame(0.05);
        let tree = rt.accessibility_json().to_string();
        assert!(
            tree.contains(
                "greyed out, outline: Outline style is only defined for primary and secondary."
            ),
            "{tree}"
        );
        let (x, y) = rt
            .find("playground/button/style/outline/container")
            .unwrap();
        rt.pointer_move(x, y);
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(0.05);
        assert!(!rt.state.play.contains_key("button/style"));
    }

    /// Needs the oracle CLI in the oracle.toml range (`DESIGN_DATA=... cargo test -- --ignored`).
    #[test]
    #[ignore]
    fn oracle_runs_every_component_at_its_defaults() {
        for (comp, _) in COMPONENTS {
            let s = crate::init();
            let args = oracle_args(&s, comp, false);
            let lines = compare(&s, comp, false, &args).expect(comp);
            assert!(
                lines.iter().any(|l| l.starts_with("container")),
                "{comp}: {lines:?}"
            );
            assert!(lines.iter().any(|l| l.contains(" = ")), "{comp}: no tokens");
            assert!(
                !lines.iter().any(|l| l.contains("DIFFERS")),
                "{comp}: {lines:?}"
            );
        }
    }

    /// The state control reaches the oracle: a hovered Button reads different tokens than one at rest.
    #[test]
    #[ignore]
    fn oracle_state_changes_the_tokens() {
        let tokens_at = |state: &str| {
            let mut s = crate::init();
            s.play.insert("/component".into(), "button".into());
            s.play.insert("button/icon".into(), "check".into());
            s.play.insert("button/state".into(), state.into());
            let args = oracle_args(&s, "button", false);
            compare(&s, "button", false, &args).expect(state)
        };
        assert_ne!(tokens_at("rest"), tokens_at("hover"));
    }

    /// Pressing Run lists the oracle's tokens in the UI (needs the CLI, like the test above).
    #[test]
    #[ignore]
    fn run_button_lists_the_oracle_tokens() {
        use rune::{Env, Runtime};
        let mut s = crate::init();
        s.story = "playground";
        let mut rt = Runtime::new(s, crate::view, (1100.0, 1400.0), Env::new(false, 1.0));
        rt.frame(0.05);
        let (x, y) = rt.find("playground/oracle/run/container").unwrap();
        rt.pointer_move(x, y);
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(0.05);
        let tree = rt.accessibility_json().to_string();
        assert!(tree.contains("    height = component-height-"), "{tree}");
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
