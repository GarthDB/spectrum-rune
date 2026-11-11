//! Pixel assertions for the Button, which `ElementInfo` cannot give: the outline border (colour and
//! 2px width, none for fills), the fill as painted over a backdrop (static colours are translucent),
//! the keyboard-focus fill, and the focus ring (colour, thickness, gap), including the white and black rings of static
//! primary and secondary. Every expected value comes from the oracle fixtures (`button_scene.json`,
//! `button_focus_ring.json`, `button_focus_fill.json`, from `design-data render button --format scene`), never from our
//! `Tokens`. See `support/pixels.rs` for how it is rendered and its limits.
//!
//! Not asserted: the pending progress ink. The oracle draws it `gray_800` (invisible on a primary
//! fill), which the Button deliberately does not copy (see its module docs), so there is no oracle
//! value to compare.

#[path = "support/pixels.rs"]
mod pixels;

use pixels::{composite, differs, near, near_within, probe_snapped, Pixels, Ring, Side};
use rune::{col, Cx, El, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::button::{Button, StaticColor, Style, Variant};
use spectrum_rune::icons::CHECKMARK_100;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

const SIZE: (u32, u32) = (300, 120);
const PAD: f32 = 30.0;
/// The page behind the button. Mid-grey-blue, so translucent white and black fills both show.
const BACKDROP: &str = "#4b6a8c";
const BACKDROP_RGB: [u8; 3] = [0x4b, 0x6a, 0x8c];
/// Left, top and bottom: the button's width is fractional (the label's), so its right edge is not
/// on a pixel boundary.
const SIDES: [Side; 3] = [Side::Left, Side::Top, Side::Bottom];

#[derive(Clone)]
struct App {
    dark: bool,
    mobile: bool,
    variant: Variant,
    style: Style,
    size: Size,
    label: String,
    disabled: bool,
    pending: bool,
    hidden: bool,
    static_color: Option<StaticColor>,
    backdrop: bool,
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
    let mut b = Button::new("bt")
        .label(a.label.clone())
        .icon(&CHECKMARK_100)
        .variant(a.variant)
        .style(a.style)
        .size(a.size)
        .disabled(a.disabled)
        .pending(a.pending)
        .label_hidden(a.hidden);
    if let Some(c) = a.static_color {
        b = b.static_color(c);
    }
    let page = col().w(SIZE.0 as f32).h(SIZE.1 as f32).px(PAD).py(PAD);
    let page = if a.backdrop {
        page.fill(rune::color::hex(BACKDROP).unwrap())
    } else {
        page
    };
    page.child(b.build(t))
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

fn container(rt: &Runtime<App>) -> rune::ElementInfo {
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some("bt/container"))
        .unwrap()
}

fn app_for(c: &Value) -> App {
    App {
        dark: c["scheme"] == "dark",
        mobile: c["scale"] == "mobile",
        variant: match c["variant"].as_str().unwrap() {
            "accent" => Variant::Accent,
            "negative" => Variant::Negative,
            "primary" => Variant::Primary,
            _ => Variant::Secondary,
        },
        style: if c["style"] == "outline" {
            Style::Outline
        } else {
            Style::Fill
        },
        size: match c["size"].as_str().unwrap() {
            "s" => Size::S,
            "m" => Size::M,
            "l" => Size::L,
            _ => Size::XL,
        },
        label: c["label"].as_str().unwrap_or("Label").into(),
        disabled: c["disabled"].as_bool().unwrap_or(false),
        pending: c["pending"].as_bool().unwrap_or(false),
        hidden: c["hidden"].as_bool().unwrap_or(false),
        static_color: match c["static"].as_str() {
            Some("white") => Some(StaticColor::White),
            Some("black") => Some(StaticColor::Black),
            _ => None,
        },
        backdrop: false,
    }
}

fn f(v: &Value) -> f32 {
    v.as_f64().unwrap() as f32
}

/// Outline buttons have the oracle's border (its `stroke` colour, `stroke_width` 2px) over the
/// container fill over the page, then the fill, and nothing outside; filled buttons have no stroke
/// (`stroke_width` 0), so the edge is the fill. Static colours are translucent, so the expected
/// pixel is the oracle's rgba blended over the page in linear light, stroke over fill over page.
/// Rest, hover and down (pressed buttons shrink about their centre by the oracle's `press`, so
/// edges are read at fractions of a pixel in), pending, hidden label and disabled included.
#[test]
fn border_and_fill_match_oracle() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/button_scene.json")).unwrap();
    assert_eq!(cases.len(), 224);
    let mut outlines = 0;
    for c in &cases {
        let mut what = c.to_string();
        what.truncate(200);
        let mut rt = runtime(App {
            backdrop: true,
            ..app_for(c)
        });
        let (x, y) = rt.find("bt/container").unwrap();
        let down = c["state"] == "down";
        match c["state"].as_str().unwrap() {
            "hover" => rt.pointer_move(x, y),
            "down" => rt.pointer_down(x, y),
            _ => {}
        }
        settle(&mut rt);
        let b = container(&rt);
        // `rect` is where Rune drew it (scaled when pressed); its border snaps to whole pixels.
        let r = b.rect;
        let px = Pixels::of(&rt, SIZE);
        let want = &c["container"];
        let sw = f(&want["stroke_width"]);
        let fill_s = want["fill"].as_str().unwrap_or("transparent");
        let stroke_s = want["stroke"].as_str().unwrap_or("transparent");
        let fill = composite(&[fill_s], BACKDROP_RGB);
        let ring = composite(&[fill_s, stroke_s], BACKDROP_RGB);
        let page = BACKDROP_RGB.map(f32::from);
        for side in SIDES {
            let w = format!("{what} {side:?}");
            let at = |d| probe_snapped(&px, r, side, d);
            near(at(-1), page, &format!("{w}: nothing outside"));
            if sw == 0.0 {
                assert_eq!(stroke_s, "transparent", "{w}: no width, no colour");
                near(at(1), fill, &format!("{w}: edge is the fill"));
                continue;
            }
            outlines += 1;
            assert_eq!(sw, 2.0, "{w}: the oracle's border is 2px");
            near_within(at(0), ring, 8.0, &format!("{w}: outer stroke pixel"));
            near(at(3), fill, &format!("{w}: fill after the stroke"));
            // Pressed, the 2px border is scaled to about 1.9 and snapped, so only the outer pixel
            // is pure stroke and the next one is part stroke; at rest two pixels are.
            if !down {
                near(at(1), ring, &format!("{w}: inner stroke pixel"));
                if ring != fill {
                    differs(at(2), ring, &format!("{w}: border is not wider"));
                }
            }
        }
    }
    assert!(outlines > 100, "{outlines} outline edges were checked");
}

