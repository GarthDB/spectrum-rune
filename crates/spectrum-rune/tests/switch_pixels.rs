//! Pixel assertions for the Switch, which `ElementInfo` cannot give: the track's border (colour and
//! 2px width), the track and handle fills as drawn, and the focus ring (colour, thickness, gap).
//! Every expected value comes from the oracle fixtures (`switch_scene.json`, `switch_rings.json`,
//! from `design-data render switch --format scene`), never from our `Tokens`. See
//! `support/pixels.rs` for how it is rendered and its limits (Rune's CPU reference renderer, sRGB
//! 8-bit, scale 1: points are pixels).

#[path = "support/pixels.rs"]
mod pixels;

use pixels::{clear, closest, composite, differs, near, probe, Pixels, Ring, Side, SIDES};
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

/// What the oracle's keyboard-focus state gives a switch (`switch_focus.json`), against rest
/// (`switch_scene.json`): the handle fill (unselected) or the track fill (selected, accent when
/// emphasized) takes the hover colour inside its radius; the label ink takes it (some pixel of the
/// text is the colour: text is anti-aliased); the ring is unchanged. NOT matched: the unselected
/// track's stroke, which the oracle takes to the hover colour and Rune cannot (no focus-visible
/// border colour, a14.14), so it is asserted at its REST colour, with the oracle's focus colour
/// recorded next to it. With the pointer also over it (hover + keyboard-focus) the stroke is the
/// hover one, which equals the oracle's. A disabled switch cannot be focused and keeps the oracle's
/// disabled colours.
#[test]
fn keyboard_focus_look_matches_oracle() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/switch_focus.json")).unwrap();
    let scene: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/switch_scene.json")).unwrap();
    let rings: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/switch_rings.json")).unwrap();
    assert_eq!(cases.len(), 34);
    let (mut handles, mut tracks, mut gaps, mut labels) = (0, 0, 0, 0);
    for c in &cases {
        let what = c.to_string();
        let (sel, disabled) = (c["selected"] == true, c["disabled"] == true);
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
        let mut rt = runtime(App {
            dark: c["scheme"] == "dark",
            mobile: c["scale"] == "mobile",
            size: Some(size_of(c["size"].as_str().unwrap())),
            selected: sel,
            emphasized: c["emphasized"] == true,
            disabled,
        });
        rt.key(KeyEvent::from(Key::Tab));
        let hover = c["state"] == "hover+keyboard-focus";
        if hover {
            let (x, y) = rt.find("sw/container").unwrap();
            rt.pointer_move(x, y);
        }
        settle(&mut rt);
        let track = rect(&rt, "track");
        let handle = rect(&rt, "handle");
        let px = Pixels::of(&rt, SIZE);
        let col = |v: &Value| composite(&[v.as_str().unwrap()], [0; 3]);

        // The part whose fill is the content colour, in its radius: the handle's centre when
        // unselected, the track away from the handle when selected; the track's own fill when not.
        let handle_c = px.at(
            (handle[0] + handle[2] / 2.0) as u32,
            (handle[1] + handle[3] / 2.0) as u32,
        );
        near(
            handle_c,
            col(&c["handle"]["fill"]),
            &format!("{what}: handle fill"),
        );
        let away = if sel { Side::Left } else { Side::Right };
        near(
            probe(&px, track, away, 4),
            col(&c["track"]["fill"]),
            &format!("{what}: track fill"),
        );
        if sel && !disabled {
            tracks += 1;
        } else if !disabled {
            handles += 1;
        }

        // Clipped to the part's radius: the corner of its bounding box is not the fill.
        let (corner, fill) = if sel {
            (
                px.at(track[0] as u32, track[1] as u32),
                col(&c["track"]["fill"]),
            )
        } else {
            (
                px.at(handle[0] as u32, handle[1] as u32),
                col(&c["handle"]["fill"]),
            )
        };
        differs(corner, fill, &format!("{what}: fill clipped at the corner"));

        // The stroke: the selected track's stroke is its fill; unselected is the gap.
        let want_stroke = col(&c["track"]["stroke"]);
        let rest_stroke = col(&rest["track"]["stroke"]);
        for side in SIDES {
            for d in 0..2 {
                let got = probe(&px, track, side, d);
                if sel || hover || disabled {
                    near(got, want_stroke, &format!("{what} {side:?}: stroke {d}"));
                } else {
                    near(
                        got,
                        rest_stroke,
                        &format!("{what} {side:?}: stroke stays at rest"),
                    );
                    differs(
                        got,
                        want_stroke,
                        &format!("{what} {side:?}: the gap to the oracle"),
                    );
                    gaps += 1;
                }
            }
        }

        // The label's ink: some pixel is the oracle's colour, and (when it changes) not the rest one.
        let label = rect(&rt, "label");
        let want = col(&c["label_color"]);
        assert!(
            closest(&px, label, want) <= 5.0,
            "{what}: label ink is not {want:?}"
        );
        if col(&rest["label_color"]) != want {
            assert!(
                closest(&px, label, col(&rest["label_color"])) > 12.0,
                "{what}: label still has the rest ink"
            );
        }
        labels += 1;

        // The ring is as before (unless disabled: not focusable, no ring).
        let ring = rings.iter().find(|r| {
            ["scheme", "scale", "size", "selected"]
                .iter()
                .all(|k| r[*k] == c[*k])
        });
        if let (Some(r), false) = (ring, disabled) {
            Ring::of(r).assert_around(&px, track, &SIDES, &format!("{what}: ring"));
        }
    }
    assert!(
        handles >= 14 && tracks >= 14 && gaps >= 40 && labels == 34,
        "{handles} {tracks} {gaps} {labels}"
    );
}

