//! Checks the Badge against the `design-data render badge --format scene` oracle. Regenerate the
//! fixture with `tests/fixtures/regen-badge.sh` (its header explains the matrix).
//!
//! Not asserted from the element, because `ElementInfo` does not expose it: the stroke (colour and
//! width) and the three corner radii after the first (`ElementInfo::radius` is the top-left one),
//! and the font family. The colour table, stroke included, and the radii are checked in
//! `src/badge.rs`'s unit test; stroke and square corners are read from pixels in
//! `tests/badge_pixels.rs`.

use rune::{col, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::badge::{Badge, Fixed, Style, Variant};
use spectrum_rune::constraints::check;
use spectrum_rune::icons::CHECKMARK_100;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

#[derive(Clone)]
struct App {
    dark: bool,
    mobile: bool,
    size: Option<Size>,
    variant: Variant,
    style: Style,
    fixed: Fixed,
    label: Option<String>,
    icon: bool,
}

fn view(a: &App, _: &mut Cx<App>) -> El {
    let t = tokens(
        if a.dark {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        },
        if a.mobile {
            Scale::Mobile
        } else {
            Scale::Desktop
        },
    );
    let mut b = Badge::new("bd")
        .variant(a.variant)
        .style(a.style)
        .fixed(a.fixed);
    if let Some(s) = a.size {
        b = b.size(s);
    }
    if let Some(l) = &a.label {
        b = b.label(l.clone());
    }
    if a.icon {
        b = b.icon(&CHECKMARK_100);
    }
    col().child(b.build(t))
}

fn runtime(a: App) -> Runtime<App> {
    let dark = a.dark;
    let mut rt = Runtime::new(a, view, (400.0, 100.0), Env::new(dark, 1.0));
    for _ in 0..40 {
        rt.frame(0.05);
    }
    rt
}

fn base() -> App {
    App {
        dark: false,
        mobile: false,
        size: None,
        variant: Variant::Neutral,
        style: Style::Bold,
        fixed: Fixed::None,
        label: Some("Label".into()),
        icon: false,
    }
}

fn part(rt: &Runtime<App>, name: &str) -> Option<ElementInfo> {
    let key = format!("bd/{name}");
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key.as_str()))
}

fn rgba(s: &str) -> [f32; 4] {
    let v: Vec<f32> = s.split(',').map(|c| c.parse().unwrap()).collect();
    [v[0], v[1], v[2], v.get(3).copied().unwrap_or(1.0)]
}

fn assert_color(got: rune::Color, want: &Value, what: &str) {
    let want = rgba(want.as_str().unwrap());
    let hex = got.hex();
    for i in 0..3 {
        let g = i32::from_str_radix(&hex[1 + 2 * i..3 + 2 * i], 16).unwrap() as f32;
        assert!(
            (g - want[i]).abs() <= 3.0,
            "{what}: got {hex}, want {want:?}"
        );
    }
    assert!(
        (got.a - want[3]).abs() < 0.02,
        "{what}: alpha {} want {}",
        got.a,
        want[3]
    );
}

fn close(got: f32, want: f64, what: &str) {
    assert!(
        (got - want as f32).abs() <= 1.0,
        "{what}: got {got}, want {want}"
    );
}

fn app_for(c: &Value) -> App {
    let find = |all: &[Variant]| *all.iter().find(|v| v.as_str() == c["variant"]).unwrap();
    App {
        dark: c["scheme"] == "dark",
        mobile: c["scale"] == "mobile",
        size: Some(match c["size"].as_str().unwrap() {
            "s" => Size::S,
            "m" => Size::M,
            "l" => Size::L,
            _ => Size::XL,
        }),
        variant: find(Variant::ALL),
        style: *Style::ALL
            .iter()
            .find(|v| v.as_str() == c["style"])
            .unwrap(),
        fixed: *Fixed::ALL
            .iter()
            .find(|v| v.as_str() == c["fixed"])
            .unwrap(),
        label: c["label"].as_str().map(Into::into),
        icon: c["icon"].is_object(),
    }
}

