//! Checks the AccordionItem against the `design-data render accordion --format scene` oracle.
//! Regenerate the fixture with `tests/fixtures/regen-accordion.sh` (its header explains the
//! matrix and why).
//!
//! The content's height is the CLI's placeholder (0 high; the item is only the recipe's padding),
//! so the test puts a 30 high body in the content and compares the content's position, its two
//! paddings (body offset inside, and what is left below the body) and that the divider sits at the
//! bottom of whatever the app supplied: item height and divider y are the oracle's plus 30.
//!
//! Not asserted from the element, because `ElementInfo` does not expose it: the chevron's rotation
//! (read from pixels in `tests/accordion_pixels.rs`, with the divider's colour and the hover
//! fill), the font family and size (the title's rect height is the line height, which follows the
//! size), and the focus ring (which is not from the oracle at all).

use rune::{col, el, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::accordion::{AccordionItem, Density};
use spectrum_rune::constraints::check;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

const BODY: f32 = 30.0;

#[derive(Clone)]
pub struct App {
    pub dark: bool,
    pub mobile: bool,
    pub size: Size,
    pub density: Option<Density>,
    pub quiet: bool,
    pub disabled: bool,
    pub open: bool,
    /// Whether `on_toggle` flips `open` (an app that owns the state) or only counts.
    pub follow: bool,
    pub toggles: u32,
    /// A focusable control inside the content.
    pub button: bool,
    pub long_title: bool,
}

impl Default for App {
    fn default() -> Self {
        App {
            dark: false,
            mobile: false,
            size: Size::M,
            density: None,
            quiet: false,
            disabled: false,
            open: false,
            follow: true,
            toggles: 0,
            button: false,
            long_title: false,
        }
    }
}

fn view(a: &App, _: &mut Cx<App>) -> El {
    let t = tokens(
        if a.dark {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        },
        if a.mobile {
            Scale::Mobile
        } else {
            Scale::Desktop
        },
    );
    let mut b = AccordionItem::new("ac")
        .title(if a.long_title {
            "A title much longer than this narrow accordion item can show on one line"
        } else {
            "Label"
        })
        .size(a.size)
        .quiet(a.quiet)
        .disabled(a.disabled)
        .open(a.open)
        .content(el().key("ac/body").h(BODY))
        .on_toggle(|s: &mut App| {
            s.toggles += 1;
            if s.follow {
                s.open = !s.open;
            }
        });
    if let Some(d) = a.density {
        b = b.density(d);
    }
    if a.button {
        b = b.content(el().key("ac/button").h(20.0).focusable(true));
    }
    let item = b.build(t);
    col().child(if a.long_title {
        col().w(160.0).child(item)
    } else {
        item
    })
}

pub fn runtime(a: App) -> Runtime<App> {
    let dark = a.dark;
    let mut rt = Runtime::new(a, view, (400.0, 300.0), Env::new(dark, 1.0));
    settle(&mut rt);
    rt
}

pub fn settle(rt: &mut Runtime<App>) {
    for _ in 0..40 {
        rt.frame(0.05);
    }
}

pub fn part(rt: &Runtime<App>, name: &str) -> Option<ElementInfo> {
    let key = format!("ac/{name}");
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key.as_str()) && !e.exiting)
}

pub fn click(rt: &mut Runtime<App>) {
    let (x, y) = rt.find("ac/header").unwrap();
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
}

fn rgba(s: &str) -> [f32; 4] {
    let v: Vec<f32> = s.split(',').map(|c| c.parse().unwrap()).collect();
    [v[0], v[1], v[2], v.get(3).copied().unwrap_or(1.0)]
}

/// `want` is the oracle's colour string; a missing one is transparent.
fn assert_color(got: rune::Color, want: &Value, what: &str) {
    let want = want.as_str().map_or([0.0; 4], rgba);
    if want[3] == 0.0 {
        assert!(got.a < 0.02, "{what}: want transparent, got {got:?}");
        return;
    }
    let hex = got.hex();
    for i in 0..3 {
        let g = i32::from_str_radix(&hex[1 + 2 * i..3 + 2 * i], 16).unwrap() as f32;
        assert!(
            (g - want[i]).abs() <= 3.0,
            "{what}: got {hex}, want {want:?}"
        );
    }
    assert!(
        (got.a - want[3]).abs() < 0.02,
        "{what}: alpha {} want {}",
        got.a,
        want[3]
    );
}

fn close(got: f32, want: f32, what: &str) {
    assert!((got - want).abs() <= 1.0, "{what}: got {got}, want {want}");
}

fn num(v: &Value) -> f32 {
    v.as_f64().unwrap() as f32
}