/// With keyboard focus the container has the oracle's ring (`--state focus` is where the oracle
/// draws it for Button): the colour (the focus colour, and white or black for static primary and
/// secondary), 2px thick at a 2px gap, at every size and on both scales; clear in the gap and
/// beyond.
#[test]
fn focus_ring_matches_oracle() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/button_focus_ring.json")).unwrap();
    assert_eq!(cases.len(), 72);
    for c in &cases {
        let mut what = c.to_string();
        what.truncate(120);
        let mut rt = runtime(App {
            hidden: false,
            ..app_for(&serde_json::json!({
                "scheme": c["scheme"], "scale": c["scale"], "size": c["size"],
                "variant": c["variant"], "style": c["style"], "static": c["static"],
            }))
        });
        rt.key(KeyEvent::from(Key::Tab));
        settle(&mut rt);
        let b = container(&rt);
        assert!(b.focus_visible, "{what}: focus is visible");
        Ring::of(c).assert_around(&Pixels::of(&rt, SIZE), b.target, &SIDES, &what);
    }
}

/// No ring without keyboard focus, and none for a disabled button that Tab cannot reach.
#[test]
fn no_ring_without_keyboard_focus() {
    let rings: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/button_focus_ring.json")).unwrap();
    let ring = Ring::of(&rings[0]);
    let base = app_for(&rings[0]);
    let off = runtime(base.clone());
    let b = container(&off).target;
    ring.assert_absent(&Pixels::of(&off, SIZE), b, &SIDES, "unfocused");
    let mut rt = runtime(App {
        disabled: true,
        ..base
    });
    rt.key(KeyEvent::from(Key::Tab));
    settle(&mut rt);
    ring.assert_absent(&Pixels::of(&rt, SIZE), b, &SIDES, "disabled, Tab pressed");
}

