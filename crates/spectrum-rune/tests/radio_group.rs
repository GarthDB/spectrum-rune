//! Checks the RadioGroup against the foundation's `radio-group` recipe oracle (regenerate the fixture with
//! `tests/fixtures/regen-radio-group.sh`) and the keyboard behaviour of a radio group.
//!
//! The oracle's radio buttons have no label, so what is compared is the gap between items and the item height, and (for the
//! `labeled` and `error` cases) where the group's label, options and help text sit in the container.

use rune::{Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::radio_group::{Orientation, RadioGroup};
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

#[derive(Clone, Default)]
struct App {
    mobile: bool,
    size: Size,
    horizontal: bool,
    disabled: bool,
    /// `labeled` or `error`: a required label and a description (the error message when `error`).
    kind: Option<String>,
    value: Option<String>,
    changes: Vec<String>,
}

fn view(a: &App, _: &mut Cx<App>) -> El {
    let t = tokens(
        ColorScheme::Light,
        if a.mobile {
            Scale::Mobile
        } else {
            Scale::Desktop
        },
    );
    let kind = a.kind.as_deref();
    let mut g = RadioGroup::new("g")
        .label("Pick one")
        .item("a", "Alpha")
        .item("b", "Beta")
        .item("c", "Gamma")
        .size(a.size)
        .orientation(if a.horizontal {
            Orientation::Horizontal
        } else {
            Orientation::Vertical
        })
        .disabled(a.disabled)
        .on_change(|s: &mut App, v: &str| {
            s.value = Some(v.to_string());
            s.changes.push(v.to_string());
        });
    if let Some(k) = kind {
        g = g
            .required(true)
            .description("Choose carefully")
            .error(k == "error");
    }
    if let Some(v) = &a.value {
        g = g.value(v.clone());
    }
    rune::col().child(g.build(t))
}

fn runtime(a: App) -> Runtime<App> {
    let mut rt = Runtime::new(a, view, (600.0, 400.0), Env::new(false, 1.0));
    settle(&mut rt);
    rt
}

fn settle(rt: &mut Runtime<App>) {
    for _ in 0..40 {
        rt.frame(0.05);
    }
}

fn el(rt: &Runtime<App>, key: &str) -> Option<ElementInfo> {
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key))
}

fn key(rt: &mut Runtime<App>, k: Key) {
    rt.key(KeyEvent::from(k));
    settle(rt);
}

fn focused_item(rt: &Runtime<App>) -> Option<char> {
    ['a', 'b', 'c']
        .into_iter()
        .find(|c| el(rt, &format!("g/{c}/container")).is_some_and(|e| e.focused))
}

#[test]
fn matches_oracle_gap_and_item_height() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/radio_group_scene.json")).unwrap();
    assert_eq!(cases.len(), 48);
    for c in cases.iter().filter(|c| c.get("kind").is_none()) {
        let horizontal = c["orientation"] == "horizontal";
        let what = format!("{} {} {}", c["scale"], c["size"], c["orientation"]);
        let items = c["items"].as_array().unwrap();
        let f = |i: usize, k: &str| items[i][k].as_f64().unwrap() as f32;
        let gap = if horizontal {
            f(1, "x") - f(0, "x") - f(0, "w")
        } else {
            f(1, "y") - f(0, "y") - f(0, "h")
        };
        let mut rt = runtime(App {
            mobile: c["scale"] == "mobile",
            size: match c["size"].as_str().unwrap() {
                "s" => Size::S,
                "m" => Size::M,
                "l" => Size::L,
                _ => Size::XL,
            },
            horizontal,
            ..App::default()
        });
        let r = |k: &str| el(&rt, k).unwrap().target;
        let (a, b, cc) = (r("g/a/container"), r("g/b/container"), r("g/c/container"));
        for (p, n) in [(a, b), (b, cc)] {
            let got = if horizontal {
                n[0] - (p[0] + p[2])
            } else {
                n[1] - (p[1] + p[3])
            };
            assert!((got - gap).abs() <= 1.0, "{what}: gap {got} want {gap}");
        }
        if !horizontal {
            assert!(
                (a[3] - f(0, "h")).abs() <= 1.0,
                "{what}: item height {} want {}",
                a[3],
                f(0, "h")
            );
        }
        rt.frame(0.0);
    }
}

