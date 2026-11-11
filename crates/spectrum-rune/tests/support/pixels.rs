//! In-repo pixel assertions without modifying Rune: a headless `Runtime` leaves the primitives it
//! paints in `rt.instances` (public), and `rune-ref` (Rune's own CPU reference renderer, pinned at
//! the same rev, behind its `gpu` feature for `Primitive::from_instance`) rasterises them. No GPU
//! or window. Colours are sRGB 8-bit, and the text/icon atlas is the runtime's own.
//!
//! Limits: the reference renderer is Rune's scalar oracle for its instance stream (blend, SDF
//! shapes, glyph sampling), not the Metal/wgpu path, so a pixel here is "what Rune says it draws".
//! Border width, ring and stroke colours are read at a chosen pixel, not from a property.

#![allow(dead_code)]

use rune::Runtime;
use rune_ref::{Atlas, Frame, Primitive, Scene};

pub struct Pixels(Frame);

impl Pixels {
    /// Paints `rt` at its viewport (scale 1 in these tests, so points are pixels).
    pub fn of<S>(rt: &Runtime<S>, (w, h): (u32, u32)) -> Self {
        let primitives = rt
            .instances
            .iter()
            .enumerate()
            .map(|(i, inst)| Primitive::from_instance(inst, i as u32, format!("instance {i}")))
            .collect();
        let atlas = &rt.text.atlas;
        let size = (atlas.data.len() as f64).sqrt() as u32;
        let scene = Scene {
            width: w,
            height: h,
            clear: [0.0; 4],
            primitives,
            atlas: Atlas {
                width: size,
                height: size,
                data: atlas.data.clone(),
            },
        };
        scene.validate().expect("a valid reference scene");
        let mut frame = Frame::new(w, h, false);
        frame.render(&scene);
        Pixels(frame)
    }

    /// The sRGB RGBA of one pixel.
    #[allow(dead_code)]
    pub fn at(&self, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * self.0.width + x) * 4) as usize;
        self.0.rgba[i..i + 4].try_into().unwrap()
    }

    /// Writes the frame as an RGBA PNG (the gallery's CI artifact).
    #[allow(dead_code)]
    pub fn write_png(&self, path: &std::path::Path) {
        let file = std::fs::File::create(path).expect("create the png");
        let mut enc = png::Encoder::new(std::io::BufWriter::new(file), self.0.width, self.0.height);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header()
            .and_then(|mut w| w.write_image_data(&self.0.rgba))
            .expect("write the png");
    }
}

/// A colour as the oracle writes it: `rgb(r, g, b)`, `rgba(r, g, b, a)`, the fixtures' `r,g,b[,a]`,
/// or `transparent`. sRGB 0..255 and alpha 0..1.
pub fn rgba(s: &str) -> [f32; 4] {
    if s == "transparent" {
        return [0.0; 4];
    }
    let v: Vec<f32> = s
        .trim_start_matches("rgba(")
        .trim_start_matches("rgb(")
        .trim_end_matches(')')
        .split(',')
        .map(|c| c.trim().parse().unwrap())
        .collect();
    [v[0], v[1], v[2], v.get(3).copied().unwrap_or(1.0)]
}

pub fn hex(c: rune::Color) -> [u8; 3] {
    let h = c.hex();
    std::array::from_fn(|i| u8::from_str_radix(&h[1 + 2 * i..3 + 2 * i], 16).unwrap())
}