fn app_for(c: &Value) -> App {
    App {
        dark: c["scheme"] == "dark",
        mobile: c["scale"] == "mobile",
        size: match c["size"].as_str().unwrap() {
            "s" => Size::S,
            "m" => Size::M,
            "l" => Size::L,
            _ => Size::XL,
        },
        density: c["density"].as_str().map(|d| {
            *Density::ALL
                .iter()
                .find(|x| x.as_str() == d)
                .expect("a catalog density")
        }),
        quiet: c["quiet"].as_bool().unwrap(),
        disabled: c["disabled"].as_bool().unwrap(),
        open: c["open"].as_bool().unwrap(),
        ..App::default()
    }
}

#[test]
fn matches_oracle_scene() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/accordion_scene.json")).unwrap();
    assert_eq!(cases.len(), 212);
    for c in &cases {
        let what = c.to_string();
        let what = &what[..what.len().min(200)];
        let open = c["open"].as_bool().unwrap();
        let disabled = c["disabled"].as_bool().unwrap();
        let mut rt = runtime(app_for(c));
        if c["state"] == "hover" {
            let (x, y) = rt.find("ac/header").unwrap();
            rt.pointer_move(x, y);
            settle(&mut rt);
        }

        // Whatever the app put in the content moves everything below it.
        let extra = if open { BODY } else { 0.0 };
        let item = part(&rt, "item").unwrap();
        let o = item.target;
        close(
            item.target[3],
            num(&c["item"]["h"]) + extra,
            &format!("{what} item h"),
        );
        // The oracle fills a disabled item on hover; the item does not react to the pointer then.
        if disabled && c["state"] == "hover" {
            assert!(
                item.fill.a < 0.02,
                "{what}: disabled item takes no hover fill"
            );
        } else {
            assert_color(item.fill, &c["item"]["fill"], &format!("{what} item fill"));
        }

        let header = part(&rt, "header").unwrap();
        close(header.target[0] - o[0], 0.0, &format!("{what} header x"));
        close(header.target[1] - o[1], 0.0, &format!("{what} header y"));
        close(
            header.target[3],
            num(&c["header"]["h"]),
            &format!("{what} header h"),
        );
        close(
            header.target[2],
            item.target[2],
            &format!("{what} header w"),
        );
        assert_eq!(header.role, "button", "{what}");
        assert_eq!(header.expanded, Some(open), "{what}: expanded follows open");
        assert_eq!(
            header.focusable, !disabled,
            "{what}: disabled is not focusable"
        );

        let icon = part(&rt, "disclosure-icon").unwrap();
        for (k, v) in [
            ("x", icon.target[0] - o[0]),
            ("y", icon.target[1] - o[1]),
            ("w", icon.target[2]),
            ("h", icon.target[3]),
        ] {
            close(v, num(&c["icon"][k]), &format!("{what} icon {k}"));
        }
        assert_color(
            icon.text_color.unwrap(),
            &c["icon"]["color"],
            &format!("{what} icon"),
        );

        let title = part(&rt, "title").unwrap();
        for (k, v) in [
            ("x", title.target[0] - o[0]),
            ("y", title.target[1] - o[1]),
            ("h", title.target[3]),
        ] {
            close(v, num(&c["title"][k]), &format!("{what} title {k}"));
        }
        assert_color(
            title.text_color.unwrap(),
            &c["title"]["color"],
            &format!("{what} title"),
        );

        let content = part(&rt, "content");
        assert_eq!(content.is_some(), open, "{what}: content present iff open");
        if let Some(content) = content {
            close(
                content.target[1] - o[1],
                num(&c["content"]["y"]),
                &format!("{what} content y"),
            );
            close(
                content.target[2],
                item.target[2],
                &format!("{what} content w"),
            );
            let body = part(&rt, "body").unwrap();
            // a14.24: the oracle's content starts at the item's left edge (x 0, the regen script
            // drops that field) and the recipe has no inline padding, so the app's content is
            // flush with the item, left of the chevron and the title.
            close(
                content.target[0],
                item.target[0],
                &format!("{what} content x"),
            );
            close(body.target[0], item.target[0], &format!("{what} body x"));
            let [top, bottom] = [num(&c["content_pad"][0]), num(&c["content_pad"][1])];
            close(
                body.target[1] - content.target[1],
                top,
                &format!("{what} content top inset"),
            );
            close(
                content.target[1] + content.target[3] - (body.target[1] + body.target[3]),
                bottom,
                &format!("{what} content bottom inset"),
            );
            close(
                body.target[2],
                content.target[2],
                &format!("{what} body stretches"),
            );
        }

        let divider = part(&rt, "divider");
        assert_eq!(
            divider.is_some(),
            !c["quiet"].as_bool().unwrap(),
            "{what}: divider present iff not quiet"
        );
        if let Some(d) = divider {
            close(
                d.target[1] - o[1],
                num(&c["divider"]["y"]) + extra,
                &format!("{what} divider y"),
            );
            close(
                d.target[3],
                num(&c["divider"]["h"]),
                &format!("{what} divider h"),
            );
            close(d.target[2], item.target[2], &format!("{what} divider w"));
            assert_color(d.fill, &c["divider"]["fill"], &format!("{what} divider"));
            // At the bottom of the item, whatever the content.
            close(
                d.target[1] + d.target[3],
                item.target[1] + item.target[3],
                &format!("{what} divider at bottom"),
            );
        }
    }
}