#[test]
fn label_options_and_help_text_sit_where_the_oracle_puts_them() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/radio_group_scene.json")).unwrap();
    for c in cases.iter().filter(|c| c.get("kind").is_some()) {
        let what = format!(
            "{} {} {} {}",
            c["scale"], c["size"], c["orientation"], c["kind"]
        );
        let rt = runtime(App {
            mobile: c["scale"] == "mobile",
            size: match c["size"].as_str().unwrap() {
                "s" => Size::S,
                "m" => Size::M,
                "l" => Size::L,
                _ => Size::XL,
            },
            horizontal: c["orientation"] == "horizontal",
            kind: c["kind"].as_str().map(str::to_owned),
            ..App::default()
        });
        let root = el(&rt, "g").unwrap().target;
        let near = |got: f32, want: &Value, what2: &str| {
            let w = want.as_f64().unwrap() as f32;
            assert!(
                (got - w).abs() <= 1.0,
                "{what}: {what2}: got {got}, want {w}"
            );
        };
        near(root[3], &c["container"]["h"], "container h");
        let label = el(&rt, "g/field-label/container").unwrap().target;
        near(label[1] - root[1], &c["label"]["y"], "label y");
        near(label[3], &c["label"]["h"], "label h");
        let options = el(&rt, "g/options").unwrap().target;
        near(options[1] - root[1], &c["options"]["y"], "options y");
        near(options[3], &c["options"]["h"], "options h");
        let help = el(&rt, "g/help-text/container").unwrap().target;
        near(help[1] - root[1], &c["help"]["y"], "help y");
        near(help[3], &c["help"]["h"], "help h");
        assert_eq!(
            el(&rt, "g/field-label/necessity-indicator").is_some(),
            c["indicator"].as_bool().unwrap(),
            "{what}: asterisk"
        );
        assert_eq!(
            el(&rt, "g/help-text/icon").is_some(),
            c["help_icon"].as_bool().unwrap(),
            "{what}: help icon"
        );
    }
}

#[test]
fn tab_lands_on_selected_and_arrows_move_the_selection() {
    let mut rt = runtime(App {
        value: Some("b".into()),
        ..App::default()
    });
    key(&mut rt, Key::Tab);
    assert_eq!(focused_item(&rt), Some('b'));
    key(&mut rt, Key::Down);
    assert_eq!(focused_item(&rt), Some('c'));
    assert_eq!(rt.state.value.as_deref(), Some("c"));
    key(&mut rt, Key::Right);
    assert_eq!(focused_item(&rt), Some('a'), "wraps");
    assert_eq!(rt.state.changes, ["c", "a"]);
}

#[test]
fn nothing_selected_tab_lands_on_first_and_press_selects() {
    let mut rt = runtime(App::default());
    key(&mut rt, Key::Tab);
    assert_eq!(focused_item(&rt), Some('a'));
    key(&mut rt, Key::Text(" ".into()));
    assert_eq!(rt.state.value.as_deref(), Some("a"));
}

#[test]
fn one_selected_and_disabled_group_is_inert() {
    let rt = runtime(App {
        value: Some("b".into()),
        ..App::default()
    });
    let dot = |c: char| {
        el(&rt, &format!("g/{c}/selection-indicator"))
            .unwrap()
            .opacity
    };
    assert!(dot('a') < 0.01 && dot('b') > 0.99 && dot('c') < 0.01);

    let mut rt = runtime(App {
        disabled: true,
        ..App::default()
    });
    key(&mut rt, Key::Tab);
    assert_eq!(focused_item(&rt), None);
    let (x, y) = rt.find("g/a/container").unwrap();
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    settle(&mut rt);
    assert!(rt.state.changes.is_empty());
}

#[test]
#[cfg(debug_assertions)] // a debug_assert!: release builds do not check
#[should_panic(expected = "duplicate item value")]
fn duplicate_item_values_are_rejected() {
    let _ = RadioGroup::new("g").item("a", "One").item("a", "Two");
}
