//! Checks the Link against the `design-data render link --format scene` oracle. Regenerate the fixture with
//! `tests/fixtures/regen-divider-link.sh`. Text widths follow the installed font, so only heights, type size and colours are
//! compared. Under keyboard focus the colour is laid over the text (`group_color`), so `ElementInfo` has the rest colour:
//! `link_pixels.rs` reads the ring from pixels.

use rune::{col, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::link::{Link, Variant};
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};

#[derive(Clone, Default)]
struct App {
    case: Value,
    pressed: u32,
}

pub fn link(c: &Value, pressed: bool) -> El {
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
    let l = Link::new("lk", "Label")
        .variant(if c["variant"] == "secondary" {
            Variant::Secondary
        } else {
            Variant::Primary
        })
        .quiet(c["quiet"] == true)
        .standalone(c["standalone"] == true);
    col().padding(20.0).child(
        if pressed {
            l.on_press(|s: &mut App| s.pressed += 1)
        } else {
            l
        }
        .build(t),
    )
}

fn view(a: &App, _: &mut Cx<App>) -> El {
    link(&a.case, true)
}

fn part(rt: &Runtime<App>, name: &str) -> Option<ElementInfo> {
    let key = format!("lk/{name}");
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key.as_str()))
}

fn rgb(s: &str) -> [f32; 3] {
    let v: Vec<f32> = s.split(',').map(|c| c.parse().unwrap()).collect();
    [v[0], v[1], v[2]]
}

fn settle(rt: &mut Runtime<App>) {
    for _ in 0..30 {
        rt.frame(0.05);
    }
}

#[test]
fn matches_oracle_scene() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("fixtures/link_scene.json")).unwrap();
    assert_eq!(cases.len(), 128);
    for c in &cases {
        let what = format!(
            "{} {} {} quiet {} standalone {} {}",
            c["scheme"], c["scale"], c["variant"], c["quiet"], c["standalone"], c["state"]
        );
        let mut rt = Runtime::new(
            App {
                case: c.clone(),
                pressed: 0,
            },
            view,
            (300.0, 120.0),
            Env::new(c["scheme"] == "dark", 1.0),
        );
        settle(&mut rt);
        let (x, y) = rt.find("lk/container").unwrap();
        match c["state"].as_str().unwrap() {
            "hover" => rt.pointer_move(x, y),
            "down" => {
                rt.pointer_move(x, y);
                rt.pointer_down(x, y);
            }
            "keyboard-focus" => rt.key(KeyEvent::from(Key::Tab)),
            _ => {}
        }
        settle(&mut rt);
        let ct = part(&rt, "container").unwrap();
        let tx = part(&rt, "text").unwrap();
        assert!(
            (ct.target[3] - c["container"]["h"].as_f64().unwrap() as f32).abs() <= 1.0,
            "{what} h"
        );
        assert!(
            (ct.radius - c["container"]["radius"].as_f64().unwrap() as f32).abs() <= 1.0,
            "{what} radius"
        );
        assert!(
            (tx.target[3] - c["text"]["h"].as_f64().unwrap() as f32).abs() <= 1.0,
            "{what} text h"
        );
        if c["state"] != "keyboard-focus" {
            let hex = tx.text_color.unwrap().hex();
            let want = rgb(c["text"]["color"].as_str().unwrap());
            for i in 0..3 {
                let g = i32::from_str_radix(&hex[1 + 2 * i..3 + 2 * i], 16).unwrap() as f32;
                assert!(
                    (g - want[i]).abs() <= 3.0,
                    "{what} text: got {hex}, want {want:?}"
                );
            }
        }
    }
}

#[test]
fn press_and_enter_follow_the_link() {
    let c = serde_json::json!({"scheme":"light","scale":"desktop","variant":"primary","quiet":false,"standalone":true});
    let mut rt = Runtime::new(
        App {
            case: c,
            pressed: 0,
        },
        view,
        (300.0, 120.0),
        Env::new(false, 1.0),
    );
    settle(&mut rt);
    let (x, y) = rt.find("lk/container").unwrap();
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    settle(&mut rt);
    assert_eq!(rt.state.pressed, 1);
    rt.pointer_move(250.0, 100.0);
    rt.key(KeyEvent::from(Key::Tab));
    rt.key(KeyEvent::from(Key::Enter));
    settle(&mut rt);
    assert_eq!(rt.state.pressed, 2);
    let tree = rt.accessibility_json().to_string();
    assert!(
        tree.contains("\"role\":\"link\"") && tree.contains("\"name\":\"Label\""),
        "{tree}"
    );
}
