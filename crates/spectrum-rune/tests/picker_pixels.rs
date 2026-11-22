//! Pixel assertions for the Picker, which `ElementInfo` cannot give: the focus ring (colour, 2px
//! thickness, 2px gap) and the error border (1px, colour from the oracle), read from
//! `picker_rings.json` and `picker_scene.json` (from `design-data render picker --format scene`),
//! never from our `Tokens`. See `support/pixels.rs` for how it is rendered and its limits.

#[path = "support/pixels.rs"]
mod pixels;

use pixels::{composite, near, probe, Pixels, Ring, SIDES};
use rune::{col, Cx, El, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::picker::Picker;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

const SIZE: (u32, u32) = (420, 140);
const PAD: f32 = 20.0;

#[derive(Clone, Default)]
struct App {
    dark: bool,
    mobile: bool,
    size: Option<Size>,
    error: bool,
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
    // No label: nothing is drawn above the field (the ring test reads pixels there).
    col().px(PAD).py(PAD).child(
        Picker::new("pk")
            .value("Canada")
            .size(a.size.unwrap_or_default())
            .error(a.error)
            .build(t),
    )
}

fn runtime(a: App) -> Runtime<App> {
    let dark = a.dark;
    let mut rt = Runtime::new(a, view, (SIZE.0 as f32, SIZE.1 as f32), Env::new(dark, 1.0));
    for _ in 0..40 {
        rt.frame(0.05);
    }
    rt
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
        error: c["error"].as_bool().unwrap(),
    }
}

fn field(rt: &Runtime<App>) -> [f32; 4] {
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some("pk/field"))
        .unwrap()
        .target
}

/// Under keyboard focus the field has the oracle's ring: its colour and thickness at its gap,
/// with clear pixels in the gap and beyond the ring. Without focus there is none.
#[test]
fn focus_ring_matches_oracle() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/picker_rings.json")).unwrap();
    assert_eq!(cases.len(), 32);
    for c in &cases {
        let what = c.to_string();
        let mut rt = runtime(app_for(c));
        let ring = Ring::of(c);
        ring.assert_absent(&Pixels::of(&rt, SIZE), field(&rt), &SIDES, &what);
        rt.key(KeyEvent::from(Key::Tab));
        for _ in 0..40 {
            rt.frame(0.05);
        }
        ring.assert_around(&Pixels::of(&rt, SIZE), field(&rt), &SIDES, &what);
    }
}

/// An error field has the oracle's 1px border inside its edge, then the field fill; without
/// error the edge pixel is the fill.
#[test]
fn error_border_matches_oracle() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/picker_scene.json")).unwrap();
    let mut n = 0;
    for c in cases.iter().filter(|c| {
        c["state"] == "rest" && c["content"] == "value" && !c["disabled"].as_bool().unwrap()
    }) {
        let want = &c["field"];
        let fill = composite(&[want["fill"].as_str().unwrap()], [0; 3]);
        let rt = runtime(app_for(c));
        let px = Pixels::of(&rt, SIZE);
        // The middle of the left and right edges is clear of the rounded corners.
        for side in [pixels::Side::Left, pixels::Side::Right] {
            let (edge, inner) = (probe(&px, field(&rt), side, 0), probe(&px, field(&rt), side, 1));
            let edge_want = match want["stroke"].as_str() {
                Some(s) => {
                    assert_eq!(want["stroke_width"].as_f64(), Some(1.0));
                    composite(&[s], [0; 3])
                }
                None => fill,
            };
            near(edge, edge_want, &format!("{c} {side:?} edge"));
            near(inner, fill, &format!("{c} {side:?} inside the edge"));
        }
        n += 1;
    }
    assert_eq!(n, 64);
}
