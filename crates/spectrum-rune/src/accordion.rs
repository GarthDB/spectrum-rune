//! Spectrum Accordion item, from `design-data` visuals/accordion.json (item > header >
//! disclosure-icon + title, content, divider). Every size, colour, padding and type value comes
//! from [`Tokens`]. One item only: the group (`items`, `isMultiple`, arrow keys between headers)
//! is [`AccordionGroup`](crate::accordion_group::AccordionGroup).
//!
//! Where the oracle (`design-data render accordion`, see oracle.toml) looks odd, the item renders what it
//! renders and says so here:
//! - The title's type is one step up from the other components' labels (S uses `font-size-100`,
//!   XL `font-size-400`: the recipe's own `sizeScale` s100..xl400), so [`Size::label_type`] is not
//!   used; the weight is bold.
//! - The header's vertical padding exists only for `regular` and `spacious` density; `compact`
//!   has none. The title adds its own vertical padding, and the horizontal padding before the
//!   title is `base_padding_horizontal_small` at S and `_medium` from M up (the header's own
//!   grows with the size).
//! - Hover fills the whole `item` (header, content and divider), in the same translucent black in
//!   both colour schemes, and darkens the title and chevron. A pressed item looks like a rested
//!   one (`down` is not visual). The oracle also fills a *disabled* item on hover; this one does
//!   not react to the pointer at all when disabled.
//! - The content has only vertical padding, no horizontal: it starts at the item's left edge, left
//!   of the chevron and the title (the oracle's `content` is at the item's x, full width, and
//!   neither the recipe nor the catalog has a token for an inset). It is kept as rendered; an app
//!   that wants the content under the title pads it itself (a14.24).
//! - Keyboard focus takes the hover ink: the title and chevron go to `neutral_content_color_hover`
//!   (41 to 19 light, 219 to 242 dark) through `hover_group_focus` + `group_color` on the header's
//!   children (a14.36); the item fill does not change, as in the oracle, which also gives hover +
//!   keyboard-focus the hover wash and ink, as here. A press (held, not focused) draws the rest
//!   ink, as the oracle's `down`. Disabled is not focusable and keeps its colour. Nothing is
//!   left unmatched here.
//!
//! - `hasActionButton` and `hasSwitch` add the recipe's trailing `direct-actions` row: an icon-only
//!   action button (hold icon always on, the recipe fixes it) and a switch with its label hidden,
//!   both at the item's size and disabled with it, `spacing_100` apart, ending at the header's
//!   inline padding. The row is a *sibling* of the header button, not inside it (a control inside
//!   a `button` role is not reachable and its Enter would toggle the item), so the header's focus
//!   ring stops before the actions, while the oracle's `header` spans them. Hover is tracked on
//!   the wrapping row instead of the header, so the item's wash and title and chevron ink hold
//!   while the pointer is over the actions (which keep their own hover, press and focus looks).
//!   Press stays on the header alone (the oracle's `down` is not visual), and keyboard focus is
//!   the header's, unchanged: the oracle draws it on the header's ink only. Without either the tree is exactly the item's alone.
//!   The oracle draws the switch unselected and the button at rest, so those states of the two
//!   have no oracle here (their own tests cover them). The parts are keyed `{key}/direct-actions`,
//!   `{key}/action-button/...` and `{key}/switch/...`.
//!
//! The header is a `button` that reports its press through `on_toggle` and exposes `expanded`
//! (ARIA `aria-expanded`); the app owns `open`. Opening and closing spring the content's height
//! and turn the chevron on [`DISCLOSURE`]. Under Reduce Motion both land at once.

use crate::action_button::ActionButton;
use crate::constraints::{self, OptionValue};
use crate::font;
use crate::icons::{CHEVRON_100, CHEVRON_200, CHEVRON_300};
use crate::interaction::{Interaction, State};
use crate::size::Size;
use crate::switch::Switch;
use crate::tokens::{Tokens, Typography};
use rune::tok::motion::DEFAULT;
use rune::{col, el, icon, row, text, Color, El, Icon, Key, Mods, Spring};
use std::f32::consts::FRAC_PI_2;
use std::rc::Rc;

