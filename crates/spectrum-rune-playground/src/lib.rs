//! The Playground story: a control per catalog option, a live preview, the options the builder reports,
//! and the catalog's constraint rules. Values come from the generated catalog (`constraints::component`);
//! a value whose choice would break an `error` rule is greyed out and the reason is listed. Each component
//! needs only an adapter (`preview`) from the chosen values to its builder. Dev-only: the token panel runs
//! the oracle CLI (`oracle.rs`) on demand.

pub mod adapters;
pub mod oracle;

use crate::adapters::{adapter, Choice, ADAPTERS, WRAPPED};
use rune::{col, el, row, text, El};
use spectrum_rune::action_button::ActionButton;
use spectrum_rune::constraints::{
    check, component, Lit, OptionDef, OptionValue, Severity, COMPONENTS as CATALOG,
};
use spectrum_rune::font;
use spectrum_rune::menu::{Menu, MenuItem, Selection};
use spectrum_rune::picker::Picker;
use spectrum_rune::status_light::{self, StatusLight};
use spectrum_rune::switch::Switch;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale, Tokens};
use spectrum_rune::Size;

type Opts = crate::adapters::Opts;

/// The CLI arguments an oracle run used, and its report or error.
pub type OracleRun = (Vec<String>, Result<Report, String>);

/// Everything the playground keeps between frames. A host holds one and exposes it through [`Host`].
#[derive(Clone)]
pub struct Play {
    /// The chosen values, keyed `component/option` (the playground's own `/component`, `/picker`, `/compare`).
    pub values: std::collections::HashMap<String, String>,
    /// `None` follows the system until the host's scheme switch is used.
    pub scheme: Option<ColorScheme>,
    pub scale: Scale,
    /// The last oracle run: the CLI arguments it ran with, and its report or error.
    pub oracle: Option<OracleRun>,
    /// The args an auto-run is waiting to run with, and when (`poll`'s clock): the debounce.
    pub oracle_due: Option<(Vec<String>, f64)>,
    /// The system scheme as the view last saw it, for `poll` (which has no `Cx`).
    pub system_dark: std::cell::Cell<bool>,
}

impl Default for Play {
    fn default() -> Self {
        Play {
            values: Default::default(),
            scheme: None,
            scale: Scale::Desktop,
            oracle: None,
            oracle_due: None,
            system_dark: std::cell::Cell::new(false),
        }
    }
}

/// An app that hosts the playground: it owns a [`Play`], and the playground's buttons reach it through here.
pub trait Host: 'static {
    fn play(&self) -> &Play;
    fn play_mut(&mut self) -> &mut Play;
}

/// Free-form options and the request width have no catalog values: a few presets each (the adapter's).
pub fn extras(comp: &str) -> crate::adapters::Extras {
    adapter(comp).map_or_else(Vec::new, |a| a.extras())
}

