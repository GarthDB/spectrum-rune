//! Checks the RadioButton against the `design-data render radio-button --format scene` oracle.
//! Regenerate the fixture with `tests/fixtures/regen-radio-button.sh`.
//!
//! Border, dot and focus ring are asserted from pixels in `radio_button_pixels.rs`. Not asserted:
//! the font family (not observable). Keyboard-focus colours are not implemented (a14.14).

use rune::{col, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::radio_button::RadioButton;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

#[derive(Clone, Default)]
struct App {
    dark: bool,
    mobile: bool,
    size: Size,
    selected: bool,
    emphasized: bool,
    disabled: bool,
    selects: u32,
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
    col().child(
        RadioButton::new("rb")
            .label("Label")
            .size(a.size)
            .selected(a.selected)
            .emphasized(a.emphasized)
            .disabled(a.disabled)
            .on_select(|s: &mut App| {
                s.selects += 1;
                s.selected = true;
            })
            .build(t),
    )
}

fn runtime(a: App) -> Runtime<App> {
    let dark = a.dark;
    let mut rt = Runtime::new(a, view, (300.0, 100.0), Env::new(dark, 1.0));
    settle(&mut rt);
    rt
}

fn settle(rt: &mut Runtime<App>) {
    for _ in 0..40 {
        rt.frame(0.05);
    }
}

/// The element for a recipe part of the one component under test (keyed `rb/<part>`).
fn part(rt: &Runtime<App>, name: &str) -> Option<ElementInfo> {
    let key = format!("rb/{name}");
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key.as_str()))
}

fn rgb(s: &str) -> [i32; 3] {
    let v: Vec<i32> = s
        .trim_start_matches("rgb(")
        .trim_end_matches(')')
        .split(',')
        .map(|c| c.trim().parse().unwrap())
        .collect();
    [v[0], v[1], v[2]]
}

fn assert_color(got: rune::Color, want: &str, what: &str) {
    let hex = got.hex();
    let g: Vec<i32> = (0..3)
        .map(|i| i32::from_str_radix(&hex[1 + 2 * i..3 + 2 * i], 16).unwrap())
        .collect();
    for (g, w) in g.iter().zip(rgb(want)) {
        assert!((g - w).abs() <= 3, "{what}: got {hex}, want {want}");
    }
}

fn assert_rect(got: [f32; 4], origin: [f32; 4], want: &Value, what: &str) {
    let w: Vec<f32> = ["x", "y", "w", "h"]
        .iter()
        .map(|k| want[k].as_f64().unwrap() as f32)
        .collect();
    let g = [got[0] - origin[0], got[1] - origin[1], got[2], got[3]];
    for (g, w) in g.iter().zip(&w) {
        assert!((g - w).abs() <= 1.0, "{what}: rect {g:?} want {w:?}");
    }
}

fn app_for(c: &Value) -> App {
    App {
        dark: c["scheme"] == "dark",
        mobile: c["scale"] == "mobile",
        size: match c["size"].as_str().unwrap() {
            "s" => Size::S,
            "m" => Size::M,
            "l" => Size::L,
            _ => Size::XL,
        },
        selected: c["selected"].as_bool().unwrap(),
        emphasized: c["emphasized"].as_bool().unwrap(),
        disabled: c["disabled"].as_bool().unwrap(),
        ..App::default()
    }
}

fn click(rt: &mut Runtime<App>) {
    let (x, y) = rt.find("rb/container").unwrap();
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    settle(rt);
}

#[test]
fn matches_oracle_scene() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/radio_button_scene.json")).unwrap();
    assert_eq!(cases.len(), 160);
    for c in &cases {
        let what = format!(
            "{} {} {} sel={} emph={} dis={} {}",
            c["scheme"],
            c["scale"],
            c["size"],
            c["selected"],
            c["emphasized"],
            c["disabled"],
            c["state"]
        );
        let mut rt = runtime(app_for(c));
        let (x, y) = rt.find("rb/container").unwrap();
        match c["state"].as_str().unwrap() {
            "hover" => rt.pointer_move(x, y),
            "down" => rt.pointer_down(x, y),
            _ => {}
        }
        settle(&mut rt);
        let origin = part(&rt, "container").unwrap().target;

        let b = part(&rt, "control").unwrap();
        let want = &c["control"];
        assert_rect(b.target, origin, want, &format!("{what} control"));
        assert_color(
            b.fill,
            want["fill"].as_str().unwrap(),
            &format!("{what} control"),
        );
        let r = want["radius"].as_f64().unwrap() as f32;
        assert!(
            (b.radius - r).abs() <= 1.0,
            "{what}: radius {} want {r}",
            b.radius
        );
        let pressed = if c["state"] == "down" {
            c["press"].as_f64().unwrap() as f32
        } else {
            1.0
        };
        assert!(
            (b.scale - pressed).abs() < 0.01,
            "{what}: control scale {} want {pressed}",
            b.scale
        );

        // The indicator keeps its key at opacity 0 when unselected; the oracle omits it.
        let dot = part(&rt, "selection-indicator").unwrap();
        let want = &c["indicator"];
        if want.is_null() {
            assert!(
                dot.opacity < 0.01,
                "{what}: indicator opacity {}",
                dot.opacity
            );
        } else {
            assert!(
                dot.opacity > 0.99,
                "{what}: indicator opacity {}",
                dot.opacity
            );
            assert_rect(dot.target, origin, want, &format!("{what} indicator"));
            assert_color(
                dot.fill,
                want["fill"].as_str().unwrap(),
                &format!("{what} indicator"),
            );
            let r = want["radius"].as_f64().unwrap() as f32;
            assert!((dot.radius - r).abs() <= 1.0, "{what}: indicator radius");
        }

        let label = part(&rt, "label").unwrap();
        assert_color(
            label.text_color.unwrap(),
            c["label"]["color"].as_str().unwrap(),
            &format!("{what} label"),
        );
        let lx = c["label"]["x"].as_f64().unwrap() as f32;
        assert!(
            (label.target[0] - origin[0] - lx).abs() <= 1.0,
            "{what}: label x"
        );
    }
}

#[test]
fn press_selects_and_disabled_does_not() {
    let mut rt = runtime(App::default());
    click(&mut rt);
    assert_eq!(rt.state.selects, 1);
    assert!(rt.state.selected);
    // A second press only selects again: a radio never turns itself off.
    click(&mut rt);
    assert!(rt.state.selected);

    let mut rt = runtime(App {
        disabled: true,
        ..App::default()
    });
    click(&mut rt);
    assert_eq!(rt.state.selects, 0);
    assert!(!part(&rt, "container").unwrap().focusable);
}

#[test]
fn indicator_fades_in_on_select() {
    let mut rt = runtime(App::default());
    assert!(part(&rt, "selection-indicator").unwrap().opacity < 0.01);
    let (x, y) = rt.find("rb/container").unwrap();
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(0.016);
    let mid = part(&rt, "selection-indicator").unwrap().opacity;
    settle(&mut rt);
    assert!(part(&rt, "selection-indicator").unwrap().opacity > 0.99);
    assert!(mid > 0.0 && mid < 1.0, "mid-flight opacity {mid}");
}

#[test]
fn space_selects_focused_radio() {
    let mut rt = runtime(App::default());
    rt.key(KeyEvent::from(Key::Tab));
    settle(&mut rt);
    assert!(part(&rt, "container").unwrap().focused);
    rt.key(KeyEvent::from(Key::Text(" ".into())));
    settle(&mut rt);
    assert_eq!(rt.state.selects, 1);
    assert!(rt.state.selected);
}
