//! One story per component. Toggles read their state from [`G`], so they really toggle; every
//! option combination goes through `constraints::check` and an `error` one is skipped.

use crate::G;
use rune::{col, el, row, text, El};
use spectrum_rune::accordion::{AccordionItem, Density};
use spectrum_rune::accordion_group::AccordionGroup;
use spectrum_rune::action_button::ActionButton;
use spectrum_rune::badge::{self, Badge};
use spectrum_rune::breadcrumbs::{self, BreadcrumbItem, Breadcrumbs};
use spectrum_rune::button::{self, Button, StaticColor};
use spectrum_rune::checkbox::Checkbox;
use spectrum_rune::clear_button::ClearButton;
use spectrum_rune::constraints::{check, OptionValue, Severity};
use spectrum_rune::divider::{self, Divider};
use spectrum_rune::font;
use spectrum_rune::icons::{ADD_100, CHECKMARK_100};
use spectrum_rune::link::{self, Link};
use spectrum_rune::menu::{Menu, MenuItem, Selection};
use spectrum_rune::meter::{self, Meter};
use spectrum_rune::picker::Picker;
use spectrum_rune::picker_menu::PickerMenu;
use spectrum_rune::popover::{Placement, Popover};
use spectrum_rune::progress_bar::ProgressBar;
use spectrum_rune::progress_circle::ProgressCircle;
use spectrum_rune::radio_button::RadioButton;
use spectrum_rune::radio_group::{Orientation, RadioGroup};
use spectrum_rune::segmented_control::{SegmentedControl, SegmentedItem};
use spectrum_rune::side_navigation::{SideNavigation, SideNavigationItem};
use spectrum_rune::status_light::{self, StatusLight};
use spectrum_rune::steplist::{Step, StepState, Steplist};
use spectrum_rune::switch::Switch;
use spectrum_rune::tabs::{self, TabItem, Tabs};
use spectrum_rune::tag::Tag;
use spectrum_rune::tokens::Tokens;
use spectrum_rune::tooltip::{self, Tooltip};
use spectrum_rune::{ActionButtonSize, Size};

type Story = fn(&G, &'static Tokens) -> El;

macro_rules! stories {
    ($($id:literal => $title:literal, $f:path;)*) => {
        pub const STORIES: &[(&str, &str, Story)] = &[$(($id, $title, $f),)*];
        /// Per story: light desktop (or `--theme`), `-dark` and `-mobile`.
        pub const SCENES: &[(&str, fn(&mut G))] = &[$(
            ($id, |s: &mut G| s.story = $id),
            (concat!($id, "-dark"), |s: &mut G| {
                s.story = $id;
                s.scheme = Some(spectrum_rune::tokens::ColorScheme::Dark);
            }),
            (concat!($id, "-mobile"), |s: &mut G| {
                s.story = $id;
                s.scale = spectrum_rune::tokens::Scale::Mobile;
            }),
        )*
        // The Picker with its menu open (a popover under the field).
        ("picker-open", |s: &mut G| {
            s.story = "picker";
            s.flip("picker/value");
        }),
        // The Tooltip shown on its trigger.
        ("tooltip-shown", |s: &mut G| {
            s.story = "tooltip";
            s.flip("tooltip/live");
        }),
        ("picker-open-dark", |s: &mut G| {
            s.story = "picker";
            s.scheme = Some(spectrum_rune::tokens::ColorScheme::Dark);
            s.flip("picker/value");
        }),
        ];
    };
}

stories! {
    "switch" => "Switch", switch;
    "checkbox" => "Checkbox", checkbox;
    "radio-button" => "Radio button", radio_button;
    "radio-group" => "Radio group", radio_group;
    "button" => "Button", button;
    "action-button" => "Action button", action_button;
    "badge" => "Badge", badge;
    "picker" => "Picker", picker;
    "menu" => "Menu", menu;
    "popover" => "Popover", popover;
    "tooltip" => "Tooltip", tooltip;
    "breadcrumbs" => "Breadcrumbs", breadcrumbs;
    "progress" => "Progress and meter", progress;
    "divider" => "Divider", divider;
    "link" => "Link", link;
    "tag" => "Tag", tag;
    "clear-button" => "Clear button", clear_button;
    "side-navigation" => "Side navigation", side_navigation;
    "segmented-control" => "Segmented control", segmented_control;
    "steplist" => "Steplist", steplist;
    "tabs" => "Tabs", tabs;
    "status-light" => "Status light", status_light;
    "accordion" => "Accordion item", accordion;
    "accordion-group" => "Accordion", accordion_group;
}

/// Backdrops for the static colours. Fixed hex values: the `blue_*`/`yellow_*` tokens invert in the
/// dark scheme, and a static colour must stay on a backdrop it was designed for.
fn blue() -> rune::Color {
    rune::color::hex("#3b63fb").unwrap()
}
fn yellow() -> rune::Color {
    rune::color::hex("#ffcd00").unwrap()
}

const SIZES: [(Size, &str); 4] = [
    (Size::S, "S"),
    (Size::M, "M"),
    (Size::L, "L"),
    (Size::XL, "XL"),
];

/// Long enough to wrap at the stories' request width.
const WRAPPED: &str = "Wrapped label text goes here";

fn valid(component: &str, options: Vec<(&'static str, OptionValue<'static>)>) -> bool {
    !check(component, &options)
        .iter()
        .any(|v| v.severity == Severity::Error)
}

fn flow() -> El {
    row().w_full().gap(16.0).center_y().flex_wrap()
}

/// A row of components centred on a common line.
fn line(items: impl IntoIterator<Item = El>) -> El {
    row().gap(16.0).center_y().children(items)
}

fn section(t: &Tokens, title: &str, body: El) -> El {
    col()
        .gap(8.0)
        .child(
            text(title)
                .font(font::text_style(&t.component_s_medium))
                .color(t.neutral_subdued_content_color_default),
        )
        .child(body)
}

fn page(sections: Vec<El>) -> El {
    col().w_full().gap(24.0).children(sections)
}

fn backdrop(color: rune::Color, body: El) -> El {
    el().padding(16.0).fill(color).radius(8.0).child(body)
}

fn flip(key: String) -> impl Fn(&mut G) + 'static {
    move |s| s.flip(&key)
}