/// What the oracle gives hover + keyboard-focus that a Button cannot: the view never sees focus, so
/// the stroke and label of an outline button, and the label of a secondary fill, stay at their
/// hover colours where the oracle lets focus win (a14.31). Only the fill is compared there.
fn focus_beats_hover(c: &Value) -> bool {
    c["state"] == "hover+keyboard-focus"
        && !c["disabled"].as_bool().unwrap_or(false)
        && (c["style"] == "outline" || c["variant"] == "secondary")
}

/// The pixel in `r` (x, y, w, h; truncated outward) closest to `want`, as its largest channel
/// distance: text and icon edges are anti-aliased, so a glyph is present when some pixel is its
/// colour.
fn closest(px: &Pixels, r: [f32; 4], want: [f32; 3]) -> f32 {
    let (x0, y0) = (r[0].floor() as u32, r[1].floor() as u32);
    let (x1, y1) = ((r[0] + r[2]).ceil() as u32, (r[1] + r[3]).ceil() as u32);
    (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| (x, y)))
        .map(|(x, y)| {
            let p = px.at(x, y);
            (0..3)
                .map(|i| (p[i] as f32 - want[i]).abs())
                .fold(0.0, f32::max)
        })
        .fold(f32::MAX, f32::min)
}

/// Keyboard focus (Tab) fills the pill with the oracle's keyboard-focus fill, `button_focus_fill.json`:
/// inside the pill (at the left, top and bottom edges, as `border_and_fill_match_oracle` does), the
/// pill's corner pixel is still the page (the fill is clipped to the radius), the icon and label
/// are still drawn over it in the oracle's colour (translucent static colours composed over the
/// fill), and an outline's stroke is unchanged. Also hover + keyboard-focus (Tab, then the pointer
/// moves in), down + keyboard-focus (Tab, then the pointer presses: a press moves focus to the
/// pointer, so Rune drops the keyboard fill and what is left is the oracle's down look), a
/// disabled button that Tab cannot reach, and pending.
#[test]
fn keyboard_focus_fill_matches_oracle() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/button_focus_fill.json")).unwrap();
    assert_eq!(cases.len(), 118);
    let (mut filled, mut stroked, mut contents) = (0, 0, 0);
    for c in &cases {
        let mut what = c.to_string();
        what.truncate(200);
        let mut rt = runtime(App {
            backdrop: true,
            ..app_for(c)
        });
        let (x, y) = rt.find("bt/container").unwrap();
        rt.key(KeyEvent::from(Key::Tab));
        match c["state"].as_str().unwrap() {
            "hover+keyboard-focus" => rt.pointer_move(x, y),
            "down+keyboard-focus" => rt.pointer_down(x, y),
            _ => {}
        }
        settle(&mut rt);
        let b = container(&rt);
        let r = b.rect;
        let down = c["state"] == "down+keyboard-focus";
        let px = Pixels::of(&rt, SIZE);
        let want = &c["container"];
        let fill_s = want["fill"].as_str().unwrap_or("transparent");
        let stroke_s = want["stroke"].as_str().unwrap_or("transparent");
        let fill = composite(&[fill_s], BACKDROP_RGB);
        let page = BACKDROP_RGB.map(f32::from);
        let gap = focus_beats_hover(c);
        let sw = f(&want["stroke_width"]);
        for side in SIDES {
            let w = format!("{what} {side:?}");
            let at = |d| probe_snapped(&px, r, side, d);
            near(at(3), fill, &format!("{w}: the fill, inside the stroke"));
            filled += 1;
            if sw > 0.0 && !gap && !down {
                near(
                    at(1),
                    composite(&[fill_s, stroke_s], BACKDROP_RGB),
                    &format!("{w}: stroke unchanged"),
                );
                stroked += 1;
            }
        }
        // Corner: the bounding box's corner pixel is outside the pill, so it must still be the page
        // where an unclipped fill would show. It is 5.9px off the arc at radius 16, clear of the
        // ring (2px gap, 2px wide); at S's radius 12 it is 4.3px, on the ring's soft outer edge,
        // so there it only must not be the fill.
        if !down {
            let (x0, y0) = (r[0].round() as u32, r[1].round() as u32);
            let corner = px.at(x0, y0);
            let what = format!("{what}: clipped at the corner");
            if f(&want["radius"]) >= 14.0 {
                near(corner, page, &what);
            } else {
                differs(corner, fill, &what);
            }
        }
        if gap || c["pending"].as_bool().unwrap_or(false) {
            continue;
        }
        // Still on top of the fill, at the oracle's colour. Pressed buttons scale about the centre.
        let origin = [r[0], r[1]];
        for (part, h) in [("icon", None), ("label_part", Some(f(&c["icon"]["h"])))] {
            let Some(p) = c.get(part) else { continue };
            let k = if down { 0.93 } else { 1.0 };
            let (pw, ph) = (f(&p["w"]) * k, h.unwrap_or_else(|| f(&p["h"])) * k);
            let box_ = [
                origin[0] + f(&p["x"]) * k,
                origin[1] + f(&p["y"]) * k,
                pw,
                ph,
            ];
            let ink = p["color"].as_str().unwrap();
            let want = composite(&[fill_s, ink], BACKDROP_RGB);
            let d = closest(&px, box_, want);
            assert!(
                d <= 12.0,
                "{what}: {part} not drawn over the fill ({d} off {want:?})"
            );
            contents += 1;
        }
    }
    assert!(
        filled > 300 && stroked > 40 && contents > 150,
        "{filled} {stroked} {contents}"
    );
}

