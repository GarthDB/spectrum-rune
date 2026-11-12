//! Checks the AccordionItem's direct actions (`hasActionButton`, `hasSwitch`) against the
//! `design-data render accordion` oracle; the fixture is `tests/fixtures/accordion_actions.json`
//! (see `regen-accordion.sh`). Geometry and fill/ink colours with the 1 px / 3-per-channel
//! tolerance. Not asserted: the switch track's stroke (`ElementInfo` has no border; the Switch's
//! own pixel tests cover it) and the states the oracle does not draw for these controls.

use rune::{col, el, row, Color, Cx, El, ElementInfo, Env, Runtime};
use serde_json::Value;
use spectrum_rune::accordion::{AccordionItem, Density};
use spectrum_rune::icons::ADD_100;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

#[derive(Clone)]
struct App {
    dark: bool,
    mobile: bool,
    size: Size,
    density: Density,
    open: bool,
    disabled: bool,
    button: bool,
    switch: bool,
    /// Hug the content like the oracle instead of filling a 400 wide column.
    natural: bool,
    toggles: u32,
    pressed: u32,
    switched: u32,
}

impl Default for App {
    fn default() -> Self {
        App {
            dark: false,
            mobile: false,
            size: Size::M,
            density: Density::Regular,
            open: false,
            disabled: false,
            button: false,
            switch: false,
            natural: false,
            toggles: 0,
            pressed: 0,
            switched: 0,
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
        .title("Label")
        .size(a.size)
        .density(a.density)
        .disabled(a.disabled)
        .open(a.open)
        .content(el().key("ac/body").h(30.0))
        .on_toggle(|s: &mut App| s.toggles += 1);
    if a.button {
        b = b.action_button(|x| {
            x.icon(&ADD_100)
                .label("Add")
                .on_press(|s: &mut App| s.pressed += 1)
        });
    }
    if a.switch {
        b = b.switch(|x| x.label("Enable").on_toggle(|s: &mut App| s.switched += 1));
    }
    let item = b.build(t);
    if a.natural {
        row().child(item)
    } else {
        col().child(item)
    }
}

fn runtime(a: App) -> Runtime<App> {
    let dark = a.dark;
    let mut rt = Runtime::new(a, view, (400.0, 300.0), Env::new(dark, 1.0));
    for _ in 0..40 {
        rt.frame(0.05);
    }
    rt
}

fn part(rt: &Runtime<App>, name: &str) -> Option<ElementInfo> {
    let key = format!("ac/{name}");
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key.as_str()) && !e.exiting)
}

fn close(got: f32, want: f32, what: &str) {
    assert!((got - want).abs() <= 1.0, "{what}: got {got}, want {want}");
}

fn num(v: &Value) -> f32 {
    v.as_f64().unwrap() as f32
}

fn color(got: Color, want: &Value, what: &str) {
    let want: Vec<f32> = want
        .as_str()
        .unwrap_or_else(|| panic!("{what}: oracle has no colour"))
        .split(',')
        .map(|c| c.parse().unwrap())
        .collect();
    let hex = got.hex();
    for i in 0..3 {
        let g = i32::from_str_radix(&hex[1 + 2 * i..3 + 2 * i], 16).unwrap() as f32;
        assert!(
            (g - want[i]).abs() <= 3.0,
            "{what}: got {hex}, want {want:?}"
        );
    }
}

/// Compares `e` with the oracle node `n`: right distance from the item, y and size.
fn geometry(e: &ElementInfo, item: &ElementInfo, n: &Value, what: &str) {
    let right = (item.target[0] + item.target[2]) - (e.target[0] + e.target[2]);
    close(right, num(&n["right"]), &format!("{what} right"));
    close(
        e.target[1] - item.target[1],
        num(&n["y"]),
        &format!("{what} y"),
    );
    close(e.target[2], num(&n["w"]), &format!("{what} w"));
    close(e.target[3], num(&n["h"]), &format!("{what} h"));
}

fn app_for(c: &Value) -> App {
    let acts = c["actions"].as_str().unwrap();
    App {
        dark: c["scheme"] == "dark",
        mobile: c["scale"] == "mobile",
        size: match c["size"].as_str().unwrap() {
            "s" => Size::S,
            "m" => Size::M,
            "l" => Size::L,
            _ => Size::XL,
        },
        density: *Density::ALL
            .iter()
            .find(|d| d.as_str() == c["density"].as_str().unwrap())
            .unwrap(),
        open: c["open"].as_bool().unwrap(),
        disabled: c["disabled"].as_bool().unwrap(),
        button: acts != "switch",
        switch: acts != "button",
        ..App::default()
    }
}

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("fixtures/accordion_actions.json")).unwrap()
}