/// Spectrum has no motion tokens. This is the spring rune-kit's `Motion.disclosure` carries in
/// its neutral theme (`DEFAULT`, 0.3 s, no bounce): a height that overshoots would draw the
/// divider past the content, so the no-bounce spring is the right one for a disclosure.
pub const DISCLOSURE: Spring = DEFAULT;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Density {
    Compact,
    /// The catalog default.
    #[default]
    Regular,
    Spacious,
}

impl Density {
    pub const ALL: [Density; 3] = [Density::Compact, Density::Regular, Density::Spacious];

    /// The catalog's option value.
    pub fn as_str(self) -> &'static str {
        match self {
            Density::Compact => "compact",
            Density::Regular => "regular",
            Density::Spacious => "spacious",
        }
    }
}

struct Dims {
    /// Header's inline padding.
    pad: f32,
    /// Padding before the title, after the chevron.
    title_pad: f32,
    /// Title's block padding.
    title_pad_v: f32,
    chevron: f32,
    glyph: &'static Icon,
}

impl Size {
    fn accordion_dims(self, t: &Tokens) -> Dims {
        match self {
            Size::S => Dims {
                pad: t.base_padding_horizontal_small,
                title_pad: t.base_padding_horizontal_small,
                title_pad_v: t.base_padding_vertical_medium,
                chevron: t.chevron_size_75,
                glyph: &CHEVRON_100,
            },
            Size::M => Dims {
                pad: t.base_padding_horizontal_medium,
                title_pad: t.base_padding_horizontal_medium,
                title_pad_v: t.base_padding_vertical_large,
                chevron: t.chevron_size_100,
                glyph: &CHEVRON_100,
            },
            Size::L => Dims {
                pad: t.base_padding_horizontal_large,
                title_pad: t.base_padding_horizontal_medium,
                title_pad_v: t.base_padding_vertical_extra_large,
                chevron: t.chevron_size_200,
                glyph: &CHEVRON_200,
            },
            Size::XL => Dims {
                pad: t.base_padding_horizontal_extra_large,
                title_pad: t.base_padding_horizontal_medium,
                title_pad_v: t.base_padding_vertical_2x_large,
                chevron: t.chevron_size_300,
                glyph: &CHEVRON_300,
            },
        }
    }

    fn title_type(self, t: &Tokens) -> Typography {
        let (size, line_height) = match self {
            Size::S => (t.font_size_100, t.line_height_font_size_100),
            Size::M => (t.font_size_200, t.line_height_font_size_200),
            Size::L => (t.font_size_300, t.line_height_font_size_300),
            Size::XL => (t.font_size_400, t.line_height_font_size_400),
        };
        Typography {
            family: t.default_font_family,
            size,
            weight: t.bold_font_weight,
            line_height,
            letter_spacing_em: 0.0,
        }
    }
}

type Press = Box<dyn FnOnce(El) -> El>;
type ActionBuild = Box<dyn FnOnce(ActionButton) -> ActionButton>;
type SwitchBuild = Box<dyn FnOnce(Switch) -> Switch>;

pub struct AccordionItem {
    pub(crate) key: String,
    title: String,
    size: Size,
    density: Density,
    quiet: bool,
    pub(crate) disabled: bool,
    open: bool,
    content: Vec<El>,
    press: Option<Press>,
    action_button: Option<ActionBuild>,
    switch: Option<SwitchBuild>,
}