/// Pointer focus never paints the fill: after a click (and the pointer leaving) the pill is the
/// oracle's rest fill and focus is not visible; a Tab afterwards does show it.
#[test]
fn pointer_focus_paints_no_focus_fill() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/button_scene.json")).unwrap();
    let rest = |variant, style| {
        let c = cases.iter().find(|c| {
            c["state"] == "rest"
                && c["variant"] == variant
                && c["style"] == style
                && c["size"] == "m"
                && c["scheme"] == "light"
                && c["scale"] == "desktop"
                && c.get("disabled").is_none()
                && c.get("static").is_none()
                && c.get("pending").is_none()
        });
        c.unwrap().clone()
    };
    for (variant, style) in [
        ("accent", "fill"),
        ("primary", "fill"),
        ("secondary", "fill"),
    ] {
        let c = rest(variant, style);
        let mut rt = runtime(App {
            backdrop: true,
            ..app_for(&c)
        });
        let (x, y) = rt.find("bt/container").unwrap();
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.pointer_move(1.0, 1.0);
        settle(&mut rt);
        let b = container(&rt);
        assert!(
            !b.focus_visible,
            "{variant}: a click shows no keyboard focus"
        );
        let fill = composite(&[c["container"]["fill"].as_str().unwrap()], BACKDROP_RGB);
        let probe = |rt: &Runtime<App>| {
            probe_snapped(&Pixels::of(rt, SIZE), container(rt).rect, Side::Left, 3)
        };
        near(
            probe(&rt),
            fill,
            &format!("{variant}: clicked, still the rest fill"),
        );
        rt.key(KeyEvent::from(Key::Tab));
        settle(&mut rt);
        assert!(container(&rt).focus_visible);
        differs(
            probe(&rt),
            fill,
            &format!("{variant}: Tab shows the focus fill"),
        );
    }
}
