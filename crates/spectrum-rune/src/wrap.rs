//! Label wrapping at a request width, shared by Switch, Checkbox, Radio button, Button and Badge
//! (recipes since spectrum-design-data #1700 to #1702 and #1704: the root has a `minHeight`, the
//! label `overflow: wrap`). What the oracle renders for `--width W`:
//!
//! - the label breaks greedily at spaces to the width the root leaves it;
//! - a row root pads each side by `(minHeight - one line) / 2` and grows by a line height per
//!   extra line, so one line gives exactly `minHeight`;
//! - the other parts (control, icon) are centred on the first line.
//!
//! Without a width nothing wraps and the root hugs its content, as before.

use rune::{text::measure, El, TextStyle};

pub(crate) struct Fit {
    /// The root's height: `minHeight` for one line, a line height more per extra line.
    pub height: f32,
    pub multi: bool,
    pad_y: f32,
    band: f32,
}

impl Fit {
    /// `avail` is the width the label gets (`None`: no request width, one line); `child_h` the
    /// tallest part beside the label (0 if none).
    pub fn new(
        label: Option<&str>,
        style: TextStyle,
        line_h: f32,
        child_h: f32,
        min_h: f32,
        avail: Option<f32>,
    ) -> Self {
        let band = line_h.max(child_h);
        let lines = match (label, avail) {
            (Some(l), Some(w)) => (measure(l, &[], style, Some(w.max(0.0))).1 / line_h)
                .round()
                .max(1.0),
            _ => 1.0,
        };
        let pad_y = (min_h - band) / 2.0;
        Fit {
            height: (2.0 * pad_y + band + (lines - 1.0) * line_h).max(min_h),
            multi: lines > 1.0,
            pad_y,
            band,
        }
    }

    /// Centred (one line) or start-aligned with the block padding (several lines).
    pub fn root(&self, root: El) -> El {
        if self.multi {
            root.items_start().py(self.pad_y)
        } else {
            root.center_y()
        }
    }

    /// A part of height `h` beside the label: centred on the first line when the label wraps.
    pub fn part(&self, e: El, h: f32) -> El {
        if self.multi {
            e.mt((self.band - h) / 2.0)
        } else {
            e
        }
    }
}