#[test]
fn matches_oracle_scene() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/badge_scene.json")).unwrap();
    assert_eq!(cases.len(), 472);
    for c in &cases {
        let what = c.to_string();
        let what = &what[..what.len().min(200)];
        let rt = runtime(app_for(c));
        let b = part(&rt, "container").unwrap();
        let origin = b.target;

        // The label's width depends on the installed font (the oracle measures Source Sans 3), so
        // the container is compared without it: padding + icon + gap must match to 1px.
        let text_w = part(&rt, "label").map_or(0.0, |l| l.target[2]);
        let want_text_w = c["label_part"]["w"].as_f64().unwrap_or(0.0);
        close(
            b.target[2] - text_w,
            c["container"]["w"].as_f64().unwrap() - want_text_w,
            &format!("{what} width"),
        );
        close(
            b.target[3],
            c["container"]["h"].as_f64().unwrap(),
            &format!("{what} height"),
        );
        assert_color(b.fill, &c["container"]["fill"], &format!("{what} fill"));
        // `ElementInfo::radius` is the top-left corner.
        let radii = c["container"]["radii"][0]
            .as_f64()
            .unwrap_or_else(|| c["container"]["radius"].as_f64().unwrap());
        assert!(
            (b.radius - radii as f32).abs() <= 0.5,
            "{what}: radius {} want {radii}",
            b.radius
        );
        assert!(!b.focusable, "{what}: a badge is not focusable");

        // A part the oracle omits must be absent here too, and the other way round.
        let icon = part(&rt, "icon");
        assert_eq!(
            icon.is_some(),
            c["icon"].is_object(),
            "{what}: icon present"
        );
        if let Some(i) = icon {
            for (k, v) in [
                ("x", i.target[0] - origin[0]),
                ("y", i.target[1] - origin[1]),
                ("w", i.target[2]),
                ("h", i.target[3]),
            ] {
                close(
                    v,
                    c["icon"][k].as_f64().unwrap(),
                    &format!("{what} icon {k}"),
                );
            }
            assert_color(
                i.text_color.unwrap(),
                &c["icon"]["color"],
                &format!("{what} icon"),
            );
        }
        let label = part(&rt, "label");
        assert_eq!(
            label.is_some(),
            c["label_part"].is_object(),
            "{what}: label present"
        );
        if let Some(l) = label {
            close(
                l.target[0] - origin[0],
                c["label_part"]["x"].as_f64().unwrap(),
                &format!("{what} label x"),
            );
            assert_color(
                l.text_color.unwrap(),
                &c["label_part"]["color"],
                &format!("{what} label"),
            );
        }
    }
}

#[test]
fn defaults_follow_the_catalog() {
    let b = Badge::new("bd");
    assert_eq!(
        b.options()[..4],
        [
            (
                "variant",
                spectrum_rune::constraints::OptionValue::Str("neutral")
            ),
            (
                "style",
                spectrum_rune::constraints::OptionValue::Str("bold")
            ),
            (
                "fixed",
                spectrum_rune::constraints::OptionValue::Str("none")
            ),
            ("size", spectrum_rune::constraints::OptionValue::Str("s")),
        ]
    );
    // Size S: 24px high on desktop (the catalog default; the CLI renders `m` unless told).
    let rt = runtime(base());
    let h = part(&rt, "container").unwrap();
    assert_eq!(h.target[3], 24.0);
}

#[test]
fn label_is_the_accessible_text() {
    let mut a = base();
    a.icon = true;
    let rt = runtime(a);
    let l = part(&rt, "label").unwrap();
    assert_eq!(l.role, "text");
    assert_eq!(l.text.as_deref(), Some("Label"));
    // `ElementInfo` does not expose a11y_hidden on the icon, so that is not asserted.
    assert!(part(&rt, "icon").is_some());
}

#[test]
fn a_badge_takes_no_focus_and_does_nothing_on_tab() {
    let mut rt = runtime(base());
    rt.key(KeyEvent::from(Key::Tab));
    for e in rt.elements() {
        assert!(!e.focusable && !e.focused, "{:?}", e.key);
    }
}

#[test]
fn badge_has_no_catalog_constraints_today() {
    use spectrum_rune::constraints::OptionValue::*;
    let o = [
        ("variant", Str("accent")),
        ("style", Str("outline")),
        ("fixed", Str("top")),
        ("size", Str("xl")),
    ];
    assert!(check("badge", &o).is_empty());
}

#[test]
fn an_empty_label_is_icon_only() {
    let mut a = base();
    a.icon = true;
    a.size = Some(Size::M);
    a.label = Some(String::new());
    let rt = runtime(a);
    assert!(part(&rt, "label").is_none());
    // Icon-only oracle layout: 7 + 20 + 7 wide.
    assert_eq!(part(&rt, "container").unwrap().target[2], 34.0);
    assert!(!Badge::new("b")
        .label("")
        .options()
        .iter()
        .any(|(k, _)| *k == "label"));
}
