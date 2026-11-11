//! Pixel assertions for the Checkbox, which `ElementInfo` cannot give: the box's border (colour and
//! 2px width, and none at all when selected or indeterminate), the glyph's ink, and the focus ring.
//! Every expected value comes from the oracle fixtures (`checkbox_scene.json`,
//! `checkbox_rings.json`, from `design-data render checkbox --format scene`), never from our
//! `Tokens`. See `support/pixels.rs` for how it is rendered and its limits.

#[path = "support/pixels.rs"]
mod pixels;

use pixels::{clear, closest, composite, differs, near, probe, scaled, Pixels, Ring, Side, SIDES};
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

/// What the oracle's keyboard-focus state gives a checkbox (`checkbox_focus.json`), against rest
/// (`checkbox_scene.json`): a selected or indeterminate emphasized, error or neutral-selected box
/// takes the hover fill inside its radius (a plain indeterminate box keeps its fill), the glyph is
/// still drawn over it in the oracle's mark colour, the label ink takes the hover colour (text is
/// anti-aliased: some fully covered pixel is the colour), and the ring is as before. NOT matched:
/// the unselected box's 2px stroke, which the oracle takes to the hover colour and Rune cannot (no
/// focus-visible border colour, a14.14): asserted at its REST colour, with the oracle's focus colour
/// recorded next to it. Hover + keyboard-focus draws the hover stroke, which is the oracle's. A
/// disabled checkbox is not focusable and keeps the oracle's disabled colours.
#[test]
fn keyboard_focus_look_matches_oracle() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/checkbox_focus.json")).unwrap();
    let scene: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/checkbox_scene.json")).unwrap();
    let rings: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/checkbox_rings.json")).unwrap();
    assert_eq!(cases.len(), 83);
    let lin = |c: [f32; 3]| c.map(pixels::to_linear);
    let col = |v: &Value| composite(&[v.as_str().unwrap()], [0; 3]);
    let (mut fills, mut gaps, mut glyphs, mut labels) = (0, 0, 0, 0);
    for c in &cases {
        let what = c.to_string();
        let disabled = c["disabled"] == true;
        let hover = c["state"] == "hover+keyboard-focus";
        let rest = scene
            .iter()
            .find(|r| {
                [
                    "scheme",
                    "scale",
                    "size",
                    "value",
                    "emphasized",
                    "error",
                    "disabled",
                ]
                .iter()
                .all(|k| r[*k] == c[*k])
                    && r["state"] == "rest"
            })
            .unwrap_or_else(|| panic!("no rest case for {what}"));
        let mut rt = runtime(app_for(c));
        rt.key(KeyEvent::from(Key::Tab));
        if hover {
            let (x, y) = rt.find("cb/container").unwrap();
            rt.pointer_move(x, y);
        }
        settle(&mut rt);
        let b = info(&rt, "box").target;
        let px = Pixels::of(&rt, SIZE);
        let fill = col(&c["box"]["fill"]);
        let on = c["value"] != "unselected";

        // The fill inside the box, after the (unselected) border; for a box with no border the edge.
        for side in SIDES {
            near(
                probe(&px, b, side, if on { 1 } else { 3 }),
                fill,
                &format!("{what} {side:?}: fill"),
            );
        }
        if on {
            assert_eq!(
                f(&c["box"]["stroke_width"]),
                0.0,
                "{what}: no oracle stroke"
            );
            for side in SIDES {
                near(
                    probe(&px, b, side, 0),
                    fill,
                    &format!("{what} {side:?}: edge is the fill"),
                );
            }
            if col(&rest["box"]["fill"]) != fill {
                fills += 1;
            }
        }
        // Clipped to the radius: the box's corner pixel is not the fill.
        differs(
            px.at(b[0] as u32, b[1] as u32),
            fill,
            &format!("{what}: clipped at the corner"),
        );

        // The unselected box's stroke is the gap; hover shows the oracle's.
        if !on {
            let want = col(&c["box"]["stroke"]);
            let at_rest = col(&rest["box"]["stroke"]);
            for side in SIDES {
                for d in 0..2 {
                    let got = probe(&px, b, side, d);
                    if hover || disabled {
                        near(got, want, &format!("{what} {side:?}: stroke {d}"));
                    } else {
                        near(
                            got,
                            at_rest,
                            &format!("{what} {side:?}: stroke stays at rest"),
                        );
                        differs(
                            got,
                            want,
                            &format!("{what} {side:?}: the gap to the oracle"),
                        );
                        gaps += 1;
                    }
                }
            }
        }

        // The glyph is still on top of the fill in the oracle's mark colour (coverage as in
        // `the_glyph_has_the_oracle_ink`).
        if !c["mark"].is_null() {
            let m = info(&rt, c["mark"]["part"].as_str().unwrap()).target;
            let ink = lin(col(&c["mark"]["color"]));
            let fl = lin(fill);
            let best = (m[1] as u32..(m[1] + m[3]).ceil() as u32)
                .flat_map(|y| (m[0] as u32..(m[0] + m[2]).ceil() as u32).map(move |x| (x, y)))
                .map(|(x, y)| {
                    let p = lin(px.at(x, y).map(|v| v as f32)[..3].try_into().unwrap());
                    (0..3)
                        .map(|i| (p[i] - fl[i]) / (ink[i] - fl[i]))
                        .sum::<f32>()
                        / 3.0
                })
                .fold(f32::MIN, f32::max);
            assert!(
                best >= 0.75,
                "{what}: the glyph covers {best} of its ink over the fill"
            );
            glyphs += 1;
        }

        // The label's ink, and not the rest one when it changes.
        let label = info(&rt, "label").target;
        let want = col(&c["label_color"]);
        assert!(
            closest(&px, label, want) <= 5.0,
            "{what}: label ink is not {want:?}"
        );
        if col(&rest["label"]["color"]) != want {
            assert!(
                closest(&px, label, col(&rest["label"]["color"])) > 12.0,
                "{what}: label still has the rest ink"
            );
        }
        labels += 1;

        // The ring is as before (the label is on the right, so three sides).
        let ring = rings.iter().find(|r| {
            ["scheme", "scale", "size", "value", "error"]
                .iter()
                .all(|k| r[*k] == c[*k])
        });
        if let (Some(r), false) = (ring, disabled) {
            let sides = [Side::Left, Side::Top, Side::Bottom];
            Ring::of(r).assert_around(&px, b, &sides, &format!("{what}: ring"));
        }
    }
    assert!(
        fills >= 40 && gaps >= 100 && glyphs >= 50 && labels == 83,
        "{fills} {gaps} {glyphs} {labels}"
    );
}