impl AccordionItem {
    /// `key` must be unique per instance: it keys the pointer state, and the parts are keyed
    /// `{key}/{recipe part}` (`item`, `header`, `disclosure-icon`, `title`, `content`, `divider`).
    /// `{key}/content-clip` is the clip the content's height springs in.
    pub fn new(key: impl Into<String>) -> Self {
        AccordionItem {
            key: key.into(),
            title: String::new(),
            size: Size::default(),
            density: Density::default(),
            quiet: false,
            disabled: false,
            open: false,
            content: Vec::new(),
            press: None,
            action_button: None,
            switch: None,
        }
    }
    /// The visible title and the header's accessible name.
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = t.into();
        self
    }
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    pub fn density(mut self, d: Density) -> Self {
        self.density = d;
        self
    }
    /// No divider under the item.
    pub fn quiet(mut self, v: bool) -> Self {
        self.quiet = v;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    /// Whether the content shows. The press only reports; flip this in `on_toggle`.
    pub fn open(mut self, v: bool) -> Self {
        self.open = v;
        self
    }
    /// Adds content below the header. Its height is the app's; the recipe adds the vertical padding
    /// only, so the content is flush with the item's left edge (not under the title). It is
    /// only built while open (and while closing), so a closed item has nothing to focus.
    pub fn content(mut self, c: El) -> Self {
        self.content.push(c);
        self
    }
    pub fn children(mut self, c: impl IntoIterator<Item = El>) -> Self {
        self.content.extend(c);
        self
    }
    /// Shows the trailing icon-only action button (`hasActionButton`). `f` receives a button the
    /// item has keyed (`{key}/action-button`), sized, disabled and given a hold icon; set its
    /// `icon`, `label` (the accessible name, required) and `on_press`. The size, hidden label, hold
    /// icon and disabled state are re-applied after `f`, since the recipe fixes them.
    pub fn action_button(mut self, f: impl FnOnce(ActionButton) -> ActionButton + 'static) -> Self {
        self.action_button = Some(Box::new(f));
        self
    }
    /// Shows the trailing switch (`hasSwitch`), keyed `{key}/switch`. Set its `label` (the
    /// accessible name; it is never drawn), `selected` and `on_toggle` in `f`.
    pub fn switch(mut self, f: impl FnOnce(Switch) -> Switch + 'static) -> Self {
        self.switch = Some(Box::new(f));
        self
    }
    /// Called on press, Space or Enter of the header; flip your own `open` state here.
    pub fn on_toggle<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        let f = Rc::new(f);
        // Rune presses a button with Space only (Return is the window's default button), but a
        // disclosure header also opens on Enter, as on the web.
        self.press = Some(Box::new(move |e| {
            let enter = f.clone();
            e.on_press(move |s: &mut S| f(s))
                .on_key(move |s: &mut S, ev, tx| {
                    let hit = ev.key == Key::Enter && ev.mods == Mods::default();
                    // Once per press, as Space does: a held Enter must not flip it again and again.
                    if hit && !tx.key_repeat() {
                        enter(s);
                    }
                    hit
                })
        }));
        self
    }

    /// The builder state under the catalog's option names (`items` and `isMultiple` belong to the
    /// group and are not visual).
    pub fn options(&self) -> Vec<(&'static str, OptionValue<'static>)> {
        vec![
            (
                "size",
                OptionValue::Str(match self.size {
                    Size::S => "s",
                    Size::M => "m",
                    Size::L => "l",
                    Size::XL => "xl",
                }),
            ),
            ("isQuiet", OptionValue::Bool(self.quiet)),
            ("isDisabled", OptionValue::Bool(self.disabled)),
            ("isOpen", OptionValue::Bool(self.open)),
            ("density", OptionValue::Str(self.density.as_str())),
            (
                "hasActionButton",
                OptionValue::Bool(self.action_button.is_some()),
            ),
            ("hasSwitch", OptionValue::Bool(self.switch.is_some())),
        ]
    }

    pub fn build(self, t: &Tokens) -> El {
        constraints::assert_valid("accordion", &self.options());
        let d = self.size.accordion_dims(t);
        let part = |name: &str| format!("{}/{name}", self.key);
        let interaction = Interaction::of(&self.key);
        let hover = !self.disabled && interaction.state() == State::Hover;
        // The hover colour while the header shows keyboard focus (`hover_group_focus`), the
        // oracle's keyboard-focus ink. Only at rest: the oracle draws a press like rest (`down` is
        // not visual), and a group colour would turn a held press to the hover colour.
        let focus_ink = (!self.disabled && interaction.state() == State::Rest)
            .then_some(t.neutral_content_color_hover);
        let tint = |e: El| match focus_ink {
            Some(c) => e.group_color(c),
            None => e,
        };

        let ink = if self.disabled {
            t.disabled_content_color
        } else if hover {
            t.neutral_content_color_hover
        } else {
            t.neutral_content_color_default
        };
        let header_pad_v = match self.density {
            Density::Compact => 0.0,
            Density::Regular => t.list_item_padding_vertical_regular,
            Density::Spacious => t.list_item_padding_vertical_spacious,
        };

        let direct = self.action_button.is_some() || self.switch.is_some();
        let mut header = row()
            .key(part("header"))
            .pl(d.pad)
            .pr(if direct { 0.0 } else { d.pad })
            .py(header_pad_v)
            .center_y()
            .role("button")
            .label(self.title.clone())
            .expanded(self.open)
            .focusable(!self.disabled)
            .disabled(self.disabled)
            // Not from the oracle (the recipe has no focus indicator): inset, so a full-width
            // header's ring is not clipped by its parent or drawn over the next item.
            .focus_ring(
                t.focus_indicator_thickness,
                -t.focus_indicator_thickness,
                t.focus_indicator_color,
            )
            .hover_group_focus()
            .child(tint(
                icon(d.glyph, d.chevron)
                    .key(part("disclosure-icon"))
                    .color(ink)
                    .a11y_hidden()
                    // The recipe's rotation is the literal 90 degrees when open.
                    .icon_rotation(if self.open { FRAC_PI_2 } else { 0.0 }),
            ))
            .child(tint(
                text(self.title.clone())
                    .key(part("title"))
                    .color(ink)
                    .a11y_hidden()
                    .font(font::text_style(&self.size.title_type(t)))
                    .wrap()
                    .grow(1.0)
                    .shrink()
                    .ml(d.title_pad)
                    .mr(d.title_pad)
                    .my(d.title_pad_v),
            ));
        if !self.disabled {
            // With actions the wrapping row owns hover, so it holds while the pointer is over
            // them; the header alone would drop it on leaving for a sibling.
            header = if direct {
                interaction.attach_press(header)
            } else {
                interaction.attach(header)
            };
            if let Some(p) = self.press {
                header = p(header);
            }
        }

        // The actions sit beside the header button, not in it; the row only exists with them.
        let top = if direct {
            let mut actions = row()
                .key(part("direct-actions"))
                .gap(t.spacing_100)
                .center_y();
            if let Some(f) = self.action_button {
                actions = actions.child(
                    f(ActionButton::new(part("action-button")))
                        .size(self.size)
                        .hide_label(true)
                        .hold_icon(true)
                        .disabled(self.disabled)
                        .build(t),
                );
            }
            if let Some(f) = self.switch {
                actions = actions.child(
                    f(Switch::new(part("switch")))
                        .size(self.size)
                        .hide_label(true)
                        .disabled(self.disabled)
                        .build(t),
                );
            }
            // The header's vertical padding also surrounds the actions, and the header stretches
            // to whichever is taller.
            let top = row()
                .items_stretch()
                .child(header.grow(1.0).shrink())
                .child(row().py(header_pad_v).pr(d.pad).center_y().child(actions));
            if self.disabled {
                top
            } else {
                interaction.attach_hover(top)
            }
        } else {
            header
        };

        // The content's height is app-supplied, so the clip cannot animate a fixed number: while
        // closed it is 0 high and holds nothing, while open it is as high as its content, and
        // the layout spring moves between the two.
        let mut clip = col()
            .key(part("content-clip"))
            .items_stretch()
            .clip_soft(0.0);
        clip = if self.open {
            clip.child(
                col()
                    .key(part("content"))
                    .role("region")
                    .label(self.title)
                    .items_stretch()
                    .pt(t.base_padding_vertical_large)
                    .pb(t.base_padding_vertical_2x_large)
                    .children(self.content),
            )
        } else {
            clip.h(0.0)
        };

        let mut item = col()
            .key(part("item"))
            .items_stretch()
            .fill(if hover {
                t.transparent_static_black_150
            } else {
                Color::TRANSPARENT
            })
            .token_motion(DISCLOSURE)
            .child(top)
            .child(clip);
        if !self.quiet {
            // The recipe writes the divider's height as the literal 1.
            item = item.child(el().key(part("divider")).h(1.0).fill(t.gray_200));
        }
        item
    }
}
