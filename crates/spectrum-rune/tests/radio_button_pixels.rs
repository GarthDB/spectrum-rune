//! Pixel assertions for the RadioButton, which `ElementInfo` cannot give: the control's border
//! (colour and 2px width, and none when selected), the 4px dot, and the focus ring. Every expected
//! value comes from the oracle fixtures (`radio_button_scene.json`, `radio_button_rings.json`,
//! from `design-data render radio-button --format scene`), never from our `Tokens`. See
//! `support/pixels.rs` for how it is rendered and its limits.

#[path = "support/pixels.rs"]
mod pixels;

use pixels::{clear, closest, composite, differs, near, probe, scaled, Pixels, Ring, Side, SIDES};
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

/// What the oracle's keyboard-focus state gives a radio button (`radio_button_focus.json`), against
/// rest (`radio_button_scene.json`): a selected control (neutral or emphasized) takes the hover fill
/// inside its circle (the bounding box's corner is not the fill), the 4px dot is still drawn over it,
/// the label ink takes the hover colour (text is anti-aliased: some fully covered pixel is the
/// colour), and the ring is as before. NOT matched: the unselected control's 2px stroke, which the
/// oracle takes to the hover colour and Rune cannot (no focus-visible border colour, a14.14):
/// asserted at its REST colour, with the oracle's focus colour recorded next to it. Hover +
/// keyboard-focus draws the hover stroke, which is the oracle's. A disabled radio is not focusable
/// and keeps the oracle's disabled colours.
#[test]
fn keyboard_focus_look_matches_oracle() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/radio_button_focus.json")).unwrap();
    let scene: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/radio_button_scene.json")).unwrap();
    let rings: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/radio_button_rings.json")).unwrap();
    assert_eq!(cases.len(), 34);
    let col = |v: &Value| composite(&[v.as_str().unwrap()], [0; 3]);
    let (mut fills, mut gaps, mut dots) = (0, 0, 0);
    for c in &cases {
        let what = c.to_string();
        let (sel, disabled) = (c["selected"] == true, c["disabled"] == true);
        let hover = c["state"] == "hover+keyboard-focus";
        let rest = scene
            .iter()
            .find(|r| {
                [
                    "scheme",
                    "scale",
                    "size",
                    "selected",
                    "emphasized",
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
            let (x, y) = rt.find("rb/container").unwrap();
            rt.pointer_move(x, y);
        }
        settle(&mut rt);
        let b = info(&rt, "control").target;
        let px = Pixels::of(&rt, SIZE);
        let fill = col(&c["control"]["fill"]);

        // The fill, inside the circle (after the border when unselected).
        for side in SIDES {
            near(
                probe(&px, b, side, if sel { 1 } else { 3 }),
                fill,
                &format!("{what} {side:?}: fill"),
            );
        }
        // Clipped to the circle: the bounding box's corner is not the fill.
        differs(
            px.at(b[0] as u32, b[1] as u32),
            fill,
            &format!("{what}: clipped at the corner"),
        );
        if sel {
            assert_eq!(f(&c["control"]["stroke_width"]), 0.0, "{what}: no stroke");
            if col(&rest["control"]["fill"]) != fill {
                fills += 1;
            }
            // The dot is still on top, in the oracle's indicator colour.
            if !c["indicator"].is_null() {
                near(
                    px.at((b[0] + b[2] / 2.0) as u32, (b[1] + b[3] / 2.0) as u32),
                    col(&c["indicator"]),
                    &format!("{what}: dot"),
                );
                dots += 1;
            }
        } else {
            // The unselected stroke is the gap; hover shows the oracle's.
            let want = col(&c["control"]["stroke"]);
            let at_rest = col(&rest["control"]["stroke"]);
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

        // The ring is as before (the label is on the right, so three sides).
        let ring = rings.iter().find(|r| {
            ["scheme", "scale", "size", "selected"]
                .iter()
                .all(|k| r[*k] == c[*k])
        });
        if let (Some(r), false) = (ring, disabled) {
            let sides = [Side::Left, Side::Top, Side::Bottom];
            Ring::of(r).assert_around(&px, b, &sides, &format!("{what}: ring"));
        }
    }
    assert!(
        fills >= 12 && gaps >= 50 && dots >= 14,
        "{fills} {gaps} {dots}"
    );
}

/// Pointer focus shows no keyboard-focus look: after a click and the pointer leaving, a selected
/// control (neutral and emphasized) is the oracle's rest fill and the label its rest ink; Tab
/// afterwards shows the focus fill. Hover and down (Tab first, then the pointer) keep the oracle's
/// label ink: the group colour must not double the hover colour or override the down one.
#[test]
fn pointer_focus_shows_no_focus_look_and_hover_down_labels_are_the_oracles() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/radio_button_scene.json")).unwrap();
    let find = |emph: bool, state: &str| {
        cases
            .iter()
            .find(|c| {
                c["scheme"] == "light"
                    && c["scale"] == "desktop"
                    && c["size"] == "m"
                    && c["selected"] == true
                    && c["emphasized"] == emph
                    && c["disabled"] == false
                    && c["state"] == state
            })
            .unwrap()
    };
    let col = |v: &Value| composite(&[v.as_str().unwrap()], [0; 3]);
    for emph in [false, true] {
        let what = format!("emph={emph}");
        let rest = find(emph, "rest");
        let app = || App {
            selected: true,
            emphasized: emph,
            ..App::default()
        };
        let mut rt = runtime(app());
        let (x, y) = rt.find("rb/container").unwrap();
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
                probe(&px, info(rt, "control").target, SIDES[0], 1),
                closest(&px, info(rt, "label").target, col(&rest["label"]["color"])),
            )
        };
        let (p, ink) = sample(&rt);
        near(
            p,
            col(&rest["control"]["fill"]),
            &format!("{what}: clicked, rest fill"),
        );
        assert!(ink <= 5.0, "{what}: clicked, rest label ink");
        rt.key(KeyEvent::from(Key::Tab));
        settle(&mut rt);
        let (p, ink) = sample(&rt);
        differs(
            p,
            col(&rest["control"]["fill"]),
            &format!("{what}: Tab shows the focus fill"),
        );
        assert!(ink > 12.0, "{what}: Tab changes the label ink");

        for state in ["hover", "down"] {
            let want = find(emph, state);
            let mut rt = runtime(app());
            let (x, y) = rt.find("rb/container").unwrap();
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

/// The oracle's down + keyboard-focus look differs from its plain down for a selected neutral radio (the control
/// takes the hover fill, 19 not 41), but Rune cannot draw it: a pointer press hides keyboard focus
/// (`focus_visible` goes false until a navigation key), so a press on a Tab-focused radio draws exactly the plain
/// down look. Asserted whole-frame, so the focus fill and the ring are gone, not merely hidden in a probe.
/// `fixtures/combined-diff.sh` prints the oracle's side (a14.31).
#[test]
fn a_press_on_a_keyboard_focused_radio_draws_the_plain_down_look() {
    for selected in [false, true] {
        let app = App {
            selected,
            ..App::default()
        };
        let pressed = |tab: bool| {
            let mut rt = runtime(app.clone());
            if tab {
                rt.key(KeyEvent::from(Key::Tab));
                settle(&mut rt);
                assert!(info(&rt, "container").focus_visible, "Tab shows focus");
            }
            let (x, y) = rt.find("rb/container").unwrap();
            rt.pointer_move(x, y);
            rt.pointer_down(x, y);
            settle(&mut rt);
            assert!(!info(&rt, "container").focus_visible, "a press hides it");
            Pixels::of(&rt, SIZE)
        };
        assert!(
            pressed(true).identical(&pressed(false)),
            "selected={selected}: Tab then press differs from press alone"
        );
    }
}
