//! Checks the Segmented control against the `design-data render segmented-control --format scene` oracle. Regenerate the fixture with
//! `tests/fixtures/regen-segmented-control.sh` (its header explains the matrix). Text widths follow the installed font, so a tab's width is
//! not compared: the gaps between the tabs, the heights, offsets, sizes and colours are.

use rune::{col, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::icons::WORKFLOW_SETTINGS;
use spectrum_rune::segmented_control::{SegmentedControl, SegmentedItem};
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
    let item = |l: &str| SegmentedItem::new(l).icon(&WORKFLOW_SETTINGS);
    col().padding(20.0).child(
        SegmentedControl::new("sc")
            .size(match c["size"].as_str().unwrap() {
                "s" => Size::S,
                "m" => Size::M,
                "l" => Size::L,
                _ => Size::XL,
            })
            .item(item("Home"))
            .item(item("Docs").selected(true))
            .item(item("Gone").disabled(true))
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
    let key = format!("sc/{name}");
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
        serde_json::from_str(include_str!("fixtures/segmented_control_scene.json")).unwrap();
    assert_eq!(cases.len(), 16);
    for c in &cases {
        let what = format!("{} {} {}", c["scheme"], c["scale"], c["size"]);
        let rt = runtime(c);
        let ct = part(&rt, "container").unwrap();
        near(ct.target[3], &c["container"]["h"], &format!("{what} h"));
        near(
            ct.radius,
            &c["container"]["radius"],
            &format!("{what} radius"),
        );
        assert_color(
            ct.fill,
            c["container"]["fill"].as_str().unwrap(),
            &format!("{what} track"),
        );
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
            near(
                item.radius,
                &want["item"]["radius"],
                &format!("{what} radius"),
            );
            if n == 1 {
                near(
                    item.target[0] - ct.target[0],
                    &want["item"]["x"],
                    &format!("{what} x"),
                );
            }
            if let Some(gap) = want["item"]["gap_after"].as_f64() {
                let next = part(&rt, &format!("item-{}", n + 1)).unwrap();
                let got = next.target[0] - (item.target[0] + item.target[2]);
                assert!(
                    (f64::from(got) - gap).abs() <= 1.0,
                    "{what} gap to the next item: got {got}, want {gap}"
                );
            }
            match want.get("fill") {
                Some(f) => assert_color(item.fill, f.as_str().unwrap(), &format!("{what} fill")),
                None => assert_eq!(item.fill.a, 0.0, "{what}: should have no fill"),
            }
            let ic = part(&rt, &format!("item-{n}.icon")).unwrap();
            near(
                ic.target[0] - item.target[0],
                &want["icon"]["x"],
                &format!("{what} icon x"),
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
            assert_color(
                l.text_color.unwrap(),
                want["label"]["color"].as_str().unwrap(),
                &format!("{what} label"),
            );
        }
    }
}

#[test]
fn press_enter_and_arrows_select_by_index_and_a_disabled_item_is_inert() {
    let c = serde_json::json!({"scheme":"light","scale":"desktop","size":"m"});
    let mut rt = runtime(&c);
    for n in [2, 3, 1] {
        let (x, y) = rt.find(&format!("sc/item-{n}")).unwrap();
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
    // Rune activates an item as focus arrives on it, so Right alone selects Docs.
    assert_eq!(rt.state.went, [1, 0, 1], "Tab to Home, Right to Docs");
    rt.key(KeyEvent::from(Key::Enter));
    for _ in 0..5 {
        rt.frame(0.05);
    }
    assert_eq!(rt.state.went, [1, 0, 1, 1], "and Enter selects it again");
    assert!(!part(&rt, "item-3").unwrap().focusable);
}
