//! Pixel assertions for the AccordionItem, which `ElementInfo` cannot give: the chevron's rotation
//! (0 closed, 90 degrees open, and in between while it turns), the 1px divider's colour and
//! height, and the hover fill's alpha over the page. See `support/pixels.rs` for how and its
//! limits (Rune's CPU reference renderer, sRGB 8-bit, scale 1).

#[path = "support/pixels.rs"]
mod pixels;

use pixels::Pixels;
use rune::{col, el, Cx, El, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::accordion::AccordionItem;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

const SIZE: (u32, u32) = (200, 160);

#[derive(Clone)]
struct App {
    dark: bool,
    open: bool,
    page: bool,
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
    let item = AccordionItem::new("ac")
        .title("Label")
        .size(a.size)
        .open(a.open)
        .content(el().key("ac/body").h(30.0))
        .on_toggle(|s: &mut App| s.open = !s.open)
        .build(t);
    let root = col().w(SIZE.0 as f32).h(SIZE.1 as f32);
    if a.page {
        root.fill(t.gray_100).child(item)
    } else {
        root.child(item)
    }
}

fn app() -> App {
    App {
        dark: false,
        open: false,
        page: false,
        size: Size::M,
    }
}

fn runtime(a: App) -> Runtime<App> {
    let dark = a.dark;
    let mut rt = Runtime::new(a, view, (SIZE.0 as f32, SIZE.1 as f32), Env::new(dark, 1.0));
    for _ in 0..40 {
        rt.frame(0.05);
    }
    rt
}

fn rect(rt: &Runtime<App>, part: &str) -> [f32; 4] {
    let key = format!("ac/{part}");
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key.as_str()) && !e.exiting)
        .unwrap()
        .target
}

fn click(rt: &mut Runtime<App>) {
    let (x, y) = rt.find("ac/header").unwrap();
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    // Off the item again, so its hover fill does not show in the chevron's alpha.
    rt.pointer_move(SIZE.0 as f32 - 2.0, SIZE.1 as f32 - 2.0);
}

/// The chevron's painted alpha over its box plus a 2px margin, row by row.
fn chevron_ink(rt: &Runtime<App>) -> Vec<Vec<f32>> {
    let r = rect(rt, "disclosure-icon");
    let px = Pixels::of(rt, SIZE);
    let (x0, y0) = (r[0] as u32 - 2, r[1] as u32 - 2);
    let n = r[2] as u32 + 4;
    assert_eq!(r[2], r[3]);
    (0..n)
        .map(|y| {
            (0..n)
                .map(|x| px.at(x0 + x, y0 + y)[3] as f32 / 255.0)
                .collect()
        })
        .collect()
}

/// A quarter turn clockwise.
fn turned(ink: &[Vec<f32>]) -> Vec<Vec<f32>> {
    let n = ink.len();
    (0..n)
        .map(|r| (0..n).map(|c| ink[n - 1 - c][r]).collect())
        .collect()
}

/// Mean absolute alpha difference per pixel.
fn distance(a: &[Vec<f32>], b: &[Vec<f32>]) -> f32 {
    let n = a.len() as f32;
    a.iter()
        .flatten()
        .zip(b.iter().flatten())
        .map(|(x, y)| (x - y).abs())
        .sum::<f32>()
        / (n * n)
}

