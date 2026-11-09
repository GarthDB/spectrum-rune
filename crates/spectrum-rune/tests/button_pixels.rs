//! Pixel assertions for the Button, which `ElementInfo` cannot give: the outline border (colour and
//! 2px width, none for fills), the fill as painted over a backdrop (static colours are translucent),
//! and the focus ring (colour, thickness, gap), including the white and black rings of static
//! primary and secondary. Every expected value comes from the oracle fixtures (`button_scene.json`,
//! `button_focus_ring.json`, from `design-data render button --format scene`), never from our
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
