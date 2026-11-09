//! Checks the Switch against the `design-data render switch --format scene` oracle.
//! Regenerate the fixture with `tests/fixtures/regen-switch.sh`.

use rune::{col, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::switch::{Size, Switch};
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};

#[derive(Clone, Default)]
struct App {
    dark: bool,
    mobile: bool,
    size: Option<Size>,
    selected: bool,
    emphasized: bool,
    disabled: bool,
    hide_label: bool,
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
        Switch::new("sw")
            .label("Wi-Fi")
            .size(a.size.unwrap_or_default())
            .selected(a.selected)
            .emphasized(a.emphasized)
            .disabled(a.disabled)
            .hide_label(a.hide_label)
            .on_toggle(|s: &mut App| {
                s.toggles += 1;
                s.selected = !s.selected;
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

/// The element for a recipe part of the one component under test (keyed `sw/<part>`).
fn part(rt: &Runtime<App>, name: &str) -> Option<ElementInfo> {
    let key = format!("sw/{name}");
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
        selected: c["selected"].as_bool().unwrap(),
        emphasized: c["emphasized"].as_bool().unwrap(),
        disabled: c["disabled"].as_bool().unwrap(),
        ..App::default()
    }
}

#[test]
fn matches_oracle_scene() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/switch_scene.json")).unwrap();
    assert!(cases.len() > 100);
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
        let (x, y) = rt.find("sw/container").unwrap();
        match c["state"].as_str().unwrap() {
            "hover" => rt.pointer_move(x, y),
            "down" => rt.pointer_down(x, y),
            // A disabled switch must look the same under the pointer (oracle: disabled hover == rest).
            _ if c["disabled"] == true => rt.pointer_down(x, y),
            _ => {}
        }
        settle(&mut rt);
        let origin = part(&rt, "container").unwrap().target;
        for (key, want) in [("track", &c["track"]), ("handle", &c["handle"])] {
            let e = part(&rt, key).unwrap();
            let what = format!("{what} {key}");
            assert_rect(e.target, origin, want, &what);
            // TODO(a14.3): assert border width/colour and focus ring once ElementInfo exposes them.
            assert_color(e.fill, want["fill"].as_str().unwrap(), &what);
            let r = want["radius"].as_f64().unwrap() as f32;
            assert!(
                (e.radius - r).abs() <= 1.0,
                "{what}: radius {} want {r}",
                e.radius
            );
        }
        assert_color(
            part(&rt, "label").unwrap().text_color.unwrap(),
            c["label_color"].as_str().unwrap(),
            &format!("{what} label"),
        );
    }
}

#[test]
fn press_toggles_and_disabled_does_not() {
    let mut rt = runtime(App::default());
    let (x, y) = rt.find("sw/container").unwrap();
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    settle(&mut rt);
    assert_eq!(rt.state.toggles, 1);
    assert!(rt.state.selected);

    let mut rt = runtime(App {
        disabled: true,
        ..App::default()
    });
    let (x, y) = rt.find("sw/container").unwrap();
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    settle(&mut rt);
    assert_eq!(rt.state.toggles, 0);
    assert!(!part(&rt, "container").unwrap().focusable);
}

#[test]
fn hide_label_removes_label() {
    assert!(part(&runtime(App::default()), "label").is_some());
    assert!(part(
        &runtime(App {
            hide_label: true,
            ..App::default()
        }),
        "label"
    )
    .is_none());
}

#[test]
fn handle_travel_animates() {
    let mut rt = runtime(App::default());
    let before = part(&rt, "handle").unwrap().rect[0];
    let (x, y) = rt.find("sw/container").unwrap();
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(0.016);
    let mid = part(&rt, "handle").unwrap().rect[0];
    settle(&mut rt);
    let after = part(&rt, "handle").unwrap().rect[0];
    assert!(after > before, "handle moved right");
    assert!(
        mid > before && mid < after,
        "mid-flight {before} < {mid} < {after}"
    );
}

#[test]
fn space_toggles_focused_switch() {
    let mut rt = runtime(App::default());
    rt.key(KeyEvent::from(Key::Tab));
    settle(&mut rt);
    assert!(
        part(&rt, "container").unwrap().focused,
        "Tab focuses the switch"
    );
    rt.key(KeyEvent::from(Key::Text(" ".into())));
    settle(&mut rt);
    assert_eq!(rt.state.toggles, 1);
    // Enter does not toggle, as for a native switch (ARIA: Space).
    rt.key(KeyEvent::from(Key::Enter));
    settle(&mut rt);
    assert_eq!(rt.state.toggles, 1);
}
