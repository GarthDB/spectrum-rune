//! Pixel assertions for the Clear button, which `ElementInfo` cannot give: the keyboard focus ring (2 pt, 2 pt off the
//! circle) and none at rest, read from `clear_button_scene.json` (from `design-data render clear-button --format scene`),
//! never from our `Tokens`. See `support/pixels.rs`.

#[path = "support/pixels.rs"]
mod pixels;

use pixels::Pixels;
use rune::{col, Cx, El, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::clear_button::ClearButton;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};

const SIZE: (u32, u32) = (120, 120);

#[derive(Clone)]
struct App {
    case: Value,
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
    col()
        .padding(30.0)
        .child(ClearButton::new("cb").on_press(|_: &mut App| {}).build(t))
}

fn rgb(s: &str) -> [i32; 3] {
    let v: Vec<i32> = s.split(',').map(|c| c.parse().unwrap()).collect();
    [v[0], v[1], v[2]]
}

#[test]
fn keyboard_focus_draws_the_oracles_ring_and_rest_draws_none() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/clear_button_scene.json")).unwrap();
    let mut n = 0;
    for c in cases.iter().filter(|c| {
        c["size"] == "m"
            && c["look"] == "plain"
            && (c["state"] == "keyboard-focus" || c["state"] == "rest")
    }) {
        n += 1;
        let what = format!("{} {} {}", c["scheme"], c["scale"], c["state"]);
        let mut rt = Runtime::new(
            App { case: c.clone() },
            view,
            (SIZE.0 as f32, SIZE.1 as f32),
            Env::new(c["scheme"] == "dark", 1.0),
        );
        for _ in 0..20 {
            rt.frame(0.05);
        }
        if c["state"] == "keyboard-focus" {
            rt.key(KeyEvent::from(Key::Tab));
            for _ in 0..20 {
                rt.frame(0.05);
            }
        }
        let px = Pixels::of(&rt, SIZE);
        let ct = rt
            .elements()
            .into_iter()
            .find(|e| e.key.as_deref() == Some("cb/container"))
            .unwrap()
            .target;
        let mid_y = (ct[1] + ct[3] / 2.0) as u32;
        let ring_x = (ct[0] + c["ring"]["x"].as_f64().unwrap_or(-3.0) as f32) as u32;
        let ring = px.at(ring_x, mid_y);
        if c["state"] == "keyboard-focus" {
            let want = rgb(c["ring"]["color"].as_str().unwrap());
            assert!(ring[3] > 200, "{what}: no ring at {ring_x}: {ring:?}");
            for i in 0..3 {
                assert!(
                    (i32::from(ring[i]) - want[i]).abs() <= 6,
                    "{what}: ring {ring:?}, want {want:?}"
                );
            }
            assert!(
                px.at(ct[0] as u32 - 1, mid_y)[3] < 20,
                "{what}: the gap to the ring is not empty"
            );
        } else {
            assert!(ring[3] < 20, "{what}: a ring at rest: {ring:?}");
        }
    }
    assert_eq!(n, 8);
}
