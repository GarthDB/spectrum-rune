//! Checks the RadioGroup against the `radio-group` recipe oracle (platforms/rune/extensions/visuals; regenerate the
//! fixture with `tests/fixtures/regen-radio-group.sh`) and the keyboard behaviour of a radio group.
//!
//! The oracle's radio buttons have no label, so what is compared is the gap between items and the item height.

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
    assert_eq!(cases.len(), 16);
    for c in &cases {
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
