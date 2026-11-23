//! Spectrum Popover: a raised surface with a tip pointing at its trigger, from `design-data` visuals/popover.json
//! (a container with a content slot, and a tip on the side that faces the trigger). Every size, radius, colour
//! and shadow value comes from [`Tokens`].
//!
//! The popover is its own element: put it anywhere, or give it a trigger with [`Popover::anchor`], which places it
//! beside that keyed element with Rune's `El::anchored` (it flips to the other side when it does not fit, and stays
//! inside the window; the tip stays on the side the placement named). Escape calls [`Popover::on_dismiss`].
//!
//! Built: the six placements (`top`, `bottom`, `left`, `right`, and `start` / `end`, which are `left` / `right` in a
//! left-to-right layout), `hideTip`, `width`, `height`. Not built (the recipe has them): the aligned placements
//! (`top left`, ...), whose tip follows the trigger's centre, and right-to-left layouts.
//!
//! Verification: geometry, fill and tip against the oracle (`tests/popover.rs`), the shadow from pixels.

use crate::tokens::Tokens;
use rune::{col, el, icon, row, El, Icon, Key, Mods, Side};
use std::rc::Rc;

/// Which side of its trigger the popover sits on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Placement {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
    /// `Left` in a left-to-right layout.
    Start,
    /// `Right` in a left-to-right layout.
    End,
}

// The recipe's tip paths, in a 16-point square (the 16 x 8 or 8 x 16 tip fills one half of it).
static TIP_BOTTOM: Icon = Icon {
    name: "popover-tip-bottom",
    view: 16.0,
    body: "<path d=\"M0 0 L6.4 6.4 Q8 8 9.6 6.4 L16 0 Z\" fill=\"currentColor\"/>",
};
static TIP_TOP: Icon = Icon {
    name: "popover-tip-top",
    view: 16.0,
    body: "<path d=\"M0 8 L6.4 1.6 Q8 0 9.6 1.6 L16 8 Z\" fill=\"currentColor\"/>",
};
static TIP_END: Icon = Icon {
    name: "popover-tip-end",
    view: 16.0,
    body: "<path d=\"M0 0 L6.4 6.4 Q8 8 6.4 9.6 L0 16 Z\" fill=\"currentColor\"/>",
};
static TIP_START: Icon = Icon {
    name: "popover-tip-start",
    view: 16.0,
    body: "<path d=\"M8 0 L1.6 6.4 Q0 8 1.6 9.6 L8 16 Z\" fill=\"currentColor\"/>",
};

type Dismiss = Rc<dyn Fn(El) -> El>;

pub struct Popover {
    key: String,
    placement: Placement,
    hide_tip: bool,
    width: Option<f32>,
    height: Option<f32>,
    offset: f32,
    anchor: Option<String>,
    label: Option<String>,
    content: Vec<El>,
    dismiss: Option<Dismiss>,
}

impl Popover {
    /// `key` must be unique per instance: the parts are keyed `{key}/container` and `{key}/tip`.
    pub fn new(key: impl Into<String>) -> Self {
        Popover {
            key: key.into(),
            placement: Placement::default(),
            hide_tip: false,
            width: None,
            height: None,
            offset: 8.0,
            anchor: None,
            label: None,
            content: Vec::new(),
            dismiss: None,
        }
    }
    pub fn placement(mut self, p: Placement) -> Self {
        self.placement = p;
        self
    }
    pub fn hide_tip(mut self, v: bool) -> Self {
        self.hide_tip = v;
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }
    pub fn height(mut self, h: f32) -> Self {
        self.height = Some(h);
        self
    }
    /// The distance from the trigger to the popover's tip (the recipe's `offset`, 8 by default).
    pub fn offset(mut self, o: f32) -> Self {
        self.offset = o;
        self
    }
    /// Places the popover beside the element keyed `trigger`, on the side `placement` names. Attach it at the root of the
    /// view, after the trigger, so it draws above later siblings and escapes clipping ancestors.
    pub fn anchor(mut self, trigger: impl Into<String>) -> Self {
        self.anchor = Some(trigger.into());
        self
    }
    /// The popover's accessible name.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    /// The popover's content (the recipe's `content` slot).
    pub fn child(mut self, c: El) -> Self {
        self.content.push(c);
        self
    }
    /// Called on Escape while focus is inside the popover.
    pub fn on_dismiss<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        let f = Rc::new(f);
        self.dismiss = Some(Rc::new(move |e: El| {
            let f = f.clone();
            e.on_key(move |s: &mut S, ev, _| {
                let hit = matches!(ev.key, Key::Escape) && ev.mods == Mods::default();
                if hit {
                    f(s);
                }
                hit
            })
        }));
        self
    }

    pub fn build(self, t: &Tokens) -> El {
        let mut surface = col()
            .key(format!("{}/container", self.key))
            .padding(t.spacing_100)
            .radius(t.corner_radius_700)
            .fill(t.background_layer_2_color)
            .shadow(
                t.drop_shadow_elevated_y,
                t.drop_shadow_elevated_blur,
                t.drop_shadow_elevated_color,
            )
            .role("dialog")
            .children(self.content);
        if let Some(l) = &self.label {
            surface = surface.label(l.clone());
        }
        if let Some(w) = self.width {
            surface = surface.w(w);
        }
        if let Some(h) = self.height {
            surface = surface.h(h);
        }
        if let Some(d) = &self.dismiss {
            surface = d(surface);
        }
        let (side, along_row) = match self.placement {
            Placement::Top => (Side::Top, false),
            Placement::Bottom => (Side::Bottom, false),
            Placement::Left | Placement::Start => (Side::Left, true),
            Placement::Right | Placement::End => (Side::Right, true),
        };
        // The tip sits on the side that faces the trigger: below a popover above it, and so on.
        let tip = (!self.hide_tip).then(|| {
            let (glyph, w, h) = match side {
                Side::Top => (&TIP_BOTTOM, t.popover_tip_width, t.popover_tip_height),
                Side::Bottom => (&TIP_TOP, t.popover_tip_width, t.popover_tip_height),
                Side::Left => (&TIP_END, t.popover_tip_height, t.popover_tip_width),
                Side::Right => (&TIP_START, t.popover_tip_height, t.popover_tip_width),
            };
            el().key(format!("{}/tip", self.key))
                .w(w)
                .h(h)
                .self_center()
                .child(icon(glyph, 16.0).color(t.background_layer_2_color))
        });
        let mut root = if along_row { row() } else { col() };
        // The popover is the side of the pair away from the trigger.
        let tip_first = matches!(side, Side::Bottom | Side::Right);
        if tip_first {
            if let Some(tip) = tip {
                root = root.child(tip);
            }
            root = root.child(surface);
        } else {
            root = root.child(surface);
            if let Some(tip) = tip {
                root = root.child(tip);
            }
        }
        // A row around the pair keeps it at its own width (the tip centres on the container, not the page).
        let mut root = crate::hug(root);
        if let Some(trigger) = self.anchor {
            // The tip fills the gap up to the offset.
            let gap = if self.hide_tip {
                self.offset
            } else {
                (self.offset - t.popover_tip_height).max(0.0)
            };
            root = root.anchored(trigger, side, rune::Align::Center, gap);
        }
        root
    }
}