pub fn lit_str(l: &Lit) -> String {
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

fn pick<'a>(s: &'a Play, comp: &str, opt: &str, default: &'a str) -> &'a str {
    s.values
        .get(&format!("{comp}/{opt}"))
        .map_or(default, String::as_str)
}

fn extra(s: &Play, comp: &str, opt: &str) -> String {
    let presets = extras(comp);
    let (_, list) = presets.iter().find(|(o, _)| *o == opt).unwrap();
    pick(s, comp, opt, list[0].0).to_owned()
}

/// The chosen values under the catalog's option names, with `over` (option, value) replacing one choice.
pub fn options(s: &Play, comp: &str, over: Option<(&str, &str)>) -> Opts {
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

pub fn errors(comp: &str, o: &Opts) -> Vec<&'static str> {
    check(comp, o)
        .into_iter()
        .filter(|v| v.severity == Severity::Error)
        .map(|v| v.description.unwrap_or("excluded option combination"))
        .collect()
}

/// Every rule the options break, errors and advisories, as (severity, description).
pub fn reasons(comp: &str, o: &Opts) -> Vec<(Severity, &'static str)> {
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
pub fn preview(s: &Play, comp: &str, t: &Tokens) -> (Opts, El) {
    preview_keyed(s, comp, t, "playground/preview")
}

/// `preview` with the key its parts are named under (`<key>/container`, ...): a second copy needs its own.
fn preview_keyed(s: &Play, comp: &str, t: &Tokens, key: &str) -> (Opts, El) {
    adapter(comp)
        .expect("a previewed component")
        .build(&choice(s, comp), t, key)
}

/// The chosen values of `comp`: its catalog options (one source of truth: the catalog values and defaults
/// `options` derives) and the preset chosen for each of its extras.
fn choice(s: &Play, comp: &str) -> Choice {
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
pub fn oracle_args(s: &Play, comp: &str, dark: bool) -> Vec<String> {
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

pub struct Preview {
    s: Play,
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
pub struct RuneEl {
    rect: [f32; 4],
    fill: (String, f32),
    radius: f32,
    text: Option<String>,
}

/// Rune's own rendering of the current selection: the preview built in a throwaway runtime (the view of
/// the real page cannot read element rects).
pub fn rune_parts(s: &Play, comp: &'static str, dark: bool) -> Vec<(String, RuneEl)> {
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
pub fn same_color(oracle: &str, rune: &(String, f32)) -> bool {
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
pub fn hex_alpha(hex: &str) -> f32 {
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
pub fn compare(
    s: &Play,
    comp: &'static str,
    dark: bool,
    args: &[String],
) -> Result<Report, String> {
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
pub fn shown_value(s: &Play, comp: &str, name: &str, v: &OptionValue) -> String {
    match (name, v) {
        ("label", OptionValue::Present) => match extra(s, comp, "label").as_str() {
            "long" => WRAPPED.to_owned(),
            _ => "Label".to_owned(),
        },
        ("icon", OptionValue::Present) => extra(s, comp, "icon"),
        _ => fmt(v),
    }
}

pub fn fmt(v: &OptionValue) -> String {
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
fn control<H: Host>(
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
            .on_press(move |g: &mut H| {
                g.play_mut().values.insert(key.clone(), id2.clone());
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
fn backdrop(s: &Play, comp: &str, t: &Tokens, built: El) -> El {
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

/// The component list's items in order: the previewed components, then the catalog-only ones, as (id, shown).
pub fn menu_order() -> Vec<(&'static str, &'static str)> {
    ADAPTERS
        .iter()
        .map(|a| (a.id(), a.name()))
        .chain(
            CATALOG
                .iter()
                .filter(|c| !previewed(c.name))
                .map(|c| (c.name, c.name)),
        )
        .collect()
}

pub fn selected(s: &Play) -> &'static str {
    CATALOG
        .iter()
        .find(|c| c.name == pick(s, "", "component", "badge"))
        .map_or("badge", |c| c.name)
}

/// Whether the playground has a preview adapter for the component (the rest show the catalog only).
pub fn previewed(comp: &str) -> bool {
    adapter(comp).is_some()
}

/// How long the options must stay unchanged before the oracle runs by itself.
const ORACLE_DELAY: f64 = 0.3;

/// Runs the oracle once the options have been still for `ORACLE_DELAY`. It blocks the UI for the CLI's
/// run (about 0.15 s warm, 2 s cold). shortcut: no background thread; move it to one if that is felt.
pub fn poll(s: &mut Play, t: f64, tx: &mut rune::Tx) -> bool {
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

pub fn playground<H: Host>(host: &H, t: &'static Tokens) -> El {
    let s = host.play();
    let comp = selected(s);
    let open = pick(s, "", "picker", "closed") == "open";
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
                .on_open(move |g: &mut H| {
                    let v = if open { "closed" } else { "open" };
                    g.play_mut().values.insert("/picker".into(), v.into());
                })
                .build(t),
        );
    if open {
        // The list is the Spectrum Menu in the page flow (not an overlay), scrolling: the previewed components
        // first, then the catalog-only ones. A menu has no surface of its own: a layer behind it.
        let mut menu = Menu::new("playground/pick-menu")
            .label("Component")
            .selection(Selection::Single)
            .on_choose(|g: &mut H, id: &str| {
                g.play_mut()
                    .values
                    .insert("/component".into(), id.to_owned());
                g.play_mut()
                    .values
                    .insert("/picker".into(), "closed".into());
            })
            .on_dismiss(|g: &mut H| {
                g.play_mut()
                    .values
                    .insert("/picker".into(), "closed".into());
            });
        for (id, name) in menu_order() {
            menu = menu.item(MenuItem::new(id, name).selected(id == comp));
        }
        picker = picker.child(
            el().key("playground/pick-list")
                .w(280.0)
                .h(320.0)
                .fill(t.background_layer_1_color)
                .radius(t.corner_radius_500)
                .padding(t.spacing_100)
                .scroll(rune::ScrollSpec {
                    stick_bottom: false,
                })
                .child(menu.build(t)),
        );
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
        controls = controls.child(control::<H>(
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
        let c = control::<H>(t, comp, opt, values, &current, None, |v| {
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
                    move |g: &mut H| {
                        let r = compare(g.play(), comp, dark, &args);
                        g.play_mut().oracle = Some((args.clone(), r));
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
                        .on_toggle(move |g: &mut H| {
                            let v = if compare { "off" } else { "on" };
                            g.play_mut().values.insert("/compare".into(), v.into());
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
