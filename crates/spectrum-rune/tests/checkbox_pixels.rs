//! Pixel assertions for the Checkbox, which `ElementInfo` cannot give: the box's border (colour and
//! 2px width, and none at all when selected or indeterminate), the glyph's ink, and the focus ring.
//! Every expected value comes from the oracle fixtures (`checkbox_scene.json`,
//! `checkbox_rings.json`, from `design-data render checkbox --format scene`), never from our
//! `Tokens`. See `support/pixels.rs` for how it is rendered and its limits.

#[path = "support/pixels.rs"]
mod pixels;

use pixels::{clear, composite, differs, near, probe, scaled, Pixels, Ring, SIDES};
use rune::{col, Cx, El, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::checkbox::{Checkbox, Size};
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};

const SIZE: (u32, u32) = (200, 100);
const PAD: f32 = 20.0;

#[derive(Clone, Default)]
struct App {
    dark: bool,
    mobile: bool,
    size: Option<Size>,
    selected: bool,
    indeterminate: bool,
    emphasized: bool,
    error: bool,
    disabled: bool,
    /// No label, so nothing is drawn beside the box (the ring test reads pixels there).
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
    let b = Checkbox::new("cb");
    let b = if a.bare { b } else { b.label("Label") };
    col().px(PAD).py(PAD).child(
        b.size(a.size.unwrap_or_default())
            .selected(a.selected)
            .indeterminate(a.indeterminate)
            .emphasized(a.emphasized)
            .error(a.error)
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
    let key = format!("cb/{part}");
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key.as_str()))
        .unwrap()
}

fn app_for(c: &Value) -> App {
    let value = c["value"].as_str().unwrap();
    App {
        dark: c["scheme"] == "dark",
        mobile: c["scale"] == "mobile",
        size: Some(match c["size"].as_str().unwrap() {
            "s" => Size::S,
            "m" => Size::M,
            "l" => Size::L,
            _ => Size::XL,
        }),
        selected: value == "selected",
        indeterminate: value == "indeterminate",
        emphasized: c["emphasized"].as_bool().unwrap_or(false),
        error: c["error"].as_bool().unwrap_or(false),
        disabled: c["disabled"].as_bool().unwrap_or(false),
        bare: false,
    }
}

fn f(v: &Value) -> f32 {
    v.as_f64().unwrap() as f32
}

