//! An options playground for Rune component libraries, hosted in Rune's `App::playground`: one story per component, the
//! live preview on the stage, and in the side column a control per option (a value a rule excludes is greyed out with the
//! reason), the options the builder reports, the rules, and an optional oracle comparison. A library implements
//! [`model::Library`] (its catalog and an [`adapters::Adapter`] per previewable component) and gives
//! [`stories`] to `Playground::story`. The chrome is drawn with `rune-kit`'s neutral theme, so it knows no design system.
//! Dev-only.

pub mod adapters;
pub mod model;
pub mod oracle;

use crate::adapters::{Adapter, Choice, WRAPPED};
use crate::model::{Env, Library, Lit, OptionDef, OptionValue, Severity};
use rune::{col, el, row, text, El};
use rune_kit::{Kit, Switch};

type Opts = crate::adapters::Opts;

/// The CLI arguments an oracle run used, and its report or error.
pub type OracleRun = (Vec<String>, Result<Report, String>);

/// Everything the playground keeps between frames. A host holds one and exposes it through [`Host`].
#[derive(Clone)]
pub struct Play {
    /// The component library being played with.
    pub lib: &'static dyn Library,
    /// The chosen values, keyed `component/option` (the playground's own `/component`, `/picker`, `/compare`).
    pub values: std::collections::HashMap<String, String>,
    /// The compact/touch scale (the side column's switch).
    pub mobile: bool,
    /// The last oracle run: the CLI arguments it ran with, and its report or error.
    pub oracle: Option<OracleRun>,
    /// The args an auto-run is waiting to run with, and when (`poll`'s clock): the debounce.
    pub oracle_due: Option<(Vec<String>, f64)>,
    /// The system scheme as the view last saw it, for `poll` (which has no `Cx`).
    pub system_dark: std::cell::Cell<bool>,
}

impl Play {
    pub fn new(lib: &'static dyn Library) -> Play {
        Play {
            lib,
            values: Default::default(),
            mobile: false,
            oracle: None,
            oracle_due: None,
            system_dark: std::cell::Cell::new(false),
        }
    }
    pub fn env(&self, dark: bool) -> Env {
        Env {
            dark,
            mobile: self.mobile,
        }
    }
    /// The adapter that previews `comp`, if it has one.
    pub fn adapter(&self, comp: &str) -> Option<&'static dyn Adapter> {
        self.lib.adapters().iter().copied().find(|a| a.id() == comp)
    }
}

/// An app that hosts the playground: it owns a [`Play`], and the playground's buttons reach it through here.
pub trait Host: 'static {
    fn play(&self) -> &Play;
    fn play_mut(&mut self) -> &mut Play;
}

/// Free-form options and the request width have no catalog values: a few presets each (the adapter's).
pub fn extras(s: &Play, comp: &str) -> crate::adapters::Extras {
    s.adapter(comp).map_or_else(Vec::new, |a| a.extras())
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
    let presets = extras(s, comp);
    let (_, list) = presets.iter().find(|(o, _)| *o == opt).unwrap();
    pick(s, comp, opt, list[0].0).to_owned()
}

/// The chosen values under the catalog's option names, with `over` (option, value) replacing one choice.
pub fn options(s: &Play, comp: &str, over: Option<(&str, &str)>) -> Opts {
    let mut o: Opts = Vec::new();
    for def in s.lib.options(comp).iter().filter(|d| !d.values.is_empty()) {
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
            _ if extras(s, comp).iter().any(|(e, _)| *e == opt) => extra(s, comp, opt),
            _ => continue,
        };
        if chosen != "none" {
            o.push((opt, OptionValue::Present));
        }
    }
    o
}

pub fn errors(s: &Play, comp: &str, o: &Opts) -> Vec<&'static str> {
    s.lib
        .check(comp, o)
        .into_iter()
        .filter(|v| v.severity == Severity::Error)
        .map(|v| v.description.unwrap_or("excluded option combination"))
        .collect()
}

