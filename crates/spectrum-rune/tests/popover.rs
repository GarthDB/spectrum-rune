//! Checks the Popover against the `design-data render popover --format scene` oracle. Regenerate the fixture with
//! `tests/fixtures/regen-popover.sh` (its header explains the matrix). The shadow is read from pixels in
//! `popover_pixels.rs`.

use rune::{col, el, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::popover::{Placement, Popover};
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};

#[derive(Clone, Default)]
struct App {
    dark: bool,
    placement: Placement,
    hide_tip: bool,
    size: (f32, f32),
    dismissed: u32,
}

fn view(a: &App, _: &mut Cx<App>) -> El {
    let t = tokens(
        if a.dark {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        },
        Scale::Desktop,
    );
    col().padding(20.0).child(
        Popover::new("pp")
            .label("Details")
            .placement(a.placement)
            .hide_tip(a.hide_tip)
            .width(a.size.0)
            .height(a.size.1)
            .child(el().key("pp/field").focusable(true).w(10.0).h(10.0))
            .on_dismiss(|s: &mut App| s.dismissed += 1)
            .build(t),
    )
}

fn runtime(a: App) -> Runtime<App> {
    let dark = a.dark;
    let mut rt = Runtime::new(a, view, (400.0, 300.0), Env::new(dark, 1.0));
    for _ in 0..20 {
        rt.frame(0.05);
    }
    rt
}

fn part(rt: &Runtime<App>, name: &str) -> Option<ElementInfo> {
    let key = format!("pp/{name}");
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key.as_str()))
}

fn rgb(s: &str) -> [f32; 3] {
    let v: Vec<f32> = s.split(',').map(|c| c.parse().unwrap()).collect();
    [v[0], v[1], v[2]]
}

fn assert_color(got: rune::Color, want: &str, what: &str) {
    let hex = got.hex();
    let w = rgb(want);
    for i in 0..3 {
        let g = i32::from_str_radix(&hex[1 + 2 * i..3 + 2 * i], 16).unwrap() as f32;
        assert!((g - w[i]).abs() <= 3.0, "{what}: got {hex}, want {want}");
    }
}

fn near(got: f32, want: &Value, what: &str) {
    let w = want.as_f64().unwrap() as f32;
    assert!((got - w).abs() <= 1.0, "{what}: got {got}, want {w}");
}

fn placement(name: &str) -> Placement {
    match name {
        "top" => Placement::Top,
        "bottom" => Placement::Bottom,
        "left" => Placement::Left,
        "right" => Placement::Right,
        "start" => Placement::Start,
        _ => Placement::End,
    }
}

#[test]
fn matches_oracle_scene() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/popover_scene.json")).unwrap();
    assert_eq!(cases.len(), 28);
    for c in &cases {
        let what = format!(
            "{} {} tip hidden {}",
            c["scheme"], c["placement"], c["hideTip"]
        );
        let rt = runtime(App {
            dark: c["scheme"] == "dark",
            placement: placement(c["placement"].as_str().unwrap()),
            hide_tip: c["hideTip"].as_bool().unwrap(),
            size: (
                c["container"]["w"].as_f64().unwrap() as f32,
                c["container"]["h"].as_f64().unwrap() as f32,
            ),
            ..App::default()
        });
        let ct = part(&rt, "container").unwrap();
        near(ct.target[2], &c["container"]["w"], &format!("{what} w"));
        near(ct.target[3], &c["container"]["h"], &format!("{what} h"));
        near(
            ct.radius,
            &c["container"]["radius"],
            &format!("{what} radius"),
        );
        assert_color(
            ct.fill,
            c["container"]["fill"].as_str().unwrap(),
            &format!("{what} fill"),
        );
        match part(&rt, "tip") {
            None => assert!(c["tip"].is_null(), "{what}: no tip drawn"),
            Some(tip) => {
                let w = &c["tip"];
                near(
                    tip.target[0] - ct.target[0],
                    &w["x"],
                    &format!("{what} tip x"),
                );
                near(
                    tip.target[1] - ct.target[1],
                    &w["y"],
                    &format!("{what} tip y"),
                );
                near(tip.target[2], &w["w"], &format!("{what} tip w"));
                near(tip.target[3], &w["h"], &format!("{what} tip h"));
            }
        }
    }
}

#[test]
fn escape_dismisses_from_inside_and_the_popover_is_a_named_dialog() {
    let mut rt = runtime(App {
        size: (120.0, 60.0),
        ..App::default()
    });
    let (x, y) = rt.find("pp/field").unwrap();
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.key(KeyEvent::from(Key::Escape));
    for _ in 0..5 {
        rt.frame(0.05);
    }
    assert_eq!(rt.state.dismissed, 1);
    let c = part(&rt, "container").unwrap();
    assert_eq!(c.role, "dialog");
    assert_eq!(c.label.as_deref(), Some("Details"));
}
