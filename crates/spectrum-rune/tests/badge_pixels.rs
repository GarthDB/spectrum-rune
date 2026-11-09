//! Pixel assertions for the Badge, which `ElementInfo` cannot give: the outline stroke (colour and
//! 2px width) and which corners are square for `fixed`. See `support/pixels.rs` for how and its
//! limits (Rune's CPU reference renderer, sRGB 8-bit, scale 1).

#[path = "support/pixels.rs"]
mod pixels;

use pixels::Pixels;
use rune::{col, Cx, El, Env, Runtime};
use spectrum_rune::badge::{Badge, Fixed, Style, Variant};
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

const SIZE: (u32, u32) = (200, 100);
const PAD: f32 = 20.0;

#[derive(Clone)]
struct App {
    dark: bool,
    variant: Variant,
    style: Style,
    fixed: Fixed,
    size: Size,
}

fn view(a: &App, _: &mut Cx<App>) -> El {
    let t = tokens(
        if a.dark {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        },
        Scale::Desktop,
    );
    col().px(PAD).py(PAD).child(
        Badge::new("bd")
            .label("Label")
            .variant(a.variant)
            .style(a.style)
            .fixed(a.fixed)
            .size(a.size)
            .build(t),
    )
}

/// The painted badge and its container rect `(x, y, w, h)` in pixels.
fn paint(a: App) -> (Pixels, (u32, u32, u32, u32)) {
    let dark = a.dark;
    let mut rt = Runtime::new(a, view, (SIZE.0 as f32, SIZE.1 as f32), Env::new(dark, 1.0));
    for _ in 0..40 {
        rt.frame(0.05);
    }
    let c = rt
        .elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some("bd/container"))
        .unwrap();
    let r = (
        c.target[0] as u32,
        c.target[1] as u32,
        c.target[2] as u32,
        c.target[3] as u32,
    );
    (Pixels::of(&rt, SIZE), r)
}

fn near(got: [u8; 4], want: [u8; 3], what: &str) {
    for i in 0..3 {
        assert!(
            (got[i] as i32 - want[i] as i32).abs() <= 3,
            "{what}: got {got:?}, want {want:?}"
        );
    }
    assert_eq!(got[3], 255, "{what}: opaque");
}

fn hex(c: rune::Color) -> [u8; 3] {
    let h = c.hex();
    std::array::from_fn(|i| u8::from_str_radix(&h[1 + 2 * i..3 + 2 * i], 16).unwrap())
}

fn app(dark: bool, variant: Variant, style: Style, fixed: Fixed, size: Size) -> App {
    App {
        dark,
        variant,
        style,
        fixed,
        size,
    }
}

/// An outline badge has a 2px stroke (`border_width_200`) in the variant's visual colour just inside
/// its edge, over the layer-2 background fill (oracle: accent light rgb(75,117,255) on white, dark
/// rgb(86,129,255) on rgb(34,34,34); also the tokens).
#[test]
fn outline_has_a_two_pixel_stroke_in_the_variant_colour() {
    for (dark, stroke, fill) in [
        (false, [75, 117, 255], [255, 255, 255]),
        (true, [86, 129, 255], [34, 34, 34]),
    ] {
        let (px, (x, y, w, h)) = paint(app(
            dark,
            Variant::Accent,
            Style::Outline,
            Fixed::None,
            Size::M,
        ));
        let (cx, cy) = (x + w / 2, y + h / 2);
        // Left edge, mid height: two stroke pixels, then the fill.
        near(px.at(x, cy), stroke, "stroke, outer pixel");
        near(px.at(x + 1, cy), stroke, "stroke, inner pixel");
        near(px.at(x + 2, cy), fill, "fill after 2px");
        // Top and bottom edges, at the middle of the badge (the label's text is not on row 0/1).
        near(px.at(cx, y), stroke, "top stroke");
        near(px.at(cx, y + 1), stroke, "top stroke, second row");
        near(px.at(cx, y + 2), fill, "fill under the top stroke");
        near(px.at(cx, y + h - 1), stroke, "bottom stroke");
        // Nothing outside the container.
        assert_eq!(px.at(x - 1, cy)[3], 0, "no stroke outside the box");
        let t = tokens(
            if dark {
                ColorScheme::Dark
            } else {
                ColorScheme::Light
            },
            Scale::Desktop,
        );
        near(
            px.at(x + 1, cy),
            hex(t.accent_visual_color),
            "stroke is accent_visual_color",
        );
        near(
            px.at(x + 4, cy),
            hex(t.background_layer_2_color),
            "fill is background_layer_2_color",
        );
    }
}

/// Bold and subtle badges draw no stroke: the edge pixel is the fill. The 19 variants without an
/// outline rule draw their outline as bold, so they have none either.
#[test]
fn no_stroke_without_an_outline() {
    let (px, (x, y, _, h)) = paint(app(
        false,
        Variant::Informative,
        Style::Bold,
        Fixed::None,
        Size::M,
    ));
    let t = tokens(ColorScheme::Light, Scale::Desktop);
    near(
        px.at(x, y + h / 2),
        hex(t.informative_background_color_default),
        "bold edge is fill",
    );
    let (px, (x, y, _, h)) = paint(app(
        false,
        Variant::Indigo,
        Style::Outline,
        Fixed::None,
        Size::M,
    ));
    near(
        px.at(x, y + h / 2),
        hex(t.indigo_background_color_default),
        "indigo outline is a solid fill",
    );
    let (px, (x, y, _, h)) = paint(app(
        false,
        Variant::Informative,
        Style::Subtle,
        Fixed::None,
        Size::M,
    ));
    near(
        px.at(x, y + h / 2),
        hex(t.informative_subtle_background_color_default),
        "subtle edge is fill",
    );
}

/// Each `fixed` edge squares its two corners (the corner pixel is filled) and leaves the others
/// rounded (the corner pixel is clear): size XL, radius 10, so the outermost pixel of a rounded
/// corner is well outside the arc.
#[test]
fn fixed_squares_the_corners_of_its_edge() {
    use Fixed::*;
    let t = tokens(ColorScheme::Light, Scale::Desktop);
    // [top-left, top-right, bottom-right, bottom-left]
    for (fixed, square) in [
        (None, [false; 4]),
        (Top, [true, true, false, false]),
        (Right, [false, true, true, false]),
        (Bottom, [false, false, true, true]),
        (Left, [true, false, false, true]),
    ] {
        let (px, (x, y, w, h)) = paint(app(false, Variant::Neutral, Style::Bold, fixed, Size::XL));
        let corners = [
            (x, y),
            (x + w - 1, y),
            (x + w - 1, y + h - 1),
            (x, y + h - 1),
        ];
        for (i, (cx, cy)) in corners.into_iter().enumerate() {
            let p = px.at(cx, cy);
            if square[i] {
                near(
                    p,
                    hex(t.neutral_subdued_background_color_default),
                    &format!("{fixed:?} corner {i} is square"),
                );
            } else {
                assert!(p[3] < 40, "{fixed:?} corner {i} is rounded, got {p:?}");
            }
        }
    }
}

/// The same on an outline badge: the stroke follows the square corner.
#[test]
fn fixed_outline_stroke_reaches_the_square_corner() {
    let (px, (x, y, _, _)) = paint(app(
        false,
        Variant::Accent,
        Style::Outline,
        Fixed::Top,
        Size::XL,
    ));
    near(px.at(x, y), [75, 117, 255], "square corner is stroke");
}