fn switch(s: &G, t: &'static Tokens) -> El {
    let sw = |name: &str, label: &str, on: bool| {
        let key = format!("switch/{name}");
        Switch::new(key.clone())
            .label(label)
            .selected(s.on(&key, on))
            .on_toggle(flip(key))
    };
    let states = flow()
        .child(sw("off", "Off", false).build(t))
        .child(sw("on", "On", true).build(t))
        .child(sw("emph", "Emphasized", true).emphasized(true).build(t))
        .child(
            sw("emph-off", "Emphasized, off", false)
                .emphasized(true)
                .build(t),
        )
        .child(sw("dis", "Disabled", false).disabled(true).build(t))
        .child(sw("dis-on", "Disabled, on", true).disabled(true).build(t))
        .child(sw("hidden", "Label hidden", true).hide_label(true).build(t));
    let sizes = line(SIZES.map(|(z, n)| sw(&format!("size-{n}"), n, true).size(z).build(t)));
    page(vec![
        section(t, "States", states),
        section(t, "Sizes", sizes),
        section(
            t,
            "Wrapped label (width 150)",
            flow().children(SIZES.map(|(z, n)| {
                Switch::new(format!("switch/wrap-{n}"))
                    .label(WRAPPED)
                    .size(z)
                    .width(150.0)
                    .build(t)
            })),
        ),
    ])
}

fn checkbox(s: &G, t: &'static Tokens) -> El {
    let cb = |name: &str, label: &str, on: bool| {
        let key = format!("checkbox/{name}");
        Checkbox::new(key.clone())
            .label(label)
            .selected(s.on(&key, on))
            .on_toggle(flip(key))
    };
    // Indeterminate until clicked, then it behaves as unselected.
    let mixed = |name: &str, label: &str| {
        let key = format!("checkbox/{name}");
        Checkbox::new(key.clone())
            .label(label)
            .indeterminate(s.on(&key, true))
            .on_toggle(flip(key))
    };
    let mut states = flow();
    let cases = [
        cb("off", "Unselected", false),
        cb("on", "Selected", true),
        mixed("mixed", "Indeterminate"),
        cb("emph", "Emphasized", true).emphasized(true),
        cb("emph-off", "Emphasized, off", false).emphasized(true),
        cb("err", "Error", false).error(true),
        cb("err-on", "Error, selected", true).error(true),
        cb("dis", "Disabled", false).disabled(true),
        cb("dis-on", "Disabled, selected", true).disabled(true),
    ];
    for c in cases {
        if valid("checkbox", c.options()) {
            states = states.child(c.build(t));
        }
    }
    let sizes = line(SIZES.map(|(z, n)| cb(&format!("size-{n}"), n, true).size(z).build(t)));
    page(vec![
        section(t, "States", states),
        section(t, "Sizes", sizes),
        section(
            t,
            "Wrapped label (width 150)",
            flow().children(SIZES.map(|(z, n)| {
                Checkbox::new(format!("checkbox/wrap-{n}"))
                    .label(WRAPPED)
                    .size(z)
                    .width(150.0)
                    .build(t)
            })),
        ),
    ])
}

fn popover(_: &G, t: &'static Tokens) -> El {
    let body = |key: &str, line: &str| {
        text(line.to_owned())
            .key(key.to_owned())
            .font(font::text_style(&t.component_m_regular))
            .color(t.neutral_content_color_default)
    };
    let one = |key: &str, p: Placement, hide: bool, w: f32, h: f32| {
        // A placement's tip points at the trigger: a little room around it shows the tip, which sits outside the surface.
        el().padding(t.spacing_200).child(
            Popover::new(format!("popover/{key}"))
                .placement(p)
                .hide_tip(hide)
                .width(w)
                .height(h)
                .child(body(&format!("popover/{key}/text"), "Popover content"))
                .build(t),
        )
    };
    let places = [
        ("top", Placement::Top),
        ("bottom", Placement::Bottom),
        ("left", Placement::Left),
        ("right", Placement::Right),
    ];
    page(vec![
        section(
            t,
            "Placements (the tip faces the trigger)",
            flow().children(places.map(|(n, p)| one(n, p, false, 140.0, 64.0))),
        ),
        section(
            t,
            "Without a tip, and sized",
            flow()
                .child(one("hide", Placement::Top, true, 140.0, 64.0))
                .child(one("small", Placement::Top, false, 90.0, 40.0))
                .child(one("large", Placement::Top, false, 220.0, 110.0)),
        ),
    ])
}

fn tag(s: &G, t: &'static Tokens) -> El {
    // Press a tag to toggle it: it is selected (or not) in this page's state.
    let tg = |key: &str, label: &str| {
        let k = key.to_owned();
        Tag::new(format!("tag/{key}"), label)
            .selected(s.on(key, false))
            .on_press(move |g: &mut G| g.flip(&k))
    };
    let sizes = flow().children(
        SIZES
            .iter()
            .take(3)
            .map(|(z, n)| tg(&format!("size-{n}"), n).size(*z).build(t)),
    );
    let looks = flow()
        .child(Tag::new("tag/plain", "Plain").build(t))
        .child(Tag::new("tag/selected", "Selected").selected(true).build(t))
        .child(
            Tag::new("tag/emphasized", "Emphasized")
                .selected(true)
                .emphasized(true)
                .build(t),
        )
        .child(Tag::new("tag/disabled", "Disabled").disabled(true).build(t));
    let live = flow().child(tg("live-neutral", "Press me").build(t)).child(
        tg("live-emphasized", "Emphasized")
            .emphasized(true)
            .build(t),
    );
    // The remove button: pressing it (or Delete or Backspace on a focused tag) flips the tag's `removed` toggle, which hides it.
    let removable = flow().children(["Design", "Rune", "Spectrum"].map(|n| {
        let key = format!("removed-{n}");
        let k = key.clone();
        if s.on(&key, false) {
            text(format!("{n} removed"))
                .font(font::text_style(&t.component_s_regular))
                .color(t.neutral_subdued_content_color_default)
        } else {
            Tag::new(format!("tag/{n}"), n)
                .on_remove(move |g: &mut G| g.flip(&k))
                .build(t)
        }
    }));
    page(vec![
        section(t, "Sizes (press to toggle)", sizes),
        section(
            t,
            "Removable (press the cross, or Delete when focused)",
            removable,
        ),
        section(t, "Looks (static)", looks),
        section(t, "Interactive: hover, press, Tab to focus", live),
    ])
}