/// What a closed chevron looks like turned 90 degrees clockwise: the reference for an open one.
fn closed_ink(size: Size) -> Vec<Vec<f32>> {
    chevron_ink(&runtime(App { size, ..app() }))
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

/// Open is the closed glyph turned 90 degrees clockwise (the recipe's `rotation: 90`), and not
/// the closed glyph itself.
fn to_linear(c: u8) -> f32 {
    let c = c as f32 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn from_linear(l: f32) -> u8 {
    let c = if l <= 0.0031308 {
        l * 12.92
    } else {
        1.055 * l.powf(1.0 / 2.4) - 0.055
    };
    (c * 255.0).round() as u8
}

#[test]
fn the_open_chevron_is_the_closed_one_turned_a_quarter() {
    for size in [Size::S, Size::M, Size::L, Size::XL] {
        let closed = closed_ink(size);
        let open = chevron_ink(&runtime(App {
            size,
            open: true,
            ..app()
        }));
        let (turn, same) = (distance(&open, &turned(&closed)), distance(&open, &closed));
        assert!(turn < 0.01, "{size:?}: open vs closed turned: {turn}");
        assert!(
            same > 0.05,
            "{size:?}: open is not the closed glyph: {same}"
        );
    }
}

#[test]
fn the_chevron_turns_while_the_item_opens_and_closes() {
    let closed = closed_ink(Size::M);
    let open = turned(&closed);
    for from_open in [false, true] {
        let mut rt = runtime(App {
            open: from_open,
            ..app()
        });
        click(&mut rt);
        let mut halfway = false;
        for _ in 0..30 {
            rt.frame(0.02);
            let now = chevron_ink(&rt);
            halfway |= distance(&now, &closed) > 0.03 && distance(&now, &open) > 0.03;
        }
        assert!(
            halfway,
            "from_open={from_open}: a mid-turn frame matches neither end"
        );
        for _ in 0..40 {
            rt.frame(0.05);
        }
        let last = chevron_ink(&rt);
        let end = if from_open { &closed } else { &open };
        assert!(distance(&last, end) < 0.01, "settles on the end state");
    }
}

#[test]
fn reduced_motion_turns_the_chevron_at_once() {
    let mut rt = runtime(app());
    rt.set_system_reduced_motion(true);
    rt.frame(0.05);
    click(&mut rt);
    rt.frame(0.001);
    assert!(distance(&chevron_ink(&rt), &turned(&closed_ink(Size::M))) < 0.01);
}

/// The divider is one row of `gray_200`: nothing above or below it, in both schemes, closed and
/// open (where it sits under the content).
#[test]
fn the_divider_is_one_pixel_of_gray_200() {
    for dark in [false, true] {
        for open in [false, true] {
            let a = App {
                dark,
                open,
                ..app()
            };
            let rt = runtime(a);
            let t = tokens(
                if dark {
                    ColorScheme::Dark
                } else {
                    ColorScheme::Light
                },
                Scale::Desktop,
            );
            let d = rect(&rt, "divider");
            assert_eq!(d[3], 1.0);
            let px = Pixels::of(&rt, SIZE);
            let (x, y) = (SIZE.0 / 2, d[1] as u32);
            let what = format!("dark={dark} open={open}");
            near(px.at(x, y), hex(t.gray_200), &format!("{what}: divider"));
            assert_eq!(px.at(x, y - 1)[3], 0, "{what}: clear above");
            assert_eq!(px.at(x, y + 1)[3], 0, "{what}: clear below");
        }
    }
}

/// Hover fills the whole item, content and all, with static black at 0.08 over the page, in both
/// schemes; at rest the page shows through.
#[test]
fn hover_fills_the_item_with_translucent_black_over_the_page() {
    for dark in [false, true] {
        for open in [false, true] {
            let mut rt = runtime(App {
                dark,
                open,
                page: true,
                ..app()
            });
            let t = tokens(
                if dark {
                    ColorScheme::Dark
                } else {
                    ColorScheme::Light
                },
                Scale::Desktop,
            );
            let page = hex(t.gray_100);
            let header = rect(&rt, "header");
            let content_y = (header[1] + header[3]) as u32 + 5;
            // Away from the title and chevron.
            let probes = [(150, 4), (150, content_y)];
            let probes = if open { &probes[..] } else { &probes[..1] };
            let px = Pixels::of(&rt, SIZE);
            for (x, y) in probes {
                near(px.at(*x, *y), page, "rest shows the page");
            }
            let (hx, hy) = rt.find("ac/header").unwrap();
            rt.pointer_move(hx, hy);
            for _ in 0..40 {
                rt.frame(0.05);
            }
            let px = Pixels::of(&rt, SIZE);
            // Rune blends in linear light, so 0.08 black over the page is not 8% off in sRGB.
            let want = page.map(|c| from_linear(to_linear(c) * (1.0 - 0.08)));
            for (x, y) in probes {
                near(
                    px.at(*x, *y),
                    want,
                    &format!("dark={dark} open={open} hover at ({x},{y})"),
                );
            }
        }
    }
}

const FSIZE: (u32, u32) = (240, 100);
/// A mid-grey page: dark and light inks both show against it.
const BG: [u8; 3] = [0x80, 0x80, 0x80];

#[derive(Clone, Copy)]
struct F {
    dark: bool,
    open: bool,
    quiet: bool,
    disabled: bool,
}

fn fview(a: &F, _: &mut Cx<F>) -> El {
    let t = tokens(
        if a.dark {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        },
        Scale::Desktop,
    );
    col()
        .w(FSIZE.0 as f32)
        .h(FSIZE.1 as f32)
        .fill(rune::color::hex("#808080").unwrap())
        .child(
            AccordionItem::new("ac")
                .title("Label")
                .open(a.open)
                .quiet(a.quiet)
                .disabled(a.disabled)
                .content(el().key("ac/body").h(20.0))
                .build(t),
        )
}

fn frt(a: F) -> Runtime<F> {
    let mut rt = Runtime::new(
        a,
        fview,
        (FSIZE.0 as f32, FSIZE.1 as f32),
        Env::new(a.dark, 1.0),
    );
    fsettle(&mut rt);
    rt
}

fn fsettle(rt: &mut Runtime<F>) {
    for _ in 0..40 {
        rt.frame(0.05);
    }
}

fn frect(rt: &Runtime<F>, part: &str) -> [f32; 4] {
    let key = format!("ac/{part}");
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key.as_str()) && !e.exiting)
        .unwrap()
        .target
}