/// Pointer focus shows no keyboard-focus look: after a click and the pointer leaving, a selected
/// box (neutral, emphasized, error) is the oracle's rest fill and the label its rest ink; Tab
/// afterwards shows the focus fill. Hover and down (Tab first, then the pointer) keep the oracle's
/// label ink: the group colour must not double the hover colour or override the down one.
#[test]
fn pointer_focus_shows_no_focus_look_and_hover_down_labels_are_the_oracles() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/checkbox_scene.json")).unwrap();
    let find = |emph: bool, err: bool, state: &str| {
        cases
            .iter()
            .find(|c| {
                c["scheme"] == "light"
                    && c["scale"] == "desktop"
                    && c["size"] == "m"
                    && c["value"] == "selected"
                    && c["emphasized"] == emph
                    && c["error"] == err
                    && c["disabled"] == false
                    && c["state"] == state
            })
            .unwrap()
    };
    let col = |v: &Value| composite(&[v.as_str().unwrap()], [0; 3]);
    for (emph, err) in [(false, false), (true, false), (false, true)] {
        let what = format!("emph={emph} err={err}");
        let rest = find(emph, err, "rest");
        let app = || App {
            selected: true,
            emphasized: emph,
            error: err,
            ..App::default()
        };
        let mut rt = runtime(app());
        let (x, y) = rt.find("cb/container").unwrap();
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.pointer_move(190.0, 95.0);
        settle(&mut rt);
        assert!(
            !info(&rt, "container").focus_visible,
            "{what}: a click shows no keyboard focus"
        );
        let sample = |rt: &Runtime<App>| {
            let px = Pixels::of(rt, SIZE);
            (
                probe(&px, info(rt, "box").target, SIDES[0], 1),
                closest(&px, info(rt, "label").target, col(&rest["label"]["color"])),
            )
        };
        let (p, ink) = sample(&rt);
        near(
            p,
            col(&rest["box"]["fill"]),
            &format!("{what}: clicked, rest fill"),
        );
        assert!(ink <= 5.0, "{what}: clicked, rest label ink");
        rt.key(KeyEvent::from(Key::Tab));
        settle(&mut rt);
        let (p, ink) = sample(&rt);
        differs(
            p,
            col(&rest["box"]["fill"]),
            &format!("{what}: Tab shows the focus fill"),
        );
        assert!(ink > 12.0, "{what}: Tab changes the label ink");

        for state in ["hover", "down"] {
            let want = find(emph, err, state);
            let mut rt = runtime(app());
            let (x, y) = rt.find("cb/container").unwrap();
            rt.key(KeyEvent::from(Key::Tab));
            if state == "hover" {
                rt.pointer_move(x, y);
            } else {
                rt.pointer_down(x, y);
            }
            settle(&mut rt);
            let px = Pixels::of(&rt, SIZE);
            let ink = col(&want["label"]["color"]);
            assert!(
                closest(&px, info(&rt, "label").target, ink) <= 5.0,
                "{what} {state}: label ink is the oracle's {ink:?}"
            );
        }
    }
}