fn steplist(s: &G, t: &'static Tokens) -> El {
    // The current step is the one last pressed (kept in `picked`); the one after the last is disabled.
    let list = |key: &str, size: Size| {
        let current = s
            .picked
            .get(key)
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1);
        let k = key.to_owned();
        let mut b = Steplist::new(format!("steplist/{key}"))
            .size(size)
            .on_select(move |g: &mut G, i| {
                g.picked.insert(k.clone(), i.to_string());
            });
        for (i, label) in ["Account", "Details", "Review", "Payment", "Done"]
            .into_iter()
            .enumerate()
        {
            let state = match i.cmp(&current) {
                std::cmp::Ordering::Less => StepState::Completed,
                std::cmp::Ordering::Equal => StepState::Current,
                std::cmp::Ordering::Greater => StepState::Incomplete,
            };
            b = b.step(Step::new(label, state).disabled(i == 3));
        }
        b.build(t)
    };
    page(vec![section(
        t,
        "Sizes (press a step; arrow keys move focus, Enter selects)",
        col()
            .gap(t.spacing_300)
            .children(SIZES.map(|(z, n)| list(&format!("size-{n}"), z))),
    )])
}

fn segmented_control(s: &G, t: &'static Tokens) -> El {
    use spectrum_rune::icons::{WORKFLOW_ADD, WORKFLOW_EDIT, WORKFLOW_SEARCH, WORKFLOW_SETTINGS};
    // The selected item is the one last pressed (kept in `picked`); the third is disabled.
    let bar = |key: &str, size: Size| {
        let current = s
            .picked
            .get(key)
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);
        let k = key.to_owned();
        let items = [
            ("Search", &WORKFLOW_SEARCH, false),
            ("Edit", &WORKFLOW_EDIT, false),
            ("Add", &WORKFLOW_ADD, true),
            ("Settings", &WORKFLOW_SETTINGS, false),
        ];
        let mut b = SegmentedControl::new(format!("segmented-control/{key}"))
            .size(size)
            .on_select(move |g: &mut G, i| {
                g.picked.insert(k.clone(), i.to_string());
            });
        for (i, (label, ic, disabled)) in items.into_iter().enumerate() {
            b = b.item(
                SegmentedItem::new(label)
                    .icon(ic)
                    .selected(i == current)
                    .disabled(disabled),
            );
        }
        b.build(t)
    };
    page(vec![section(
        t,
        "Sizes (press an item; arrow keys move and select)",
        col()
            .gap(t.spacing_200)
            .children(SIZES.map(|(z, n)| bar(&format!("size-{n}"), z))),
    )])
}

fn tabs(s: &G, t: &'static Tokens) -> El {
    use spectrum_rune::icons::{WORKFLOW_ADD, WORKFLOW_EDIT, WORKFLOW_SEARCH, WORKFLOW_SETTINGS};
    // The selected tab is the one last pressed (kept in `picked`); the third is disabled.
    let bar = |key: &str, size: Size, density: tabs::Density| {
        let current = s
            .picked
            .get(key)
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);
        let k = key.to_owned();
        let items = [
            ("Search", &WORKFLOW_SEARCH, false),
            ("Edit", &WORKFLOW_EDIT, false),
            ("Add", &WORKFLOW_ADD, true),
            ("Settings", &WORKFLOW_SETTINGS, false),
        ];
        let mut b = Tabs::new(format!("tabs/{key}"))
            .size(size)
            .density(density)
            .on_select(move |g: &mut G, i| {
                g.picked.insert(k.clone(), i.to_string());
            });
        for (i, (label, ic, disabled)) in items.into_iter().enumerate() {
            b = b.item(
                TabItem::new(label)
                    .icon(ic)
                    .selected(i == current)
                    .disabled(disabled),
            );
        }
        b.build(t)
    };
    page(vec![
        section(
            t,
            "Sizes (press a tab; arrow keys move and select)",
            col()
                .gap(t.spacing_200)
                .children(SIZES.map(|(z, n)| bar(&format!("size-{n}"), z, tabs::Density::Regular))),
        ),
        section(
            t,
            "Compact",
            flow().child(bar("compact", Size::M, tabs::Density::Compact)),
        ),
    ])
}

fn side_navigation(s: &G, t: &'static Tokens) -> El {
    use spectrum_rune::icons::{WORKFLOW_ADD, WORKFLOW_EDIT, WORKFLOW_SEARCH, WORKFLOW_SETTINGS};
    // The selected item is the one last pressed (kept in `picked`); the second is disabled.
    let nav = |key: &str, title: Option<&str>| {
        let current = s
            .picked
            .get(key)
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);
        let k = key.to_owned();
        let items = [
            ("Search", &WORKFLOW_SEARCH, false),
            ("Reports", &WORKFLOW_EDIT, true),
            ("Create", &WORKFLOW_ADD, false),
            ("Settings", &WORKFLOW_SETTINGS, false),
        ];
        let mut n =
            SideNavigation::new(format!("side-navigation/{key}")).on_select(move |g: &mut G, i| {
                g.picked.insert(k.clone(), i.to_string());
            });
        if let Some(title) = title {
            n = n.title(title);
        }
        for (i, (label, ic, disabled)) in items.into_iter().enumerate() {
            n = n.item(
                SideNavigationItem::new(label)
                    .icon(ic)
                    .selected(i == current)
                    .disabled(disabled),
            );
        }
        n.build(t)
    };
    page(vec![
        section(
            t,
            "With a section title (press an item; arrows move focus)",
            flow().child(nav("with-title", Some("Workspace"))),
        ),
        section(t, "Without", flow().child(nav("plain", None))),
    ])
}

