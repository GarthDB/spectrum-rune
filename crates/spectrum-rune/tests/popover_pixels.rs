//! Pixel assertions for the Popover, which `ElementInfo` cannot give: its drop shadow (offset 2, blur 8; 16% black in
//! light, 48% in dark, from `popover_scene.json`). See `support/pixels.rs`.

#[path = "support/pixels.rs"]
mod pixels;

use pixels::Pixels;
use rune::{col, el, Cx, El, Env, Runtime};
use serde_json::Value;
use spectrum_rune::popover::Popover;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};

const SIZE: (u32, u32) = (300, 200);

#[derive(Clone)]
struct App {
    dark: bool,
    tip: bool,
}

fn view(a: &App, _: &mut Cx<App>) -> El {
    let t = tokens(
        if a.dark {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        },
        Scale::Desktop,
    );
    col().padding(60.0).child(
        Popover::new("pp")
            .hide_tip(!a.tip)
            .width(120.0)
            .height(60.0)
            .child(el())
            .build(t),
    )
}

/// The shadow's alpha 3 points below the container's bottom edge, over a transparent canvas.
fn shadow_alpha(dark: bool) -> (u8, u8) {
    let mut rt = Runtime::new(
        App { dark, tip: false },
        view,
        (SIZE.0 as f32, SIZE.1 as f32),
        Env::new(dark, 1.0),
    );
    for _ in 0..20 {
        rt.frame(0.05);
    }
    let px = Pixels::of(&rt, SIZE);
    let c = rt
        .elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some("pp/container"))
        .unwrap()
        .target;
    let x = (c[0] + c[2] / 2.0) as u32;
    let below = px.at(x, (c[1] + c[3]) as u32 + 3)[3];
    let far = px.at(x, (c[1] + c[3]) as u32 + 40)[3];
    (below, far)
}

#[test]
fn the_shadow_falls_below_the_container_and_is_stronger_in_dark() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/popover_scene.json")).unwrap();
    let alpha = |scheme: &str| -> f64 {
        let c = cases
            .iter()
            .find(|c| c["scheme"] == scheme && c["hideTip"] == true)
            .unwrap();
        c["container"]["shadow"][0]["color"]
            .as_str()
            .unwrap()
            .split(',')
            .nth(3)
            .unwrap()
            .parse()
            .unwrap()
    };
    let (light_below, light_far) = shadow_alpha(false);
    let (dark_below, dark_far) = shadow_alpha(true);
    assert!(
        light_below > 4,
        "light: no shadow below the container ({light_below})"
    );
    assert_eq!(
        (light_far, dark_far),
        (0, 0),
        "the shadow is within its blur"
    );
    assert!(
        dark_below > light_below,
        "dark {dark_below} is not stronger than light {light_below}"
    );
    // The oracle says dark is three times as opaque; the blurred edge keeps the ratio roughly.
    assert!(alpha("dark") > 2.5 * alpha("light"));
}

/// The tip is the container's colour (the oracle's `tip.fill`), drawn opaque just outside the container's edge.
#[test]
fn the_tip_is_drawn_in_the_surface_colour() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/popover_scene.json")).unwrap();
    for dark in [false, true] {
        let c = cases
            .iter()
            .find(|c| {
                c["scheme"] == if dark { "dark" } else { "light" }
                    && c["placement"] == "top"
                    && c["hideTip"] == false
            })
            .unwrap();
        let want: Vec<u8> = c["tip"]["fill"]
            .as_str()
            .unwrap()
            .split(',')
            .map(|v| v.parse().unwrap())
            .collect();
        let mut rt = Runtime::new(
            App { dark, tip: true },
            view,
            (SIZE.0 as f32, SIZE.1 as f32),
            Env::new(dark, 1.0),
        );
        for _ in 0..20 {
            rt.frame(0.05);
        }
        let px = Pixels::of(&rt, SIZE);
        let el = |k: &str| {
            rt.elements()
                .into_iter()
                .find(|e| e.key.as_deref() == Some(k))
                .unwrap()
                .target
        };
        let tip = el("pp/tip");
        // Two points under the tip's top edge, on its middle: inside the 16 x 8 triangle.
        let got = px.at((tip[0] + tip[2] / 2.0) as u32, (tip[1] + 2.0) as u32);
        assert_eq!(
            got[3], 255,
            "dark {dark}: the tip is not opaque there: {got:?}"
        );
        for i in 0..3 {
            assert!(
                (i32::from(got[i]) - i32::from(want[i])).abs() <= 3,
                "dark {dark}: tip {got:?}, want {want:?}"
            );
        }
    }
}
