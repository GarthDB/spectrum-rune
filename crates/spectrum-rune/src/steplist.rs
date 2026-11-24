//! Spectrum Steplist: steps side by side, each a label over a row of a circle and a track running on to the next step, from
//! `design-data` visuals/steplist.json. Every size, gap, colour and type value comes from [`Tokens`].
//!
//! Built: horizontal steplists of the four sizes (`S` to `XL`); a step is `Completed` (filled circle, filled track),
//! `Current` (dark circle, grey track) or `Incomplete` (outlined circle, grey track), and may be `disabled`; the last step
//! has no track; pressing or Enter on a step ([`Steplist::on_select`] with its index) and the arrow keys moving focus along
//! the row (Enter then selects). The app keeps the current step. Limits, mostly the oracle's: it sizes each step to its label (the track fills the rest of that width, so
//! the steplist is not as wide as its container); vertical orientation is not built; a completed step's hover colours (its
//! label and circle, in the oracle a `completed hover` state) are drawn only on the label, because Rune cannot tint the
//! circle from the step's hover; and the oracle draws no focus look, so none beyond Rune's focus ring. The current step is
//! announced (as a navigation region of steps) with ", current step" after its label (Rune has no `aria-current`).
//!
//! Verification: geometry, type and colours against the oracle (`tests/steplist.rs`); text widths follow the installed
//! font, so a step's width is not compared, only that the track runs from the circle to the step's right edge.

use crate::font;
use crate::tokens::{Tokens, Typography};
use crate::Size;
use rune::{col, el, row, text, El, Key, Mods};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepState {
    Completed,
    Current,
    #[default]
    Incomplete,
}

#[derive(Clone)]
pub struct Step {
    label: String,
    state: StepState,
    disabled: bool,
}

impl Step {
    pub fn new(label: impl Into<String>, state: StepState) -> Self {
        Step {
            label: label.into(),
            state,
            disabled: false,
        }
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
}

type Select = Rc<dyn Fn(El, usize) -> El>;

pub struct Steplist {
    key: String,
    steps: Vec<Step>,
    size: Size,
    label: String,
    select: Option<Select>,
}

impl Steplist {
    /// `key` must be unique per instance: the parts are keyed `{key}/container`, `{key}/step-N` (from 1) and
    /// `{key}/step-N.label`, `.row`, `.visual`, `.track`, as in the recipe.
    pub fn new(key: impl Into<String>) -> Self {
        Steplist {
            key: key.into(),
            steps: Vec::new(),
            size: Size::M,
            label: "Steps".to_owned(),
            select: None,
        }
    }
    pub fn step(mut self, step: Step) -> Self {
        self.steps.push(step);
        self
    }
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    /// The list's accessible name (`Steps` by default).
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Called with the step's index on press and on Enter. A disabled step does not call it. Without it the steps are not
    /// focusable.
    pub fn on_select<S: 'static>(mut self, f: impl Fn(&mut S, usize) + 'static) -> Self {
        let f = Rc::new(f);
        self.select = Some(Rc::new(move |e: El, i: usize| {
            let (a, b) = (f.clone(), f.clone());
            e.on_press(move |s: &mut S| a(s, i))
                .on_key(move |s: &mut S, ev, tx| {
                    let hit = matches!(ev.key, Key::Enter) && ev.mods == Mods::default();
                    if hit && !tx.key_repeat() {
                        b(s, i);
                    }
                    hit
                })
        }));
        self
    }

    pub fn build(self, t: &Tokens) -> El {
        let (visual, gap, font_size) = match self.size {
            Size::S => (
                t.steplist_visual_size_small,
                t.base_gap_small,
                t.font_size_75,
            ),
            Size::M => (
                t.steplist_visual_size_medium,
                t.base_gap_medium,
                t.font_size_100,
            ),
            Size::L => (
                t.steplist_visual_size_large,
                t.base_gap_large,
                t.font_size_200,
            ),
            Size::XL => (
                t.steplist_visual_size_extra_large,
                t.base_gap_extra_large,
                t.font_size_300,
            ),
        };
        let part = |name: &str| format!("{}/{name}", self.key);
        let style = font::text_style(&Typography {
            size: font_size,
            weight: t.regular_font_weight,
            line_height: font_size * t.line_height_100,
            ..t.component_m_regular
        });
        let mut list = row()
            .key(part("container"))
            .role("navigation")
            .label(self.label.clone())
            .focus_group();
        let last = self.steps.len().saturating_sub(1);
        for (i, st) in self.steps.iter().enumerate() {
            let key = part(&format!("step-{}", i + 1));
            let completed = st.state == StepState::Completed;
            let current = st.state == StepState::Current;
            let ink = if st.disabled {
                t.disabled_content_color
            } else if current {
                t.neutral_content_color_default
            } else {
                t.neutral_subdued_content_color_default
            };
            let mut label = text(st.label.clone())
                .key(format!("{key}.label"))
                .font(style)
                .color(ink)
                .a11y_hidden();
            if completed && !st.disabled {
                label = label.group_color(t.neutral_subdued_content_color_hover);
            }
            let mut circle = el()
                .key(format!("{key}.visual"))
                .w(visual)
                .h(visual)
                .radius(visual / 2.0);
            circle = match (st.state, st.disabled) {
                (_, true) if completed || current => circle.fill(t.disabled_background_color),
                (StepState::Completed, _) => {
                    circle.fill(t.neutral_subdued_background_color_default)
                }
                (StepState::Current, _) => circle.fill(t.neutral_background_color_default),
                _ => circle.border(t.border_width_200, t.neutral_visual_color),
            };
            let mut steps_row = row().key(format!("{key}.row")).center_y().child(circle);
            if i != last {
                steps_row = steps_row.child(
                    el().key(format!("{key}.track"))
                        .grow(1.0)
                        // shortcut: a label narrower than the circle (XL) leaves the oracle's track no width; one point keeps the box valid.
                        .min_w(1.0)
                        .h(t.bar_thickness_extra_small)
                        .fill(if completed {
                            t.neutral_subdued_background_color_default
                        } else {
                            t.neutral_visual_color
                        }),
                );
            }
            let name = if current {
                format!("{}, current step", st.label)
            } else {
                st.label.clone()
            };
            let mut step = col()
                .key(key)
                .gap(gap)
                .role("listitem")
                .label(name)
                .hover_group_focus()
                .child(label)
                .child(steps_row);
            if let Some(s) = &self.select {
                if !st.disabled {
                    step = s(step.focusable(true), i);
                }
            }
            list = list.child(step);
        }
        crate::hug(col().child(list))
    }
}