pub fn to_linear(c: f32) -> f32 {
    let c = c / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn from_linear(l: f32) -> f32 {
    let c = if l <= 0.0031308 {
        l * 12.92
    } else {
        1.055 * l.powf(1.0 / 2.4) - 0.055
    };
    c * 255.0
}

/// What the oracle's colours `layers` (bottom first, each `rgb(..)`/`rgba(..)`) look like painted
/// over an opaque `backdrop`. Rune blends in linear light, so a translucent colour is mixed there
/// and not in sRGB; the result is sRGB 0..255, unrounded.
pub fn composite(layers: &[&str], backdrop: [u8; 3]) -> [f32; 3] {
    let mut out = backdrop.map(|c| to_linear(c as f32));
    for l in layers {
        let c = rgba(l);
        for i in 0..3 {
            out[i] = to_linear(c[i]) * c[3] + out[i] * (1.0 - c[3]);
        }
    }
    out.map(from_linear)
}

/// `got` is opaque and within 3 per channel of `want`: the tolerance of the other oracle tests
/// (the oracle prints 8-bit sRGB, and Rune's reference renderer quantises once more). "Opaque" is
/// alpha 245+: the outermost pixel of a curved edge is a few percent short of full coverage.
pub fn near(got: [u8; 4], want: [f32; 3], what: &str) {
    near_within(got, want, 3.0, what)
}

/// [`near`] with a looser per-channel tolerance, for the outermost pixel of a curved edge over a
/// page: it is ~98% covered, and the missing 2% of the page shows through (more in linear light).
pub fn near_within(got: [u8; 4], want: [f32; 3], tol: f32, what: &str) {
    for i in 0..3 {
        assert!(
            (got[i] as f32 - want[i]).abs() <= tol,
            "{what}: got {got:?}, want {want:?}"
        );
    }
    assert!(got[3] >= 245, "{what}: not opaque, got {got:?}");
}

/// The pixel in `r` (x, y, w, h; truncated outward) closest to `want`, as its largest channel
/// distance: text and icon edges are anti-aliased, so a glyph is present when some pixel is its
/// colour (and a different ink is told apart when none is). Pixels under 98% alpha are skipped:
/// over a clear page a half-covered glyph edge is darker than its ink.
pub fn closest(px: &Pixels, r: [f32; 4], want: [f32; 3]) -> f32 {
    let (x0, y0) = (r[0].floor() as u32, r[1].floor() as u32);
    let (x1, y1) = ((r[0] + r[2]).ceil() as u32, (r[1] + r[3]).ceil() as u32);
    (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| (x, y)))
        .map(|(x, y)| {
            let p = px.at(x, y);
            if p[3] < 250 {
                return f32::MAX;
            }
            (0..3)
                .map(|i| (p[i] as f32 - want[i]).abs())
                .fold(0.0, f32::max)
        })
        .fold(f32::MAX, f32::min)
}

/// `got` is not the colour `unwanted` (a neighbour that must NOT be the stroke colour):
/// transparent counts as different.
pub fn differs(got: [u8; 4], unwanted: [f32; 3], what: &str) {
    let same = got[3] == 255 && (0..3).all(|i| (got[i] as f32 - unwanted[i]).abs() <= 3.0);
    assert!(
        !same,
        "{what}: {got:?} is the colour {unwanted:?} it should not be"
    );
}

/// Nothing is drawn at `got`: alpha 8 (3%) or less, because a ring's soft inner edge bleeds a
/// little into the gap pixel next to it.
pub fn clear(got: [u8; 4], what: &str) {
    assert!(got[3] <= 8, "{what}: not clear, got {got:?}");
}

#[derive(Clone, Copy, Debug)]
pub enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

/// `[x, y, w, h]` scaled about its centre (a pressed control shrinks that way).
pub fn scaled(r: [f32; 4], k: f32) -> [f32; 4] {
    let (w, h) = (r[2] * k, r[3] * k);
    [r[0] + (r[2] - w) / 2.0, r[1] + (r[3] - h) / 2.0, w, h]
}

