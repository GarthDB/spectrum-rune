//! In-repo pixel assertions without modifying Rune: a headless `Runtime` leaves the primitives it
//! paints in `rt.instances` (public), and `rune-ref` (Rune's own CPU reference renderer, pinned at
//! the same rev, behind its `gpu` feature for `Primitive::from_instance`) rasterises them. No GPU
//! or window. Colours are sRGB 8-bit, and the text/icon atlas is the runtime's own.
//!
//! Limits: the reference renderer is Rune's scalar oracle for its instance stream (blend, SDF
//! shapes, glyph sampling), not the Metal/wgpu path, so a pixel here is "what Rune says it draws".
//! Border width, ring and stroke colours are read at a chosen pixel, not from a property.

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
