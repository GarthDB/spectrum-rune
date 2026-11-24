//! Checks the Tabs against the `design-data render tabs --format scene` oracle. Regenerate the fixture with
//! `tests/fixtures/regen-tabs.sh` (its header explains the matrix). Text widths follow the installed font, so a tab's width is
//! not compared: the gaps between the tabs, the heights, offsets, sizes and colours are.

use rune::{col, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::icons::WORKFLOW_SETTINGS;
use spectrum_rune::tabs::{Density, TabItem, Tabs};
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

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
    let item = |l: &str| TabItem::new(l).icon(&WORKFLOW_SETTINGS);
    col().padding(20.0).child(
        Tabs::new("tb")
            .size(match c["size"].as_str().unwrap() {
                "s" => Size::S,
                "m" => Size::M,
                "l" => Size::L,
                _ => Size::XL,
            })
            .density(if c["density"] == "compact" {
                Density::Compact
            } else {
                Density::Regular
            })
            .item(item("Home"))
            .item(item("Docs").selected(true))
            .item(item("Gone").disabled(true))
            .item(item("More"))
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
        (600.0, 300.0),
        Env::new(c["scheme"] == "dark", 1.0),
    );
    for _ in 0..20 {
        rt.frame(0.05);
    }
    rt
}

fn part(rt: &Runtime<App>, name: &str) -> Option<ElementInfo> {
    let key = format!("tb/{name}");
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
    let cases: Vec<Value> = serde_json::from_str(include_str!("fixtures/tabs_scene.json")).unwrap();
    assert_eq!(cases.len(), 32);
    for c in &cases {
        let what = format!(
            "{} {} {} {}",
            c["scheme"], c["scale"], c["size"], c["density"]
        );
        let rt = runtime(c);
        let ct = part(&rt, "container").unwrap();
        near(ct.target[3], &c["container"]["h"], &format!("{what} h"));
        for (i, want) in c["items"].as_array().unwrap().iter().enumerate() {
            let n = i + 1;
            let what = format!("{what} tab {n}");
            let item = part(&rt, &format!("tab-item-{n}")).unwrap();
            near(
                item.target[1] - ct.target[1],
                &want["item"]["y"],
                &format!("{what} y"),
            );
            near(item.target[3], &want["item"]["h"], &format!("{what} h"));
            if n == 1 {
                near(
                    item.target[0] - ct.target[0],
                    &want["item"]["x"],
                    &format!("{what} x"),
                );
            }
            if let Some(gap) = want["item"]["gap_after"].as_f64() {
                let next = part(&rt, &format!("tab-item-{}", n + 1)).unwrap();
                let got = next.target[0] - (item.target[0] + item.target[2]);
                assert!(
                    (f64::from(got) - gap).abs() <= 1.0,
                    "{what} gap to the next tab: got {got}, want {gap}"
                );
            }
            let ic = part(&rt, &format!("tab-item-{n}.icon")).unwrap();
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
            let l = part(&rt, &format!("tab-item-{n}.label")).unwrap();
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
                part(&rt, &format!("tab-item-{n}.selection-indicator")),
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
                    near(s.target[3], &w["h"], &format!("{what} indicator h"));
                    assert_eq!(w["w_matches_item"], true);
                    assert!(
                        (s.target[2] - item.target[2]).abs() <= 1.0,
                        "{what}: the indicator is not as wide as the tab"
                    );
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
fn press_enter_and_arrows_select_by_index_and_a_disabled_tab_is_inert() {
    let c = serde_json::json!({"scheme":"light","scale":"desktop","size":"m","density":"regular"});
    let mut rt = runtime(&c);
    for n in [2, 3, 1] {
        let (x, y) = rt.find(&format!("tb/tab-item-{n}")).unwrap();
        rt.pointer_move(x, y);
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(0.05);
    }
    assert_eq!(rt.state.went, [1, 0], "Docs and Home; Gone is disabled");
    rt.pointer_move(590.0, 290.0);
    rt.key(KeyEvent::from(Key::Tab));
    rt.frame(0.05);
    rt.key(KeyEvent::from(Key::Right));
    rt.frame(0.05);
    // Rune activates a tab as focus arrives on it (automatic activation), so Right alone selects Docs.
    assert_eq!(rt.state.went, [1, 0, 1], "Tab to Home, Right to Docs");
    rt.key(KeyEvent::from(Key::Enter));
    for _ in 0..5 {
        rt.frame(0.05);
    }
    assert_eq!(rt.state.went, [1, 0, 1, 1], "and Enter selects it again");
    assert!(!part(&rt, "tab-item-3").unwrap().focusable);
    let tree = rt.accessibility_json().to_string();
    assert!(
        tree.contains("\"role\":\"tab-list\"") || tree.contains("tab"),
        "{tree}"
    );
}