fn clear_button(s: &G, t: &'static Tokens) -> El {
    let cb = |key: &str| {
        let k = key.to_owned();
        ClearButton::new(format!("clear-button/{key}")).on_press(move |g: &mut G| g.flip(&k))
    };
    let sizes = flow().children(SIZES.map(|(z, n)| cb(&format!("size-{n}")).size(z).build(t)));
    let states = flow().child(cb("rest").build(t)).child(
        ClearButton::new("clear-button/disabled")
            .disabled(true)
            .build(t),
    );
    // Each press flips the button's own toggle, so the count shows that pressing works.
    let pressed = s
        .flipped
        .iter()
        .filter(|k| k.starts_with("clear-button/"))
        .count();
    page(vec![
        section(t, "Sizes (press, Tab to focus)", sizes),
        section(t, "States", states),
        section(
            t,
            "Pressed",
            flow().child(
                text(format!("{pressed} button(s) toggled"))
                    .font(font::text_style(&t.component_m_regular))
                    .color(t.neutral_content_color_default),
            ),
        ),
    ])
}

fn divider(_: &G, t: &'static Tokens) -> El {
    let h = |key: &str, z: Size| {
        col()
            .w(240.0)
            .gap(t.spacing_75)
            .child(
                text(format!("size {key}"))
                    .font(font::text_style(&t.component_s_regular))
                    .color(t.neutral_subdued_content_color_default),
            )
            .child(Divider::new(format!("divider/h-{key}")).size(z).build(t))
    };
    let v = |key: &str, z: Size| {
        row()
            .h(48.0)
            .gap(t.spacing_200)
            .center_y()
            .child(
                text(key.to_owned())
                    .font(font::text_style(&t.component_s_regular))
                    .color(t.neutral_content_color_default),
            )
            .child(
                Divider::new(format!("divider/v-{key}"))
                    .size(z)
                    .orientation(divider::Orientation::Vertical)
                    .build(t),
            )
            .child(
                text("after".to_owned())
                    .font(font::text_style(&t.component_s_regular))
                    .color(t.neutral_content_color_default),
            )
    };
    page(vec![
        section(
            t,
            "Horizontal (fills its parent)",
            flow().children(SIZES.iter().take(3).map(|(z, n)| h(n, *z))),
        ),
        section(
            t,
            "Vertical",
            flow().children(SIZES.iter().take(3).map(|(z, n)| v(n, *z))),
        ),
    ])
}

fn link(s: &G, t: &'static Tokens) -> El {
    let l = |key: &str, label: &str| {
        let k = key.to_owned();
        Link::new(format!("link/{key}"), label).on_press(move |g: &mut G| g.flip(&k))
    };
    let pressed = |key: &str| s.on(key, false);
    let _ = pressed;
    let variants = flow().child(l("primary", "Primary link").build(t)).child(
        l("secondary", "Secondary link")
            .variant(link::Variant::Secondary)
            .build(t),
    );
    let quiet = flow()
        .child(
            l("quiet", "Quiet (underlined on hover)")
                .quiet(true)
                .build(t),
        )
        .child(
            l("quiet-secondary", "Quiet secondary")
                .quiet(true)
                .variant(link::Variant::Secondary)
                .build(t),
        );
    // The inline form sits in a sentence: regular weight.
    let inline = row()
        .gap(t.spacing_75)
        .center_y()
        .flex_wrap()
        .child(
            text("Read the".to_owned())
                .font(font::text_style(&t.component_m_regular))
                .color(t.neutral_content_color_default),
        )
        .child(l("inline", "documentation").standalone(false).build(t))
        .child(
            text("first.".to_owned())
                .font(font::text_style(&t.component_m_regular))
                .color(t.neutral_content_color_default),
        );
    page(vec![
        section(t, "Variants (press or Enter)", variants),
        section(t, "Quiet", quiet),
        section(t, "Inline (not standalone)", flow().child(inline)),
    ])
}

fn progress(_: &G, t: &'static Tokens) -> El {
    let bar = |key: &str, z: Size, v: f32| {
        ProgressBar::new(format!("progress/{key}"))
            .size(z)
            .label("Loading")
            .value(v)
            .width(220.0)
            .build(t)
    };
    let sizes = flow().children(SIZES.map(|(z, n)| bar(&format!("size-{n}"), z, 40.0)));
    let values =
        flow().children([0.0, 25.0, 60.0, 100.0].map(|v| bar(&format!("v{v}"), Size::M, v)));
    let other = flow()
        .child(
            ProgressBar::new("progress/indeterminate")
                .label("Working")
                .indeterminate(true)
                .width(220.0)
                .build(t),
        )
        .child(
            ProgressBar::new("progress/value-label")
                .label("Uploading")
                .value(40.0)
                .value_label("2 of 5")
                .width(220.0)
                .build(t),
        )
        .child(
            ProgressBar::new("progress/hidden")
                .label("Hidden label")
                .hide_label(true)
                .value(40.0)
                .width(220.0)
                .build(t),
        );
    let variants = flow().children(
        [
            ("informative", meter::Variant::Informative),
            ("positive", meter::Variant::Positive),
            ("notice", meter::Variant::Notice),
            ("negative", meter::Variant::Negative),
        ]
        .map(|(n, v)| {
            Meter::new(format!("progress/meter-{n}"))
                .label(n)
                .variant(v)
                .value(70.0)
                .help_text("Help text")
                .width(220.0)
                .build(t)
        }),
    );
    let circles = flow()
        .children([Size::S, Size::M, Size::L].map(|z| {
            ProgressCircle::new(format!("progress/circle-{z:?}"))
                .size(z)
                .label("Loading")
                .value(65.0)
                .build(t)
        }))
        .children([0.0, 25.0, 50.0, 100.0].map(|v| {
            ProgressCircle::new(format!("progress/circle-v{v}"))
                .label("Loading")
                .value(v)
                .build(t)
        }))
        .child(
            ProgressCircle::new("progress/circle-indeterminate")
                .label("Working")
                .indeterminate(true)
                .build(t),
        );
    page(vec![
        section(t, "Progress bar sizes (40%)", sizes),
        section(t, "Progress bar values", values),
        section(t, "Indeterminate, value label, hidden label", other),
        section(t, "Meter variants", variants),
        section(t, "Progress circle: sizes, values, indeterminate", circles),
    ])
}

