//! Pixel assertions for the ActionButton, which `ElementInfo` cannot give: the focus ring and what
//! is (not) drawn around the container. See `support/pixels.rs` for how and its limits.

#[path = "support/pixels.rs"]
mod pixels;

use pixels::Pixels;
use rune::{col, Cx, El, Env, Key, KeyEvent, Runtime};
use spectrum_rune::action_button::ActionButton;
use spectrum_rune::icons::CHECKMARK_100;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};

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