#[test]
fn the_oracle_cases_are_valid_option_combinations() {
    use spectrum_rune::constraints::OptionValue::*;
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/accordion_scene.json")).unwrap();
    for c in &cases {
        let flag = |k: &str| Bool(c[k].as_bool().unwrap());
        let o = [
            ("size", Str(c["size"].as_str().unwrap())),
            ("isQuiet", flag("quiet")),
            ("isDisabled", flag("disabled")),
            ("isOpen", flag("open")),
        ];
        assert!(check("accordion", &o).is_empty(), "{c}");
    }
}

/// The catalog default is `regular` (the CLI renders it when no density is given: the fixture's
/// last cases have none) and `m`.
#[test]
fn defaults_follow_the_catalog() {
    use spectrum_rune::constraints::OptionValue::*;
    let o = AccordionItem::new("a").options();
    assert!(o.contains(&("density", Str("regular"))), "{o:?}");
    assert!(o.contains(&("size", Str("m"))), "{o:?}");
    assert_eq!(Density::default(), Density::Regular);
    assert_eq!(
        Density::ALL.map(Density::as_str),
        ["compact", "regular", "spacious"]
    );
    let rt = runtime(App::default());
    assert_eq!(part(&rt, "header").unwrap().target[3], 48.0);
}

#[test]
fn press_reports_and_the_app_owns_open() {
    let mut rt = runtime(App {
        follow: false,
        ..App::default()
    });
    click(&mut rt);
    settle(&mut rt);
    assert_eq!(rt.state.toggles, 1);
    assert_eq!(
        part(&rt, "header").unwrap().expanded,
        Some(false),
        "a press alone does not open"
    );
    assert!(part(&rt, "content").is_none());

    let mut rt = runtime(App::default());
    click(&mut rt);
    settle(&mut rt);
    assert_eq!(part(&rt, "header").unwrap().expanded, Some(true));
    assert!(part(&rt, "content").is_some());
    click(&mut rt);
    settle(&mut rt);
    assert_eq!(rt.state.toggles, 2);
    assert_eq!(part(&rt, "header").unwrap().expanded, Some(false));
    assert!(part(&rt, "content").is_none());
}

#[test]
fn space_and_enter_toggle_the_focused_header() {
    for key in [Key::Text(" ".into()), Key::Enter] {
        let mut rt = runtime(App::default());
        rt.key(KeyEvent::from(Key::Tab));
        settle(&mut rt);
        let h = part(&rt, "header").unwrap();
        assert!(
            h.focused && h.focus_visible,
            "keyboard focus shows the ring"
        );
        rt.key(KeyEvent::from(key.clone()));
        settle(&mut rt);
        assert_eq!(rt.state.toggles, 1, "{key:?}");
        assert_eq!(part(&rt, "header").unwrap().expanded, Some(true), "{key:?}");
    }
}

#[test]
fn disabled_does_not_react_or_take_focus() {
    let mut rt = runtime(App {
        disabled: true,
        ..App::default()
    });
    click(&mut rt);
    rt.key(KeyEvent::from(Key::Tab));
    rt.key(KeyEvent::from(Key::Enter));
    settle(&mut rt);
    assert_eq!(rt.state.toggles, 0);
    let h = part(&rt, "header").unwrap();
    assert!(!h.focusable && !h.focused);
    assert_eq!(h.expanded, Some(false));
    // The pointer is over it and nothing changed: no hover fill, rest colours.
    assert!(part(&rt, "item").unwrap().fill.a < 0.02);
    assert_eq!(
        part(&rt, "title").unwrap().text_color.unwrap().hex(),
        tokens(ColorScheme::Light, Scale::Desktop)
            .disabled_content_color
            .hex()
    );
}