/// The box's border is the oracle's colour and `stroke_width` (2px unselected: default, hover,
/// down, error, disabled), with the box fill after it and nothing outside; selected and
/// indeterminate boxes have no border (`stroke_width` 0), so the fill reaches the edge. Pressed
/// boxes are 24/26 smaller about their centre, so there the stroke is read one pixel in, where the
/// shifted edge cannot reach, and the fill one pixel after that.
#[test]
fn box_border_matches_oracle() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/checkbox_scene.json")).unwrap();
    for c in &cases {
        let what = format!(
            "{} {} {} {} emph={} err={} dis={} {}",
            c["scheme"],
            c["scale"],
            c["size"],
            c["value"],
            c["emphasized"],
            c["error"],
            c["disabled"],
            c["state"]
        );
        let mut rt = runtime(app_for(c));
        let (x, y) = rt.find("cb/container").unwrap();
        let down = c["state"] == "down";
        match c["state"].as_str().unwrap() {
            "hover" => rt.pointer_move(x, y),
            "down" => rt.pointer_down(x, y),
            _ => {}
        }
        settle(&mut rt);
        let b = info(&rt, "box");
        let r = if down {
            scaled(b.target, b.scale)
        } else {
            b.target
        };
        let px = Pixels::of(&rt, SIZE);
        let want = &c["box"];
        let sw = f(&want["stroke_width"]) as i32;
        let fill = composite(&[want["fill"].as_str().unwrap()], [0; 3]);
        let stroke = composite(&[want["stroke"].as_str().unwrap()], [0; 3]);
        for side in SIDES {
            let w = format!("{what} {side:?}");
            if sw == 0 {
                // Selected: no border, so the very edge is the fill.
                assert!(
                    c["value"] != "unselected",
                    "{w}: an unselected box has a border"
                );
                near(
                    probe(&px, r, side, if down { 1 } else { 0 }),
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
            if stroke != fill {
                differs(
                    probe(&px, r, side, sw),
                    stroke,
                    &format!("{w}: border is not wider"),
                );
            }
            clear(
                probe(&px, r, side, -1 - down as i32),
                &format!("{w}: nothing outside"),
            );
        }
    }
}

/// A selected or indeterminate box shows its glyph in the oracle's mark colour. The glyph is 1 to
/// 2px strokes, so no pixel is fully covered: the best pixel's coverage of the ink over the box
/// fill, measured in linear light (where Rune blends), is at least 0.75 (the L dash is 1.7px thick and peaks near 0.83), averaged over the channels.
/// A glyph in another colour would not reach that, but a close shade could.
#[test]
fn the_glyph_has_the_oracle_ink() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/checkbox_scene.json")).unwrap();
    let lin = |c: [f32; 3]| c.map(pixels::to_linear);
    let mut worst = 1.0f32;
    for c in cases
        .iter()
        .filter(|c| c["state"] == "rest" && !c["mark"].is_null())
    {
        let rt = runtime(app_for(c));
        let m = info(&rt, c["mark"]["part"].as_str().unwrap()).target;
        let px = Pixels::of(&rt, SIZE);
        let ink = lin(composite(&[c["mark"]["color"].as_str().unwrap()], [0; 3]));
        let fill = lin(composite(&[c["box"]["fill"].as_str().unwrap()], [0; 3]));
        let best = (m[1] as u32..(m[1] + m[3]).ceil() as u32)
            .flat_map(|y| (m[0] as u32..(m[0] + m[2]).ceil() as u32).map(move |x| (x, y)))
            .map(|(x, y)| {
                let p = lin(px.at(x, y).map(|v| v as f32)[..3].try_into().unwrap());
                (0..3)
                    .map(|i| (p[i] - fill[i]) / (ink[i] - fill[i]))
                    .sum::<f32>()
                    / 3.0
            })
            .fold(f32::MIN, f32::max);
        worst = worst.min(best);
        assert!(
            best >= 0.75,
            "{c}: the best glyph pixel covers {best} of the ink"
        );
    }
    println!("worst {worst}");
}

/// Under keyboard focus the box has the oracle's ring: its colour and 2px thickness at a 2px gap
/// from the box, for every value (unselected, selected, indeterminate), error or not, with clear
/// pixels in the gap and beyond the ring.
#[test]
fn focus_ring_matches_oracle() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/checkbox_rings.json")).unwrap();
    assert_eq!(cases.len(), 96);
    for c in &cases {
        let what = format!(
            "{} {} {} {} err={}",
            c["scheme"], c["scale"], c["size"], c["value"], c["error"]
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
        let b = info(&rt, "box").target;
        Ring::of(c).assert_around(&Pixels::of(&rt, SIZE), b, &SIDES, &what);
    }
}

/// Without keyboard focus, and for a disabled box that Tab cannot reach, no ring is drawn.
#[test]
fn no_ring_without_keyboard_focus() {
    let rings: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/checkbox_rings.json")).unwrap();
    let ring = Ring::of(&rings[0]);
    let off = runtime(App {
        bare: true,
        ..App::default()
    });
    let b = info(&off, "box").target;
    ring.assert_absent(&Pixels::of(&off, SIZE), b, &SIDES, "unfocused");
    let mut rt = runtime(App {
        disabled: true,
        bare: true,
        ..App::default()
    });
    rt.key(KeyEvent::from(Key::Tab));
    settle(&mut rt);
    ring.assert_absent(&Pixels::of(&rt, SIZE), b, &SIDES, "disabled, Tab pressed");
}