fn fcol(s: &str) -> [f32; 3] {
    let c = pixels::rgba(s);
    [c[0], c[1], c[2]]
}

/// The pixel of `r` (x, y, w, h) farthest from the backdrop `bg`, in linear light. Thin glyphs and
/// anti-aliased text never fully cover a pixel, so the ink itself cannot be read: this pixel is
/// the ink at the glyph's best coverage.
fn extreme(px: &Pixels, r: [f32; 4], bg: [f32; 3]) -> [u8; 4] {
    let lin = pixels::to_linear;
    let mut best = ([0u8; 4], -1.0f32);
    for y in r[1] as u32..(r[1] + r[3]).ceil() as u32 {
        for x in r[0] as u32..(r[0] + r[2]).ceil() as u32 {
            let p = px.at(x, y);
            let d = (lin(p[0] as f32) - lin(bg[0])).abs();
            if d > best.1 {
                best = (p, d);
            }
        }
    }
    best.0
}

/// The glyph's best coverage (0..1) of `ink` over `bg`, from its most covered pixel.
fn coverage(got: [u8; 4], ink: [f32; 3], bg: [f32; 3]) -> f32 {
    let lin = pixels::to_linear;
    (lin(got[0] as f32) - lin(bg[0])).abs() / (lin(ink[0]) - lin(bg[0])).abs()
}

/// What the oracle's `ink` looks like at `coverage` over `bg`.
fn predicted(ink: [f32; 3], coverage: f32, bg: [u8; 3]) -> [f32; 3] {
    pixels::composite(
        &[&format!("{},{},{},{coverage}", ink[0], ink[1], ink[2])],
        bg,
    )
}

