//! Pixel assertions for the ActionButton, which `ElementInfo` cannot give: the focus ring and what
//! is (not) drawn around the container, and the keyboard-focus fill (oracle fixture
//! `action_button_focus.json`). See `support/pixels.rs` for how and its limits.

#[path = "support/pixels.rs"]
mod pixels;

use pixels::{closest, composite, differs, near_within, Pixels};
use rune::{col, Cx, El, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::action_button::ActionButton;
use spectrum_rune::button::StaticColor;
use spectrum_rune::icons::CHECKMARK_100;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::ActionButtonSize;

const SIZE: (u32, u32) = (200, 100);
const PAD: f32 = 20.0;

/// State: (dark, selected + emphasized).
fn view(&(dark, selected): &(bool, bool), _: &mut Cx<(bool, bool)>) -> El {
    let t = tokens(
        if dark {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        },
        Scale::Desktop,
    );
    col().px(PAD).py(PAD).child(
        ActionButton::new("bt")
            .label("Label")
            .icon(&CHECKMARK_100)
            .hold_icon(true)
            .selected(selected)
            .emphasized(selected)
            .build(t),
    )
}

fn focused(dark: bool) -> Runtime<(bool, bool)> {
    let mut rt = Runtime::new(
        (dark, false),
        view,
        (SIZE.0 as f32, SIZE.1 as f32),
        Env::new(dark, 1.0),
    );
    rt.key(KeyEvent::from(Key::Tab));
    for _ in 0..40 {
        rt.frame(0.05);
    }
    rt
}

fn hex(c: rune::Color) -> [u8; 3] {
    let h = c.hex();
    std::array::from_fn(|i| u8::from_str_radix(&h[1 + 2 * i..3 + 2 * i], 16).unwrap())
}

fn near(got: [u8; 4], want: [u8; 3], what: &str) {
    for i in 0..3 {
        assert!(
            (got[i] as i32 - want[i] as i32).abs() <= 3,
            "{what}: got {got:?}, want {want:?}"
        );
    }
}

/// The ring is 2px (`focus_indicator_thickness`) at 2px (`focus_ring_gap`) from the container,
/// in `focus_indicator_color` (oracle `--state keyboard-focus`: rgb(75,117,255) light, (64,105,253)
/// dark; also checked against the tokens here).
#[test]
fn focus_ring_is_drawn_in_the_focus_colour_outside_the_container() {
    for dark in [false, true] {
        let rt = focused(dark);
        let t = tokens(
            if dark {
                ColorScheme::Dark
            } else {
                ColorScheme::Light
            },
            Scale::Desktop,
        );
        let c = rt
            .elements()
            .into_iter()
            .find(|e| e.key.as_deref() == Some("bt/container"))
            .unwrap();
        assert!(c.focus_visible);
        let px = Pixels::of(&rt, SIZE);
        let (x, y, h) = (c.target[0] as u32, c.target[1] as u32, c.target[3] as u32);
        let ring = hex(t.focus_indicator_color);
        near(
            px.at(x - 3, y + h / 2),
            ring,
            "ring, left edge (gap 2, thickness 2)",
        );
        near(px.at(x - 4, y + h / 2), ring, "ring, outer pixel");
        // The gap between the ring and the container is empty, and so is outside the ring.
        assert_eq!(px.at(x - 2, y + h / 2)[3], 0, "gap is clear");
        assert_eq!(px.at(x - 6, y + h / 2)[3], 0, "nothing beyond the ring");
        // Inside, the container fill (gray_100 at rest).
        near(px.at(x + 3, y + h / 2), hex(t.gray_100), "container fill");
        let want = if dark { [64, 105, 253] } else { [75, 117, 255] };
        near(px.at(x - 3, y + h / 2), want, "ring, oracle colour");
    }
}

/// The recipe has no border on any ActionButton state: unfocused, nothing is drawn outside the
/// container's edge.
#[test]
fn there_is_no_border_or_ring_without_keyboard_focus() {
    let mut rt = Runtime::new(
        (false, false),
        view,
        (SIZE.0 as f32, SIZE.1 as f32),
        Env::new(false, 1.0),
    );
    for _ in 0..40 {
        rt.frame(0.05);
    }
    let c = rt
        .elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some("bt/container"))
        .unwrap();
    let px = Pixels::of(&rt, SIZE);
    let (x, y, h) = (c.target[0] as u32, c.target[1] as u32, c.target[3] as u32);
    for dx in 1..=4 {
        assert_eq!(
            px.at(x - dx, y + h / 2)[3],
            0,
            "no stroke {dx}px left of the container"
        );
    }
}

