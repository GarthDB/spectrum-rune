//! Checks the Clear button against the `design-data render clear-button --format scene` oracle. Regenerate the fixture
//! with `tests/fixtures/regen-clear-button.sh` (its header explains the matrix). Under keyboard focus the colour is laid
//! over the cross, so `ElementInfo` has the rest one: `clear_button_pixels.rs` reads the ring from pixels.

use rune::{col, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::clear_button::ClearButton;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

#[derive(Clone, Default)]
struct App {
    case: Value,
    pressed: u32,
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
    col().padding(20.0).child(
        ClearButton::new("cb")
            .size(match c["size"].as_str().unwrap() {
                "s" => Size::S,
                "m" => Size::M,
                "l" => Size::L,
                _ => Size::XL,
            })
            .disabled(c["look"] == "disabled")
            .on_press(|s: &mut App| s.pressed += 1)
            .build(t),
    )
}

fn part(rt: &Runtime<App>, name: &str) -> Option<ElementInfo> {
    let key = format!("cb/{name}");
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key.as_str()))
}

fn rgb(s: &str) -> [f32; 3] {
    let v: Vec<f32> = s.split(',').map(|c| c.parse().unwrap()).collect();
    [v[0], v[1], v[2]]
}

fn near(got: f32, want: &Value, what: &str) {
    let w = want.as_f64().unwrap() as f32;
    assert!((got - w).abs() <= 1.0, "{what}: got {got}, want {w}");
}

fn settle(rt: &mut Runtime<App>) {
    for _ in 0..30 {
        rt.frame(0.05);
    }
}

#[test]
fn matches_oracle_scene() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/clear_button_scene.json")).unwrap();
    assert_eq!(cases.len(), 44);
    for c in &cases {
        let what = format!(
            "{} {} {} {} {}",
            c["scheme"], c["scale"], c["size"], c["look"], c["state"]
        );
        let mut rt = Runtime::new(
            App {
                case: c.clone(),
                pressed: 0,
            },
            view,
            (200.0, 120.0),
            Env::new(c["scheme"] == "dark", 1.0),
        );
        settle(&mut rt);
        let (x, y) = rt.find("cb/container").unwrap();
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
        near(ct.target[2], &c["container"]["w"], &format!("{what} w"));
        near(ct.target[3], &c["container"]["h"], &format!("{what} h"));
        near(
            ct.radius,
            &c["container"]["radius"],
            &format!("{what} radius"),
        );
        let ic = part(&rt, "icon").unwrap();
        near(
            ic.target[0] - ct.target[0],
            &c["icon"]["x"],
            &format!("{what} icon x"),
        );
        near(
            ic.target[1] - ct.target[1],
            &c["icon"]["y"],
            &format!("{what} icon y"),
        );
        near(ic.target[2], &c["icon"]["w"], &format!("{what} icon w"));
        if c["state"] != "keyboard-focus" {
            let hex = ic.text_color.unwrap().hex();
            let want = rgb(c["icon"]["color"].as_str().unwrap());
            for i in 0..3 {
                let g = i32::from_str_radix(&hex[1 + 2 * i..3 + 2 * i], 16).unwrap() as f32;
                assert!(
                    (g - want[i]).abs() <= 3.0,
                    "{what} icon: got {hex}, want {want:?}"
                );
            }
        }
    }
}

#[test]
fn press_and_enter_call_the_handler_and_a_disabled_button_does_not() {
    let plain = serde_json::json!({"scheme":"light","scale":"desktop","size":"m","look":"plain"});
    let mut rt = Runtime::new(
        App {
            case: plain,
            pressed: 0,
        },
        view,
        (200.0, 120.0),
        Env::new(false, 1.0),
    );
    settle(&mut rt);
    let (x, y) = rt.find("cb/container").unwrap();
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    settle(&mut rt);
    assert_eq!(rt.state.pressed, 1);
    rt.pointer_move(190.0, 110.0);
    rt.key(KeyEvent::from(Key::Tab));
    rt.key(KeyEvent::from(Key::Enter));
    settle(&mut rt);
    assert_eq!(rt.state.pressed, 2);
    let tree = rt.accessibility_json().to_string();
    assert!(tree.contains("\"name\":\"Clear\""), "{tree}");
    let disabled =
        serde_json::json!({"scheme":"light","scale":"desktop","size":"m","look":"disabled"});
    let mut rt = Runtime::new(
        App {
            case: disabled,
            pressed: 0,
        },
        view,
        (200.0, 120.0),
        Env::new(false, 1.0),
    );
    settle(&mut rt);
    let (x, y) = rt.find("cb/container").unwrap();
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    settle(&mut rt);
    assert_eq!(rt.state.pressed, 0);
    assert!(!part(&rt, "container").unwrap().focusable);
}
