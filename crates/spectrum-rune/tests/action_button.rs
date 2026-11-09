//! Checks the ActionButton against the `design-data render action-button --format scene` oracle.
//! Regenerate the fixture with `tests/fixtures/regen-action-button.sh` (its header explains the
//! matrix).
//!
//! Not asserted from the element, because `ElementInfo` does not expose it (a14.3): border (the
//! recipe has none), the focus ring and the font family. The colour table, hold-icon colour
//! included, is checked in `src/action_button.rs`'s unit test; the ring colour and one fill are
//! also read from pixels in `tests/pixels.rs`. Keyboard-focus colours are not implemented (a14.14).

use rune::{col, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::action_button::ActionButton;
use spectrum_rune::button::StaticColor;
use spectrum_rune::constraints::{check, Severity};
use spectrum_rune::icons::CHECKMARK_100;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::{ActionButtonSize, Size};

#[derive(Clone)]
struct App {
    dark: bool,
    mobile: bool,
    size: ActionButtonSize,
    label: String,
    icon: bool,
    quiet: bool,
    selected: Option<bool>,
    emphasized: bool,
    disabled: bool,
    pending: bool,
    hidden: bool,
    hold: bool,
    static_color: Option<StaticColor>,
    presses: u32,
}

impl Default for App {
    fn default() -> Self {
        App {
            dark: false,
            mobile: false,
            size: ActionButtonSize::M,
            label: "Label".into(),
            icon: true,
            quiet: false,
            selected: None,
            emphasized: false,
            disabled: false,
            pending: false,
            hidden: false,
            hold: false,
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
    let mut b = ActionButton::new("bt")
        .label(a.label.clone())
        .size(a.size)
        .quiet(a.quiet)
        .emphasized(a.emphasized)
        .disabled(a.disabled)
        .pending(a.pending)
        .hide_label(a.hidden)
        .hold_icon(a.hold)
        .on_press(|s: &mut App| s.presses += 1);
    if let Some(on) = a.selected {
        b = b.selected(on);
    }
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
    let flag = |k: &str| c[k].as_bool().unwrap_or(false);
    App {
        dark: c["scheme"] == "dark",
        mobile: c["scale"] == "mobile",
        size: match c["size"].as_str().unwrap() {
            "xs" => ActionButtonSize::XS,
            "s" => ActionButtonSize::S,
            "m" => ActionButtonSize::M,
            "l" => ActionButtonSize::L,
            _ => ActionButtonSize::XL,
        },
        label: c["label"].as_str().unwrap().into(),
        quiet: flag("quiet"),
        selected: flag("selected").then_some(true),
        emphasized: flag("emphasized"),
        disabled: flag("disabled"),
        pending: flag("pending"),
        hidden: flag("hidden"),
        hold: flag("hold_icon"),
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
        serde_json::from_str(include_str!("fixtures/action_button_scene.json")).unwrap();
    assert_eq!(cases.len(), 412);
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
        // The label's width depends on the installed font (the oracle measures Source Sans 3), so
        // the container is compared without it: padding + icon + gap must match to 1px.
        let text_w = part(&rt, "label").map_or(0.0, |l| l.target[2]);
        let want_text_w = c["label_part"]["w"].as_f64().unwrap_or(0.0) as f32;
        let want_w = c["container"]["w"].as_f64().unwrap() as f32;
        close(
            b.target[2] - text_w,
            &(want_w - want_text_w).into(),
            1.0,
            &format!("{what} width"),
        );
        close(
            b.target[3],
            &c["container"]["h"],
            1.0,
            &format!("{what} height"),
        );
        assert_color(
            b.fill,
            &c["container"]["fill"],
            &format!("{what} container"),
        );
        close(
            b.radius,
            &c["container"]["radius"],
            1.0,
            &format!("{what} radius"),
        );
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
        let hold = part(&rt, "hold-icon");
        assert_eq!(
            hold.is_some(),
            !c["hold"].is_null(),
            "{what}: hold icon present"
        );
        if let Some(h) = hold {
            assert_color(
                h.text_color.unwrap(),
                &c["hold"]["color"],
                &format!("{what} hold icon"),
            );
            // Pinned to the bottom and end corner: compare the distance to those edges, which
            // does not depend on the label's width.
            let end = |x: f32, w: f32, cw: f32| cw - x - w;
            let h_end = end(h.target[0] - origin[0], h.target[2], b.target[2]);
            let want_end = end(c["hold"]["x"].as_f64().unwrap() as f32, 6.0, want_w);
            close(
                h_end,
                &want_end.into(),
                0.5,
                &format!("{what} hold end inset"),
            );
            close(
                h.target[1] - origin[1],
                &c["hold"]["y"],
                0.5,
                &format!("{what} hold y"),
            );
            close(h.target[2], &c["hold"]["w"], 0.5, &format!("{what} hold w"));
            close(h.target[3], &c["hold"]["h"], 0.5, &format!("{what} hold h"));
        }
        let progress = part(&rt, "progress");
        assert_eq!(
            progress.is_some(),
            c["pending"] == true,
            "{what}: progress present"
        );
        if let Some(p) = progress {
            // The oracle's progress carries no usable ink here: Loader draws its own ring.
            assert_rect(
                p.target,
                origin,
                &c["progress"],
                &format!("{what} progress"),
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

#[test]
fn pending_hides_label_icon_and_hold_icon_and_ignores_press() {
    let mut rt = runtime(App {
        pending: true,
        hold: true,
        ..App::default()
    });
    assert!(part(&rt, "label").is_none());
    assert!(part(&rt, "icon").is_none());
    assert!(part(&rt, "hold-icon").is_none());
    assert!(part(&rt, "progress").is_some());
    assert!(part(&rt, "container").unwrap().focusable);
    click(&mut rt);
    assert_eq!(rt.state.presses, 0);
}

#[test]
fn hide_label_is_a_square_icon_button_with_an_accessible_name() {
    for size in [
        ActionButtonSize::XS,
        ActionButtonSize::S,
        ActionButtonSize::M,
        ActionButtonSize::L,
        ActionButtonSize::XL,
    ] {
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

fn node<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    if v["key"] == key {
        return Some(v);
    }
    v["children"].as_array()?.iter().find_map(|c| node(c, key))
}

fn states(rt: &Runtime<App>) -> Vec<String> {
    let tree = rt.accessibility_json();
    let n = node(&tree, "bt/container").expect("container node");
    assert_eq!(n["role"], "button");
    n["states"]
        .as_array()
        .map(|s| s.iter().map(|x| x.as_str().unwrap().to_owned()).collect())
        .unwrap_or_default()
}

/// `selected` marks a toggle (pressed state); the press only reports and the app flips it.
#[test]
fn selected_makes_a_toggle_and_the_app_owns_the_state() {
    assert!(states(&runtime(App::default()))
        .iter()
        .all(|s| s != "checked" && s != "unchecked"));
    let on = |v: Option<bool>| {
        states(&runtime(App {
            selected: v,
            ..App::default()
        }))
    };
    assert!(on(Some(true)).contains(&"checked".into()));
    assert!(on(Some(false)).contains(&"unchecked".into()));

    let mut rt = runtime(App {
        selected: Some(false),
        ..App::default()
    });
    click(&mut rt);
    assert_eq!(rt.state.presses, 1);
    assert!(
        states(&rt).contains(&"unchecked".into()),
        "a press alone does not select"
    );
}

#[test]
fn size_accepts_the_shared_size_and_xs() {
    let height = |s: ActionButtonSize| {
        let rt = runtime(App {
            size: s,
            ..App::default()
        });
        part(&rt, "container").unwrap().target[3]
    };
    assert_eq!(height(Size::M.into()), 32.0);
    assert_eq!(height(ActionButtonSize::XS), 20.0);
}

fn errors(b: &ActionButton) -> Vec<&'static str> {
    check("action-button", &b.options())
        .iter()
        .filter(|v| v.severity == Severity::Error)
        .map(|v| v.description.unwrap())
        .collect()
}

#[test]
fn check_reports_an_icon_less_hidden_label_as_an_error() {
    assert_eq!(
        errors(&ActionButton::new("a").hide_label(true)),
        ["Icon must be present if the label is not defined."]
    );
    assert!(errors(&ActionButton::new("a").hide_label(true).icon(&CHECKMARK_100)).is_empty());
}

#[test]
fn emphasized_without_selected_is_advisory_only() {
    let b = ActionButton::new("a").emphasized(true);
    assert!(errors(&b).is_empty());
    let v = check("action-button", &b.options());
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].severity, Severity::Advisory);
    // Valid: builds without a debug assertion.
    let _ = b.build(tokens(ColorScheme::Light, Scale::Desktop));
    // Selected, it is fine.
    assert!(check(
        "action-button",
        &ActionButton::new("a")
            .emphasized(true)
            .selected(true)
            .options()
    )
    .is_empty());
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "Icon must be present")]
fn building_an_error_combination_asserts_in_debug() {
    let _ = ActionButton::new("a")
        .hide_label(true)
        .build(tokens(ColorScheme::Light, Scale::Desktop));
}
