//! Checks the Side navigation against the `design-data render side-navigation --format scene` oracle. Regenerate the fixture
//! with `tests/fixtures/regen-side-navigation.sh` (its header explains the matrix). Text widths follow the installed font,
//! so only heights, offsets, sizes and colours are compared.

use rune::{col, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::icons::WORKFLOW_SETTINGS;
use spectrum_rune::side_navigation::{SideNavigation, SideNavigationItem};
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};

#[derive(Clone, Default)]
struct App {
    case: Value,
    went: Vec<usize>,
}

fn view(a: &App, _: &mut Cx<App>) -> El {
    let c = &a.case;
    let t = tokens(
        if c["scheme"] == "dark" {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        },
        if c["scale"] == "mobile" {
            Scale::Mobile
        } else {
            Scale::Desktop
        },
    );
    let item = |l: &str| SideNavigationItem::new(l).icon(&WORKFLOW_SETTINGS);
    let mut n = SideNavigation::new("sn");
    if c["title"] == "with" {
        n = n.title("Section");
    }
    col().padding(20.0).child(
        n.item(item("Home"))
            .item(item("Docs").selected(true))
            .item(item("Gone").disabled(true))
            .item(item("Settings"))
            .on_select(|s: &mut App, i| s.went.push(i))
            .build(t),
    )
}

fn runtime(c: &Value) -> Runtime<App> {
    let mut rt = Runtime::new(
        App {
            case: c.clone(),
            went: Vec::new(),
        },
        view,
        (400.0, 400.0),
        Env::new(c["scheme"] == "dark", 1.0),
    );
    for _ in 0..20 {
        rt.frame(0.05);
    }
    rt
}

fn part(rt: &Runtime<App>, name: &str) -> Option<ElementInfo> {
    let key = format!("sn/{name}");
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
        serde_json::from_str(include_str!("fixtures/side_navigation_scene.json")).unwrap();
    assert_eq!(cases.len(), 8);
    for c in &cases {
        let what = format!("{} {} title {}", c["scheme"], c["scale"], c["title"]);
        let rt = runtime(c);
        let ct = part(&rt, "container").unwrap();
        near(ct.target[2], &c["container"]["w"], &format!("{what} w"));
        near(ct.target[3], &c["container"]["h"], &format!("{what} h"));
        match (part(&rt, "section-header"), c.get("header")) {
            (None, None) => {}
            (Some(h), Some(w)) => {
                near(
                    h.target[1] - ct.target[1],
                    &w["y"],
                    &format!("{what} header y"),
                );
                near(h.target[3], &w["h"], &format!("{what} header h"));
                assert_color(
                    h.text_color.unwrap(),
                    w["color"].as_str().unwrap(),
                    &format!("{what} header"),
                );
            }
            (got, want) => panic!(
                "{what}: header drawn {}, oracle {}",
                got.is_some(),
                want.is_some()
            ),
        }
        for (i, want) in c["items"].as_array().unwrap().iter().enumerate() {
            let n = i + 1;
            let what = format!("{what} item {n}");
            let item = part(&rt, &format!("item-{n}")).unwrap();
            near(
                item.target[1] - ct.target[1],
                &want["item"]["y"],
                &format!("{what} y"),
            );
            near(item.target[3], &want["item"]["h"], &format!("{what} h"));
            near(item.target[2], &want["item"]["w"], &format!("{what} w"));
            let ic = part(&rt, &format!("item-{n}.icon")).unwrap();
            near(
                ic.target[0] - item.target[0],
                &want["icon"]["x"],
                &format!("{what} icon x"),
            );
            near(
                ic.target[1] - item.target[1],
                &want["icon"]["y"],
                &format!("{what} icon y"),
            );
            near(ic.target[2], &want["icon"]["w"], &format!("{what} icon w"));
            assert_color(
                ic.text_color.unwrap(),
                want["icon"]["color"].as_str().unwrap(),
                &format!("{what} icon"),
            );
            let l = part(&rt, &format!("item-{n}.label")).unwrap();
            near(
                l.target[0] - item.target[0],
                &want["label"]["x"],
                &format!("{what} label x"),
            );
            near(
                l.target[1] - item.target[1],
                &want["label"]["y"],
                &format!("{what} label y"),
            );
            near(l.target[3], &want["label"]["h"], &format!("{what} label h"));
            assert_color(
                l.text_color.unwrap(),
                want["label"]["color"].as_str().unwrap(),
                &format!("{what} label"),
            );
            match (
                part(&rt, &format!("item-{n}.selection-indicator")),
                want.get("indicator"),
            ) {
                (None, None) => {}
                (Some(s), Some(w)) => {
                    near(
                        s.target[0] - item.target[0],
                        &w["x"],
                        &format!("{what} indicator x"),
                    );
                    near(
                        s.target[1] - item.target[1],
                        &w["y"],
                        &format!("{what} indicator y"),
                    );
                    near(s.target[2], &w["w"], &format!("{what} indicator w"));
                    near(s.target[3], &w["h"], &format!("{what} indicator h"));
                    assert_color(
                        s.fill,
                        w["fill"].as_str().unwrap(),
                        &format!("{what} indicator"),
                    );
                }
                (got, want) => panic!(
                    "{what}: indicator drawn {}, oracle {}",
                    got.is_some(),
                    want.is_some()
                ),
            }
        }
    }
}

#[test]
fn press_and_enter_select_by_index_and_a_disabled_item_is_inert() {
    let c = serde_json::json!({"scheme":"light","scale":"desktop","title":"without"});
    let mut rt = runtime(&c);
    for n in [2, 3, 1] {
        let (x, y) = rt.find(&format!("sn/item-{n}")).unwrap();
        rt.pointer_move(x, y);
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(0.05);
    }
    assert_eq!(rt.state.went, [1, 0], "Docs and Home; Gone is disabled");
    rt.pointer_move(390.0, 390.0);
    rt.key(KeyEvent::from(Key::Tab));
    rt.key(KeyEvent::from(Key::Down));
    rt.key(KeyEvent::from(Key::Enter));
    for _ in 0..5 {
        rt.frame(0.05);
    }
    assert_eq!(rt.state.went, [1, 0, 1], "Tab to Home, Down to Docs, Enter");
    assert!(!part(&rt, "item-3").unwrap().focusable);
    let tree = rt.accessibility_json().to_string();
    assert!(tree.contains("\"name\":\"Side navigation\""), "{tree}");
}
