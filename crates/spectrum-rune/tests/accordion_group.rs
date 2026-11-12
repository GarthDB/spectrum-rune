//! Checks the AccordionGroup: the open-set rules and the keyboard (no oracle exists for either),
//! and that stacked items add up to the item oracle (`accordion_scene.json`, the `rest` cases;
//! the recipe has no group part, so the group's own layout is gap 0 and nothing else).

use rune::{col, el, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime, ScrollSpec};
use serde_json::Value;
use spectrum_rune::accordion::{AccordionItem, Density};
use spectrum_rune::accordion_group::AccordionGroup;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

const BODY: f32 = 30.0;

#[derive(Clone)]
struct App {
    dark: bool,
    mobile: bool,
    size: Size,
    density: Density,
    quiet: bool,
    multiple: bool,
    disabled: bool,
    /// Item `b` is disabled.
    b_disabled: bool,
    open: Vec<String>,
    /// A focusable control in the content of `a`.
    button: bool,
    /// Five items in a short scroll viewport.
    scroll: bool,
    changes: Vec<Vec<String>>,
}

impl Default for App {
    fn default() -> Self {
        App {
            dark: false,
            mobile: false,
            size: Size::M,
            density: Density::Regular,
            quiet: false,
            multiple: false,
            disabled: false,
            b_disabled: false,
            open: vec![],
            button: false,
            scroll: false,
            changes: vec![],
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
    let item = |k: &str| {
        AccordionItem::new(k)
            .title(k)
            .content(el().key(format!("g/{k}/body")).h(BODY))
    };
    let mut a_item = item("a");
    if a.button {
        a_item = a_item.content(el().key("g/a/btn").h(20.0).focusable(true));
    }
    let mut g = AccordionGroup::new("g")
        .label("Details")
        .item(a_item)
        .item(item("b").disabled(a.b_disabled))
        .item(item("c"))
        .size(a.size)
        .density(a.density)
        .quiet(a.quiet)
        .multiple(a.multiple)
        .disabled(a.disabled)
        .open(a.open.clone())
        .on_change(|s: &mut App, open: &[String]| {
            s.open = open.to_vec();
            s.changes.push(open.to_vec());
        });
    if a.scroll {
        g = g.item(item("d")).item(item("e"));
    }
    let g = g.build(t);
    if a.scroll {
        col().child(
            col()
                .key("view")
                .h(120.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(g),
        )
    } else {
        col().child(g)
    }
}

fn runtime(a: App) -> Runtime<App> {
    let dark = a.dark;
    let mut rt = Runtime::new(a, view, (400.0, 600.0), Env::new(dark, 1.0));
    settle(&mut rt);
    rt
}

fn settle(rt: &mut Runtime<App>) {
    for _ in 0..40 {
        rt.frame(0.05);
    }
}

fn el_of(rt: &Runtime<App>, key: &str) -> Option<ElementInfo> {
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key) && !e.exiting)
}

fn key(rt: &mut Runtime<App>, k: Key) {
    rt.key(KeyEvent::from(k));
    settle(rt);
}

fn click(rt: &mut Runtime<App>, item: &str) {
    let (x, y) = rt.find(&format!("g/{item}/header")).unwrap();
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    settle(rt);
}

fn focused(rt: &Runtime<App>) -> Option<String> {
    rt.elements()
        .into_iter()
        .find(|e| e.focused)
        .and_then(|e| e.key)
}

fn on_header(rt: &Runtime<App>, item: &str) -> bool {
    focused(rt).as_deref() == Some(&format!("g/{item}/header"))
}

fn is_open(rt: &Runtime<App>, item: &str) -> bool {
    el_of(rt, &format!("g/{item}/header"))
        .unwrap()
        .expanded
        .unwrap()
}

fn open_items(rt: &Runtime<App>) -> Vec<&'static str> {
    ["a", "b", "c"]
        .into_iter()
        .filter(|i| is_open(rt, i))
        .collect()
}

fn close(got: f32, want: f32, what: &str) {
    assert!((got - want).abs() <= 1.0, "{what}: got {got}, want {want}");
}