/// The backdrop of the header: the page, with the oracle's item fill (hover wash) over it.
fn backdrop(item_fill: &Value) -> [u8; 3] {
    match item_fill.as_str() {
        Some(f) => pixels::composite(&[f], BG).map(|c| c.round() as u8),
        None => BG,
    }
}

/// Runs `case` (a `accordion_focus.json` entry) to its settled state: Tab, and the pointer over the
/// header for hover, with the focus Tab for the combined state.
fn run(case: &Value) -> Runtime<F> {
    let mut rt = frt(F {
        dark: case["scheme"] == "dark",
        open: case["open"] == true,
        quiet: case["quiet"] == true,
        disabled: case["disabled"] == true,
    });
    let state = case["state"].as_str().unwrap();
    if state.contains("keyboard-focus") {
        rt.key(KeyEvent::from(Key::Tab));
    }
    if state.starts_with("hover") {
        let (x, y) = rt.find("ac/header").unwrap();
        rt.pointer_move(x, y);
    }
    fsettle(&mut rt);
    rt
}

/// What the oracle's keyboard-focus state gives an accordion item (`accordion_focus.json`): the
/// title and the chevron go to the hover ink, 41 to 19 light and 219 to 242 dark (a glyph's most
/// covered pixel is the oracle's ink at the coverage the same glyph reaches at rest, calibrated per
/// case from the oracle's rest ink); the item fill does NOT change (no wash: the page pixel beside
/// the header's content is the page), while hover has the oracle's wash over the page, hover +
/// keyboard-focus has both the wash and the ink, and a disabled item (not focusable) keeps the
/// oracle's disabled ink. The ring is still drawn on the header, inset.
#[test]
fn keyboard_focus_look_matches_oracle() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/accordion_focus.json")).unwrap();
    assert_eq!(cases.len(), 36);
    let (mut inks, mut changed) = (0, 0);
    for c in &cases {
        let what = c.to_string();
        let same = |r: &&Value| {
            ["scheme", "open", "quiet", "disabled"]
                .iter()
                .all(|k| r[*k] == c[*k])
                && r["state"] == "rest"
        };
        let rest = cases.iter().find(same).unwrap();
        let rest_rt = run(rest);
        let rest_px = Pixels::of(&rest_rt, FSIZE);
        let rt = run(c);
        let px = Pixels::of(&rt, FSIZE);
        let bg = backdrop(&c["item_fill"]);
        for (part, key) in [("title", "title"), ("disclosure-icon", "icon")] {
            let ink = fcol(c[key].as_str().unwrap());
            let rest_ink = fcol(rest[key].as_str().unwrap());
            let cov = coverage(
                extreme(&rest_px, frect(&rest_rt, part), BG.map(f32::from)),
                rest_ink,
                BG.map(f32::from),
            );
            assert!(cov > 0.5, "{what}: the rest {part} is drawn ({cov})");
            let got = extreme(&px, frect(&rt, part), bg.map(f32::from));
            pixels::near(got, predicted(ink, cov, bg), &format!("{what}: {part} ink"));
            inks += 1;
            if c[key] != rest[key] {
                changed += 1;
                pixels::differs(
                    got,
                    predicted(rest_ink, cov, bg),
                    &format!("{what}: {part} is not the rest ink"),
                );
            }
        }
        // The item fill: the page beside the title, wash only where the oracle has one.
        let h = frect(&rt, "header");
        let beside = px.at((h[0] + h[2] - 12.0) as u32, (h[1] + h[3] / 2.0) as u32);
        pixels::near(beside, bg.map(f32::from), &format!("{what}: item fill"));
        // The ring: drawn inside the header's edge when it shows keyboard focus.
        let edge = px.at(h[0] as u32, (h[1] + h[3] / 2.0) as u32);
        let focused =
            c["state"].as_str().unwrap().contains("keyboard-focus") && c["disabled"] != true;
        if focused {
            pixels::differs(
                edge,
                bg.map(f32::from),
                &format!("{what}: ring on the header"),
            );
        } else {
            pixels::near(edge, bg.map(f32::from), &format!("{what}: no ring"));
        }
    }
    assert!(inks == 72 && changed >= 36, "{inks} {changed}");
}