/// The pixel `depth` pixels in from the middle of `side` of `r` (0 is the outermost pixel inside
/// it, -1 the first one outside), at scale 1. An edge at a fraction is rounded outward.
pub fn probe(px: &Pixels, r: [f32; 4], side: Side, depth: i32) -> [u8; 4] {
    let (cx, cy) = ((r[0] + r[2] / 2.0).floor(), (r[1] + r[3] / 2.0).floor());
    let (x, y) = match side {
        Side::Left => (r[0].floor() + depth as f32, cy),
        Side::Right => ((r[0] + r[2]).ceil() - 1.0 - depth as f32, cy),
        Side::Top => (cx, r[1].floor() + depth as f32),
        Side::Bottom => (cx, (r[1] + r[3]).ceil() - 1.0 - depth as f32),
    };
    px.at(x as u32, y as u32)
}

/// Like [`probe`] with the edges of `r` rounded to the nearest pixel, which is where Rune
/// snaps a border: for a pressed (scaled) control, whose `ElementInfo::rect` is off the grid.
pub fn probe_snapped(px: &Pixels, r: [f32; 4], side: Side, depth: i32) -> [u8; 4] {
    let (x0, y0) = (r[0].round(), r[1].round());
    let (x1, y1) = ((r[0] + r[2]).round(), (r[1] + r[3]).round());
    probe(px, [x0, y0, x1 - x0, y1 - y0], side, depth)
}

pub const SIDES: [Side; 4] = [Side::Left, Side::Right, Side::Top, Side::Bottom];

/// A focus ring from the oracle (`ring.jq`): `stroke` colour, `width` and `gap` in pixels.
pub struct Ring {
    pub stroke: [f32; 3],
    pub width: i32,
    pub gap: i32,
}

impl Ring {
    pub fn of(v: &serde_json::Value) -> Self {
        let r = &v["ring"];
        let n = |k: &str| r[k].as_f64().unwrap();
        // The oracle strokes the ring on a rect that sits gap + width/2 outside the part.
        let t = &v["target"];
        let out = n("gap") + n("width") / 2.0;
        let d = |a: &str, b: &str| t[a].as_f64().unwrap() - r[b].as_f64().unwrap();
        // x and y are `out` further out; w and h are `2 * out` bigger.
        for (got, want, axis) in [
            (d("x", "x"), out, "x"),
            (d("y", "y"), out, "y"),
            (
                r["w"].as_f64().unwrap() - t["w"].as_f64().unwrap(),
                2.0 * out,
                "w",
            ),
            (
                r["h"].as_f64().unwrap() - t["h"].as_f64().unwrap(),
                2.0 * out,
                "h",
            ),
        ] {
            assert!(
                (got - want).abs() < 0.01,
                "ring fixture {axis}: {got} vs {want}: {v}"
            );
        }
        let c = rgba(r["stroke"].as_str().unwrap());
        Ring {
            stroke: [c[0], c[1], c[2]],
            width: n("width") as i32,
            gap: n("gap") as i32,
        }
    }

    /// Around `r` (the part, at scale 1) on `sides`: `gap` clear pixels, `width` pixels of ring,
    /// then clear again, so the thickness is checked at both of its edges.
    pub fn assert_around(&self, px: &Pixels, r: [f32; 4], sides: &[Side], what: &str) {
        for &s in sides {
            for k in 1..=self.gap {
                clear(probe(px, r, s, -k), &format!("{what} {s:?}: gap pixel {k}"));
            }
            for k in self.gap + 1..=self.gap + self.width {
                near(
                    probe(px, r, s, -k),
                    self.stroke,
                    &format!("{what} {s:?}: ring pixel {k}"),
                );
            }
            let beyond = self.gap + self.width + 1;
            clear(
                probe(px, r, s, -beyond),
                &format!("{what} {s:?}: nothing beyond the ring"),
            );
        }
    }

    /// No ring at all: every pixel out to the ring's outer edge is clear.
    pub fn assert_absent(&self, px: &Pixels, r: [f32; 4], sides: &[Side], what: &str) {
        for &s in sides {
            for k in 1..=self.gap + self.width {
                clear(probe(px, r, s, -k), &format!("{what} {s:?}: pixel {k} out"));
            }
        }
    }
}
