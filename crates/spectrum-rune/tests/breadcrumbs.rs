//! Checks the Breadcrumbs against the `design-data render breadcrumbs --format scene` oracle. Regenerate the fixture with
//! `tests/fixtures/regen-breadcrumbs.sh` (its header explains the matrix). Text widths depend on the installed font, so
//! only what does not is compared: the heights, the offsets inside an item, the gap after the title, type and colours.

use rune::{col, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::breadcrumbs::{BreadcrumbItem, Breadcrumbs, Separator, Size};
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};

#[derive(Clone, Default)]
struct App {
    dark: bool,
    mobile: bool,
    large: bool,
    no_separator: bool,
    went: Vec<usize>,
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
        Breadcrumbs::new("bc")
            .size(if a.large { Size::L } else { Size::M })
            .separator(if a.no_separator {
                Separator::None
            } else {
                Separator::Chevron
            })
            .item(BreadcrumbItem::new("Home"))
            .item(BreadcrumbItem::new("Docs"))
            .item(BreadcrumbItem::new("Current").current(true))
            .item(BreadcrumbItem::new("Gone").disabled(true))
            .on_navigate(|s: &mut App, i| s.went.push(i))
            .build(t),
    )
}

fn runtime(a: App) -> Runtime<App> {
    let dark = a.dark;
    let mut rt = Runtime::new(a, view, (500.0, 200.0), Env::new(dark, 1.0));
    for _ in 0..20 {
        rt.frame(0.05);
    }
    rt
}

fn part(rt: &Runtime<App>, name: &str) -> Option<ElementInfo> {
    let key = format!("bc/{name}");
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

#[test]
fn matches_oracle_scene() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/breadcrumbs_scene.json")).unwrap();
    assert_eq!(cases.len(), 16);
    for c in &cases {
        let what = format!(
            "{} {} {} {}",
            c["scheme"], c["scale"], c["size"], c["separator"]
        );
        let rt = runtime(App {
            dark: c["scheme"] == "dark",
            mobile: c["scale"] == "mobile",
            large: c["size"] == "l",
            no_separator: c["separator"] == "none",
            ..App::default()
        });
        let ct = part(&rt, "container").unwrap();
        near(
            ct.target[3],
            &c["container"]["h"],
            &format!("{what} height"),
        );
        for (i, want) in c["items"].as_array().unwrap().iter().enumerate() {
            let n = i + 1;
            let what = format!("{what} item {n}");
            let item = part(&rt, &format!("breadcrumb-item-{n}")).unwrap();
            let title = part(&rt, &format!("breadcrumb-item-{n}.breadcrumb-title")).unwrap();
            near(
                item.target[1] - ct.target[1],
                &want["item"]["y"],
                &format!("{what} y"),
            );
            near(item.target[3], &want["item"]["h"], &format!("{what} h"));
            near(
                title.target[0] - item.target[0],
                &want["title"]["x"],
                &format!("{what} title x"),
            );
            near(
                title.target[1] - item.target[1],
                &want["title"]["y"],
                &format!("{what} title y"),
            );
            near(
                title.target[3],
                &want["title"]["h"],
                &format!("{what} title h"),
            );
            assert_color(
                title.text_color.unwrap(),
                want["title"]["color"].as_str().unwrap(),
                &format!("{what} title"),
            );
            match part(&rt, &format!("breadcrumb-item-{n}.separator")) {
                None => assert!(want["separator"].is_null(), "{what}: no separator drawn"),
                Some(s) => {
                    let w = &want["separator"];
                    near(
                        s.target[0] - (title.target[0] + title.target[2]),
                        &w["gap"],
                        &format!("{what} separator gap"),
                    );
                    near(
                        s.target[1] - item.target[1],
                        &w["y"],
                        &format!("{what} separator y"),
                    );
                    near(s.target[2], &w["w"], &format!("{what} separator w"));
                    assert_color(
                        s.text_color.unwrap(),
                        w["color"].as_str().unwrap(),
                        &format!("{what} separator"),
                    );
                }
            }
        }
    }
}

#[test]
fn pressing_a_link_navigates_but_the_current_and_disabled_items_do_not() {
    let mut rt = runtime(App::default());
    for n in [2, 3, 4, 1] {
        let (x, y) = rt.find(&format!("bc/breadcrumb-item-{n}")).unwrap();
        rt.pointer_move(x, y);
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(0.05);
    }
    assert_eq!(
        rt.state.went,
        [1, 0],
        "only Docs and Home navigate (by index)"
    );
    let tree = rt.accessibility_json().to_string();
    assert!(tree.contains("\"name\":\"Breadcrumbs\""), "{tree}");
}

#[test]
fn enter_follows_the_focused_link() {
    let mut rt = runtime(App::default());
    rt.key(KeyEvent::from(Key::Tab));
    rt.key(KeyEvent::from(Key::Enter));
    for _ in 0..3 {
        rt.frame(0.05);
    }
    assert_eq!(rt.state.went, [0]);
}