/// Pointer focus shows no keyboard-focus look: after a click (and the pointer leaving) the title
/// and chevron are the oracle's rest ink; Tab afterwards shows the focus ink. A held press (no
/// keyboard focus) draws the rest ink too, as the oracle's `down` does: the group colour must not
/// turn it to the hover colour.
#[test]
fn pointer_focus_shows_no_focus_ink_and_a_press_is_the_rest_ink() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/accordion_focus.json")).unwrap();
    let find = |state: &str| {
        cases
            .iter()
            .find(|c| {
                c["scheme"] == "light"
                    && c["open"] == false
                    && c["quiet"] == false
                    && c["disabled"] == false
                    && c["state"] == state
            })
            .unwrap()
    };
    let (rest, focus) = (find("rest"), find("keyboard-focus"));
    let bg = BG.map(f32::from);
    let rest_rt = run(rest);
    let rest_px = Pixels::of(&rest_rt, FSIZE);
    let (r_ink, f_ink) = (
        fcol(rest["title"].as_str().unwrap()),
        fcol(focus["title"].as_str().unwrap()),
    );
    let title = |rt: &Runtime<F>| extreme(&Pixels::of(rt, FSIZE), frect(rt, "title"), bg);
    let cov = coverage(extreme(&rest_px, frect(&rest_rt, "title"), bg), r_ink, bg);
    let is = |got: [u8; 4], ink, what: &str| pixels::near(got, predicted(ink, cov, BG), what);

    let mut rt = frt(F {
        dark: false,
        open: false,
        quiet: false,
        disabled: false,
    });
    let (x, y) = rt.find("ac/header").unwrap();
    rt.pointer_down(x, y);
    fsettle(&mut rt);
    // The press toggles nothing yet (the app flips on release) but the item is `Down`.
    is(title(&rt), r_ink, "held press: the rest ink");
    rt.pointer_up(x, y);
    rt.pointer_move(FSIZE.0 as f32 - 2.0, FSIZE.1 as f32 - 2.0);
    fsettle(&mut rt);
    is(title(&rt), r_ink, "clicked: the rest ink");
    pixels::differs(
        title(&rt),
        predicted(f_ink, cov, BG),
        "clicked: not the focus ink",
    );
    rt.key(KeyEvent::from(Key::Tab));
    fsettle(&mut rt);
    is(title(&rt), f_ink, "Tab: the focus ink");
}

/// Same as the radio's: the oracle gives down + keyboard-focus the focus look (title and chevron 19 not 41), Rune
/// cannot enter it (a pointer press hides keyboard focus), so a press on a Tab-focused header draws the plain
/// down look (the same as rest, as the oracle's down is). `fixtures/combined-diff.sh` prints the oracle's side.
#[test]
fn a_press_on_a_keyboard_focused_header_draws_the_plain_down_look() {
    let header = |rt: &Runtime<App>| {
        rt.elements()
            .into_iter()
            .find(|e| e.key.as_deref() == Some("ac/header"))
            .expect("the header")
    };
    let pressed = |tab: bool| {
        let mut rt = runtime(app());
        if tab {
            rt.key(KeyEvent::from(Key::Tab));
            for _ in 0..40 {
                rt.frame(0.05);
            }
            assert!(header(&rt).focus_visible, "Tab shows focus on the header");
        }
        let (x, y) = rt.find("ac/header").unwrap();
        rt.pointer_move(x, y);
        rt.pointer_down(x, y);
        for _ in 0..40 {
            rt.frame(0.05);
        }
        assert!(!header(&rt).focus_visible, "a press hides it");
        Pixels::of(&rt, SIZE)
    };
    assert!(pressed(true).identical(&pressed(false)));
}