fn breadcrumbs(s: &G, t: &'static Tokens) -> El {
    use breadcrumbs::{Separator as Sep, Size as Z};
    // The trail ends at the page chosen by pressing a link: the links after it drop away.
    let trail = |key: &str, size: Z, sep: Sep| {
        let names = ["Home", "Docs", "Components", "Breadcrumbs"];
        let current = s
            .picked
            .get(key)
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(2)
            .min(names.len() - 1);
        let k = key.to_owned();
        let mut b =
            Breadcrumbs::new(key)
                .size(size)
                .separator(sep)
                .on_navigate(move |g: &mut G, i| {
                    g.picked.insert(k.clone(), i.to_string());
                });
        for (i, n) in names.iter().enumerate().take(current + 1) {
            b = b.item(BreadcrumbItem::new(*n).current(i == current));
        }
        b.build(t)
    };
    page(vec![
        section(
            t,
            "Sizes (press a link: the trail ends there)",
            flow()
                .child(trail("breadcrumbs/m", Z::M, Sep::Chevron))
                .child(trail("breadcrumbs/l", Z::L, Sep::Chevron)),
        ),
        section(
            t,
            "No separator",
            flow().child(trail("breadcrumbs/none", Z::M, Sep::None)),
        ),
        section(
            t,
            "States: current, disabled",
            flow().child(
                Breadcrumbs::new("breadcrumbs/states")
                    .item(BreadcrumbItem::new("Home"))
                    .item(BreadcrumbItem::new("Archive").disabled(true))
                    .item(BreadcrumbItem::new("Report").current(true))
                    .build(t),
            ),
        ),
    ])
}

fn tooltip(s: &G, t: &'static Tokens) -> El {
    use tooltip::{Placement as P, Variant as V};
    let one = |key: &str, v: V, p: P| {
        // The tip sits outside the surface: a little room around each shows it.
        el().padding(t.spacing_200).child(
            Tooltip::new(format!("tooltip/{key}"), "Label")
                .variant(v)
                .placement(p)
                .build(t),
        )
    };
    let variants = flow().children([
        one("neutral", V::Neutral, P::Top),
        one("informative", V::Informative, P::Top),
        one("negative", V::Negative, P::Top),
    ]);
    let placements = flow().children([
        one("top", V::Neutral, P::Top),
        one("bottom", V::Neutral, P::Bottom),
        one("left", V::Neutral, P::Left),
        one("right", V::Neutral, P::Right),
    ]);
    let wrapped = el().padding(t.spacing_200).child(
        Tooltip::new(
            "tooltip/wrapped",
            "A longer label wraps at the maximum width of 160",
        )
        .build(t),
    );
    // The pointer resting on the button for 1.5 s shows its tooltip (mounted at the root, see `overlays`).
    let live = Tooltip::bind(
        el().key("tooltip/trigger")
            .child(ActionButton::new("tooltip/live").label("Hover me").build(t)),
        |g: &mut G, on| {
            if g.on("tooltip/live", false) != on {
                g.flip("tooltip/live");
            }
        },
    );
    let _ = s;
    page(vec![
        section(t, "Variants", variants),
        section(t, "Placements (the tip faces the trigger)", placements),
        section(t, "Wrapped at the maximum width", wrapped),
        section(
            t,
            "On a trigger (rest the pointer on it)",
            flow().child(live),
        ),
    ])
}

fn menu(s: &G, t: &'static Tokens) -> El {
    // The chosen item per menu is kept in `picked`; the first is chosen until another is pressed.
    let items = |id: &str, selection: Selection, size: Size| {
        let key = format!("menu/{id}");
        let chosen = s.picked.get(&key).map_or("copy", String::as_str).to_owned();
        let k = key.clone();
        let item = |v: &str, label: &str| {
            MenuItem::new(v, label).selected(selection == Selection::Single && chosen == v)
        };
        // A menu has no surface of its own (the recipe's container is not visual): a layer behind it.
        el().fill(t.background_layer_2_color)
            .padding(t.spacing_100)
            .radius(t.corner_radius_500)
            .child(
                Menu::new(key)
                    .label("Edit")
                    .size(size)
                    .selection(selection)
                    .item(item("copy", "Copy").value("⌘C"))
                    .item(item("paste", "Paste").value("⌘V"))
                    .item(item("cut", "Cut"))
                    .item(MenuItem::new("delete", "Delete").disabled(true))
                    .on_choose(move |s: &mut G, v: &str| {
                        s.picked.insert(k.clone(), v.to_owned());
                    })
                    .build(t),
            )
    };
    page(vec![
        section(
            t,
            "Plain, with values and a disabled item",
            flow().child(items("plain", Selection::None, Size::M)),
        ),
        section(
            t,
            "Single selection (click to choose)",
            flow().child(items("single", Selection::Single, Size::M)),
        ),
        section(
            t,
            "Sizes",
            flow().children(SIZES.map(|(z, n)| items(&format!("size-{n}"), Selection::Single, z))),
        ),
    ])
}

fn radio_group(s: &G, t: &'static Tokens) -> El {
    // The chosen value per group is kept in `picked`; nothing is chosen until one is pressed.
    let group = |id: &str, size: Size, orientation: Orientation| {
        let key = format!("radio-group/{id}");
        let k = key.clone();
        let mut g = RadioGroup::new(key.clone())
            .label("Fruit")
            .item("apple", "Apple")
            .item("pear", "Pear")
            .item("plum", "Plum")
            .size(size)
            .orientation(orientation)
            .on_change(move |s: &mut G, v: &str| {
                s.picked.insert(k.clone(), v.to_owned());
            });
        if let Some(v) = s.picked.get(&key) {
            g = g.value(v.clone());
        }
        g.build(t)
    };
    page(vec![
        section(
            t,
            "Vertical (no gap: the radio buttons' height spaces them)",
            flow().children(SIZES.map(|(z, n)| group(&format!("v-{n}"), z, Orientation::Vertical))),
        ),
        section(
            t,
            "Horizontal (spacing-300 between)",
            col()
                .gap(t.spacing_200)
                .children(SIZES.map(|(z, n)| group(&format!("h-{n}"), z, Orientation::Horizontal))),
        ),
    ])
}

