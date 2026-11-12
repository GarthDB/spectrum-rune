//! One story per component. Toggles read their state from [`G`], so they really toggle; every
//! option combination goes through `constraints::check` and an `error` one is skipped.

use crate::G;
use rune::{col, el, row, text, El};
use spectrum_rune::accordion::{AccordionItem, Density};
use spectrum_rune::accordion_group::AccordionGroup;
use spectrum_rune::action_button::ActionButton;
use spectrum_rune::badge::{self, Badge};
use spectrum_rune::button::{self, Button, StaticColor};
use spectrum_rune::checkbox::Checkbox;
use spectrum_rune::constraints::{check, OptionValue, Severity};
use spectrum_rune::font;
use spectrum_rune::icons::{ADD_100, CHECKMARK_100};
use spectrum_rune::radio_button::RadioButton;
use spectrum_rune::switch::Switch;
use spectrum_rune::tokens::Tokens;
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
        )*];
    };
}

stories! {
    "switch" => "Switch", switch;
    "checkbox" => "Checkbox", checkbox;
    "radio-button" => "Radio button", radio_button;
    "button" => "Button", button;
    "action-button" => "Action button", action_button;
    "badge" => "Badge", badge;
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
    let fixed = flow().children(badge::Fixed::ALL.iter().map(|f| {
        Badge::new(format!("badge/fixed-{}", f.as_str()))
            .label(f.as_str())
            .variant(badge::Variant::Informative)
            .fixed(*f)
            .build(t)
    }));
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
    sections.push(section(
        t,
        "Fixed edges (that edge's corners are square)",
        fixed,
    ));
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
    let long = stack(vec![item(
        "long",
        "A long title that has to wrap onto a second line because the item is only 360 points wide",
        true,
    )]);
    page(vec![
        section(t, "States (click or Space/Enter)", basic),
        section(t, "Density", density),
        section(t, "Sizes", sizes),
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
