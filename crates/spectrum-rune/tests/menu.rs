//! Checks the Menu against the `design-data render menu --format scene` oracle. Regenerate the fixture with
//! `tests/fixtures/regen-menu.sh` (its header explains the matrix). In the oracle a state applies to every item;
//! here the pointer or the keyboard is on one, so each state is compared for that item. Text widths depend on
//! the installed font, so item and container widths are not compared.

use rune::{col, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::menu::{Menu, MenuItem, Selection};
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

#[derive(Clone, Default)]
struct App {
    dark: bool,
    mobile: bool,
    size: Option<Size>,
    single: bool,
    chosen: Vec<String>,
    dismissed: u32,
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
        Menu::new("mn")
            .label("Actions")
            .size(a.size.unwrap_or_default())
            .selection(if a.single {
                Selection::Single
            } else {
                Selection::None
            })
            .item(MenuItem::new("one", "One"))
            .item(MenuItem::new("two", "Two").value("V").selected(a.single))
            .item(MenuItem::new("three", "Three").disabled(true))
            .on_choose(|s: &mut App, id: &str| s.chosen.push(id.to_owned()))
            .on_dismiss(|s: &mut App| s.dismissed += 1)
            .build(t),
    )
}

fn runtime(a: App) -> Runtime<App> {
    let dark = a.dark;
    let mut rt = Runtime::new(a, view, (400.0, 300.0), Env::new(dark, 1.0));
    settle(&mut rt);
    rt
}

fn settle(rt: &mut Runtime<App>) {
    for _ in 0..40 {
        rt.frame(0.05);
    }
}

fn part(rt: &Runtime<App>, name: &str) -> Option<ElementInfo> {
    let key = format!("mn/{name}");
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key.as_str()))
}

fn rgba(s: &str) -> [f32; 4] {
    let v: Vec<f32> = s.split(',').map(|c| c.parse().unwrap()).collect();
    [v[0], v[1], v[2], v.get(3).copied().unwrap_or(1.0)]
}

fn assert_color(got: rune::Color, want: &str, what: &str) {
    let hex = got.hex();
    let w = rgba(want);
    for i in 0..3 {
        let g = i32::from_str_radix(&hex[1 + 2 * i..3 + 2 * i], 16).unwrap() as f32;
        assert!((g - w[i]).abs() <= 3.0, "{what}: got {hex}, want {want}");
    }
}

fn near(got: f32, want: &Value, what: &str) {
    let w = want.as_f64().unwrap() as f32;
    assert!((got - w).abs() <= 1.0, "{what}: got {got}, want {w}");
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
        single: c["variant"] == "single",
        ..App::default()
    }
}

/// One item against the oracle's: its box and fill, label, value and checkmark.
fn check_item(
    rt: &Runtime<App>,
    c: &Value,
    i: usize,
    origin: [f32; 4],
    with_fill: bool, // false while keyboard-focused: the fill and the inks are then `menu_pixels.rs`'s
    what: &str,
) {
    let want = &c["items"][i];
    let n = i + 1;
    let item = part(rt, &format!("menu-item-{n}")).unwrap();
    let what = format!("{what} item {n}");
    near(
        item.target[0] - origin[0],
        &want["item"]["x"],
        &format!("{what} x"),
    );
    near(
        item.target[1] - origin[1],
        &want["item"]["y"],
        &format!("{what} y"),
    );
    near(item.target[3], &want["item"]["h"], &format!("{what} h"));
    near(
        item.radius,
        &want["item"]["radius"],
        &format!("{what} radius"),
    );
    if with_fill {
        match want["item"]["fill"].as_str() {
            Some(f) => assert_color(item.fill, f, &format!("{what} fill")),
            None => assert!(
                item.fill.a < 0.02,
                "{what}: fill {:?}, want none",
                item.fill
            ),
        }
    }
    let label = part(rt, &format!("menu-item-{n}.label")).unwrap();
    near(
        label.target[0] - item.target[0],
        &want["label"]["x"],
        &format!("{what} label x"),
    );
    near(
        label.target[1] - item.target[1],
        &want["label"]["y"],
        &format!("{what} label y"),
    );
    near(
        label.target[3],
        &want["label"]["h"],
        &format!("{what} label h"),
    );
    if with_fill {
        assert_color(
            label.text_color.unwrap(),
            want["label"]["color"].as_str().unwrap(),
            &format!("{what} label"),
        );
    }
    match part(rt, &format!("menu-item-{n}.value")) {
        None => assert!(want["value"].is_null(), "{what}: no value drawn"),
        Some(v) => {
            let w = &want["value"];
            near(
                (item.target[0] + item.target[2]) - (v.target[0] + v.target[2]),
                &w["right"],
                &format!("{what} value right"),
            );
            near(
                v.target[1] - item.target[1],
                &w["y"],
                &format!("{what} value y"),
            );
            if with_fill {
                assert_color(
                    v.text_color.unwrap(),
                    w["color"].as_str().unwrap(),
                    &format!("{what} value"),
                );
            }
        }
    }
    match part(rt, &format!("menu-item-{n}.checkmark")) {
        None => assert!(want["checkmark"].is_null(), "{what}: no checkmark drawn"),
        Some(k) => {
            let w = &want["checkmark"];
            near(
                k.target[0] - item.target[0],
                &w["x"],
                &format!("{what} check x"),
            );
            near(
                k.target[1] - item.target[1],
                &w["y"],
                &format!("{what} check y"),
            );
            near(k.target[2], &w["w"], &format!("{what} check w"));
            if with_fill {
                assert_color(
                    k.text_color.unwrap(),
                    w["color"].as_str().unwrap(),
                    &format!("{what} check"),
                );
            }
        }
    }
}

