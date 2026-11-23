//! Pixel assertions for the Tag, which `ElementInfo` cannot give: under keyboard focus the fill (`El::focus_fill` is laid
//! over the tag) and the ring (2 pt, 2 pt off the container), read from `tag_scene.json` (from `design-data render tag
//! --format scene`), never from our `Tokens`. See `support/pixels.rs`.

#[path = "support/pixels.rs"]
mod pixels;

use pixels::Pixels;
use rune::{col, Cx, El, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::tag::Tag;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};

const SIZE: (u32, u32) = (200, 100);

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
    let look = c["look"].as_str().unwrap();
    col().padding(30.0).child(
        Tag::new("tg", "Label")
            .selected(look != "plain")
            .emphasized(look == "emphasized")
            .on_press(|_: &mut App| {})
            .build(t),
    )
}

fn rgb(s: &str) -> [i32; 3] {
    let v: Vec<i32> = s.split(',').map(|c| c.parse().unwrap()).collect();
    [v[0], v[1], v[2]]
}

#[test]
fn keyboard_focus_takes_the_hover_fill_and_draws_the_ring() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("fixtures/tag_scene.json")).unwrap();
    let mut n = 0;
    for c in cases.iter().filter(|c| c["state"] == "keyboard-focus") {
        n += 1;
        let what = format!("{} {} {}", c["scheme"], c["scale"], c["look"]);
        let mut rt = Runtime::new(
            App { case: c.clone() },
            view,
            (SIZE.0 as f32, SIZE.1 as f32),
            Env::new(c["scheme"] == "dark", 1.0),
        );
        for _ in 0..20 {
            rt.frame(0.05);
        }
        rt.key(KeyEvent::from(Key::Tab));
        for _ in 0..20 {
            rt.frame(0.05);
        }
        let px = Pixels::of(&rt, SIZE);
        let ct = rt
            .elements()
            .into_iter()
            .find(|e| e.key.as_deref() == Some("tg/container"))
            .unwrap()
            .target;
        let mid_y = (ct[1] + ct[3] / 2.0) as u32;
        // Clear of the label and the rounded corners: a few points in from the left edge.
        let fill = px.at(ct[0] as u32 + 3, mid_y);
        let want = rgb(c["container"]["fill"].as_str().unwrap());
        assert!(fill[3] > 250, "{what}: the fill is not opaque: {fill:?}");
        for i in 0..3 {
            assert!(
                (i32::from(fill[i]) - want[i]).abs() <= 4,
                "{what}: fill {fill:?}, want {want:?}"
            );
        }
        let ring_x = (ct[0] + c["ring"]["x"].as_f64().unwrap() as f32) as u32;
        let ring = px.at(ring_x, mid_y);
        let ring_want = rgb(c["ring"]["color"].as_str().unwrap());
        assert!(ring[3] > 200, "{what}: no ring at {ring_x}: {ring:?}");
        for i in 0..3 {
            assert!(
                (i32::from(ring[i]) - ring_want[i]).abs() <= 6,
                "{what}: ring {ring:?}, want {ring_want:?}"
            );
        }
    }
    assert_eq!(n, 12);
}