fn radio_button(s: &G, t: &'static Tokens) -> El {
    // One radio per group member; the first is chosen until another is picked.
    let radio = |group: &str, id: &str, label: &str| {
        let group_key = format!("radio-button/{group}");
        let chosen = s.picked.get(&group_key).map_or("a", String::as_str) == id;
        let (g, id) = (group_key.clone(), id.to_owned());
        RadioButton::new(format!("{group_key}/{id}"))
            .label(label)
            .selected(chosen)
            .on_select(move |s: &mut G| {
                s.picked.insert(g.clone(), id.clone());
            })
    };
    let states = flow()
        .child(radio("basic", "a", "Selected first").build(t))
        .child(radio("basic", "b", "Unselected").build(t))
        .child(radio("emph", "a", "Emphasized").emphasized(true).build(t))
        .child(
            radio("emph", "b", "Emphasized, unselected")
                .emphasized(true)
                .build(t),
        )
        .child(radio("dis", "a", "Disabled").disabled(true).build(t))
        .child(
            radio("dis", "b", "Disabled, unselected")
                .disabled(true)
                .build(t),
        );
    let sizes = line(SIZES.map(|(z, n)| radio(&format!("size-{n}"), "a", n).size(z).build(t)));
    page(vec![
        section(t, "States (click to pick within a pair)", states),
        section(t, "Sizes", sizes),
        section(
            t,
            "Wrapped label (width 150)",
            flow().children(SIZES.map(|(z, n)| {
                RadioButton::new(format!("radio-button/wrap-{n}"))
                    .label(WRAPPED)
                    .size(z)
                    .width(150.0)
                    .build(t)
            })),
        ),
    ])
}

fn button(_: &G, t: &'static Tokens) -> El {
    use button::{Style, Variant};
    let variants = [
        (Variant::Accent, "Accent"),
        (Variant::Negative, "Negative"),
        (Variant::Primary, "Primary"),
        (Variant::Secondary, "Secondary"),
    ];
    let styled = |tag: &str, style: Style, tweak: &dyn Fn(Button) -> Button| {
        let mut r = flow();
        for (v, n) in variants {
            let b = tweak(
                Button::new(format!("button/{tag}/{v:?}-{style:?}"))
                    .label(n)
                    .variant(v)
                    .style(style),
            );
            if valid("button", b.options()) {
                r = r.child(b.build(t));
            }
        }
        r
    };
    let none = |b: Button| b;
    let mut sections = vec![
        section(t, "Fill", styled("fill", Style::Fill, &none)),
        section(
            t,
            "Outline (accent and negative have none)",
            styled("outline", Style::Outline, &none),
        ),
        section(
            t,
            "With icon",
            styled("icon", Style::Fill, &|b| b.icon(&ADD_100)),
        ),
        section(
            t,
            "Icon only",
            styled("icon-only", Style::Fill, &|b| {
                b.icon(&ADD_100).label_hidden(true)
            }),
        ),
        section(
            t,
            "Pending",
            styled("pending", Style::Fill, &|b| b.pending(true)),
        ),
        section(
            t,
            "Disabled",
            styled("disabled", Style::Fill, &|b| b.disabled(true)),
        ),
    ];
    let statics = |c: StaticColor| {
        let mut r = flow();
        for style in [Style::Fill, Style::Outline] {
            for (v, n) in [
                (Variant::Primary, "Primary"),
                (Variant::Secondary, "Secondary"),
            ] {
                let b = Button::new(format!("button/static-{c:?}-{style:?}-{n}"))
                    .label(n)
                    .variant(v)
                    .style(style)
                    .static_color(c);
                if valid("button", b.options()) {
                    r = r.child(b.build(t));
                }
            }
        }
        r
    };
    sections.push(section(
        t,
        "Static white",
        backdrop(blue(), statics(StaticColor::White)),
    ));
    sections.push(section(
        t,
        "Static black",
        backdrop(yellow(), statics(StaticColor::Black)),
    ));
    let sizes = line(SIZES.map(|(z, n)| {
        Button::new(format!("button/size-{n}"))
            .label(n)
            .size(z)
            .build(t)
    }));
    sections.push(section(t, "Sizes", sizes));
    sections.push(section(
        t,
        "Wrapped label (width 150; min width 2.25 x height)",
        flow().children(SIZES.map(|(z, n)| {
            Button::new(format!("button/wrap-{n}"))
                .label(WRAPPED)
                .icon(&ADD_100)
                .size(z)
                .width(150.0)
                .build(t)
        })),
    ));
    page(sections)
}

fn action_button(s: &G, t: &'static Tokens) -> El {
    let ab =
        |name: &str, label: &str| ActionButton::new(format!("action-button/{name}")).label(label);
    // A toggle: the app flips `selected`.
    let tog = |name: &str, label: &str, on: bool| {
        let key = format!("action-button/{name}");
        ActionButton::new(key.clone())
            .label(label)
            .selected(s.on(&key, on))
            .on_press(flip(key))
    };
    let cases = vec![
        ab("default", "Default"),
        ab("quiet", "Quiet").quiet(true),
        tog("selected", "Selected", true),
        tog("emph", "Emphasized", true).emphasized(true),
        ab("dis", "Disabled").disabled(true),
        ab("pending", "Pending").pending(true),
        ab("hold", "Hold icon").hold_icon(true),
        ab("icon", "Add").icon(&ADD_100),
        ab("icon-only", "Add").icon(&ADD_100).hide_label(true),
        ab("icon-quiet", "Add")
            .icon(&ADD_100)
            .hide_label(true)
            .quiet(true),
    ];
    let mut states = flow();
    for c in cases {
        if valid("action-button", c.options()) {
            states = states.child(c.build(t));
        }
    }
    let statics = |c: StaticColor, bg| {
        let mut r = flow();
        for b in [
            ab(&format!("s-default-{c:?}"), "Static").static_color(c),
            tog(&format!("s-selected-{c:?}"), "Selected", true).static_color(c),
            ab(&format!("s-icon-{c:?}"), "Add")
                .icon(&ADD_100)
                .hide_label(true)
                .static_color(c),
            ab(&format!("s-dis-{c:?}"), "Disabled")
                .disabled(true)
                .static_color(c),
        ] {
            if valid("action-button", b.options()) {
                r = r.child(b.build(t));
            }
        }
        backdrop(bg, r)
    };
    let sizes = line(
        [
            ActionButtonSize::XS,
            ActionButtonSize::S,
            ActionButtonSize::M,
            ActionButtonSize::L,
            ActionButtonSize::XL,
        ]
        .map(|z| {
            ab(&format!("size-{z:?}"), &format!("{z:?}"))
                .size(z)
                .build(t)
        }),
    );
    let icons = line(
        [
            ActionButtonSize::XS,
            ActionButtonSize::S,
            ActionButtonSize::M,
            ActionButtonSize::L,
            ActionButtonSize::XL,
        ]
        .map(|z| {
            ab(&format!("isize-{z:?}"), "Add")
                .icon(&ADD_100)
                .hide_label(true)
                .size(z)
                .build(t)
        }),
    );
    page(vec![
        section(t, "States", states),
        section(
            t,
            "Static white (Disabled stays an opaque grey pill: the oracle ignores the static colour when disabled)",
            statics(StaticColor::White, blue()),
        ),
        section(
            t,
            "Static black (same: the oracle draws Disabled like a plain disabled button)",
            statics(StaticColor::Black, yellow()),
        ),
        section(t, "Sizes", sizes),
        section(t, "Icon-only sizes", icons),
    ])
}