#[test]
fn stacked_items_add_up_to_the_item_oracle() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/accordion_scene.json")).unwrap();
    let mut n = 0;
    for c in cases.iter().filter(|c| c["state"] == "rest") {
        n += 1;
        let what = format!("{} {} {}", c["scheme"], c["scale"], c["size"]);
        let open = c["open"].as_bool().unwrap();
        let quiet = c["quiet"].as_bool().unwrap();
        let density = c["density"].as_str().map_or(Density::Regular, |d| {
            *Density::ALL.iter().find(|x| x.as_str() == d).unwrap()
        });
        let rt = runtime(App {
            dark: c["scheme"] == "dark",
            mobile: c["scale"] == "mobile",
            size: match c["size"].as_str().unwrap() {
                "s" => Size::S,
                "m" => Size::M,
                "l" => Size::L,
                _ => Size::XL,
            },
            density,
            quiet,
            multiple: true,
            disabled: c["disabled"].as_bool().unwrap(),
            open: if open {
                vec!["a".into(), "b".into(), "c".into()]
            } else {
                vec![]
            },
            ..App::default()
        });
        let want = c["item"]["h"].as_f64().unwrap() as f32 + if open { BODY } else { 0.0 };
        let group = el_of(&rt, "g").unwrap().target;
        let mut y = group[1];
        for i in ["a", "b", "c"] {
            let item = el_of(&rt, &format!("g/{i}/item")).unwrap().target;
            close(item[3], want, &format!("{what} {i} height"));
            close(item[1], y, &format!("{what} {i} touches the one above"));
            close(item[2], group[2], &format!("{what} {i} width"));
            if !quiet {
                let d = el_of(&rt, &format!("g/{i}/divider")).unwrap().target;
                close(
                    d[1] + d[3],
                    item[1] + item[3],
                    &format!("{what} {i} divider"),
                );
            }
            y += item[3];
        }
        close(group[3], 3.0 * want, &format!("{what} group height"));
    }
    assert!(n > 100, "{n} rest cases");
}

#[test]
fn single_mode_keeps_one_open() {
    let mut rt = runtime(App::default());
    assert!(open_items(&rt).is_empty(), "none open is allowed");
    click(&mut rt, "a");
    assert_eq!(open_items(&rt), ["a"]);
    click(&mut rt, "c");
    assert_eq!(open_items(&rt), ["c"], "opening another closes the first");
    click(&mut rt, "c");
    assert!(
        open_items(&rt).is_empty(),
        "pressing the open one closes it"
    );
    assert_eq!(rt.state.changes, [vec!["a"], vec!["c"], Vec::<&str>::new()]);
}

#[test]
fn multiple_mode_toggles_membership_in_item_order() {
    let mut rt = runtime(App {
        multiple: true,
        ..App::default()
    });
    click(&mut rt, "c");
    click(&mut rt, "a");
    assert_eq!(open_items(&rt), ["a", "c"]);
    click(&mut rt, "c");
    assert_eq!(open_items(&rt), ["a"]);
    assert_eq!(rt.state.changes, [vec!["c"], vec!["a", "c"], vec!["a"]]);
}

#[test]
fn the_app_owns_the_open_set() {
    let rt = runtime(App {
        open: vec!["b".into(), "nope".into()],
        ..App::default()
    });
    assert_eq!(open_items(&rt), ["b"], "unknown values are ignored");
    assert!(rt.state.changes.is_empty());
}

#[test]
fn group_settings_reach_every_item() {
    let rt = runtime(App {
        quiet: true,
        size: Size::XL,
        density: Density::Spacious,
        ..App::default()
    });
    let g = el_of(&rt, "g").unwrap();
    assert_eq!(g.role, "group");
    assert_eq!(g.label.as_deref(), Some("Details"));
    for i in ["a", "b", "c"] {
        assert!(el_of(&rt, &format!("g/{i}/divider")).is_none(), "quiet");
    }
    let h = |i: &str| el_of(&rt, &format!("g/{i}/header")).unwrap().target[3];
    assert!(h("a") == h("b") && h("b") == h("c") && h("a") > 60.0);
}