#[test]
fn matches_oracle_scene() {
    let cases = cases();
    assert_eq!(cases.len(), 69);
    for c in &cases {
        let what = c.to_string();
        let what = &what[..what.len().min(140)];
        let open = c["open"].as_bool().unwrap();
        let rt = runtime(app_for(c));
        let item = part(&rt, "item").unwrap();
        let header = part(&rt, "header").unwrap();
        close(
            item.target[3],
            num(&c["item_h"]) + if open { 30.0 } else { 0.0 },
            &format!("{what} item h"),
        );
        close(
            header.target[3],
            num(&c["header_h"]),
            &format!("{what} header h"),
        );
        // The header button stops before the actions: it is a sibling, not their parent.
        assert_eq!(header.role, "button", "{what}");

        let direct = part(&rt, "direct-actions").unwrap();
        geometry(
            &direct,
            &item,
            &c["direct"],
            &format!("{what} direct-actions"),
        );
        assert!(
            direct.target[0] >= header.target[0] + header.target[2] - 1.0,
            "{what}: actions overlap the header button"
        );

        let button = part(&rt, "action-button/container");
        assert_eq!(button.is_some(), !c["button"].is_null(), "{what}: button");
        if let Some(b) = button {
            geometry(&b, &item, &c["button"], &format!("{what} button"));
            color(b.fill, &c["button"]["fill"], &format!("{what} button fill"));
            let i = part(&rt, "action-button/icon").unwrap();
            geometry(&i, &item, &c["button_icon"], &format!("{what} button icon"));
            color(
                i.text_color.unwrap(),
                &c["button_icon"]["color"],
                &format!("{what} icon ink"),
            );
            let h = part(&rt, "action-button/hold-icon").expect("hold icon is always on");
            geometry(&h, &item, &c["button_hold"], &format!("{what} hold icon"));
            color(
                h.text_color.unwrap(),
                &c["button_hold"]["color"],
                &format!("{what} hold ink"),
            );
            assert_eq!(b.focusable, !c["disabled"].as_bool().unwrap(), "{what}");
        }

        let switch = part(&rt, "switch/container");
        assert_eq!(switch.is_some(), !c["switch"].is_null(), "{what}: switch");
        if let Some(s) = switch {
            geometry(&s, &item, &c["switch"], &format!("{what} switch"));
            let t = part(&rt, "switch/track").unwrap();
            geometry(&t, &item, &c["track"], &format!("{what} track"));
            color(t.fill, &c["track"]["fill"], &format!("{what} track fill"));
            let h = part(&rt, "switch/handle").unwrap();
            geometry(&h, &item, &c["handle"], &format!("{what} handle"));
            color(h.fill, &c["handle"]["fill"], &format!("{what} handle fill"));
            assert!(
                part(&rt, "switch/label").is_none(),
                "{what}: label is hidden"
            );
            assert_eq!(s.role, "switch", "{what}");
        }
    }
}

#[test]
fn title_to_actions_gap_matches_oracle_at_natural_width() {
    let mut seen = 0;
    for c in cases()
        .iter()
        .filter(|c| c["scheme"] == "light" && c["scale"] == "desktop")
    {
        // From M up the oracle item is at its min width, so only S is content-sized.
        if c["size"] != "s"
            || c["open"] == true
            || c["disabled"] == true
            || c["density"] != "regular"
        {
            continue;
        }
        let rt = runtime(App {
            natural: true,
            ..app_for(c)
        });
        let title = part(&rt, "title").unwrap();
        let direct = part(&rt, "direct-actions").unwrap();
        let gap = direct.target[0] - (title.target[0] + title.target[2]);
        close(
            gap,
            num(&c["title_gap"]),
            &format!("{} {} gap", c["size"], c["actions"]),
        );
        seen += 1;
    }
    assert_eq!(seen, 3);
}

/// The oracle clamps M and up to the minimum width and overflows a long title, so only the gap
/// (not the wrapping or the right edge) is comparable there; ours is laid out at natural width.
#[test]
fn title_to_actions_gap_matches_oracle_in_overflow_case() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/accordion_actions_gap.json")).unwrap();
    assert_eq!(cases.len(), 3);
    for c in &cases {
        let rt = runtime(App {
            natural: true,
            switch: true,
            size: match c["size"].as_str().unwrap() {
                "m" => Size::M,
                "l" => Size::L,
                _ => Size::XL,
            },
            ..App::default()
        });
        let title = part(&rt, "title").unwrap();
        let direct = part(&rt, "direct-actions").unwrap();
        let gap = direct.target[0] - (title.target[0] + title.target[2]);
        close(gap, num(&c["gap"]), &format!("{} overflow gap", c["size"]));
    }
}

#[test]
fn default_item_has_no_direct_actions() {
    let rt = runtime(App::default());
    assert!(part(&rt, "direct-actions").is_none());
    assert!(part(&rt, "action-button/container").is_none());
    assert!(part(&rt, "switch/container").is_none());
}

#[test]
fn actions_press_without_toggling_the_item() {
    let mut rt = runtime(App {
        button: true,
        switch: true,
        ..App::default()
    });
    for key in ["ac/action-button/container", "ac/switch/container"] {
        let (x, y) = rt.find(key).unwrap();
        rt.pointer_move(x, y);
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
    }
    for _ in 0..10 {
        rt.frame(0.05);
    }
    let s = &rt.state;
    assert_eq!((s.pressed, s.switched, s.toggles), (1, 1, 0));
    let (x, y) = rt.find("ac/header").unwrap();
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(0.05);
    assert_eq!(rt.state.toggles, 1);
}

#[test]
fn disabled_item_disables_its_actions() {
    let rt = runtime(App {
        button: true,
        switch: true,
        disabled: true,
        ..App::default()
    });
    for key in ["action-button/container", "switch/container"] {
        assert!(!part(&rt, key).unwrap().focusable, "{key}");
    }
}