fn badge(_: &G, t: &'static Tokens) -> El {
    let base = |name: &str, v: badge::Variant, st: badge::Style| {
        Badge::new(format!("badge/{name}-{}-{}", v.as_str(), st.as_str()))
            .label(v.as_str())
            .variant(v)
            .style(st)
    };
    let mut sections = Vec::new();
    for st in badge::Style::ALL {
        let mut r = flow().gap(8.0);
        for v in badge::Variant::ALL {
            let b = base("v", *v, *st);
            if valid("badge", b.options()) {
                r = r.child(b.build(t));
            }
        }
        let note = if *st == badge::Style::Outline {
            " (only six variants have an outline; the rest render bold, adobe/spectrum-design-data#1693)"
        } else {
            ""
        };
        sections.push(section(t, &format!("{}{note}", st.as_str()), r));
    }
    let icon = flow().children(badge::Variant::ALL.iter().take(6).map(|v| {
        Badge::new(format!("badge/icon-{}", v.as_str()))
            .label(v.as_str())
            .variant(*v)
            .icon(&CHECKMARK_100)
            .build(t)
    }));
    let sizes = line(SIZES.map(|(z, n)| {
        Badge::new(format!("badge/size-{n}"))
            .label(n)
            .variant(badge::Variant::Accent)
            .size(z)
            .build(t)
    }));
    sections.push(section(t, "With icon", icon));
    sections.push(section(t, "Sizes", sizes));
    sections.push(section(
        t,
        "Wrapped label (width 150)",
        flow().children(SIZES.map(|(z, n)| {
            Badge::new(format!("badge/wrap-{n}"))
                .label(WRAPPED)
                .variant(badge::Variant::Informative)
                .size(z)
                .width(150.0)
                .build(t)
        })),
    ));
    page(sections)
}

/// The closed field only: pressing, Space, Enter or Arrow Down flips `expanded` (there is no menu).
const COUNTRIES: [(&str, &str); 4] = [
    ("ca", "Canada"),
    ("mx", "Mexico"),
    ("us", "United States"),
    ("jp", "Japan"),
];

/// The country a picker shows: the one chosen from its menu, else `default`.
fn country<'a>(s: &'a G, key: &str, default: Option<&'a str>) -> Option<&'a str> {
    s.picked
        .get(key)
        .map(|id| {
            COUNTRIES
                .iter()
                .find(|(i, _)| i == id)
                .map_or("", |(_, n)| *n)
        })
        .or(default)
}

fn picker(s: &G, t: &'static Tokens) -> El {
    let pk = |name: &str, default: Option<&str>| {
        let key = format!("picker/{name}");
        let p = Picker::new(key.clone())
            .label("Country")
            .placeholder("Select")
            .open(s.on(&key, false))
            .on_open(flip(key.clone()));
        match country(s, &key, default) {
            Some(v) => p.value(v),
            None => p,
        }
    };
    let states = flow()
        .child(pk("placeholder", None).build(t))
        .child(pk("value", Some("Canada")).build(t))
        .child(pk("quiet", Some("Canada")).quiet(true).build(t))
        .child(pk("error", Some("Canada")).error(true).build(t))
        .child(pk("dis", Some("Canada")).disabled(true).build(t))
        .child(pk("dis-ph", None).disabled(true).build(t));
    let sizes =
        flow().children(SIZES.map(|(z, n)| pk(&format!("size-{n}"), Some(n)).size(z).build(t)));
    let truncated = pk(
        "truncated",
        Some("A value too long for the field is cut off at the end"),
    )
    .width(160.0)
    .build(t);
    page(vec![
        section(t, "States (press one: its menu opens)", states),
        section(t, "Sizes", sizes),
        section(t, "Truncated value (width 160)", truncated),
    ])
}

/// The menus of the open pickers, to mount at the root of the view (they draw above the page and escape its clipping).
pub fn overlays(s: &G, t: &'static Tokens) -> Vec<El> {
    if s.story == "tooltip" && s.on("tooltip/live", false) {
        return vec![Tooltip::new("tooltip/shown", "Hover tooltip")
            .anchor("tooltip/trigger")
            .build(t)];
    }
    if s.story != "picker" {
        return Vec::new();
    }
    let mut out = Vec::new();
    let sized =
        |n: &str| -> Option<Size> { SIZES.iter().find(|(_, name)| *name == n).map(|(z, _)| *z) };
    for name in [
        "placeholder",
        "value",
        "quiet",
        "error",
        "size-s",
        "size-m",
        "size-l",
        "size-xl",
        "truncated",
    ] {
        let key = format!("picker/{name}");
        if !s.on(&key, false) {
            continue;
        }
        let chosen = COUNTRIES
            .iter()
            .find(|(_, n)| {
                Some(*n)
                    == country(
                        s,
                        &key,
                        if name == "placeholder" {
                            None
                        } else {
                            Some("Canada")
                        },
                    )
            })
            .map(|(i, _)| *i);
        let (k1, k2) = (key.clone(), key.clone());
        let mut menu = PickerMenu::new(key.clone())
            .label("Country")
            .size(
                name.strip_prefix("size-")
                    .and_then(sized)
                    .unwrap_or(Size::M),
            )
            .width(if name == "truncated" { 160.0 } else { 240.0 })
            .on_choose(move |g: &mut G, id: &str| {
                g.picked.insert(k1.clone(), id.to_owned());
                g.flip(&k1);
            })
            .on_close(move |g: &mut G| g.flip(&k2));
        for (id, label) in COUNTRIES {
            menu = menu.item(id, label);
        }
        if let Some(id) = chosen {
            menu = menu.selected(id);
        }
        out.push(menu.build(t));
    }
    out
}

