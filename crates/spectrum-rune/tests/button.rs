//! Checks the Button against the `design-data render button --format scene` oracle.
//! Regenerate the fixture with `tests/fixtures/regen-button.sh` (its header explains the matrix).
//!
//! Not asserted, because `ElementInfo` does not expose it yet (a14.3): border width and colour
//! (the colour table, border included, is checked in `src/button.rs`'s unit test instead), the
//! focus ring, the font family. Keyboard-focus colours are not implemented (a14.14).

use rune::{col, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::button::{Button, StaticColor, Style, Variant};
use spectrum_rune::icons::CHECKMARK_100;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

#[derive(Clone)]
struct App {
    dark: bool,
    mobile: bool,
    variant: Variant,
    style: Style,
    size: Size,
    label: String,
    icon: bool,
    disabled: bool,
    pending: bool,
    hidden: bool,
    static_color: Option<StaticColor>,
    presses: u32,
}

impl Default for App {
    fn default() -> Self {
        App {
            dark: false,
            mobile: false,
            variant: Variant::Accent,
            style: Style::Fill,
            size: Size::M,
            label: "Label".into(),
            icon: true,
            disabled: false,
            pending: false,
            hidden: false,
            static_color: None,
            presses: 0,
        }
    }
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
    let mut b = Button::new("bt")
        .label(a.label.clone())
        .variant(a.variant)
        .style(a.style)
        .size(a.size)
        .disabled(a.disabled)
        .pending(a.pending)
        .label_hidden(a.hidden)
        .on_press(|s: &mut App| s.presses += 1);
    if a.icon {
        b = b.icon(&CHECKMARK_100);
    }
    if let Some(c) = a.static_color {
        b = b.static_color(c);
    }
    col().child(b.build(t))
}

fn runtime(a: App) -> Runtime<App> {
    let dark = a.dark;
    let mut rt = Runtime::new(a, view, (400.0, 100.0), Env::new(dark, 1.0));
    settle(&mut rt);
    rt
}

fn settle(rt: &mut Runtime<App>) {
    for _ in 0..40 {
        rt.frame(0.05);
    }
}

/// The element for a recipe part of the one component under test (keyed `bt/<part>`).
fn part(rt: &Runtime<App>, name: &str) -> Option<ElementInfo> {
    let key = format!("bt/{name}");
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key.as_str()))
}

/// The fixture writes colours as `r,g,b[,a]`, or `transparent`.
fn rgba(s: &str) -> [f32; 4] {
    if s == "transparent" {
        return [0.0; 4];
    }
    let v: Vec<f32> = s.split(',').map(|c| c.parse().unwrap()).collect();
    [v[0], v[1], v[2], v.get(3).copied().unwrap_or(1.0)]
}