/// Every rule the options break, errors and advisories, as (severity, description).
pub fn reasons(s: &Play, comp: &str, o: &Opts) -> Vec<(Severity, &'static str)> {
    s.lib
        .check(comp, o)
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
pub fn preview(s: &Play, comp: &str, env: Env) -> (Opts, El) {
    preview_keyed(s, comp, env, "playground/preview")
}

/// `preview` with the key its parts are named under (`<key>/container`, ...): a second copy needs its own.
fn preview_keyed(s: &Play, comp: &str, env: Env, key: &str) -> (Opts, El) {
    s.adapter(comp)
        .expect("a previewed component")
        .build(&choice(s, comp), env, key)
}

/// The chosen values of `comp`: its catalog options (one source of truth: the catalog values and defaults
/// `options` derives) and the preset chosen for each of its extras.
fn choice(s: &Play, comp: &str) -> Choice {
    Choice {
        options: options(s, comp, None),
        extras: extras(s, comp)
            .iter()
            .map(|(opt, _)| (*opt, extra(s, comp, opt)))
            .collect(),
    }
}

/// The oracle's arguments for the chosen values: empty when the library has no oracle.
pub fn oracle_args(s: &Play, comp: &str, dark: bool) -> Vec<String> {
    s.lib
        .oracle()
        .map_or_else(Vec::new, |o| o.args(comp, &choice(s, comp), s.env(dark)))
}

pub struct Preview {
    s: Play,
    comp: &'static str,
    env: Env,
}

fn preview_view(p: &Preview, _: &mut rune::Cx<Preview>) -> El {
    col().child(preview(&p.s, p.comp, p.env).1)
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
    use rune::{Env as RuneEnv, Runtime};
    let state = Preview {
        s: s.clone(),
        comp,
        env: s.env(dark),
    };
    let mut rt = Runtime::new(state, preview_view, (800.0, 400.0), RuneEnv::new(dark, 1.0));
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
    if let Some(why) = errors(s, comp, &options(s, comp, None)).first() {
        return Err(format!("not defined: {why}"));
    }
    let oracle = s.lib.oracle().ok_or("the library has no oracle")?;
    let (version, parts) = oracle.run(comp, args)?;
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

/// A coloured dot and a line of text: a status.
fn status(k: &Kit, key: String, dot: rune::Color, line: String) -> El {
    row()
        .key(key)
        .gap(6.0)
        .center_y()
        .child(el().w(8.0).h(8.0).radius(4.0).fill(dot))
        .child(text(line).font(k.ty.caption()).color(k.c.text_base).wrap())
}

/// The oracle panel's body: per part a header (matches / differs / no such part in Rune) and its token rows in
/// aligned columns.
fn report_view(k: &Kit, r: &Report) -> El {
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
    let cell = |s: String, w: f32| caption(k, s).w(w);
    let mut c = col().gap(12.0).child(caption(
        k,
        format!(
            "oracle {} vs Rune; boxes relative to the container",
            r.version
        ),
    ));
    for (i, p) in r.parts.iter().enumerate() {
        let (dot, word) = match p.verdict {
            Verdict::Matches => (k.c.success.fg, "matches"),
            Verdict::Differs => (k.c.danger.fg, "differs"),
            Verdict::NoPart => (k.c.warning.fg, "no such part in Rune"),
        };
        let mut head = format!("{} · {word} · oracle {}", p.name, boxed(p.oracle_box));
        if let (Verdict::Differs, Some(b)) = (&p.verdict, p.rune_box) {
            if b.iter().zip(p.oracle_box).any(|(a, o)| (a - o).abs() > 1.0) {
                head.push_str(&format!(", Rune {}", boxed(b)));
            }
        }
        let mut part =
            col()
                .gap(2.0)
                .child(status(k, format!("playground/oracle/part/{i}"), dot, head));
        for row_ in &p.rows {
            let mark = match &row_.rune {
                None => String::new(),
                Some(Ok(())) => "Rune: same".to_owned(),
                Some(Err(v)) => format!("Rune: differs, {v}"),
            };
            let mut line = row()
                .gap(8.0)
                .child(cell(row_.prop.clone(), 140.0))
                .child(cell(row_.token.clone(), 300.0))
                .child(cell(row_.value.clone(), 150.0));
            line = line.child(match &row_.rune {
                Some(Err(_)) => text(mark).font(k.ty.caption()).color(k.c.danger.fg),
                _ => caption(k, mark),
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

fn heading(k: &Kit, s: impl Into<String>) -> El {
    text(s.into()).font(k.ty.ui_medium).color(k.c.text_strong)
}

fn caption(k: &Kit, s: impl Into<String>) -> El {
    text(s.into())
        .font(k.ty.caption())
        .color(k.c.text_weak)
        .w_full()
        .wrap()
}

/// A note under a control, in a status colour.
fn note(k: &Kit, s: String, color: rune::Color) -> El {
    text(s).font(k.ty.caption()).color(color).w_full().wrap()
}

/// A choice: the selected one filled, a disabled one faint.
fn pill<H: Host>(
    k: &Kit,
    key: String,
    label: String,
    on: bool,
    disabled: bool,
    press: impl Fn(&mut H) + 'static,
) -> El {
    let ink = if disabled {
        k.c.text_faint
    } else {
        k.c.text_strong
    };
    let mut e = row()
        .key(key)
        .py(5.0)
        .pl(10.0)
        .pr(10.0)
        .radius(6.0)
        .center_y()
        .role("button")
        .label(label.clone())
        .focusable(!disabled)
        .disabled(disabled)
        .child(text(label).font(k.ty.ui).color(ink).a11y_hidden());
    if on {
        e = e.fill(k.c.surface_raised_base);
    }
    if disabled {
        e
    } else {
        k.interactive(e).on_press(press)
    }
}

/// One option: its values as buttons, the chosen one selected, an excluded one disabled. A value that
/// would break an error rule says so; one that would only break an advisory stays available and says so.
fn control<H: Host>(
    k: &Kit,
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
                Severity::Error => note(
                    k,
                    format!("{shown}: not available, Error · {rule}"),
                    k.c.danger.fg,
                ),
                Severity::Advisory => {
                    note(k, format!("{shown}: Advisory · {rule}"), k.c.warning.fg)
                }
            });
        }
        let key = format!("{comp}/{opt}");
        let id2 = id.clone();
        let label = if default == Some(id.as_str()) {
            format!("{shown} (default)")
        } else {
            shown
        };
        pill::<H>(
            k,
            format!("playground/{comp}/{opt}/{id}"),
            label,
            id == current,
            blocked,
            move |g: &mut H| {
                g.play_mut().values.insert(key.clone(), id2.clone());
            },
        )
    });
    let row_ = row().gap(6.0).flex_wrap().children(buttons);
    col()
        .gap(6.0)
        .child(caption(k, opt))
        .child(row_)
        .children(notes)
}

/// The component list's items in order: the previewed components, then the catalog-only ones, as (id, shown).
pub fn menu_order(s: &Play) -> Vec<(&'static str, &'static str)> {
    s.lib
        .adapters()
        .iter()
        .map(|a| (a.id(), a.name()))
        .chain(
            s.lib
                .components()
                .into_iter()
                .filter(|c| !previewed(s, c))
                .map(|c| (c, c)),
        )
        .collect()
}

pub fn selected(s: &Play) -> &'static str {
    let want = pick(s, "", "component", "badge");
    let all = s.lib.components();
    all.iter().copied().find(|c| *c == want).unwrap_or(all[0])
}

/// Whether the playground has a preview adapter for the component (the rest show the catalog only).
pub fn previewed(s: &Play, comp: &str) -> bool {
    s.adapter(comp).is_some()
}

/// How long the options must stay unchanged before the oracle runs by itself.
const ORACLE_DELAY: f64 = 0.3;

/// Runs the oracle once the options have been still for `ORACLE_DELAY`. It blocks the UI for the oracle's
/// run (the design-data CLI: about 0.15 s warm, 2 s cold). shortcut: no background thread; move it to one if
/// that is felt.
pub fn poll(s: &mut Play, t: f64, tx: &mut rune::Tx) -> bool {
    if s.lib.oracle().is_none() {
        return false;
    }
    let dark = s.system_dark.get();
    let comp = selected(s);
    if !previewed(s, comp) {
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

/// The pieces of one component's playground, built from its state and the appearance it shows in.
struct Parts {
    summary: String,
    shown: El,
    compare: El,
    controls: El,
    panel: El,
    rules: El,
    oracle: Option<El>,
}

fn parts<H: Host>(s: &Play, k: &Kit, env: Env) -> Parts {
    let comp = selected(s);
    let now_reasons = reasons(s, comp, &options(s, comp, None));
    let mut controls = col().gap(12.0).child(heading(k, "Options"));
    for def in s.lib.options(comp).iter().filter(|d| !d.values.is_empty()) {
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
            k,
            comp,
            def.name,
            values,
            &current,
            Some(&dflt),
            |v| {
                inherit(
                    &now_reasons,
                    reasons(s, comp, &options(s, comp, Some((def.name, v)))),
                )
            },
        ));
    }
    let mut state_control = None;
    for (opt, presets) in extras(s, comp) {
        let current = extra(s, comp, opt);
        let values = presets
            .iter()
            .map(|(id, shown)| (id.to_string(), shown.to_string()))
            .collect();
        let c = control::<H>(k, comp, opt, values, &current, None, |v| {
            if opt == "width" || opt == "state" {
                Vec::new()
            } else {
                inherit(
                    &now_reasons,
                    reasons(s, comp, &options(s, comp, Some((opt, v)))),
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
    let broken = errors(s, comp, &now);
    let (reported, shown) = if !previewed(s, comp) {
        (
            Vec::new(),
            text(format!(
                "Not previewed: the playground has no adapter for {comp} yet. The options and rules are the catalog's."
            ))
            .font(k.ty.caption())
            .color(k.c.text_weak),
        )
    } else if broken.is_empty() {
        let ad = s.adapter(comp).unwrap();
        let ch = choice(s, comp);
        let (reported, built) = preview(s, comp, env);
        let built = ad.stage(&ch, env, built);
        if pick(s, "", "compare", "off") == "on" {
            let other = Env {
                dark: !env.dark,
                ..env
            };
            let (_, alt) = preview_keyed(s, comp, other, "playground/alt");
            let alt = ad.stage(&ch, other, alt);
            let side = |label: &str, kk: &Kit, e: El| {
                col()
                    .gap(6.0)
                    .padding(16.0)
                    .fill(kk.c.surface_base)
                    .child(
                        text(label.to_owned())
                            .font(kk.ty.caption())
                            .color(kk.c.text_weak),
                    )
                    .child(e)
            };
            let names = if env.dark {
                ("dark", "light")
            } else {
                ("light", "dark")
            };
            let ko = Kit::neutral(other.dark);
            (
                reported,
                row()
                    .gap(16.0)
                    .flex_wrap()
                    .child(side(names.0, k, built))
                    .child(side(names.1, &ko, alt)),
            )
        } else {
            (reported, built)
        }
    } else {
        (
            Vec::new(),
            text(format!("Not defined: {}", broken.join("; ")))
                .font(k.ty.ui)
                .color(k.c.danger.fg),
        )
    };
    let mut panel = col()
        .gap(2.0)
        .child(heading(k, "Builder options"))
        .child(caption(
            k,
            "every catalog option, as the builder's options() reports it",
        ));
    let mut names: Vec<&str> = s.lib.options(comp).iter().map(|d| d.name).collect();
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
                k,
                format!(
                    "{n} = {}{}",
                    shown_value(s, comp, n, v),
                    if is_reported { "" } else { " · not reported" }
                ),
            ),
            None => text(format!("{name}: not reported"))
                .font(k.ty.caption())
                .color(k.c.text_faint),
        };
        panel = panel.child(line);
    }
    let mut rules = col().gap(2.0).child(heading(k, "Catalog rules"));
    let all_rules = s.lib.rules(comp);
    if all_rules.is_empty() {
        rules = rules.child(caption(
            k,
            "none: the catalog has no rules for this component",
        ));
    }
    let broken_now = s.lib.check(comp, &now);
    for (i, r) in all_rules.iter().enumerate() {
        let hit = broken_now.iter().any(|v| v.rule == i);
        let (dot, state) = match (hit, r.severity) {
            (true, Severity::Error) => (k.c.danger.fg, "active"),
            (true, Severity::Advisory) => (k.c.warning.fg, "advisory"),
            _ => (k.c.text_faint, "inactive"),
        };
        rules = rules.child(status(
            k,
            format!("playground/rule/{i}"),
            dot,
            format!(
                "{state} · {:?} · {}",
                r.severity,
                r.description.unwrap_or("")
            ),
        ));
    }
    let oracle = s.lib.oracle().map(|_| {
        if !previewed(s, comp) {
            col().child(caption(
                k,
                "oracle: nothing to compare, Rune has no preview of this component here",
            ))
        } else {
            let args = oracle_args(s, comp, env.dark);
            let mut oracle = col().gap(2.0);
            if let Some(c) = state_control {
                oracle = oracle.child(c).child(caption(
                    k,
                    "rest, hover and down also show in the preview; keyboard focus is oracle only (Rune draws its ring only on real keyboard focus: Tab to the preview)",
                ));
            }
            let dark = env.dark;
            let retry_args = args.clone();
            oracle = oracle.child(pill::<H>(
                k,
                "playground/oracle/run".to_owned(),
                "Retry oracle".to_owned(),
                false,
                false,
                move |g: &mut H| {
                    let r = compare(g.play(), comp, dark, &retry_args);
                    g.play_mut().oracle = Some((retry_args.clone(), r));
                },
            ));
            match &s.oracle {
                None => oracle.child(caption(
                    k,
                    "tokens each part reads, from the oracle: runs when the options stop changing",
                )),
                Some((ran, _)) if *ran != args => oracle.child(caption(
                    k,
                    "the options changed: the oracle runs when they stop changing",
                )),
                Some((_, Ok(report))) => oracle.child(report_view(k, report)),
                Some((_, Err(e))) => oracle.child(caption(k, format!("oracle failed: {e}"))),
            }
        }
    });
    let name = s.adapter(comp).map_or(comp, |a| a.name());
    let summary = now
        .iter()
        .fold(name.to_owned(), |a, (n, v)| format!("{a} · {n} {}", fmt(v)));
    let compare = pick(s, "", "compare", "off") == "on";
    let compare = Switch::new(compare)
        .label("Compare light and dark")
        .on_toggle(move |g: &mut H| {
            let v = if compare { "off" } else { "on" };
            g.play_mut().values.insert("/compare".into(), v.into());
        })
        .build(k);
    Parts {
        summary,
        shown,
        compare,
        controls,
        panel,
        rules,
        oracle,
    }
}

/// Rune's neutral kit for the appearance the stage is showing, and that appearance.
fn kit_env(s: &Play, cx: &rune::Cx<impl Host>) -> (Kit, Env) {
    s.system_dark.set(cx.system_dark());
    (Kit::neutral(cx.system_dark()), s.env(cx.system_dark()))
}

/// The stage: the preview, as Rune's playground shows it in its stage (its appearance), and a line saying what it is.
pub fn stage_view<H: Host>(h: &H, cx: &mut rune::Cx<H>) -> El {
    let s = h.play();
    let (k, env) = kit_env(s, cx);
    let p = parts::<H>(s, &k, env);
    col()
        .key("playground/stage")
        .gap(8.0)
        .child(caption(&k, p.summary))
        .child(p.shown)
}

/// The side column: the compare and scale switches, a control per catalog option, the builder's options, the
/// catalog's rules and the oracle's tokens.
pub fn side_view<H: Host>(h: &H, cx: &mut rune::Cx<H>) -> El {
    let s = h.play();
    let (k, env) = kit_env(s, cx);
    let mobile = s.mobile;
    let p = parts::<H>(s, &k, env);
    let mut c = col()
        .w_full()
        .gap(16.0)
        .child(p.compare)
        .child(
            Switch::new(mobile)
                .label("Mobile scale")
                .on_toggle(move |g: &mut H| {
                    g.play_mut().mobile = !mobile;
                })
                .build(&k),
        )
        .child(p.controls)
        .child(p.panel)
        .child(p.rules);
    if let Some(o) = p.oracle {
        c = c.child(col().gap(2.0).child(heading(&k, "Oracle")).child(o));
    }
    c
}

/// The state of one playground story. Rune keeps one per story, apart from the app's.
#[derive(Clone)]
pub struct Page {
    pub play: Play,
}

impl Host for Page {
    fn play(&self) -> &Play {
        &self.play
    }
    fn play_mut(&mut self) -> &mut Play {
        &mut self.play
    }
}

impl Page {
    /// A page for `lib` with `component` chosen.
    pub fn of(lib: &'static dyn Library, component: &str) -> Page {
        let mut p = Page {
            play: Play::new(lib),
        };
        p.play.values.insert("/component".into(), component.into());
        p
    }
}

/// One story per component of `lib`, under "Components/", the previewed ones first: give each to
/// `rune::Playground::story`.
pub fn stories(lib: &'static dyn Library) -> Vec<rune::Story<Page>> {
    menu_order(&Play::new(lib))
        .into_iter()
        .map(|(id, name)| {
            // Rune takes story names as `&'static str`; the catalog is fixed, so this leaks a few dozen.
            let path: &'static str = Box::leak(format!("Components/{name}").into_boxed_str());
            rune::Story::new(path, move || Page::of(lib, id), stage_view::<Page>)
                .side(side_view::<Page>)
                .poll(|p: &mut Page, t, tx| poll(&mut p.play, t, tx))
        })
        .collect()
}
