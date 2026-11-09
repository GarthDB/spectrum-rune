//! Checks the Checkbox against the `design-data render checkbox --format scene` oracle.
//! Regenerate the fixture with `tests/fixtures/regen-checkbox.sh`.
//! Border, glyph ink and focus ring: `checkbox_pixels.rs`. Not asserted: the font family, keyboard-focus colours (a14.14).

use rune::{col, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::checkbox::{Checkbox, Size};
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};

#[derive(Clone, Default)]
struct App {
    dark: bool,
    mobile: bool,
    size: Option<Size>,
    selected: bool,
    indeterminate: bool,
    emphasized: bool,
    error: bool,
    disabled: bool,
    toggles: u32,
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
        Checkbox::new("cb")
            .label("Label")
            .size(a.size.unwrap_or_default())
            .selected(a.selected)
            .indeterminate(a.indeterminate)
            .emphasized(a.emphasized)
            .error(a.error)
            .disabled(a.disabled)
            .on_toggle(|s: &mut App| {
                s.toggles += 1;
                s.selected = !s.selected;
                s.indeterminate = false;
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

/// The element for a recipe part of the one component under test (keyed `cb/<part>`).
fn part(rt: &Runtime<App>, name: &str) -> Option<ElementInfo> {
    let key = format!("cb/{name}");
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
        size: Some(match c["size"].as_str().unwrap() {
            "s" => Size::S,
            "m" => Size::M,
            "l" => Size::L,
            _ => Size::XL,
        }),
        selected: c["value"] == "selected",
        indeterminate: c["value"] == "indeterminate",
        emphasized: c["emphasized"].as_bool().unwrap(),
        error: c["error"].as_bool().unwrap(),
        disabled: c["disabled"].as_bool().unwrap(),
        ..App::default()
    }
}

fn click(rt: &mut Runtime<App>) {
    let (x, y) = rt.find("cb/container").unwrap();
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    settle(rt);
}

#[test]
fn matches_oracle_scene() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/checkbox_scene.json")).unwrap();
    assert!(cases.len() > 400);
    for c in &cases {
        let what = format!(
            "{} {} {} {} emph={} err={} dis={} {}",
            c["scheme"],
            c["scale"],
            c["size"],
            c["value"],
            c["emphasized"],
            c["error"],
            c["disabled"],
            c["state"]
        );
        let mut rt = runtime(app_for(c));
        let (x, y) = rt.find("cb/container").unwrap();
        match c["state"].as_str().unwrap() {
            "hover" => rt.pointer_move(x, y),
            "down" => rt.pointer_down(x, y),
            _ => {}
        }
        settle(&mut rt);
        let origin = part(&rt, "container").unwrap().target;

        let b = part(&rt, "box").unwrap();
        let want = &c["box"];
        assert_rect(b.target, origin, want, &format!("{what} box"));
        // Border and ring are asserted from pixels in checkbox_pixels.rs.
        assert_color(
            b.fill,
            want["fill"].as_str().unwrap(),
            &format!("{what} box"),
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
            "{what}: box scale {} want {pressed}",
            b.scale
        );

        // Only the icon the oracle draws is visible; the other keeps its key at opacity 0.
        for key in ["checkmark", "dash"] {
            let e = part(&rt, key).unwrap();
            let want = &c["mark"];
            let shown = want["part"] == key;
            assert_eq!(
                e.opacity > 0.99,
                shown,
                "{what}: {key} opacity {}",
                e.opacity
            );
            assert!(
                shown || e.opacity < 0.01,
                "{what}: {key} opacity {}",
                e.opacity
            );
            if shown {
                assert_rect(e.target, origin, want, &format!("{what} {key}"));
                assert_color(
                    e.text_color.unwrap(),
                    want["color"].as_str().unwrap(),
                    &format!("{what} {key}"),
                );
            }
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
fn press_toggles_and_disabled_does_not() {
    let mut rt = runtime(App::default());
    click(&mut rt);
    assert_eq!(rt.state.toggles, 1);
    assert!(rt.state.selected);

    let mut rt = runtime(App {
        disabled: true,
        ..App::default()
    });
    click(&mut rt);
    assert_eq!(rt.state.toggles, 0);
    assert!(!part(&rt, "container").unwrap().focusable);
}

#[test]
fn indeterminate_shows_dash_not_checkmark() {
    let rt = runtime(App {
        selected: true,
        indeterminate: true,
        ..App::default()
    });
    assert!(part(&rt, "dash").unwrap().opacity > 0.99);
    assert!(part(&rt, "checkmark").unwrap().opacity < 0.01);
    let rt = runtime(App {
        selected: true,
        ..App::default()
    });
    assert!(part(&rt, "checkmark").unwrap().opacity > 0.99);
    assert!(part(&rt, "dash").unwrap().opacity < 0.01);
}

#[test]
fn check_fades_in_on_toggle() {
    let mut rt = runtime(App::default());
    assert!(part(&rt, "checkmark").unwrap().opacity < 0.01);
    let (x, y) = rt.find("cb/container").unwrap();
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(0.016);
    let mid = part(&rt, "checkmark").unwrap().opacity;
    settle(&mut rt);
    assert!(part(&rt, "checkmark").unwrap().opacity > 0.99);
    assert!(mid > 0.0 && mid < 1.0, "mid-flight opacity {mid}");
}

#[test]
fn space_toggles_focused_checkbox() {
    let mut rt = runtime(App::default());
    rt.key(KeyEvent::from(Key::Tab));
    settle(&mut rt);
    assert!(part(&rt, "container").unwrap().focused);
    rt.key(KeyEvent::from(Key::Text(" ".into())));
    settle(&mut rt);
    assert_eq!(rt.state.toggles, 1);
}
