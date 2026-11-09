//! Pixel assertions for the RadioButton, which `ElementInfo` cannot give: the control's border
//! (colour and 2px width, and none when selected), the 4px dot, and the focus ring. Every expected
//! value comes from the oracle fixtures (`radio_button_scene.json`, `radio_button_rings.json`,
//! from `design-data render radio-button --format scene`), never from our `Tokens`. See
//! `support/pixels.rs` for how it is rendered and its limits.

#[path = "support/pixels.rs"]
mod pixels;

use pixels::{clear, composite, differs, near, probe, scaled, Pixels, Ring, SIDES};
use rune::{col, Cx, El, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::radio_button::RadioButton;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

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
    /// No label, so nothing is drawn beside the control (the ring test reads pixels there).
    bare: bool,
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
    let b = RadioButton::new("rb");
    let b = if a.bare { b } else { b.label("Label") };
    col().px(PAD).py(PAD).child(
        b.size(a.size.unwrap_or_default())
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

fn info(rt: &Runtime<App>, part: &str) -> rune::ElementInfo {
    let key = format!("rb/{part}");
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key.as_str()))
        .unwrap()
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
        selected: c["selected"].as_bool().unwrap(),
        emphasized: c["emphasized"].as_bool().unwrap_or(false),
        disabled: c["disabled"].as_bool().unwrap_or(false),
        bare: false,
    }
}

fn f(v: &Value) -> f32 {
    v.as_f64().unwrap() as f32
}

/// The control's border is the oracle's colour and `stroke_width` (2px unselected, in every state
/// and disabled), with the control fill after it and nothing outside; a selected control has no
/// border (`stroke_width` 0), so its fill reaches the edge, and holds the oracle's 4px dot in
/// the middle: the dot's fill at its centre and the control fill just outside it. A pressed
/// control is 24/26 smaller about its centre, so there the stroke is read one pixel in, where the
/// shifted edge cannot reach.
#[test]
fn control_border_and_dot_match_oracle() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/radio_button_scene.json")).unwrap();
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
        let mut rt = runtime(app_for(c));
        let (x, y) = rt.find("rb/container").unwrap();
        let down = c["state"] == "down";
        match c["state"].as_str().unwrap() {
            "hover" => rt.pointer_move(x, y),
            "down" => rt.pointer_down(x, y),
            _ => {}
        }
        settle(&mut rt);
        let ctl = info(&rt, "control");
        let r = if down {
            scaled(ctl.target, ctl.scale)
        } else {
            ctl.target
        };
        let px = Pixels::of(&rt, SIZE);
        let want = &c["control"];
        let sw = f(&want["stroke_width"]) as i32;
        let fill = composite(&[want["fill"].as_str().unwrap()], [0; 3]);
        let stroke = composite(&[want["stroke"].as_str().unwrap()], [0; 3]);
        for side in SIDES {
            let w = format!("{what} {side:?}");
            if sw == 0 {
                assert!(
                    c["selected"] == true,
                    "{w}: an unselected control has a border"
                );
                near(
                    probe(&px, r, side, down as i32),
                    fill,
                    &format!("{w}: edge is the fill"),
                );
                continue;
            }
            assert_eq!(sw, 2, "{w}: the oracle's border is 2px");
            let depths: Vec<i32> = if down { vec![1] } else { (0..sw).collect() };
            for d in depths {
                near(
                    probe(&px, r, side, d),
                    stroke,
                    &format!("{w}: stroke pixel {d}"),
                );
            }
            // The fill is read after the border's soft inner edge (see the Switch test).
            near(
                probe(&px, r, side, sw + 1),
                fill,
                &format!("{w}: fill after the stroke"),
            );
            differs(
                probe(&px, r, side, sw),
                stroke,
                &format!("{w}: border is not wider"),
            );
            clear(
                probe(&px, r, side, -1 - down as i32),
                &format!("{w}: nothing outside"),
            );
        }
        if let Some(dot) = c["indicator"].as_object().filter(|_| c["selected"] == true) {
            let dot = Value::Object(dot.clone());
            assert_eq!(
                (f(&dot["w"]), f(&dot["h"])),
                (4.0, 4.0),
                "{what}: the dot is 4px"
            );
            let cx = ctl.target[0] + f(&dot["x"]) - f(&want["x"]) + f(&dot["w"]) / 2.0;
            let cy = ctl.target[1] + f(&dot["y"]) - f(&want["y"]) + f(&dot["h"]) / 2.0;
            let ink = composite(&[dot["fill"].as_str().unwrap()], [0; 3]);
            near(
                px.at(cx as u32, cy as u32),
                ink,
                &format!("{what}: dot centre"),
            );
            near(
                px.at(cx as u32 - 1, cy as u32 - 1),
                ink,
                &format!("{what}: dot, near its edge"),
            );
            near(
                px.at(cx as u32 + 3, cy as u32),
                fill,
                &format!("{what}: control fill beside the dot"),
            );
            near(
                px.at(cx as u32, cy as u32 - 4),
                fill,
                &format!("{what}: control fill above the dot"),
            );
        } else {
            // Unselected: the dot is invisible, so the middle is the control fill.
            let (cx, cy) = (r[0] + r[2] / 2.0, r[1] + r[3] / 2.0);
            near(
                px.at(cx as u32, cy as u32),
                fill,
                &format!("{what}: no dot"),
            );
        }
    }
}

/// Under keyboard focus the control has the oracle's ring: its colour and 2px thickness at a 2px
/// gap from the control, selected or not, with clear pixels in the gap and beyond the ring.
#[test]
fn focus_ring_matches_oracle() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/radio_button_rings.json")).unwrap();
    assert_eq!(cases.len(), 32);
    for c in &cases {
        let what = format!(
            "{} {} {} sel={}",
            c["scheme"], c["scale"], c["size"], c["selected"]
        );
        let mut rt = runtime(App {
            bare: true,
            ..app_for(c)
        });
        rt.key(KeyEvent::from(Key::Tab));
        settle(&mut rt);
        assert!(
            info(&rt, "container").focus_visible,
            "{what}: focus is visible"
        );
        let ctl = info(&rt, "control").target;
        Ring::of(c).assert_around(&Pixels::of(&rt, SIZE), ctl, &SIDES, &what);
    }
}

/// Without keyboard focus, and for a disabled radio that Tab cannot reach, no ring is drawn.
#[test]
fn no_ring_without_keyboard_focus() {
    let rings: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/radio_button_rings.json")).unwrap();
    let ring = Ring::of(&rings[0]);
    let off = runtime(App {
        bare: true,
        ..App::default()
    });
    let ctl = info(&off, "control").target;
    ring.assert_absent(&Pixels::of(&off, SIZE), ctl, &SIDES, "unfocused");
    let mut rt = runtime(App {
        disabled: true,
        bare: true,
        ..App::default()
    });
    rt.key(KeyEvent::from(Key::Tab));
    settle(&mut rt);
    ring.assert_absent(&Pixels::of(&rt, SIZE), ctl, &SIDES, "disabled, Tab pressed");
}