/// The hold icon's triangle fills the lower-right half of its 6px box: sample near the corner, in
/// the colour the oracle gives it (`neutral_content_color_default` unselected, white when selected
/// and emphasized).
#[test]
fn hold_icon_pixel_has_the_hold_colour() {
    for selected in [false, true] {
        let mut rt = Runtime::new(
            (false, selected),
            view,
            (SIZE.0 as f32, SIZE.1 as f32),
            Env::new(false, 1.0),
        );
        for _ in 0..40 {
            rt.frame(0.05);
        }
        let t = tokens(ColorScheme::Light, Scale::Desktop);
        let h = rt
            .elements()
            .into_iter()
            .find(|e| e.key.as_deref() == Some("bt/hold-icon"))
            .unwrap();
        let px = Pixels::of(&rt, SIZE);
        let (x, y) = ((h.target[0] + 4.0) as u32, (h.target[1] + 4.0) as u32);
        let want = if selected {
            t.white
        } else {
            t.neutral_content_color_default
        };
        near(px.at(x, y), hex(want), "hold icon pixel");
    }
}

const FSIZE: (u32, u32) = (300, 120);
const FPAD: f32 = 30.0;
/// The page behind the button: mid grey-blue, so translucent static fills show.
const BACKDROP: &str = "#4b6a8c";
const BACKDROP_RGB: [u8; 3] = [0x4b, 0x6a, 0x8c];

#[derive(Clone)]
struct F {
    dark: bool,
    mobile: bool,
    size: ActionButtonSize,
    quiet: bool,
    selected: bool,
    emphasized: bool,
    stat: Option<StaticColor>,
    disabled: bool,
    pending: bool,
}

fn fview(a: &F, _: &mut Cx<F>) -> El {
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
    let mut b = ActionButton::new("bt")
        .label("Label")
        .icon(&CHECKMARK_100)
        .size(a.size)
        .quiet(a.quiet)
        .selected(a.selected)
        .emphasized(a.emphasized)
        .disabled(a.disabled)
        .pending(a.pending);
    if let Some(c) = a.stat {
        b = b.static_color(c);
    }
    col()
        .w(FSIZE.0 as f32)
        .h(FSIZE.1 as f32)
        .px(FPAD)
        .py(FPAD)
        .fill(rune::color::hex(BACKDROP).unwrap())
        .child(b.build(t))
}

fn fsettle(rt: &mut Runtime<F>) {
    for _ in 0..40 {
        rt.frame(0.05);
    }
}

fn fapp(c: &Value) -> F {
    let b = |k: &str| c[k].as_bool().unwrap_or(false);
    F {
        dark: c["scheme"] == "dark",
        mobile: c["scale"] == "mobile",
        size: match c["size"].as_str().unwrap() {
            "xs" => ActionButtonSize::XS,
            "s" => ActionButtonSize::S,
            "m" => ActionButtonSize::M,
            "l" => ActionButtonSize::L,
            _ => ActionButtonSize::XL,
        },
        quiet: b("quiet"),
        selected: b("selected"),
        emphasized: b("emphasized"),
        stat: match c["static"].as_str() {
            Some("white") => Some(StaticColor::White),
            Some("black") => Some(StaticColor::Black),
            _ => None,
        },
        disabled: b("disabled"),
        pending: b("pending"),
    }
}

fn frt(a: F) -> Runtime<F> {
    let dark = a.dark;
    let mut rt = Runtime::new(
        a,
        fview,
        (FSIZE.0 as f32, FSIZE.1 as f32),
        Env::new(dark, 1.0),
    );
    fsettle(&mut rt);
    rt
}

fn fcontainer(rt: &Runtime<F>) -> rune::ElementInfo {
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some("bt/container"))
        .unwrap()
}

fn fnum(v: &Value) -> f32 {
    v.as_f64().unwrap() as f32
}