#[test]
fn closed_content_is_not_reachable() {
    let mut rt = runtime(App {
        button: true,
        ..App::default()
    });
    assert!(part(&rt, "content").is_none() && part(&rt, "button").is_none());
    assert!(rt
        .elements()
        .iter()
        .all(|e| e.key.as_deref() != Some("ac/button")));
    // Tab visits the header only.
    rt.key(KeyEvent::from(Key::Tab));
    settle(&mut rt);
    assert!(part(&rt, "header").unwrap().focused);
    rt.key(KeyEvent::from(Key::Tab));
    settle(&mut rt);
    assert!(rt
        .elements()
        .iter()
        .all(|e| !e.focused || e.key.as_deref() != Some("ac/button")));

    // Open, the content takes focus in order.
    let mut rt = runtime(App {
        button: true,
        open: true,
        ..App::default()
    });
    rt.key(KeyEvent::from(Key::Tab));
    rt.key(KeyEvent::from(Key::Tab));
    settle(&mut rt);
    assert!(part(&rt, "button").unwrap().focused);

    // Closing: while the height springs down the content is clipped and cannot be reached.
    let mut rt = runtime(App {
        button: true,
        open: true,
        ..App::default()
    });
    click(&mut rt);
    rt.frame(0.03);
    let ghost = rt
        .elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some("ac/button"))
        .expect("the content is still drawn while it closes");
    assert!(ghost.exiting && ghost.inert, "{ghost:?}");
    let clip = rt
        .elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some("ac/content-clip"))
        .unwrap();
    assert!(clip.rect[3] > 0.0 && clip.rect[3] < clip.visible[3] + 0.01);
}

#[test]
fn the_height_springs_between_the_oracle_end_states() {
    for open in [false, true] {
        let mut rt = runtime(App {
            open,
            ..App::default()
        });
        let at_rest = part(&rt, "item").unwrap().rect[3];
        click(&mut rt);
        let mut heights = vec![];
        for _ in 0..40 {
            rt.frame(0.02);
            heights.push(part(&rt, "item").unwrap().rect[3]);
        }
        let end = part(&rt, "item").unwrap().target[3];
        let (lo, hi) = (at_rest.min(end), at_rest.max(end));
        assert!(hi - lo > BODY, "closed and open differ by the content");
        assert!(
            heights.iter().any(|h| *h > lo + 1.0 && *h < hi - 1.0),
            "opens={}: mid-flight heights {heights:?} lie strictly between {lo} and {hi}",
            !open
        );
        assert!(
            heights.iter().all(|h| *h >= lo - 0.01 && *h <= hi + 0.01),
            "no overshoot: {heights:?}"
        );
        for pair in heights.windows(2) {
            let d = pair[1] - pair[0];
            assert!(
                if open { d <= 0.01 } else { d >= -0.01 },
                "monotonic: {heights:?}"
            );
        }
        assert!((heights[39] - end).abs() < 0.5, "settles on the end state");
        // The divider follows the height; the title and chevron do not move.
        let d = part(&rt, "divider").unwrap();
        assert!((d.target[1] + 1.0 - (part(&rt, "item").unwrap().target[1] + end)).abs() < 0.5);
    }
}

#[test]
fn reduced_motion_lands_at_once() {
    for open in [false, true] {
        let mut rt = runtime(App {
            open,
            ..App::default()
        });
        rt.set_system_reduced_motion(true);
        rt.frame(0.05);
        click(&mut rt);
        rt.frame(0.001);
        let item = part(&rt, "item").unwrap();
        assert_eq!(item.rect[3], item.target[3], "opens={}", !open);
        let want = if open { 49.0 } else { 49.0 + BODY + 25.0 };
        assert_eq!(item.rect[3], want);
    }
}

#[test]
fn defaults_and_part_keys() {
    let rt = runtime(App {
        open: true,
        ..App::default()
    });
    for k in [
        "item",
        "header",
        "disclosure-icon",
        "title",
        "content",
        "divider",
    ] {
        assert!(part(&rt, k).is_some(), "{k}");
    }
    let content = part(&rt, "content").unwrap();
    assert_eq!(content.role, "region");
    assert_eq!(content.label.as_deref(), Some("Label"));
    assert_eq!(part(&rt, "header").unwrap().label.as_deref(), Some("Label"));
}

#[test]
fn a_held_enter_toggles_once() {
    let mut rt = runtime(App::default());
    rt.key(KeyEvent::from(Key::Tab));
    rt.key(KeyEvent::from(Key::Enter));
    rt.frame(0.0);
    for _ in 0..3 {
        rt.repeat_key(KeyEvent::from(Key::Enter));
        rt.frame(0.0);
    }
    assert_eq!(rt.state.toggles, 1);
}

#[test]
fn a_long_title_wraps_and_keeps_the_chevron() {
    let mut rt = runtime(App {
        long_title: true,
        ..App::default()
    });
    settle(&mut rt);
    let (item, icon, title) = (
        part(&rt, "item").unwrap(),
        part(&rt, "disclosure-icon").unwrap(),
        part(&rt, "title").unwrap(),
    );
    assert_eq!(icon.target[2], 10.0, "chevron keeps its size");
    assert!(
        title.target[0] + title.target[2] <= item.target[0] + item.target[2],
        "title stays inside: {title:?}"
    );
    assert!(
        title.target[3] > 20.0,
        "wrapped to more than one line: {}",
        title.target[3]
    );
}