/// Pointer focus shows no keyboard-focus look: after a click and the pointer leaving, the handle
/// (unselected) or track (selected) is the oracle's rest fill and the label its rest ink; Tab
/// afterwards shows the focus look. Hover and down (no Tab) keep the oracle's label ink: the group
/// colour must not double the hover colour or override the down one.
#[test]
fn pointer_focus_shows_no_focus_look_and_hover_down_labels_are_the_oracles() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/switch_scene.json")).unwrap();
    let find = |sel: bool, emph: bool, state: &str| {
        cases
            .iter()
            .find(|c| {
                c["scheme"] == "light"
                    && c["scale"] == "desktop"
                    && c["size"] == "m"
                    && c["selected"] == sel
                    && c["emphasized"] == emph
                    && c["disabled"] == false
                    && c["state"] == state
            })
            .unwrap()
    };
    let col = |v: &Value| composite(&[v.as_str().unwrap()], [0; 3]);
    let app = |sel, emph| App {
        selected: sel,
        emphasized: emph,
        ..App::default()
    };
    for (sel, emph) in [(false, false), (true, false), (true, true)] {
        let c = find(sel, emph, "rest");
        let mut rt = runtime(app(sel, emph));
        let (x, y) = rt.find("sw/container").unwrap();
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.pointer_move(190.0, 95.0);
        settle(&mut rt);
        let e = rt
            .elements()
            .into_iter()
            .find(|e| e.key.as_deref() == Some("sw/container"))
            .unwrap();
        assert!(!e.focus_visible, "a click shows no keyboard focus");
        let sample = |rt: &Runtime<App>| {
            let px = Pixels::of(rt, SIZE);
            let (h, t) = (rect(rt, "handle"), rect(rt, "track"));
            let p = if sel {
                probe(&px, t, Side::Left, 4)
            } else {
                px.at((h[0] + h[2] / 2.0) as u32, (h[1] + h[3] / 2.0) as u32)
            };
            (p, closest(&px, rect(rt, "label"), col(&c["label_color"])))
        };
        let part = if sel {
            &c["track"]["fill"]
        } else {
            &c["handle"]["fill"]
        };
        let (p, ink) = sample(&rt);
        near(
            p,
            col(part),
            &format!("sel={sel} emph={emph}: clicked, rest fill"),
        );
        assert!(ink <= 5.0, "sel={sel} emph={emph}: clicked, rest label ink");
        rt.key(KeyEvent::from(Key::Tab));
        settle(&mut rt);
        let (p, ink) = sample(&rt);
        differs(
            p,
            col(part),
            &format!("sel={sel} emph={emph}: Tab shows the focus fill"),
        );
        assert!(
            ink > 12.0,
            "sel={sel} emph={emph}: Tab changes the label ink"
        );

        for state in ["hover", "down"] {
            let want = find(sel, emph, state);
            let mut rt = runtime(app(sel, emph));
            let (x, y) = rt.find("sw/container").unwrap();
            // Tab first, so keyboard focus and the pointer are both there (a press then takes
            // focus to the pointer; hover keeps both).
            rt.key(KeyEvent::from(Key::Tab));
            if state == "hover" {
                rt.pointer_move(x, y);
            } else {
                rt.pointer_down(x, y);
            }
            settle(&mut rt);
            let px = Pixels::of(&rt, SIZE);
            let ink = col(&want["label_color"]);
            assert!(
                closest(&px, rect(&rt, "label"), ink) <= 5.0,
                "sel={sel} emph={emph} {state}: label ink is the oracle's {ink:?}"
            );
        }
    }
}
