//! Pixel evidence for the Rune keyboard-focus API the five focus looks (a14.32 to a14.36) rely on,
//! on small standalone elements: `hover_group_focus` + `group_color` (text and icon colour),
//! `focus_fill` on a child of the focusable root, a selected Switch track's border under a focus
//! fill, and a disabled root. Colours are loud (green rest, red focus, blue down, yellow fill) so
//! a ~12-per-channel match of the closest pixel proves the glyph is drawn in that colour.

#[path = "support/pixels.rs"]
mod pixels;

use pixels::{near, Pixels};
use rune::color::hex;
use rune::{col, el, icon, row, text, Color, Cx, El, Env, Key, KeyEvent, Runtime};
use spectrum_rune::icons::CHECKMARK_100;
use spectrum_rune::interaction::{Interaction, State};

const SIZE: (u32, u32) = (320, 120);
const GREEN: [f32; 3] = [0.0, 255.0, 0.0];
const RED: [f32; 3] = [255.0, 0.0, 0.0];
const BLUE: [f32; 3] = [0.0, 0.0, 255.0];
const YELLOW: [f32; 3] = [255.0, 255.0, 0.0];

fn c(s: &str) -> Color {
    hex(s).unwrap()
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    /// `group_color(red)` always, text colour from `Interaction` (green/red/blue).
    Naive,
    /// `group_color(red)` only while the view is at rest.
    AtRest,
}

#[derive(Clone, Copy)]
struct App {
    mode: Mode,
    focusable: bool,
}

fn view(a: &App, _: &mut Cx<App>) -> El {
    let i = Interaction::of("g");
    let ink = match i.state() {
        State::Rest => c("#00ff00"),
        State::Hover => c("#ff0000"),
        State::Down => c("#0000ff"),
    };
    let group = (a.mode == Mode::Naive || i.state() == State::Rest).then(|| c("#ff0000"));
    let tint = |e: El| match group {
        Some(g) => e.group_color(g),
        None => e,
    };
    let root = row()
        .key("g")
        .w(300.0)
        .h(80.0)
        .gap(10.0)
        .items_center()
        .focusable(a.focusable)
        .disabled(!a.focusable)
        .no_focus_ring()
        .hover_group_focus()
        .child(tint(text("MMMM").key("t").color(ink)))
        .child(tint(icon(&CHECKMARK_100, 30.0).key("i").color(ink)))
        .child(
            el().key("c")
                .size(40.0, 40.0)
                .radius(12.0)
                .fill(c("#000080"))
                .focus_fill(c("#ffff00")),
        );
    col()
        .w(SIZE.0 as f32)
        .h(SIZE.1 as f32)
        .px(10.0)
        .py(10.0)
        .fill(c("#000000"))
        .child(if a.focusable { i.attach(root) } else { root })
}

fn start(a: App) -> Runtime<App> {
    let mut rt = Runtime::new(
        a,
        view,
        (SIZE.0 as f32, SIZE.1 as f32),
        Env::new(false, 1.0),
    );
    settle(&mut rt);
    rt
}

fn settle(rt: &mut Runtime<App>) {
    for _ in 0..40 {
        rt.frame(0.05);
    }
}

fn tab(rt: &mut Runtime<App>) {
    rt.key(KeyEvent::from(Key::Tab));
    settle(rt);
}

fn rect(rt: &Runtime<App>, key: &str) -> [f32; 4] {
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key))
        .unwrap_or_else(|| panic!("no element {key}"))
        .rect
}

/// Largest-channel distance from `want` of the pixel in `key`'s rect closest to it.
fn closest(rt: &Runtime<App>, key: &str, want: [f32; 3]) -> f32 {
    let r = rect(rt, key);
    let px = Pixels::of(rt, SIZE);
    let (x0, y0) = (r[0].floor() as u32, r[1].floor() as u32);
    let (x1, y1) = ((r[0] + r[2]).ceil() as u32, (r[1] + r[3]).ceil() as u32);
    (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| (x, y)))
        .map(|(x, y)| {
            let p = px.at(x, y);
            (0..3)
                .map(|i| (p[i] as f32 - want[i]).abs())
                .fold(0.0, f32::max)
        })
        .fold(f32::MAX, f32::min)
}

/// Text and icon are `want` and not `other`: some pixel is the colour, no pixel is the other.
fn ink_is(rt: &Runtime<App>, want: [f32; 3], other: [f32; 3], what: &str) {
    for key in ["t", "i"] {
        assert!(
            closest(rt, key, want) <= 12.0,
            "{what}: {key} is not drawn in {want:?}"
        );
        assert!(
            closest(rt, key, other) > 60.0,
            "{what}: {key} still has {other:?}"
        );
    }
}

fn centre(rt: &Runtime<App>, key: &str) -> [u8; 4] {
    let r = rect(rt, key);
    Pixels::of(rt, SIZE).at((r[0] + r[2] / 2.0) as u32, (r[1] + r[3] / 2.0) as u32)
}

const APP: App = App {
    mode: Mode::AtRest,
    focusable: true,
};

/// (a) Keyboard focus turns a child text and a child `Icon` (the generated icons) to the group
/// colour; a click, then the pointer leaving, does not.
#[test]
fn group_colour_follows_keyboard_focus_for_text_and_icon_not_pointer_focus() {
    let mut rt = start(APP);
    ink_is(&rt, GREEN, RED, "rest");
    tab(&mut rt);
    ink_is(&rt, RED, GREEN, "Tab");

    let mut rt = start(APP);
    let (x, y) = rt.find("g").unwrap();
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.pointer_move(310.0, 110.0);
    settle(&mut rt);
    ink_is(&rt, GREEN, RED, "after a click and the pointer leaving");
    tab(&mut rt);
    ink_is(&rt, RED, GREEN, "click, then Tab");
}