#[test]
fn matches_oracle_scene() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("fixtures/menu_scene.json")).unwrap();
    assert_eq!(cases.len(), 56);
    for c in &cases {
        let what = format!(
            "{} {} {} {} {}",
            c["scheme"], c["scale"], c["size"], c["variant"], c["state"]
        );
        let mut rt = runtime(app_for(c));
        let origin = part(&rt, "container").unwrap().target;
        // The state goes to one enabled item at a time; the others stay at rest.
        let on = |rt: &mut Runtime<App>, n: usize| {
            let (x, y) = rt.find(&format!("mn/menu-item-{n}")).unwrap();
            match c["state"].as_str().unwrap() {
                "hover" => rt.pointer_move(x, y),
                "down" => {
                    rt.pointer_move(x, y);
                    rt.pointer_down(x, y);
                }
                _ => {}
            }
        };
        if c["state"] == "rest" {
            for i in 0..3 {
                check_item(&rt, c, i, origin, true, &what);
            }
            continue;
        }
        for n in [1, 2] {
            let mut rt = runtime(app_for(c));
            if c["state"] == "keyboard-focus" {
                // Tab enters the menu on its first item; the arrow keys move within it.
                rt.key(KeyEvent::from(Key::Tab));
                for _ in 1..n {
                    rt.key(KeyEvent::from(Key::Down));
                }
                settle(&mut rt);
            } else {
                on(&mut rt, n);
                settle(&mut rt);
            }
            // The key-focus fill is laid over the item, so `ElementInfo` has none: `menu_pixels.rs` reads it.
            let fill = c["state"] != "keyboard-focus";
            check_item(&rt, c, n - 1, origin, fill, &format!("{what} #{n}"));
        }
        let _ = &mut rt;
    }
}

#[test]
fn enter_space_and_press_choose_and_escape_dismisses() {
    let mut rt = runtime(App::default());
    rt.key(KeyEvent::from(Key::Tab));
    settle(&mut rt);
    rt.key(KeyEvent::from(Key::Enter));
    settle(&mut rt);
    assert_eq!(rt.state.chosen, ["one"]);
    rt.key(KeyEvent::from(Key::Down));
    settle(&mut rt);
    rt.key(KeyEvent::from(Key::Text(" ".into())));
    settle(&mut rt);
    assert_eq!(rt.state.chosen, ["one", "two"]);
    rt.key(KeyEvent::from(Key::Escape));
    settle(&mut rt);
    assert_eq!(rt.state.dismissed, 1);
    let (x, y) = rt.find("mn/menu-item-1").unwrap();
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    settle(&mut rt);
    assert_eq!(rt.state.chosen, ["one", "two", "one"]);
}

#[test]
fn a_disabled_item_is_not_focusable_nor_chosen() {
    let mut rt = runtime(App::default());
    let three = part(&rt, "menu-item-3").unwrap();
    assert!(!three.focusable);
    let (x, y) = rt.find("mn/menu-item-3").unwrap();
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    settle(&mut rt);
    assert!(rt.state.chosen.is_empty());
}

#[test]
fn the_menu_is_a_named_menu_of_menuitems_with_a_checked_selection() {
    let rt = runtime(App {
        single: true,
        ..App::default()
    });
    let menu = part(&rt, "container").unwrap();
    assert_eq!(menu.role, "menu");
    assert_eq!(menu.label.as_deref(), Some("Actions"));
    let two = part(&rt, "menu-item-2").unwrap();
    assert_eq!(two.role, "menuitem");
    let tree = rt.accessibility_json().to_string();
    assert!(
        tree.contains(r#""name":"Two","position":2,"role":"menu-item","states":["checked"]"#),
        "{tree}"
    );
    assert!(
        tree.contains(r#""name":"One","position":1,"role":"menu-item","states":["unchecked"]"#),
        "{tree}"
    );
}