/// Keyboard focus (Tab) gives the oracle's keyboard-focus look, `action_button_focus.json`: the
/// container fill inside the pill (left, top and bottom edges; selected neutral, emphasized, quiet
/// or not, and static selected + emphasized, which go to the hover colour; unselected and the other
/// static colours keep their rest fill, which `focus_fill` must not stack on), clipped to the radius
/// (the bounding box's corner pixel is not the fill), the icon and label
/// still drawn over it in the oracle's colour (some fully covered pixel within 12 of it; text is
/// anti-aliased) and the ring unchanged. Also hover + keyboard-focus (Tab, then the pointer moves
/// in: the oracle's fill, once), disabled (not focusable: the oracle's disabled fill, no ring) and
/// pending (the oracle's gray fill).
#[test]
fn keyboard_focus_fill_matches_oracle() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/action_button_focus.json")).unwrap();
    assert_eq!(cases.len(), 104);
    let scene: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/action_button_scene.json")).unwrap();
    let page = BACKDROP_RGB.map(f32::from);
    let (mut changed, mut contents, mut clipped, mut gaps) = (0, 0, 0, 0);
    for c in &cases {
        let mut what = c.to_string();
        what.truncate(160);
        let disabled = c["disabled"] == true;
        let mut rt = frt(fapp(c));
        let (x, y) = rt.find("bt/container").unwrap();
        rt.key(KeyEvent::from(Key::Tab));
        if c["state"] == "hover+keyboard-focus" {
            rt.pointer_move(x, y);
        }
        fsettle(&mut rt);
        let b = fcontainer(&rt);
        assert_eq!(b.focus_visible, !disabled, "{what}: focus visible");
        let r = b.rect;
        let px = Pixels::of(&rt, FSIZE);
        let want = &c["container"];
        let fill_s = want["fill"].as_str().unwrap();
        // NOT matched: an unselected button with the pointer over it AND keyboard focus. The oracle
        // lets focus win over hover (the rest fill, the rest label ink; quiet: the hover wash with
        // the rest ink), the view cannot see focus (as for Button, a14.31), so the oracle's HOVER
        // look is drawn: pinned here, with the oracle's focus value recorded next to it.
        let gap = c["state"] == "hover+keyboard-focus" && c["selected"] != true && !disabled;
        let hover_case = gap.then(|| {
            let same = |k: &str| c[k].as_bool().unwrap_or(false);
            scene
                .iter()
                .find(|h| {
                    let hb = |k: &str| h[k].as_bool().unwrap_or(false);
                    h["state"] == "hover"
                        && h["scheme"] == c["scheme"]
                        && h["scale"] == c["scale"]
                        && h["size"] == c["size"]
                        && hb("quiet") == same("quiet")
                        && hb("selected") == same("selected")
                        && hb("emphasized") == same("emphasized")
                        && h["static"] == c["static"]
                })
                .expect("the oracle's hover case")
        });
        let want_fill = hover_case.map_or(fill_s, |h| h["container"]["fill"].as_str().unwrap());
        let fill = composite(&[want_fill], BACKDROP_RGB);
        let (x0, y0) = (r[0].round() as u32, r[1].round() as u32);
        let (w, h) = (r[2].round() as u32, r[3].round() as u32);
        let edge = |side: usize| match side {
            0 => px.at(x0 + 2, y0 + h / 2),
            1 => px.at(x0 + w / 2, y0 + 2),
            _ => px.at(x0 + w / 2, y0 + h - 3),
        };
        for side in 0..3 {
            if want_fill != "transparent" {
                near_within(edge(side), fill, 3.0, &format!("{what}: fill, side {side}"));
                if gap && want_fill != fill_s {
                    differs(
                        edge(side),
                        composite(&[fill_s], BACKDROP_RGB),
                        &format!("{what}: the gap to the oracle's focus fill, side {side}"),
                    );
                    gaps += 1;
                }
            } else {
                near_within(
                    edge(side),
                    page,
                    3.0,
                    &format!("{what}: no fill, side {side}"),
                );
            }
        }
        if !disabled && want_fill != "transparent" {
            // The bounding box's corner is outside the pill, so an unclipped fill would show
            // there. (On the ring's soft edge at these radii, so it only must not be the fill.)
            differs(
                px.at(x0, y0),
                fill,
                &format!("{what}: clipped at the corner"),
            );
            clipped += 1;
        }
        // The ring is as before (the colours of the existing ring test), absent when disabled.
        let ring = if c["scheme"] == "dark" {
            [64.0, 105.0, 253.0]
        } else {
            [75.0, 117.0, 255.0]
        };
        if disabled {
            assert_eq!(
                px.at(x0 - 3, y0 + h / 2)[..3],
                px.at(x0 - 9, y0 + h / 2)[..3],
                "{what}: no ring"
            );
        } else {
            near_within(
                px.at(x0 - 3, y0 + h / 2),
                ring,
                3.0,
                &format!("{what}: ring"),
            );
        }
        if c["pending"] == true {
            continue;
        }
        for part in ["icon", "label_part"] {
            let p = &c[part];
            let hh = if part == "icon" {
                fnum(&p["h"])
            } else {
                fnum(&c["icon"]["h"])
            };
            let box_ = [
                r[0] + fnum(&p["x"]),
                r[1] + fnum(&p["y"]),
                fnum(&p["w"]),
                hh,
            ];
            if gap {
                // The label's (and the icon's) ink is the hover one: not the oracle's.
                continue;
            }
            let ink = composite(&[fill_s, p["color"].as_str().unwrap()], BACKDROP_RGB);
            let d = closest(&px, box_, ink);
            assert!(
                d <= 12.0,
                "{what}: {part} not drawn over the fill ({d} off {ink:?})"
            );
            contents += 1;
        }
        // Counted for the guard below: selected buttons are the ones whose fill changes.
        if c["selected"] == true && !disabled && fill_s != "transparent" {
            changed += 1;
        }
    }
    assert!(
        changed > 30 && contents > 150 && clipped > 60 && gaps >= 12,
        "{changed} {contents} {clipped} {gaps}"
    );
}