fn status_light(_: &G, t: &'static Tokens) -> El {
    // The recipe's `Semantic (Dot)` variants; every other one is a named colour (`Non-semantic (Dot)`).
    use status_light::Variant as V;
    let semantic = |v: &V| {
        matches!(
            v,
            V::Informative | V::Accent | V::Neutral | V::Positive | V::Notice | V::Negative
        )
    };
    let variants = |want: bool| {
        flow().children(
            status_light::Variant::ALL
                .iter()
                .filter(|v| semantic(v) == want)
                .map(|v| {
                    StatusLight::new(format!("status-light/v-{}", v.as_str()), v.as_str())
                        .variant(*v)
                        .build(t)
                }),
        )
    };
    let sizes = line(SIZES.map(|(z, n)| {
        StatusLight::new(format!("status-light/size-{n}"), n)
            .variant(status_light::Variant::Positive)
            .size(z)
            .build(t)
    }));
    // The dot stays on the first line; React Spectrum's `LongLabel` story is this text at 128px.
    let wrapped = flow().children(SIZES.map(|(z, n)| {
        StatusLight::new(
            format!("status-light/wrap-{n}"),
            "StatusLight with very long label so we can see wrapping",
        )
        .variant(status_light::Variant::Positive)
        .size(z)
        .width(128.0)
        .build(t)
    }));
    page(vec![
        section(t, "Semantic variants", variants(true)),
        section(t, "Non-semantic variants", variants(false)),
        section(t, "Sizes", sizes),
        section(t, "Wrapped label (width 128)", wrapped),
    ])
}

fn accordion(s: &G, t: &'static Tokens) -> El {
    let item = |name: &str, title: &str, open: bool| {
        let key = format!("accordion/{name}");
        AccordionItem::new(key.clone())
            .title(title)
            .open(s.on(&key, open))
            .on_toggle(flip(key))
            .content(
                text("Content below the header. Its height is the app's; the recipe adds only the vertical padding, so it is flush with the item edge.")
                    .wrap()
                    .font(font::text_style(&t.component_m_regular))
                    .color(t.neutral_content_color_default),
            )
    };
    let stack = |items: Vec<AccordionItem>| {
        let mut c = col().w(360.0);
        for i in items {
            if valid("accordion", i.options()) {
                c = c.child(i.build(t));
            }
        }
        c
    };
    let basic = stack(vec![
        item("closed", "Closed", false),
        item("open", "Open", true),
        item("quiet", "Quiet, no divider", true).quiet(true),
        item("dis", "Disabled", false).disabled(true),
    ]);
    let density = stack(vec![
        item("d-compact", "Compact", true).density(Density::Compact),
        item("d-regular", "Regular", true).density(Density::Regular),
        item("d-spacious", "Spacious", true).density(Density::Spacious),
    ]);
    let sizes = stack(
        SIZES
            .map(|(z, n)| item(&format!("size-{n}"), &format!("Size {n}"), false).size(z))
            .into(),
    );
    let sw = |name: &str| {
        let key = format!("accordion/{name}/switch-on");
        let on = s.on(&key, false);
        move |b: Switch| b.label("Enable").selected(on).on_toggle(flip(key))
    };
    let actions = stack(vec![
        item("act-button", "Action button", false).action_button(|b| b.icon(&ADD_100).label("Add")),
        item("act-switch", "Switch", false).switch(sw("act-switch")),
        item("act-both", "Both", true)
            .action_button(|b| b.icon(&ADD_100).label("Add"))
            .switch(sw("act-both")),
        item("act-dis", "Disabled", false)
            .disabled(true)
            .action_button(|b| b.icon(&ADD_100).label("Add"))
            .switch(|s| s.label("Enable")),
    ]);
    let long = stack(vec![item(
        "long",
        "A long title that has to wrap onto a second line because the item is only 360 points wide",
        true,
    )]);
    page(vec![
        section(t, "States (click or Space/Enter)", basic),
        section(t, "Density", density),
        section(t, "Sizes", sizes),
        section(t, "Direct actions", actions),
        section(t, "Long title", long),
    ])
}

fn accordion_group(s: &G, t: &'static Tokens) -> El {
    let group = |name: &str, multiple: bool, first_open: &str| {
        let key = format!("accordion-group/{name}");
        let item = |k: &str, title: &str| {
            AccordionItem::new(k).title(title).content(
                text("Content below the header.")
                    .font(font::text_style(&t.component_m_regular))
                    .color(t.neutral_content_color_default),
            )
        };
        let k = key.clone();
        AccordionGroup::new(key.clone())
            .label(name.to_owned())
            .item(item("shipping", "Shipping"))
            .item(item("returns", "Returns"))
            .item(item("sizing", "Sizing").disabled(true))
            .item(item("care", "Care"))
            .multiple(multiple)
            .open(
                s.open
                    .get(&key)
                    .cloned()
                    .unwrap_or_else(|| vec![first_open.to_owned()]),
            )
            .on_change(move |s: &mut G, open: &[String]| {
                s.open.insert(k.clone(), open.to_vec());
            })
    };
    let w = |g: AccordionGroup| col().w(360.0).child(g.build(t));
    page(vec![
        section(
            t,
            "Single (one open; Up/Down, Home/End between headers)",
            w(group("single", false, "shipping")),
        ),
        section(t, "Multiple", w(group("multiple", true, "returns"))),
        section(
            t,
            "Quiet, size L",
            w(group("quiet", false, "care").quiet(true).size(Size::L)),
        ),
        section(
            t,
            "Disabled",
            w(group("disabled", false, "shipping").disabled(true)),
        ),
    ])
}
