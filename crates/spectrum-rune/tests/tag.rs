//! Checks the Tag against the `design-data render tag --format scene` oracle. Regenerate the fixture with
//! `tests/fixtures/regen-tag.sh` (its header explains the matrix). Text widths follow the installed font, so only heights,
//! offsets, type size and colours are compared. Under keyboard focus the fill and colours are laid over the tag, so
//! `ElementInfo` has the rest ones: `tag_pixels.rs` reads them and the ring from pixels.

use rune::{col, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::tag::Tag;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

#[derive(Clone, Default)]
pub struct App {
    pub case: Value,
    pub pressed: u32,
}

pub fn tag(c: &Value) -> El {
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
    let look = c["look"].as_str().unwrap();
    let t1 = Tag::new("tg", "Label")
        .size(match c["size"].as_str().unwrap() {
            "s" => Size::S,
            "m" => Size::M,
            _ => Size::L,
        })
        .selected(look == "selected" || look == "emphasized")
        .emphasized(look == "emphasized")
        .disabled(look == "disabled")
        .on_press(|s: &mut App| s.pressed += 1);
    col().padding(20.0).child(t1.build(t))
}

fn view(a: &App, _: &mut Cx<App>) -> El {
    tag(&a.case)
}

fn part(rt: &Runtime<App>, name: &str) -> Option<ElementInfo> {
    let key = format!("tg/{name}");
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

fn settle(rt: &mut Runtime<App>) {
    for _ in 0..30 {
        rt.frame(0.05);
    }
}

#[test]
fn matches_oracle_scene() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("fixtures/tag_scene.json")).unwrap();
    assert_eq!(cases.len(), 84);
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
            (300.0, 120.0),
            Env::new(c["scheme"] == "dark", 1.0),
        );
        settle(&mut rt);
        let (x, y) = rt.find("tg/container").unwrap();
        let focus = c["state"] == "keyboard-focus";
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
        near(ct.target[3], &c["container"]["h"], &format!("{what} h"));
        near(
            ct.radius,
            &c["container"]["radius"],
            &format!("{what} radius"),
        );
        let l = part(&rt, "label").unwrap();
        near(
            l.target[0] - ct.target[0],
            &c["label"]["x"],
            &format!("{what} label x"),
        );
        near(
            l.target[1] - ct.target[1],
            &c["label"]["y"],
            &format!("{what} label y"),
        );
        near(l.target[3], &c["label"]["h"], &format!("{what} label h"));
        if !focus {
            assert_color(
                ct.fill,
                c["container"]["fill"].as_str().unwrap(),
                &format!("{what} fill"),
            );
            assert_color(
                l.text_color.unwrap(),
                c["label"]["color"].as_str().unwrap(),
                &format!("{what} label"),
            );
        }
    }
}

#[test]
fn press_and_enter_call_the_handler_and_a_disabled_tag_does_not() {
    let plain = serde_json::json!({"scheme":"light","scale":"desktop","size":"m","look":"plain"});
    let mut rt = Runtime::new(
        App {
            case: plain,
            pressed: 0,
        },
        view,
        (300.0, 120.0),
        Env::new(false, 1.0),
    );
    settle(&mut rt);
    let (x, y) = rt.find("tg/container").unwrap();
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    settle(&mut rt);
    assert_eq!(rt.state.pressed, 1);
    rt.pointer_move(280.0, 100.0);
    rt.key(KeyEvent::from(Key::Tab));
    rt.key(KeyEvent::from(Key::Enter));
    settle(&mut rt);
    assert_eq!(rt.state.pressed, 2);
    let disabled =
        serde_json::json!({"scheme":"light","scale":"desktop","size":"m","look":"disabled"});
    let mut rt = Runtime::new(
        App {
            case: disabled,
            pressed: 0,
        },
        view,
        (300.0, 120.0),
        Env::new(false, 1.0),
    );
    settle(&mut rt);
    let (x, y) = rt.find("tg/container").unwrap();
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    settle(&mut rt);
    assert_eq!(rt.state.pressed, 0);
    assert!(!part(&rt, "container").unwrap().focusable);
}