fn assert_color(got: rune::Color, want: &Value, what: &str) {
    let want = rgba(want.as_str().unwrap_or("transparent"));
    // A transparent fill has no meaningful rgb.
    if want[3] == 0.0 {
        assert!(got.a < 0.02, "{what}: alpha {} want 0", got.a);
        return;
    }
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

fn close(got: f32, want: &Value, tol: f32, what: &str) {
    let w = want.as_f64().unwrap() as f32;
    assert!((got - w).abs() <= tol, "{what}: got {got}, want {w}");
}

fn assert_rect(got: [f32; 4], origin: [f32; 4], want: &Value, what: &str) {
    let g = [got[0] - origin[0], got[1] - origin[1], got[2], got[3]];
    for (g, k) in g.iter().zip(["x", "y", "w", "h"]) {
        if !want[k].is_null() {
            close(*g, &want[k], 1.0, &format!("{what} {k}"));
        }
    }
}

fn app_for(c: &Value) -> App {
    App {
        dark: c["scheme"] == "dark",
        mobile: c["scale"] == "mobile",
        variant: match c["variant"].as_str().unwrap() {
            "accent" => Variant::Accent,
            "negative" => Variant::Negative,
            "primary" => Variant::Primary,
            _ => Variant::Secondary,
        },
        style: if c["style"] == "outline" {
            Style::Outline
        } else {
            Style::Fill
        },
        size: match c["size"].as_str().unwrap() {
            "s" => Size::S,
            "m" => Size::M,
            "l" => Size::L,
            _ => Size::XL,
        },
        label: c["label"].as_str().unwrap().into(),
        disabled: c["disabled"].as_bool().unwrap_or(false),
        pending: c["pending"].as_bool().unwrap_or(false),
        hidden: c["hidden"].as_bool().unwrap_or(false),
        static_color: match c["static"].as_str() {
            Some("white") => Some(StaticColor::White),
            Some("black") => Some(StaticColor::Black),
            _ => None,
        },
        ..App::default()
    }
}

fn click(rt: &mut Runtime<App>) {
    let (x, y) = rt.find("bt/container").unwrap();
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    settle(rt);
}

#[test]
fn matches_oracle_scene() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/button_scene.json")).unwrap();
    assert_eq!(cases.len(), 264);
    for c in &cases {
        let what = c.to_string();
        let what = &what[..what.len().min(200)];
        let mut rt = runtime(app_for(c));
        let (x, y) = rt.find("bt/container").unwrap();
        match c["state"].as_str().unwrap() {
            "hover" => rt.pointer_move(x, y),
            "down" => rt.pointer_down(x, y),
            _ => {}
        }
        settle(&mut rt);

        let b = part(&rt, "container").unwrap();
        let origin = b.target;
        let want = &c["container"];
        // The label's width depends on the installed font (the oracle measures Source Sans 3), so
        // the container is compared without it: padding + icon + gap must match to 1px.
        let text_w = part(&rt, "label").map_or(0.0, |l| l.target[2]);
        let want_text_w = c["label_part"]["w"].as_f64().unwrap_or(0.0) as f32;
        let mut want = want.clone();
        want["w"] = (want["w"].as_f64().unwrap() as f32 - want_text_w).into();
        assert_rect(
            [b.target[0], b.target[1], b.target[2] - text_w, b.target[3]],
            origin,
            &want,
            &format!("{what} container"),
        );
        let want = &c["container"];
        assert_color(b.fill, &want["fill"], &format!("{what} container"));
        close(b.radius, &want["radius"], 1.0, &format!("{what} radius"));
        let pressed = if c["state"] == "down" {
            c["press"].clone()
        } else {
            1.0.into()
        };
        close(b.scale, &pressed, 0.01, &format!("{what} scale"));

        // A part the oracle omits must be absent here too, and the other way round.
        let icon = part(&rt, "icon");
        assert_eq!(icon.is_some(), !c["icon"].is_null(), "{what}: icon present");
        if let Some(i) = icon {
            assert_rect(i.target, origin, &c["icon"], &format!("{what} icon"));
            assert_color(
                i.text_color.unwrap(),
                &c["icon"]["color"],
                &format!("{what} icon"),
            );
        }
        let label = part(&rt, "label");
        assert_eq!(
            label.is_some(),
            !c["label_part"].is_null(),
            "{what}: label present"
        );
        if let Some(l) = label {
            assert_color(
                l.text_color.unwrap(),
                &c["label_part"]["color"],
                &format!("{what} label"),
            );
            close(
                l.target[0] - origin[0],
                &c["label_part"]["x"],
                1.0,
                &format!("{what} label x"),
            );
        }
        let progress = part(&rt, "progress");
        assert_eq!(
            progress.is_some(),
            c["pending"] == true,
            "{what}: progress present"
        );
        if let Some(p) = progress {
            // The oracle's progress carries no colour in the fixture: Loader draws its own ring.
            let (w, h) = (p.target[2], p.target[3]);
            assert!(w == h && w > 0.0 && w < want["w"].as_f64().unwrap() as f32);
            assert_eq!(
                (p.target[0] - origin[0]).round(),
                ((want["w"].as_f64().unwrap() as f32 - w) / 2.0).round(),
                "{what}: progress x (centred)"
            );
        }
    }
}

#[test]
fn press_fires_and_disabled_does_not() {
    let mut rt = runtime(App::default());
    click(&mut rt);
    assert_eq!(rt.state.presses, 1);

    let mut rt = runtime(App {
        disabled: true,
        ..App::default()
    });
    click(&mut rt);
    assert_eq!(rt.state.presses, 0);
    assert!(!part(&rt, "container").unwrap().focusable);
}

#[test]
fn space_presses_focused_button() {
    let mut rt = runtime(App::default());
    rt.key(KeyEvent::from(Key::Tab));
    settle(&mut rt);
    assert!(part(&rt, "container").unwrap().focused);
    rt.key(KeyEvent::from(Key::Text(" ".into())));
    settle(&mut rt);
    assert_eq!(rt.state.presses, 1);
}

/// The catalog: a second click while pending must not submit again. The oracle still shows the
/// hover and down colours and the press scale (fixture block D), so only `on_press` is muted.
#[test]
fn pending_hides_label_shows_progress_and_ignores_press() {
    let mut rt = runtime(App {
        pending: true,
        ..App::default()
    });
    assert!(part(&rt, "label").is_none());
    assert!(part(&rt, "icon").is_none());
    assert!(part(&rt, "progress").is_some());
    assert!(part(&rt, "container").unwrap().focusable);
    click(&mut rt);
    assert_eq!(rt.state.presses, 0);

    let mut rt = runtime(App {
        pending: false,
        ..App::default()
    });
    assert!(part(&rt, "label").is_some());
    assert!(part(&rt, "progress").is_none());
    click(&mut rt);
    assert_eq!(rt.state.presses, 1);
}

#[test]
fn hidden_label_is_a_square_icon_button_with_an_accessible_name() {
    for size in [Size::S, Size::M, Size::L, Size::XL] {
        let rt = runtime(App {
            hidden: true,
            size,
            ..App::default()
        });
        let c = part(&rt, "container").unwrap();
        assert!(part(&rt, "label").is_none());
        assert!(part(&rt, "icon").is_some());
        assert_eq!(c.target[2], c.target[3], "{size:?} is square");
        assert_eq!(c.label.as_deref(), Some("Label"));
    }
}

/// Not in the oracle, which always draws the icon: a label-only button has no icon part and
/// takes the half-height `component-pill-edge-to-text` padding.
#[test]
fn label_only_button_has_no_icon() {
    let rt = runtime(App {
        icon: false,
        ..App::default()
    });
    let c = part(&rt, "container").unwrap();
    let l = part(&rt, "label").unwrap();
    assert!(part(&rt, "icon").is_none());
    assert_eq!(l.target[0] - c.target[0], 16.0, "M text padding");
}