/// Pointer focus never paints the fill: after a click (and the pointer leaving) a selected button
/// is its rest fill and focus is not visible; Tab afterwards shows the oracle's focus fill. An
/// unselected one has the same fill before and after Tab (the oracle: focus = rest).
#[test]
fn pointer_focus_paints_no_focus_fill() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/action_button_scene.json")).unwrap();
    let focus: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/action_button_focus.json")).unwrap();
    let find = |v: &[Value], sel: bool, emp: bool, state: &str| {
        v.iter()
            .find(|c| {
                c["scheme"] == "light"
                    && c["scale"] == "desktop"
                    && c["size"] == "m"
                    && c["selected"].as_bool().unwrap_or(false) == sel
                    && c["emphasized"].as_bool().unwrap_or(false) == emp
                    && c.get("quiet").is_none()
                    && c.get("static").is_none()
                    && c.get("disabled").is_none()
                    && c.get("pending").is_none()
                    && c["state"] == state
            })
            .unwrap()
            .clone()
    };
    for (sel, emp) in [(true, false), (true, true), (false, false)] {
        let rest = find(&cases, sel, emp, "rest");
        let key = find(&focus, sel, emp, "keyboard-focus");
        let rest_fill = composite(&[rest["container"]["fill"].as_str().unwrap()], BACKDROP_RGB);
        let key_fill = composite(&[key["container"]["fill"].as_str().unwrap()], BACKDROP_RGB);
        let mut rt = frt(fapp(&rest));
        let (x, y) = rt.find("bt/container").unwrap();
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.pointer_move(1.0, 1.0);
        fsettle(&mut rt);
        assert!(
            !fcontainer(&rt).focus_visible,
            "a click shows no keyboard focus"
        );
        let probe = |rt: &Runtime<F>| {
            let r = fcontainer(rt).rect;
            Pixels::of(rt, FSIZE).at(r[0] as u32 + 2, (r[1] + r[3] / 2.0) as u32)
        };
        let what = format!("sel={sel} emph={emp}");
        near_within(
            probe(&rt),
            rest_fill,
            3.0,
            &format!("{what}: clicked, the rest fill"),
        );
        rt.key(KeyEvent::from(Key::Tab));
        fsettle(&mut rt);
        assert!(fcontainer(&rt).focus_visible);
        near_within(
            probe(&rt),
            key_fill,
            3.0,
            &format!("{what}: Tab shows the oracle's focus fill"),
        );
        if sel {
            differs(
                probe(&rt),
                rest_fill,
                &format!("{what}: Tab changes the fill"),
            );
        }
    }
}
