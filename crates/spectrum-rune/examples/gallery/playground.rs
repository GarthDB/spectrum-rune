//! The Playground story: a control per catalog option, a live preview, the options the builder reports,
//! and the catalog's constraint rules. Values come from the generated catalog (`constraints::component`);
//! a value whose choice would break an `error` rule is greyed out and the reason is listed. Each component
//! needs only an adapter (`preview`) from the chosen values to its builder. Dev-only: the token panel runs
//! the oracle CLI (`oracle.rs`) on demand.

use crate::adapters::{adapter, Choice, ADAPTERS, WRAPPED};
use crate::G;
use rune::{col, row, text, El};
use spectrum_rune::action_button::ActionButton;
use spectrum_rune::constraints::{
    check, component, Lit, OptionDef, OptionValue, Severity, COMPONENTS as CATALOG,
};
use spectrum_rune::font;
use spectrum_rune::picker::Picker;
use spectrum_rune::status_light::{self, StatusLight};
use spectrum_rune::switch::Switch;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale, Tokens};
use spectrum_rune::Size;

type Opts = crate::adapters::Opts;

/// Free-form options and the request width have no catalog values: a few presets each (the adapter's).
fn extras(comp: &str) -> crate::adapters::Extras {
    adapter(comp).map_or_else(Vec::new, |a| a.extras())
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

/// Every rule the options break, errors and advisories, as (severity, description).
fn reasons(comp: &str, o: &Opts) -> Vec<(Severity, &'static str)> {
    check(comp, o)
        .into_iter()
        .map(|v| {
            (
                v.severity,
                v.description.unwrap_or("excluded option combination"),
            )
        })
        .collect()
}

/// `reasons` with whether each one is already broken by the current selection (`now`).
fn inherit(
    now: &[(Severity, &'static str)],
    r: Vec<(Severity, &'static str)>,
) -> Vec<(Severity, &'static str, bool)> {
    r.into_iter()
        .map(|w| (w.0, w.1, now.contains(&w)))
        .collect()
}

/// The adapter: chosen values to the builder. Returns the builder's own `options()` and the built element.
fn preview(s: &G, comp: &str, t: &Tokens) -> (Opts, El) {
    preview_keyed(s, comp, t, "playground/preview")
}

/// `preview` with the key its parts are named under (`<key>/container`, ...): a second copy needs its own.
fn preview_keyed(s: &G, comp: &str, t: &Tokens, key: &str) -> (Opts, El) {
    adapter(comp)
        .expect("a previewed component")
        .build(&choice(s, comp), t, key)
}

/// The chosen values of `comp`: its catalog options (one source of truth: the catalog values and defaults
/// `options` derives) and the preset chosen for each of its extras.
fn choice(s: &G, comp: &str) -> Choice {
    Choice {
        options: options(s, comp, None),
        extras: extras(comp)
            .iter()
            .map(|(opt, _)| (*opt, extra(s, comp, opt)))
            .collect(),
    }
}

/// The `design-data render` arguments for the chosen values (the catalog option names, `--size`, scheme, scale),
/// then what the component's adapter adds (label, icon, state) and the width.
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
    let mut size = None;
    for (n, v) in options(s, comp, None) {
        let mut opt = |kv: String| a.extend(["--option".to_owned(), kv]);
        match (n, v) {
            ("size", OptionValue::Str(x)) => size = Some(x),
            // The free-form options the adapter names itself.
            ("icon" | "label", _) => {}
            (n, OptionValue::Str(x)) => opt(format!("{n}=\"{x}\"")),
            (n, OptionValue::Bool(b)) => opt(format!("{n}={b}")),
            (n, OptionValue::Num(x)) => opt(format!("{n}={x}")),
            (_, OptionValue::Present) => {}
        }
    }
    if let Some(ad) = adapter(comp) {
        a.extend(ad.oracle_args(&choice(s, comp)));
    }
    if let Some(x) = size {
        a.extend(["--size".to_owned(), x.to_owned()]);
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

/// What Rune drew for one part: its box relative to the container, fill (sRGB hex, alpha), corner radius
/// and text colour. `ElementInfo` has no stroke or font size.
struct RuneEl {
    rect: [f32; 4],
    fill: (String, f32),
    radius: f32,
    text: Option<String>,
}

/// Rune's own rendering of the current selection: the preview built in a throwaway runtime (the view of
/// the real page cannot read element rects).
fn rune_parts(s: &G, comp: &'static str, dark: bool) -> Vec<(String, RuneEl)> {
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
    let mut all: Vec<_> = rt
        .elements()
        .into_iter()
        .filter_map(|e| {
            let name = e.key.as_deref()?.strip_prefix(prefix)?.to_owned();
            Some((
                name,
                RuneEl {
                    rect: e.rect,
                    fill: (e.fill.hex(), e.fill.a),
                    radius: e.radius,
                    text: e.text_color.map(|c| c.hex()),
                },
            ))
        })
        .collect();
    let c = all
        .iter()
        .find(|(n, _)| n == "container")
        .map_or([0.0; 4], |(_, r)| r.rect);
    for (_, r) in &mut all {
        r.rect = [r.rect[0] - c[0], r.rect[1] - c[1], r.rect[2], r.rect[3]];
    }
    all
}

/// Whether the oracle's `rgb(..)`/`rgba(..)`/`transparent` is within 3 per channel of Rune's hex.
fn same_color(oracle: &str, rune: &(String, f32)) -> bool {
    let n: Vec<f32> = oracle
        .trim_start_matches("rgba(")
        .trim_start_matches("rgb(")
        .trim_end_matches(')')
        .split(',')
        .filter_map(|c| c.trim().parse().ok())
        .collect();
    if n.len() < 3 {
        return rune.1 < 0.02;
    }
    let a = n.get(3).copied().unwrap_or(1.0);
    (a - rune.1).abs() < 0.02
        && (0..3).all(|i| {
            let g = i32::from_str_radix(&rune.0[1 + 2 * i..3 + 2 * i], 16).unwrap_or(0) as f32;
            (g - n[i]).abs() <= 3.0
        })
}

/// The alpha of Rune's `#rrggbb` or `#rrggbbaa` colour.
fn hex_alpha(hex: &str) -> f32 {
    hex.get(7..9)
        .and_then(|a| u8::from_str_radix(a, 16).ok())
        .map_or(1.0, |a| f32::from(a) / 255.0)
}

/// One token row of a part: its property, token and value, and for the properties Rune exposes (fill,
/// corner radius, text colour) whether Rune drew the same, with Rune's value when not.
#[derive(Clone, Debug)]
pub struct Row {
    pub prop: String,
    pub token: String,
    pub value: String,
    pub rune: Option<Result<(), String>>,
}

#[derive(Clone, Debug)]
pub enum Verdict {
    Matches,
    Differs,
    NoPart,
}

#[derive(Clone, Debug)]
pub struct PartReport {
    pub name: String,
    pub oracle_box: [f32; 4],
    pub rune_box: Option<[f32; 4]>,
    pub verdict: Verdict,
    pub rows: Vec<Row>,
}

#[derive(Clone, Debug)]
pub struct Report {
    pub version: String,
    pub parts: Vec<PartReport>,
}

/// Oracle vs Rune for the current selection: each oracle part's box (flagged when it differs by more
/// than 1px), fill, radius and text colour against Rune's, and the tokens it reads. Parts that differ
/// come first.
fn compare(s: &G, comp: &'static str, dark: bool, args: &[String]) -> Result<Report, String> {
    if let Some(why) = errors(comp, &options(s, comp, None)).first() {
        return Err(format!("not defined: {why}"));
    }
    let (version, parts) = crate::oracle::run(comp, args)?;
    let rune = rune_parts(s, comp, dark);
    let c = parts
        .iter()
        .find(|p| p.name == "container")
        .map_or([0.0; 4], |p| p.rect);
    let mut out: Vec<PartReport> = parts
        .iter()
        .map(|p| {
            let o = [p.rect[0] - c[0], p.rect[1] - c[1], p.rect[2], p.rect[3]];
            let got = rune.iter().find(|(n, _)| *n == p.name).map(|(_, r)| r);
            let rows: Vec<Row> = p
                .tokens
                .iter()
                .map(|t| {
                    let rune = got.and_then(|g| match t.prop.as_str() {
                        "fill" => Some(if same_color(&t.value, &g.fill) {
                            Ok(())
                        } else {
                            Err(g.fill.0.clone())
                        }),
                        "color" => g.text.as_ref().map(|c| {
                            if same_color(&t.value, &(c.clone(), hex_alpha(c))) {
                                Ok(())
                            } else {
                                Err(c.clone())
                            }
                        }),
                        "cornerRadius" => Some(if (g.radius - p.radius).abs() <= 1.0 {
                            Ok(())
                        } else {
                            Err(format!("{}px", g.radius))
                        }),
                        _ => None,
                    });
                    Row {
                        prop: t.prop.clone(),
                        token: t.token.clone(),
                        value: t.value.clone(),
                        rune,
                    }
                })
                .collect();
            let verdict = match got {
                None => Verdict::NoPart,
                Some(g)
                    if g.rect.iter().zip(o).all(|(a, b)| (a - b).abs() <= 1.0)
                        && rows.iter().all(|r| !matches!(r.rune, Some(Err(_)))) =>
                {
                    Verdict::Matches
                }
                Some(_) => Verdict::Differs,
            };
            PartReport {
                name: p.name.clone(),
                oracle_box: o,
                rune_box: got.map(|g| g.rect),
                verdict,
                rows,
            }
        })
        .collect();
    let rank = |v: &Verdict| match v {
        Verdict::Differs => 0,
        Verdict::NoPart => 1,
        Verdict::Matches => 2,
    };
    out.sort_by_key(|p| rank(&p.verdict));
    Ok(Report {
        version,
        parts: out,
    })
}

/// The oracle panel's body: per part a header (Status light: matches / differs / no such part in Rune)
/// and its token rows in aligned columns.
fn report_view(t: &'static Tokens, r: &Report) -> El {
    let r1 = |x: f32| (x * 10.0).round() / 10.0;
    let boxed = |b: [f32; 4]| {
        format!(
            "x {} y {} w {} h {}",
            r1(b[0]),
            r1(b[1]),
            r1(b[2]),
            r1(b[3])
        )
    };
    let cell = |s: String, w: f32| caption(t, s).w(w);
    let mut c = col().gap(t.spacing_100).child(caption(
        t,
        format!(
            "oracle {} vs Rune; boxes relative to the container",
            r.version
        ),
    ));
    for (i, p) in r.parts.iter().enumerate() {
        let (variant, word) = match p.verdict {
            Verdict::Matches => (status_light::Variant::Positive, "matches"),
            Verdict::Differs => (status_light::Variant::Negative, "differs"),
            Verdict::NoPart => (status_light::Variant::Notice, "no such part in Rune"),
        };
        let mut head = format!("{} · {word} · oracle {}", p.name, boxed(p.oracle_box));
        if let (Verdict::Differs, Some(b)) = (&p.verdict, p.rune_box) {
            if b.iter().zip(p.oracle_box).any(|(a, o)| (a - o).abs() > 1.0) {
                head.push_str(&format!(", Rune {}", boxed(b)));
            }
        }
        let mut part = col().gap(t.spacing_50).child(
            StatusLight::new(format!("playground/oracle/part/{i}"), head)
                .variant(variant)
                .size(Size::S)
                .build(t),
        );
        for row_ in &p.rows {
            let mark = match &row_.rune {
                None => String::new(),
                Some(Ok(())) => "Rune: same".to_owned(),
                Some(Err(v)) => format!("Rune: differs, {v}"),
            };
            let mut line = row()
                .gap(t.spacing_200)
                .child(cell(row_.prop.clone(), 140.0))
                .child(cell(row_.token.clone(), 300.0))
                .child(cell(row_.value.clone(), 150.0));
            line = line.child(match &row_.rune {
                Some(Err(_)) => text(mark)
                    .font(font::text_style(&t.component_s_regular))
                    .color(t.negative_content_color_default),
                _ => caption(t, mark),
            });
            part = part.child(line);
        }
        c = c.child(part);
    }
    c
}

/// An option's value for the panel: free-form ones (`label`, `icon`) as what the preview shows.
fn shown_value(s: &G, comp: &str, name: &str, v: &OptionValue) -> String {
    match (name, v) {
        ("label", OptionValue::Present) => match extra(s, comp, "label").as_str() {
            "long" => WRAPPED.to_owned(),
            _ => "Label".to_owned(),
        },
        ("icon", OptionValue::Present) => extra(s, comp, "icon"),
        _ => fmt(v),
    }
}

fn fmt(v: &OptionValue) -> String {
    match v {
        OptionValue::Str(s) => (*s).to_owned(),
        OptionValue::Bool(b) => b.to_string(),
        OptionValue::Num(n) => n.to_string(),
        OptionValue::Present => "(set)".to_owned(),
    }
}

/// A section heading. Spectrum has no heading styles: this composes font-size-300, bold-font-weight and
/// title-color (gap, a14.56).
fn heading(t: &Tokens, s: impl Into<String>) -> El {
    let base = &t.component_s_regular;
    let style = font::text_style(&spectrum_rune::tokens::Typography {
        size: t.font_size_300,
        weight: t.bold_font_weight,
        line_height: t.line_height_font_size_300,
        letter_spacing_em: 0.0,
        ..*base
    });
    text(s.into()).font(style).color(t.title_color)
}

fn caption(t: &Tokens, s: impl Into<String>) -> El {
    text(s.into())
        .font(font::text_style(&t.component_s_regular))
        .color(t.neutral_subdued_content_color_default)
}

/// One option: its values as buttons, the chosen one selected, an excluded one disabled. A value that
/// would break an error rule says so; one that would only break an advisory stays available and says so.
fn control(
    t: &'static Tokens,
    comp: &'static str,
    opt: &'static str,
    values: Vec<(String, String)>,
    current: &str,
    default: Option<&str>,
    rules: impl Fn(&str) -> Vec<(Severity, &'static str, bool)>,
) -> El {
    let mut notes = Vec::new();
    let buttons = values.into_iter().map(|(id, shown)| {
        let why = rules(&id);
        let blocked = why.iter().any(|(sev, ..)| *sev == Severity::Error);
        // `inherited`: the current selection already breaks that rule, so it is not this value's doing
        // and the stage says it once.
        for (sev, rule, _) in why.into_iter().filter(|w| !w.2) {
            notes.push(match sev {
                Severity::Error => text(format!("{shown}: not available, Error · {rule}"))
                    .font(font::text_style(&t.component_s_regular))
                    .color(t.negative_content_color_default),
                Severity::Advisory => text(format!("{shown}: Advisory · {rule}"))
                    .font(font::text_style(&t.component_s_regular))
                    // shortcut: Spectrum has no notice text colour; neutral stands in, see a14.56
                    .color(t.neutral_content_color_default),
            });
        }
        let key = format!("{comp}/{opt}");
        let id2 = id.clone();
        let label = if default == Some(id.as_str()) {
            format!("{shown} (default)")
        } else {
            shown
        };
        ActionButton::new(format!("playground/{comp}/{opt}/{id}"))
            .label(label)
            .quiet(true)
            .selected(id == current)
            .disabled(blocked)
            .on_press(move |g: &mut G| {
                g.play.insert(key.clone(), id2.clone());
            })
            .build(t)
    });
    let row_ = row().gap(t.spacing_75).flex_wrap().children(buttons);
    col()
        .gap(t.spacing_75)
        .child(caption(t, opt))
        .child(row_)
        .children(notes)
}

/// A static white or black Button is invisible on the stage's own surface, so it sits on a contrasting
/// one: the darker of the two tokens behind white, the lighter behind black, in either scheme.
/// shortcut: Spectrum has no backdrop token for static colours; these are stand-ins, see a14.56.
fn backdrop(s: &G, comp: &str, t: &Tokens, built: El) -> El {
    let on = match (comp, pick(s, comp, "staticColor", "none")) {
        ("button", c @ ("white" | "black")) => c,
        _ => return built,
    };
    let dark = std::ptr::eq(t, tokens(ColorScheme::Dark, s.scale));
    let (dark_surface, light_surface) = if dark {
        (t.background_layer_2_color, t.neutral_content_color_default)
    } else {
        (t.neutral_content_color_default, t.background_layer_2_color)
    };
    col()
        .key("playground/stage/backdrop")
        .padding(t.spacing_300)
        .fill(if on == "white" {
            dark_surface
        } else {
            light_surface
        })
        .child(built)
}

fn selected(s: &G) -> &'static str {
    CATALOG
        .iter()
        .find(|c| c.name == pick(s, "", "component", "badge"))
        .map_or("badge", |c| c.name)
}

/// Whether the playground has a preview adapter for the component (the rest show the catalog only).
fn previewed(comp: &str) -> bool {
    adapter(comp).is_some()
}

/// How long the options must stay unchanged before the oracle runs by itself.
const ORACLE_DELAY: f64 = 0.3;

/// Runs the oracle once the options have been still for `ORACLE_DELAY`. It blocks the UI for the CLI's
/// run (about 0.15 s warm, 2 s cold). shortcut: no background thread; move it to one if that is felt.
pub fn poll(s: &mut G, t: f64, tx: &mut rune::Tx) -> bool {
    if s.story != "playground" {
        return false;
    }
    let dark = s
        .scheme
        .map_or(s.system_dark.get(), |c| c == ColorScheme::Dark);
    let comp = selected(s);
    if !previewed(comp) {
        return false;
    }
    let args = oracle_args(s, comp, dark);
    if s.oracle.as_ref().is_some_and(|(a, _)| *a == args) {
        s.oracle_due = None;
        return false;
    }
    let delay = std::time::Duration::from_secs_f64(ORACLE_DELAY);
    match &s.oracle_due {
        Some((a, due)) if *a == args && t >= *due => {
            let r = compare(s, comp, dark, &args);
            s.oracle = Some((args, r));
            s.oracle_due = None;
            true
        }
        Some((a, due)) if *a == args => {
            tx.wake_after(std::time::Duration::from_secs_f64(due - t));
            false
        }
        _ => {
            s.oracle_due = Some((args, t + ORACLE_DELAY));
            tx.wake_after(delay);
            false
        }
    }
}

pub fn playground(s: &G, t: &'static Tokens) -> El {
    let comp = selected(s);
    let open = pick(s, "", "picker", "closed") == "open";
    let pick_button = |id: &'static str, name: &str| {
        ActionButton::new(format!("playground/pick/{id}"))
            .label(name.to_owned())
            .quiet(true)
            .selected(id == comp)
            .on_press(move |g: &mut G| {
                g.play.insert("/component".into(), id.into());
                g.play.insert("/picker".into(), "closed".into());
            })
            .build(t)
    };
    let shown_name = adapter(comp).map_or(comp, |a| a.name());
    let mut picker = col()
        .gap(t.spacing_75)
        .child(heading(t, "Component"))
        .child(
            Picker::new("playground/pick")
                .value(shown_name)
                .placeholder("Select a component")
                .size(Size::M)
                .width(280.0)
                .open(open)
                .on_open(move |g: &mut G| {
                    let v = if open { "closed" } else { "open" };
                    g.play.insert("/picker".into(), v.into());
                })
                .build(t),
        );
    if open {
        // shortcut: Spectrum has no Menu or Popover recipe yet (a14.56), so the list is composed of quiet Action
        // buttons on a layer surface, shown in the page flow rather than as an overlay.
        let list = col()
            .gap(t.spacing_75)
            .padding(t.spacing_200)
            .fill(t.background_layer_1_color)
            .child(caption(t, "previewed here"))
            .child(
                row()
                    .gap(t.spacing_75)
                    .flex_wrap()
                    .children(ADAPTERS.iter().map(|a| pick_button(a.id(), a.name()))),
            )
            .child(caption(
                t,
                format!(
                    "catalog only, options and rules without a preview ({})",
                    CATALOG.len() - ADAPTERS.len()
                ),
            ))
            .child(
                row().gap(t.spacing_75).flex_wrap().children(
                    CATALOG
                        .iter()
                        .filter(|c| !previewed(c.name))
                        .map(|c| pick_button(c.name, c.name)),
                ),
            );
        picker = picker.child(list);
    }
    let now_reasons = reasons(comp, &options(s, comp, None));
    let mut controls = col().gap(t.spacing_200).child(heading(t, "Options"));
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
            .then(|| ("none".to_owned(), "unset".to_owned()));
        let values = unset
            .into_iter()
            .chain(def.values.iter().map(|l| (lit_str(l), lit_str(l))))
            .collect();
        let dflt = default_of(def);
        controls = controls.child(control(
            t,
            comp,
            def.name,
            values,
            &current,
            Some(&dflt),
            |v| {
                inherit(
                    &now_reasons,
                    reasons(comp, &options(s, comp, Some((def.name, v)))),
                )
            },
        ));
    }
    let mut state_control = None;
    for (opt, presets) in extras(comp) {
        let current = extra(s, comp, opt);
        let values = presets
            .iter()
            .map(|(id, shown)| (id.to_string(), shown.to_string()))
            .collect();
        let c = control(t, comp, opt, values, &current, None, |v| {
            if opt == "width" || opt == "state" {
                Vec::new()
            } else {
                inherit(
                    &now_reasons,
                    reasons(comp, &options(s, comp, Some((opt, v)))),
                )
            }
        });
        if opt == "state" {
            state_control = Some(c);
        } else {
            controls = controls.child(c);
        }
    }

    let now = options(s, comp, None);
    let broken = errors(comp, &now);
    let (reported, shown) = if !previewed(comp) {
        (
            Vec::new(),
            text(format!(
                "Not previewed: the playground has no adapter for {comp} yet. The options and rules are the catalog's."
            ))
            .font(font::text_style(&t.component_s_regular))
            .color(t.neutral_subdued_content_color_default),
        )
    } else if broken.is_empty() {
        let (reported, built) = preview(s, comp, t);
        let built = backdrop(s, comp, t, built);
        if pick(s, "", "compare", "off") == "on" {
            let dark_now = std::ptr::eq(t, tokens(ColorScheme::Dark, s.scale));
            let other = tokens(scheme(!dark_now), s.scale);
            let (_, alt) = preview_keyed(s, comp, other, "playground/alt");
            let side = |label: &str, t: &Tokens, e: El| {
                col()
                    .gap(t.spacing_75)
                    .padding(t.spacing_300)
                    .fill(t.background_layer_1_color)
                    .child(
                        text(label.to_owned())
                            .font(font::text_style(&t.component_s_regular))
                            .color(t.neutral_subdued_content_color_default),
                    )
                    .child(e)
            };
            let names = if dark_now {
                ("dark", "light")
            } else {
                ("light", "dark")
            };
            (
                reported,
                row()
                    .gap(t.spacing_300)
                    .flex_wrap()
                    .child(side(names.0, t, built))
                    .child(side(names.1, other, backdrop(s, comp, other, alt))),
            )
        } else {
            (reported, built)
        }
    } else {
        (
            Vec::new(),
            text(format!("Not defined: {}", broken.join("; ")))
                .color(t.negative_content_color_default),
        )
    };
    let mut panel = col()
        .gap(t.spacing_50)
        .child(heading(t, "Builder options"))
        .child(caption(
            t,
            "every catalog option, as the builder's options() reports it",
        ));
    let mut names: Vec<&str> = component(comp)
        .unwrap()
        .options
        .iter()
        .map(|d| d.name)
        .collect();
    for (n, _) in &now {
        if !names.contains(n) {
            names.push(n);
        }
    }
    for name in names {
        let chosen = now.iter().find(|(n, _)| *n == name);
        let is_reported = reported.iter().any(|(n, _)| *n == name);
        let line = match chosen {
            Some((n, v)) => caption(
                t,
                format!(
                    "{n} = {}{}",
                    shown_value(s, comp, n, v),
                    if is_reported { "" } else { " · not reported" }
                ),
            ),
            None => text(format!("{name}: not reported"))
                .font(font::text_style(&t.component_s_regular))
                .color(t.disabled_content_color),
        };
        panel = panel.child(line);
    }
    let mut rules = col().gap(t.spacing_50).child(heading(t, "Catalog rules"));
    let c = component(comp).unwrap();
    if c.rules.is_empty() {
        rules = rules.child(caption(
            t,
            "none: the catalog has no rules for this component",
        ));
    }
    let broken_now = check(comp, &now);
    for (i, r) in c.rules.iter().enumerate() {
        let hit = broken_now.iter().any(|v| std::ptr::eq(v.rule, r));
        let (variant, state) = match (hit, r.severity) {
            (true, Severity::Error) => (status_light::Variant::Negative, "active"),
            (true, Severity::Advisory) => (status_light::Variant::Notice, "advisory"),
            _ => (status_light::Variant::Neutral, "inactive"),
        };
        rules = rules.child(
            StatusLight::new(
                format!("playground/rule/{i}"),
                format!(
                    "{state} · {:?} · {}",
                    r.severity,
                    r.description.unwrap_or("")
                ),
            )
            .variant(variant)
            .size(Size::S)
            .build(t),
        );
    }
    let oracle = if !previewed(comp) {
        col().child(caption(
            t,
            "oracle: nothing to compare, Rune has no preview of this component here",
        ))
    } else {
        let dark = std::ptr::eq(t, tokens(ColorScheme::Dark, s.scale));
        let args = oracle_args(s, comp, dark);
        let mut oracle = col().gap(t.spacing_50);
        if let Some(c) = state_control {
            oracle = oracle.child(c).child(caption(
            t,
            "rest, hover and down also show in the preview; keyboard focus is oracle only (Rune draws its ring only on real keyboard focus: Tab to the preview)",
        ));
        }
        oracle = oracle.child(
            ActionButton::new("playground/oracle/run")
                .label("Retry oracle (design-data CLI)")
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
                "tokens each part reads, from the oracle: runs when the options stop changing",
            )),
            Some((ran, _)) if *ran != args => oracle.child(caption(
                t,
                "the options changed: the oracle runs when they stop changing",
            )),
            Some((_, Ok(report))) => oracle.child(report_view(t, report)),
            Some((_, Err(e))) => oracle.child(caption(t, format!("oracle failed: {e}"))),
        };
        oracle
    };
    let name = adapter(comp).map_or(comp, |a| a.name());
    let summary = now
        .iter()
        .fold(name.to_owned(), |a, (n, v)| format!("{a} · {n} {}", fmt(v)));
    let compare = pick(s, "", "compare", "off") == "on";
    let stage = col()
        .key("playground/stage")
        .w_full()
        .fill(t.background_layer_1_color)
        .padding(t.spacing_300)
        .gap(t.spacing_100)
        .sticky_top(0.0)
        .child(
            row()
                .gap(t.spacing_300)
                .center_y()
                .flex_wrap()
                .child(caption(t, summary))
                .child(
                    Switch::new("playground/compare")
                        .label("Compare light and dark")
                        .selected(compare)
                        .on_toggle(move |g: &mut G| {
                            let v = if compare { "off" } else { "on" };
                            g.play.insert("/compare".into(), v.into());
                        })
                        .build(t),
                ),
        )
        .child(shown);
    col()
        .w_full()
        .gap(t.spacing_400)
        .child(picker)
        .child(stage)
        .child(
            row()
                .gap(t.spacing_500)
                .flex_wrap()
                .child(controls.w(520.0))
                .child(
                    col().gap(t.spacing_400).child(panel).child(rules).child(
                        col()
                            .gap(t.spacing_50)
                            .child(heading(t, "Oracle"))
                            .child(oracle),
                    ),
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
        for ad in ADAPTERS {
            let comp = ad.id();
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
        for ad in ADAPTERS {
            let comp = ad.id();
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
                        // Button and Picker builders do not report their label (no visual option of their own).
                        if matches!(comp, "button" | "picker") && name == "label" {
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
        for ad in ADAPTERS {
            let comp = ad.id();
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
                "outline: not available, Error · Outline style is only defined for primary and secondary."
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

    /// Writes the playground in a few states to `target/gallery/playground-*.png` (for PRs). Needs the
    /// oracle CLI (`DESIGN_DATA`) for the oracle panel; run with `--release -- --ignored`.
    #[test]
    #[ignore = "slow unoptimised; needs the oracle CLI"]
    fn render_playground_shots() {
        use rune::{Env, Runtime};
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/gallery");
        std::fs::create_dir_all(&dir).unwrap();
        let shots: [(&str, &[(&str, &str)]); 13] = [
            ("badge", &[]),
            (
                "button-hover",
                &[
                    ("/component", "button"),
                    ("button/variant", "accent"),
                    ("button/state", "hover"),
                    ("button/icon", "check"),
                ],
            ),
            (
                "button-excluded",
                &[
                    ("/component", "button"),
                    ("button/variant", "accent"),
                    ("button/style", "outline"),
                ],
            ),
            ("catalog-only", &[("/component", "tabs")]),
            (
                "selector-open",
                &[("/component", "button"), ("/picker", "open")],
            ),
            (
                "status-light-s",
                &[("/component", "status-light"), ("status-light/size", "s")],
            ),
            (
                "status-light-xl",
                &[("/component", "status-light"), ("status-light/size", "xl")],
            ),
            ("picker", &[("/component", "picker")]),
            (
                "picker-help",
                &[
                    ("/component", "picker"),
                    ("picker/isRequired", "true"),
                    ("picker/description", "help"),
                ],
            ),
            (
                "picker-error",
                &[
                    ("/component", "picker"),
                    ("picker/isError", "true"),
                    ("picker/description", "help"),
                ],
            ),
            (
                "button-compare",
                &[
                    ("/component", "button"),
                    ("button/variant", "accent"),
                    ("/compare", "on"),
                ],
            ),
            (
                "button-static-white",
                &[("/component", "button"), ("button/staticColor", "white")],
            ),
            (
                "button-static-black",
                &[("/component", "button"), ("button/staticColor", "black")],
            ),
        ];
        for (name, play) in shots {
            for dark in [false, true] {
                let mut s = crate::init();
                s.story = "playground";
                s.scheme = Some(scheme(dark));
                for (k, v) in play {
                    s.play.insert((*k).into(), (*v).into());
                }
                let (w, h) = (1100u32, 2500u32);
                let mut rt =
                    Runtime::new(s, crate::view, (w as f32, h as f32), Env::new(dark, 1.0));
                rt.poll = Some(poll);
                for _ in 0..12 {
                    rt.frame(0.1);
                }
                let tag = if dark { "dark" } else { "light" };
                crate::pixels::Pixels::of(&rt, (w, h))
                    .write_png(&dir.join(format!("playground-{name}-{tag}.png")));
            }
        }
    }

    fn tree_of(play: &[(&str, &str)]) -> String {
        use rune::{Env, Runtime};
        let mut s = crate::init();
        s.story = "playground";
        for (k, v) in play {
            s.play.insert((*k).into(), (*v).into());
        }
        let mut rt = Runtime::new(s, crate::view, (1100.0, 2500.0), Env::new(false, 1.0));
        rt.frame(0.05);
        rt.accessibility_json().to_string()
    }

    /// The catalog default is marked, an option without one reads "unset", and the options panel shows
    /// the real label text.
    #[test]
    fn defaults_are_marked_and_the_panel_shows_real_values() {
        let tree = tree_of(&[("/component", "button")]);
        assert!(tree.contains("fill (default)"), "{tree}");
        assert!(tree.contains("unset (default)"), "{tree}");
        assert!(tree.contains("label = Label"), "{tree}");
        assert!(tree.contains("Builder options"), "{tree}");
    }

    /// When the current selection is itself excluded, the reason is said once (on the stage), not under
    /// every value of every control.
    #[test]
    fn an_excluded_selection_does_not_repeat_its_reason() {
        let tree = tree_of(&[
            ("/component", "button"),
            ("button/variant", "accent"),
            ("button/style", "outline"),
        ]);
        assert_eq!(
            tree.matches("Not defined: Outline style").count(),
            1,
            "{tree}"
        );
        assert_eq!(
            tree.matches("not available, Error · Outline style").count(),
            0
        );
    }

    #[test]
    fn oracle_colours_compare_within_three_per_channel() {
        let rune = ("#3b63fb".to_owned(), 1.0);
        assert!(same_color("rgb(59, 99, 251)", &rune));
        assert!(same_color("rgb(61, 97, 250)", &rune));
        assert!(!same_color("rgb(59, 99, 240)", &rune));
        // A translucent text colour: Rune's hex carries the alpha, the oracle's rgba its own.
        let text = "#000000ed".to_owned();
        assert!(same_color(
            "rgba(0, 0, 0, 0.93)",
            &(text.clone(), hex_alpha(&text))
        ));
        assert!(same_color("transparent", &("#000000".to_owned(), 0.0)));
        assert!(!same_color("transparent", &rune));
    }

    /// A static white or black Button is previewed on a surface that contrasts with it.
    #[test]
    fn static_colour_buttons_get_a_contrasting_backdrop() {
        use rune::{Env, Runtime};
        for (static_color, dark) in [
            ("white", false),
            ("white", true),
            ("black", false),
            ("black", true),
        ] {
            let mut s = crate::init();
            s.story = "playground";
            s.scheme = Some(scheme(dark));
            s.play.insert("/component".into(), "button".into());
            s.play
                .insert("button/staticColor".into(), static_color.into());
            let mut rt = Runtime::new(s, crate::view, (1100.0, 1500.0), Env::new(dark, 1.0));
            for _ in 0..10 {
                rt.frame(0.05);
            }
            let fill = |key: &str| {
                rt.elements()
                    .iter()
                    .find(|e| e.key.as_deref() == Some(key))
                    .unwrap()
                    .fill
            };
            let back = fill("playground/stage/backdrop");
            let lum = |c: rune::Color| c.r + c.g + c.b;
            if static_color == "white" {
                assert!(lum(back) < 1.5, "white needs a dark backdrop: {back:?}");
            } else {
                assert!(lum(back) > 1.5, "black needs a light backdrop: {back:?}");
            }
        }
    }

    /// "Compare light and dark" shows a second copy under the other scheme's tokens.
    #[test]
    fn compare_shows_a_copy_under_the_other_scheme() {
        use rune::{Env, Runtime};
        for dark in [false, true] {
            let mut s = crate::init();
            s.story = "playground";
            s.scheme = Some(scheme(dark));
            s.play.insert("/component".into(), "button".into());
            s.play.insert("/compare".into(), "on".into());
            let mut rt = Runtime::new(s, crate::view, (1100.0, 1500.0), Env::new(dark, 1.0));
            for _ in 0..10 {
                rt.frame(0.05);
            }
            let fill = |key: &str| {
                rt.elements()
                    .iter()
                    .find(|e| e.key.as_deref() == Some(key))
                    .unwrap()
                    .fill
            };
            let (here, there) = (
                fill("playground/preview/container"),
                fill("playground/alt/container"),
            );
            assert_ne!(here, there, "dark={dark}: both copies look the same");
        }
    }

    fn press(rt: &mut rune::Runtime<G>, key: &str) {
        let (x, y) = rt.find(key).unwrap();
        rt.pointer_move(x, y);
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(0.05);
    }

    /// The component Picker opens a list, choosing from it selects the component and closes the list.
    #[test]
    fn the_component_picker_opens_and_selects() {
        use rune::{Env, Runtime};
        let mut s = crate::init();
        s.story = "playground";
        let mut rt = Runtime::new(s, crate::view, (1100.0, 1400.0), Env::new(false, 1.0));
        rt.frame(0.05);
        assert!(
            rt.find("playground/pick/tabs/container").is_none(),
            "closed: no list"
        );
        press(&mut rt, "playground/pick/field");
        assert!(
            rt.find("playground/pick/tabs/container").is_some(),
            "open: the catalog is listed"
        );
        press(&mut rt, "playground/pick/tabs/container");
        assert_eq!(rt.state.play["/component"], "tabs");
        assert!(
            rt.find("playground/pick/tabs/container").is_none(),
            "closed again"
        );
        let tree = rt.accessibility_json().to_string();
        assert!(tree.contains("Not previewed"), "{tree}");
    }

    /// A rule the current options break is listed as active, the others as inactive.
    #[test]
    fn catalog_rules_show_their_live_state() {
        use rune::{Env, Runtime};
        let text_for = |variant: &str| {
            let mut s = crate::init();
            s.story = "playground";
            s.play.insert("/component".into(), "button".into());
            s.play.insert("button/variant".into(), variant.into());
            s.play.insert("button/style".into(), "outline".into());
            let mut rt = Runtime::new(s, crate::view, (1100.0, 1400.0), Env::new(false, 1.0));
            rt.frame(0.05);
            rt.accessibility_json().to_string()
        };
        assert!(text_for("accent").contains("active · Error · Outline style"));
        assert!(text_for("primary").contains("inactive · Error · Outline style"));
    }

    /// Needs the oracle CLI in the oracle.toml range (`DESIGN_DATA=... cargo test -- --ignored`).
    #[test]
    #[ignore]
    fn oracle_runs_every_component_at_its_defaults() {
        for ad in ADAPTERS {
            let comp = ad.id();
            let s = crate::init();
            let args = oracle_args(&s, comp, false);
            let r = compare(&s, comp, false, &args).expect(comp);
            assert!(r.parts.iter().any(|p| p.name == "container"), "{comp}");
            assert!(
                r.parts.iter().any(|p| !p.rows.is_empty()),
                "{comp}: no tokens"
            );
            let bad: Vec<_> = r
                .parts
                .iter()
                .filter(|p| !matches!(p.verdict, Verdict::Matches))
                .map(|p| &p.name)
                .collect();
            assert!(bad.is_empty(), "{comp}: {bad:?}");
        }
    }

    /// Static white and black Buttons match the oracle too (translucent fills and text colours).
    #[test]
    #[ignore]
    fn oracle_matches_static_colour_buttons() {
        for c in ["white", "black"] {
            let mut s = crate::init();
            s.play.insert("/component".into(), "button".into());
            s.play.insert("button/staticColor".into(), c.into());
            let args = oracle_args(&s, "button", false);
            let r = compare(&s, "button", false, &args).expect(c);
            let bad: Vec<_> = r
                .parts
                .iter()
                .filter(|p| !matches!(p.verdict, Verdict::Matches))
                .map(|p| &p.name)
                .collect();
            assert!(bad.is_empty(), "{c}: {bad:?}");
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
            let r = compare(&s, "button", false, &args).expect(state);
            r.parts
                .iter()
                .flat_map(|p| p.rows.iter().map(|r| (r.prop.clone(), r.token.clone())))
                .collect::<Vec<_>>()
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
        assert!(tree.contains("base-height-"), "{tree}");
    }

    /// The oracle runs by itself once the options have been still for the delay, and not before.
    /// It needs no CLI: a missing or out-of-range one is an error result, still a run.
    #[test]
    fn oracle_runs_after_the_options_settle() {
        use rune::{Env, Runtime};
        let mut s = crate::init();
        s.story = "playground";
        let mut rt = Runtime::new(s, crate::view, (1100.0, 900.0), Env::new(false, 1.0));
        rt.poll = Some(poll);
        rt.frame(0.05);
        assert!(rt.state.oracle.is_none(), "ran before the delay");
        for _ in 0..4 {
            rt.frame(0.1);
        }
        assert!(rt.state.oracle.is_some(), "did not run after the delay");
    }

    /// The preview stage stays at the top while the page scrolls.
    #[test]
    fn the_stage_stays_pinned_when_the_page_scrolls() {
        use rune::{Env, Runtime};
        let mut s = crate::init();
        s.story = "playground";
        let mut rt = Runtime::new(s, crate::view, (1100.0, 400.0), Env::new(false, 1.0));
        rt.frame(0.05);
        let y = |rt: &Runtime<G>, key: &str| {
            rt.elements()
                .iter()
                .find(|e| e.key.as_deref() == Some(key))
                .unwrap()
                .rect[1]
        };
        let ctl = y(&rt, "playground/pick/field");
        let top = y(&rt, "gallery/body");
        assert!(
            y(&rt, "playground/stage") > top + 80.0,
            "the stage starts below the picker"
        );
        for _ in 0..10 {
            rt.wheel(500.0, 300.0, -200.0);
            rt.frame(0.05);
        }
        assert!(
            y(&rt, "playground/pick/field") < ctl - 100.0,
            "the page did not scroll"
        );
        assert!(
            y(&rt, "playground/stage") - top < 80.0,
            "the stage scrolled away"
        );
    }

    /// The state control drives the Button preview: hover and down fill differently from rest.
    #[test]
    fn the_state_control_drives_the_button_preview() {
        use rune::{Env, Runtime};
        let fill_at = |state: &str| {
            let mut s = crate::init();
            s.story = "playground";
            s.play.insert("/component".into(), "button".into());
            s.play.insert("button/state".into(), state.into());
            let mut rt = Runtime::new(s, crate::view, (1100.0, 900.0), Env::new(false, 1.0));
            for _ in 0..10 {
                rt.frame(0.05);
            }
            let e = rt.elements();
            e.iter()
                .find(|e| e.key.as_deref() == Some("playground/preview/container"))
                .unwrap()
                .fill
        };
        assert_ne!(fill_at("rest"), fill_at("hover"));
        assert_ne!(fill_at("rest"), fill_at("down"));
        assert_eq!(fill_at("keyboard-focus"), fill_at("rest"));
    }

    /// Every catalog component builds in the playground, previewed or not, without the oracle.
    #[test]
    fn every_catalog_component_builds() {
        use rune::{Env, Runtime};
        for c in CATALOG {
            let mut s = crate::init();
            s.story = "playground";
            s.play.insert("/component".into(), c.name.into());
            let mut rt = Runtime::new(s, crate::view, (1100.0, 900.0), Env::new(false, 1.0));
            rt.poll = Some(poll);
            for _ in 0..10 {
                rt.frame(0.1);
            }
            let tree = rt.accessibility_json().to_string();
            assert!(tree.contains("Catalog rules"), "{}", c.name);
            assert_eq!(
                tree.contains("Not previewed"),
                !previewed(c.name),
                "{}",
                c.name
            );
        }
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
