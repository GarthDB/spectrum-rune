//! Pixel assertions for the Switch, which `ElementInfo` cannot give: the track's border (colour and
//! 2px width), the track and handle fills as drawn, and the focus ring (colour, thickness, gap).
//! Every expected value comes from the oracle fixtures (`switch_scene.json`, `switch_rings.json`,
//! from `design-data render switch --format scene`), never from our `Tokens`. See
//! `support/pixels.rs` for how it is rendered and its limits (Rune's CPU reference renderer, sRGB
//! 8-bit, scale 1: points are pixels).

#[path = "support/pixels.rs"]
mod pixels;

use pixels::{clear, composite, differs, near, probe, Pixels, Ring, Side, SIDES};
use rune::{col, Cx, El, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::switch::{Size, Switch};
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};

const SIZE: (u32, u32) = (200, 100);
const PAD: f32 = 20.0;

#[derive(Clone, Default)]
struct App {
    dark: bool,
    mobile: bool,
    size: Option<Size>,
    selected: bool,
    emphasized: bool,
    disabled: bool,
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
    col().px(PAD).py(PAD).child(
        Switch::new("sw")
            .label("Wi-Fi")
            .size(a.size.unwrap_or_default())
            .selected(a.selected)
            .emphasized(a.emphasized)
            .disabled(a.disabled)
            .build(t),
    )
}

fn settle(rt: &mut Runtime<App>) {
    for _ in 0..40 {
        rt.frame(0.05);
    }
}

fn runtime(a: App) -> Runtime<App> {
    let dark = a.dark;
    let mut rt = Runtime::new(a, view, (SIZE.0 as f32, SIZE.1 as f32), Env::new(dark, 1.0));
    settle(&mut rt);
    rt
}

fn rect(rt: &Runtime<App>, part: &str) -> [f32; 4] {
    let key = format!("sw/{part}");
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key.as_str()))
        .unwrap()
        .target
}

fn size_of(s: &str) -> Size {
    match s {
        "s" => Size::S,
        "m" => Size::M,
        "l" => Size::L,
        _ => Size::XL,
    }
}

/// The track has the oracle's border (its colour and 2px width, from the oracle's `stroke` and
/// `stroke_width`), then the track fill, and nothing outside it. The handle's centre pixel is its
/// oracle fill. Selected switches have a stroke equal to their fill (the oracle says so), so
/// there the colour is checked but the width cannot be seen. Hover and down are at size M,
/// disabled at rest.
#[test]
fn track_border_and_fills_match_oracle() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/switch_scene.json")).unwrap();
    for c in &cases {
        let what = format!(
            "{} {} {} sel={} emph={} dis={} {}",
            c["scheme"],
            c["scale"],
            c["size"],
            c["selected"],
            c["emphasized"],
            c["disabled"],
            c["state"]
        );
        let mut rt = runtime(App {
            dark: c["scheme"] == "dark",
            mobile: c["scale"] == "mobile",
            size: Some(size_of(c["size"].as_str().unwrap())),
            selected: c["selected"].as_bool().unwrap(),
            emphasized: c["emphasized"].as_bool().unwrap(),
            disabled: c["disabled"].as_bool().unwrap(),
        });
        let (x, y) = rt.find("sw/container").unwrap();
        match c["state"].as_str().unwrap() {
            "hover" => rt.pointer_move(x, y),
            "down" => rt.pointer_down(x, y),
            _ if c["disabled"] == true => rt.pointer_down(x, y),
            _ => {}
        }
        settle(&mut rt);
        let track = rect(&rt, "track");
        let px = Pixels::of(&rt, SIZE);
        let t = &c["track"];
        let sw = t["stroke_width"].as_f64().unwrap() as i32;
        assert_eq!(sw, 2, "{what}: the oracle's border is 2px");
        let stroke = composite(&[t["stroke"].as_str().unwrap()], [0; 3]);
        let fill = composite(&[t["fill"].as_str().unwrap()], [0; 3]);
        for side in SIDES {
            let w = format!("{what} {side:?}");
            for d in 0..sw {
                near(
                    probe(&px, track, side, d),
                    stroke,
                    &format!("{w}: stroke pixel {d}"),
                );
            }
            // The border's soft inner edge leaks about 3% of the stroke into the next pixel (in
            // linear light, which shows in dark), so the fill is read one pixel further in.
            // (A selected track's handle sits there on the right.)
            if !(c["selected"] == true && matches!(side, Side::Right)) {
                near(
                    probe(&px, track, side, sw + 1),
                    fill,
                    &format!("{w}: fill after the stroke"),
                );
            }
            if stroke != fill {
                differs(
                    probe(&px, track, side, sw),
                    stroke,
                    &format!("{w}: border is not wider"),
                );
            }
            clear(
                probe(&px, track, side, -1),
                &format!("{w}: nothing outside"),
            );
        }
        // The handle's centre, from the oracle's offset inside the track.
        let h = &c["handle"];
        let f = |v: &Value| v.as_f64().unwrap() as f32;
        let hx = track[0] + f(&h["x"]) - f(&t["x"]) + f(&h["w"]) / 2.0;
        let hy = track[1] + f(&h["y"]) - f(&t["y"]) + f(&h["h"]) / 2.0;
        near(
            px.at(hx as u32, hy as u32),
            composite(&[h["fill"].as_str().unwrap()], [0; 3]),
            &format!("{what}: handle fill"),
        );
    }
}

fn focused(a: App) -> Runtime<App> {
    let mut rt = runtime(a);
    rt.key(KeyEvent::from(Key::Tab));
    settle(&mut rt);
    rt
}

/// Under keyboard focus the track has the oracle's ring: its colour and 2px thickness at a 2px gap
/// from the track (`keyboard-focus` ring part), with clear pixels in the gap and beyond the ring.
#[test]
fn focus_ring_matches_oracle() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/switch_rings.json")).unwrap();
    assert_eq!(cases.len(), 32);
    for c in &cases {
        let what = format!(
            "{} {} {} sel={}",
            c["scheme"], c["scale"], c["size"], c["selected"]
        );
        let rt = focused(App {
            dark: c["scheme"] == "dark",
            mobile: c["scale"] == "mobile",
            size: Some(size_of(c["size"].as_str().unwrap())),
            selected: c["selected"].as_bool().unwrap(),
            ..App::default()
        });
        let e = rt
            .elements()
            .into_iter()
            .find(|e| e.key.as_deref() == Some("sw/container"))
            .unwrap();
        assert!(e.focus_visible, "{what}: keyboard focus is visible");
        let track = rect(&rt, "track");
        Ring::of(c).assert_around(&Pixels::of(&rt, SIZE), track, &SIDES, &what);
    }
}

/// A disabled switch cannot take focus, so Tab draws no ring; and an unfocused one has none.
#[test]
fn no_ring_without_keyboard_focus() {
    let ring: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/switch_rings.json")).unwrap();
    let ring = Ring::of(&ring[0]);
    let off = runtime(App::default());
    let track = rect(&off, "track");
    ring.assert_absent(&Pixels::of(&off, SIZE), track, &SIDES, "unfocused");
    let mut rt = runtime(App {
        disabled: true,
        ..App::default()
    });
    rt.key(KeyEvent::from(Key::Tab));
    settle(&mut rt);
    ring.assert_absent(
        &Pixels::of(&rt, SIZE),
        track,
        &SIDES,
        "disabled, Tab pressed",
    );
}
