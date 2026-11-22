//! Checks the Picker (closed field) against the `design-data render picker --format scene` oracle.
//! Regenerate the fixture with `tests/fixtures/regen-picker.sh` (its header explains the matrix).
//! The focus ring and the error border are asserted from pixels in `picker_pixels.rs`. Not asserted:
//! the font family (not observable) and text widths (they depend on the installed font).

use rune::{col, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::picker::Picker;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

#[derive(Clone, Default)]
struct App {
    dark: bool,
    mobile: bool,
    size: Option<Size>,
    placeholder: bool,
    quiet: bool,
    error: bool,
    disabled: bool,
    opens: u32,
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
    let p = Picker::new("pk").label("Country");
    let p = if a.placeholder {
        p.placeholder("Select")
    } else {
        p.value("Canada")
    };
    col().child(
        p.size(a.size.unwrap_or_default())
            .quiet(a.quiet)
            .error(a.error)
            .disabled(a.disabled)
            .on_open(|s: &mut App| s.opens += 1)
            .build(t),
    )
}

fn runtime(a: App) -> Runtime<App> {
    let dark = a.dark;
    let mut rt = Runtime::new(a, view, (400.0, 200.0), Env::new(dark, 1.0));
    settle(&mut rt);
    rt
}

fn settle(rt: &mut Runtime<App>) {
    for _ in 0..40 {
        rt.frame(0.05);
    }
}

fn part(rt: &Runtime<App>, name: &str) -> Option<ElementInfo> {
    let key = format!("pk/{name}");
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

fn near(got: f32, want: &Value, what: &str) {
    let w = want.as_f64().unwrap() as f32;
    assert!((got - w).abs() <= 1.0, "{what}: got {got}, want {w}");
}

fn assert_pos(got: &ElementInfo, origin: [f32; 4], want: &Value, what: &str) {
    near(got.target[0] - origin[0], &want["x"], &format!("{what} x"));
    near(got.target[1] - origin[1], &want["y"], &format!("{what} y"));
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
        placeholder: c["content"] == "placeholder",
        quiet: c["quiet"].as_bool().unwrap(),
        error: c["error"].as_bool().unwrap(),
        disabled: c["disabled"].as_bool().unwrap(),
        ..App::default()
    }
}

fn click(rt: &mut Runtime<App>) {
    let (x, y) = rt.find("pk/field").unwrap();
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    settle(rt);
}

#[test]
fn matches_oracle_scene() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/picker_scene.json")).unwrap();
    assert!(cases.len() > 300);
    for c in &cases {
        let what = format!(
            "{} {} {} {} quiet={} err={} dis={} {}",
            c["scheme"],
            c["scale"],
            c["size"],
            c["content"],
            c["quiet"],
            c["error"],
            c["disabled"],
            c["state"]
        );
        let mut rt = runtime(app_for(c));
        let (x, y) = rt.find("pk/field").unwrap();
        match c["state"].as_str().unwrap() {
            "hover" => rt.pointer_move(x, y),
            "down" => rt.pointer_down(x, y),
            "keyboard-focus" => {
                rt.key(KeyEvent::from(Key::Tab));
            }
            _ => {}
        }
        settle(&mut rt);
        let origin = part(&rt, "container").unwrap().target;
        near(
            origin[2],
            &c["container"]["w"],
            &format!("{what} container w"),
        );
        near(
            origin[3],
            &c["container"]["h"],
            &format!("{what} container h"),
        );

        let field = part(&rt, "field").unwrap();
        let want = &c["field"];
        assert_pos(&field, origin, want, &format!("{what} field"));
        near(field.target[2], &want["w"], &format!("{what} field w"));
        near(field.target[3], &want["h"], &format!("{what} field h"));
        near(field.radius, &want["radius"], &format!("{what} radius"));
        assert_color(
            field.fill,
            want["fill"].as_str().unwrap(),
            &format!("{what} field"),
        );
        let pressed = c["press"].as_f64().map_or(1.0, |p| p as f32);
        assert!(
            (field.scale - pressed).abs() < 0.01,
            "{what}: scale {} want {pressed}",
            field.scale
        );

        let name = c["content"].as_str().unwrap();
        let text = part(&rt, name).unwrap();
        assert_pos(&text, origin, &c["text"], &format!("{what} {name}"));
        assert_color(
            text.text_color.unwrap(),
            c["text"]["color"].as_str().unwrap(),
            &format!("{what} {name}"),
        );
        // The other of value / placeholder is not drawn.
        let other = if name == "value" {
            "placeholder"
        } else {
            "value"
        };
        assert!(part(&rt, other).is_none(), "{what}: {other} drawn");

        let icon = part(&rt, "disclosure-icon").unwrap();
        assert_pos(&icon, origin, &c["icon"], &format!("{what} icon"));
        near(icon.target[2], &c["icon"]["w"], &format!("{what} icon w"));
        assert_color(
            icon.text_color.unwrap(),
            c["icon"]["color"].as_str().unwrap(),
            &format!("{what} icon"),
        );

        let label = part(&rt, "field-label.label").unwrap();
        assert_pos(&label, origin, &c["label"], &format!("{what} label"));
        assert_color(
            label.text_color.unwrap(),
            c["label"]["color"].as_str().unwrap(),
            &format!("{what} label"),
        );
    }
}

#[test]
fn opens_on_press_enter_space_and_arrow_down() {
    let mut rt = runtime(App::default());
    click(&mut rt);
    assert_eq!(rt.state.opens, 1);
    for (n, key) in [Key::Enter, Key::Down, Key::Text(" ".into())]
        .into_iter()
        .enumerate()
    {
        let mut rt = runtime(App::default());
        rt.key(KeyEvent::from(Key::Tab));
        settle(&mut rt);
        rt.key(KeyEvent::from(key));
        settle(&mut rt);
        assert_eq!(rt.state.opens, 1, "key {n}");
    }
}

#[test]
fn disabled_is_not_focusable_and_does_not_open() {
    let mut rt = runtime(App {
        disabled: true,
        ..App::default()
    });
    click(&mut rt);
    assert_eq!(rt.state.opens, 0);
    let f = part(&rt, "field").unwrap();
    assert!(!f.focusable);
    rt.key(KeyEvent::from(Key::Tab));
    settle(&mut rt);
    assert!(!part(&rt, "field").unwrap().focused);
}

#[test]
fn field_is_a_combobox_named_by_its_label() {
    let rt = runtime(App::default());
    let f = part(&rt, "field").unwrap();
    assert_eq!(f.role, "combobox");
    assert_eq!(f.label.as_deref(), Some("Country"));
    assert_eq!(f.expanded, Some(false));
}

#[test]
fn long_value_truncates_before_the_icon() {
    let mut rt = Runtime::new(
        App::default(),
        |_: &App, _: &mut Cx<App>| {
            let t = tokens(ColorScheme::Light, Scale::Desktop);
            col().child(
                Picker::new("pk")
                    .value("A value far too long for a field this narrow to show in full")
                    .width(120.0)
                    .build(t),
            )
        },
        (400.0, 200.0),
        Env::new(false, 1.0),
    );
    settle(&mut rt);
    let (v, i) = (
        part(&rt, "value").unwrap().target,
        part(&rt, "disclosure-icon").unwrap().target,
    );
    assert!(v[0] + v[2] <= i[0] + 0.5, "value {v:?} reaches icon {i:?}");
}