/// (b) Pointer hover: Interaction rebuilds the colour to the hover red and the group colour is the
/// same red, so hover (alone, or on top of keyboard focus) is red, once. Down: the naive
/// `group_color(red)` always on overrides the down blue with the held group's red (they fight);
/// setting it only while the view is at rest leaves the down blue. Both modes, with and without Tab.
#[test]
fn hover_and_down_do_not_double_or_fight_when_group_colour_is_set_only_at_rest() {
    for focused in [false, true] {
        for mode in [Mode::Naive, Mode::AtRest] {
            let mut rt = start(App {
                mode,
                focusable: true,
            });
            if focused {
                tab(&mut rt);
            }
            let (x, y) = rt.find("g").unwrap();
            rt.pointer_move(x, y);
            settle(&mut rt);
            ink_is(&rt, RED, GREEN, &format!("hover, focused={focused}"));
            rt.pointer_down(x, y);
            settle(&mut rt);
            let down_is_blue = closest(&rt, "t", BLUE) <= 12.0 && closest(&rt, "t", RED) > 60.0;
            // Pressing moves focus to the pointer, so Rune drops the keyboard focus on its own.
            match mode {
                Mode::AtRest => assert!(down_is_blue, "down, focused={focused}: the down colour"),
                Mode::Naive => assert!(
                    !down_is_blue,
                    "down, focused={focused}: naive group colour overrides the down colour"
                ),
            }
        }
    }
}

/// (c) `focus_fill` on a child of the focusable root paints under Tab inside that child and is
/// clipped to its radius (the corner of the child's box stays the page), and not after a click.
#[test]
fn focus_fill_on_a_child_paints_clipped_under_keyboard_focus_only() {
    let mut rt = start(APP);
    near(centre(&rt, "c"), [0.0, 0.0, 128.0], "rest");
    tab(&mut rt);
    near(centre(&rt, "c"), YELLOW, "Tab: the child's focus fill");
    let r = rect(&rt, "c");
    let corner = Pixels::of(&rt, SIZE).at(r[0] as u32, r[1] as u32);
    near(
        corner,
        [0.0; 3],
        "Tab: clipped at the child's corner (radius 12)",
    );

    let mut rt = start(APP);
    let (x, y) = rt.find("c").unwrap();
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.pointer_move(310.0, 110.0);
    settle(&mut rt);
    near(centre(&rt, "c"), [0.0, 0.0, 128.0], "click: no fill");
}

/// (d) A disabled root cannot be focused: Tab paints no fill and changes no colour.
#[test]
fn disabled_paints_nothing() {
    let mut rt = start(App {
        mode: Mode::AtRest,
        focusable: false,
    });
    tab(&mut rt);
    near(centre(&rt, "c"), [0.0, 0.0, 128.0], "disabled: no fill");
    ink_is(&rt, GREEN, RED, "disabled: no colour");
}

/// (e) A selected Switch track with a 2px border and a focus fill: the border is drawn over the
/// fill, so a rest-coloured rim stays around the focus fill. Without the border the fill reaches
/// the edge. Both tracks are 52x28 pills, the border inside the box.
#[test]
fn focus_fill_under_a_track_border_leaves_a_rim() {
    fn track(border: bool) -> El {
        let t = el()
            .key("tr")
            .size(52.0, 28.0)
            .radius(14.0)
            .fill(c("#00ff00"))
            .focus_fill(c("#ff0000"));
        if border {
            t.border(2.0, c("#00ff00"))
        } else {
            t
        }
    }
    for border in [true, false] {
        let mut rt = Runtime::new(
            border,
            |b: &bool, _: &mut Cx<bool>| {
                col()
                    .w(100.0)
                    .h(60.0)
                    .px(10.0)
                    .py(10.0)
                    .fill(c("#000000"))
                    .child(
                        row()
                            .key("g")
                            .focusable(true)
                            .no_focus_ring()
                            .child(track(*b)),
                    )
            },
            (100.0, 60.0),
            Env::new(false, 1.0),
        );
        let px_of = |rt: &Runtime<bool>| {
            let r = rt
                .elements()
                .into_iter()
                .find(|e| e.key.as_deref() == Some("tr"))
                .unwrap()
                .rect;
            let px = Pixels::of(rt, (100, 60));
            let (y, x0) = ((r[1] + r[3] / 2.0) as u32, r[0] as u32);
            [
                px.at(x0, y),
                px.at(x0 + 1, y),
                px.at(x0 + 3, y),
                px.at(x0 + 6, y),
            ]
        };
        for _ in 0..20 {
            rt.frame(0.05);
        }
        rt.key(KeyEvent::from(Key::Tab));
        for _ in 0..20 {
            rt.frame(0.05);
        }
        let p = px_of(&rt);
        near(p[3], RED, "interior is the focus fill");
        if border {
            near(p[0], GREEN, "outer rim pixel keeps the border colour");
            near(p[1], GREEN, "second rim pixel keeps the border colour");
            near(
                p[2],
                RED,
                "then the focus fill (pixel 2 is the border's soft inner edge)",
            );
        } else {
            near(p[0], RED, "no border: the fill reaches the edge");
        }
    }
}