#[test]
fn every_enabled_header_is_a_tab_stop_and_arrows_move_between_them() {
    let mut rt = runtime(App {
        b_disabled: true,
        ..App::default()
    });
    key(&mut rt, Key::Tab);
    assert!(on_header(&rt, "a"));
    key(&mut rt, Key::Tab);
    assert!(on_header(&rt, "c"), "Tab skips the disabled item");
    key(&mut rt, Key::Down);
    assert!(on_header(&rt, "a"), "Down wraps and skips b");
    key(&mut rt, Key::Up);
    assert!(on_header(&rt, "c"), "Up wraps");
    key(&mut rt, Key::Up);
    assert!(on_header(&rt, "a"), "Up skips b");
    key(&mut rt, Key::End);
    assert!(on_header(&rt, "c"));
    key(&mut rt, Key::Home);
    assert!(on_header(&rt, "a"));
    key(&mut rt, Key::Right);
    assert!(on_header(&rt, "a"), "Right does not move");
    assert!(rt.state.changes.is_empty(), "arrows never toggle");
    let (a, c) = (
        el_of(&rt, "g/a/header").unwrap(),
        el_of(&rt, "g/c/header").unwrap(),
    );
    assert!(a.focus_visible && a.focusable && c.focusable);
    assert!(!el_of(&rt, "g/b/header").unwrap().focusable);
}

#[test]
fn space_and_enter_toggle_the_focused_header() {
    let mut rt = runtime(App::default());
    key(&mut rt, Key::Tab);
    key(&mut rt, Key::Down);
    key(&mut rt, Key::Enter);
    assert_eq!(open_items(&rt), ["b"]);
    key(&mut rt, Key::Text(" ".into()));
    assert!(open_items(&rt).is_empty());
}

#[test]
fn a_control_in_an_open_panel_keeps_its_keys_and_its_tab_stop() {
    let mut rt = runtime(App {
        button: true,
        open: vec!["a".into()],
        ..App::default()
    });
    key(&mut rt, Key::Tab);
    key(&mut rt, Key::Tab);
    assert_eq!(focused(&rt).as_deref(), Some("g/a/btn"));
    key(&mut rt, Key::Down);
    key(&mut rt, Key::End);
    assert_eq!(
        focused(&rt).as_deref(),
        Some("g/a/btn"),
        "arrows are not the group's while a panel control has focus"
    );
    key(&mut rt, Key::Tab);
    assert!(on_header(&rt, "b"));
}

#[test]
fn arrows_reveal_a_header_in_a_scroll_viewport() {
    let mut rt = runtime(App {
        scroll: true,
        ..App::default()
    });
    key(&mut rt, Key::Tab);
    key(&mut rt, Key::End);
    assert_eq!(focused(&rt).as_deref(), Some("g/e/header"));
    let view = el_of(&rt, "view").unwrap().rect;
    let h = el_of(&rt, "g/e/header").unwrap();
    assert!(
        h.rect[1] >= view[1] - 0.5 && h.rect[1] + h.rect[3] <= view[1] + view[3] + 0.5,
        "header {:?} inside viewport {view:?}",
        h.rect
    );
    key(&mut rt, Key::Home);
    let h = el_of(&rt, "g/a/header").unwrap();
    assert!(h.rect[1] >= view[1] - 0.5, "scrolled back: {:?}", h.rect);
}

#[test]
fn a_disabled_group_is_inert() {
    let mut rt = runtime(App {
        disabled: true,
        ..App::default()
    });
    key(&mut rt, Key::Tab);
    assert_eq!(focused(&rt), None);
    click(&mut rt, "a");
    assert!(rt.state.changes.is_empty() && open_items(&rt).is_empty());
}

#[test]
fn a_disabled_item_does_not_toggle() {
    let mut rt = runtime(App {
        b_disabled: true,
        ..App::default()
    });
    click(&mut rt, "b");
    assert!(rt.state.changes.is_empty());
    click(&mut rt, "a");
    assert_eq!(open_items(&rt), ["a"]);
}

#[test]
#[cfg(debug_assertions)] // a debug_assert!: release builds do not check
#[should_panic(expected = "duplicate item key")]
fn duplicate_item_keys_are_rejected() {
    let _ = AccordionGroup::new("g")
        .item(AccordionItem::new("a"))
        .item(AccordionItem::new("a"));
}

#[test]
fn stale_state_with_two_open_in_single_mode_shows_the_first() {
    let mut rt = runtime(App {
        open: vec!["c".into(), "b".into()],
        ..App::default()
    });
    assert_eq!(open_items(&rt), ["b"]);
    click(&mut rt, "b");
    assert!(
        rt.state.open.is_empty(),
        "pressing the shown one closes all"
    );
}
