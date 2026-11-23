//! Checks the Tooltip against the `design-data render tooltip --format scene` oracle. Regenerate the fixture with
//! `tests/fixtures/regen-tooltip.sh` (its header explains the matrix). Text widths depend on the installed font, so the
//! container's width is not compared.

use rune::{col, el, Cx, El, ElementInfo, Env, Runtime};
use serde_json::Value;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::tooltip::{Placement, Tooltip, Variant};

#[derive(Clone, Default)]
struct App {
    dark: bool,
    mobile: bool,
    variant: Variant,
    placement: Placement,
}

fn tokens_of(a: &App) -> &'static spectrum_rune::tokens::Tokens {
    tokens(
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
    )
}

fn view(a: &App, _: &mut Cx<App>) -> El {
    let t = tokens_of(a);
    col().padding(20.0).child(
        Tooltip::new("tt", "Label")
            .variant(a.variant)
            .placement(a.placement)
            .build(t),
    )
}

fn runtime(a: App) -> Runtime<App> {
    let dark = a.dark;
    let mut rt = Runtime::new(a, view, (400.0, 300.0), Env::new(dark, 1.0));
    for _ in 0..20 {
        rt.frame(0.05);
    }
    rt
}

fn part(rt: &Runtime<App>, name: &str) -> Option<ElementInfo> {
    let key = format!("tt/{name}");
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key.as_str()))
}

fn rgb(s: &str) -> [f32; 3] {
    let v: Vec<f32> = s.split(',').map(|c| c.parse().unwrap()).collect();
    [v[0], v[1], v[2]]
}

fn assert_color(got: rune::Color, want: &str, what: &str) {
    let hex = got.hex();
    let w = rgb(want);
    for i in 0..3 {
        let g = i32::from_str_radix(&hex[1 + 2 * i..3 + 2 * i], 16).unwrap() as f32;
        assert!((g - w[i]).abs() <= 3.0, "{what}: got {hex}, want {want}");
    }
}

fn near(got: f32, want: &Value, what: &str) {
    let w = want.as_f64().unwrap() as f32;
    assert!((got - w).abs() <= 1.0, "{what}: got {got}, want {w}");
}

#[test]
fn matches_oracle_scene() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/tooltip_scene.json")).unwrap();
    assert_eq!(cases.len(), 48);
    for c in &cases {
        let what = format!(
            "{} {} {} {}",
            c["scheme"], c["scale"], c["variant"], c["placement"]
        );
        let rt = runtime(App {
            dark: c["scheme"] == "dark",
            mobile: c["scale"] == "mobile",
            variant: match c["variant"].as_str().unwrap() {
                "neutral" => Variant::Neutral,
                "informative" => Variant::Informative,
                _ => Variant::Negative,
            },
            placement: match c["placement"].as_str().unwrap() {
                "top" => Placement::Top,
                "bottom" => Placement::Bottom,
                "left" => Placement::Left,
                _ => Placement::Right,
            },
        });
        let ct = part(&rt, "container").unwrap();
        near(ct.target[3], &c["container"]["h"], &format!("{what} h"));
        near(
            ct.radius,
            &c["container"]["radius"],
            &format!("{what} radius"),
        );
        assert_color(
            ct.fill,
            c["container"]["fill"].as_str().unwrap(),
            &format!("{what} fill"),
        );
        let l = part(&rt, "label").unwrap();
        let w = &c["label"];
        near(
            l.target[0] - ct.target[0],
            &w["x"],
            &format!("{what} label x"),
        );
        near(
            l.target[1] - ct.target[1],
            &w["y"],
            &format!("{what} label y"),
        );
        near(l.target[3], &w["h"], &format!("{what} label h"));
        assert_color(
            l.text_color.unwrap(),
            w["color"].as_str().unwrap(),
            &format!("{what} label"),
        );
        let tip = part(&rt, "tip").unwrap();
        let w = &c["tip"];
        near(
            tip.target[0] - ct.target[0],
            &w["x"],
            &format!("{what} tip x"),
        );
        near(
            tip.target[1] - ct.target[1],
            &w["y"],
            &format!("{what} tip y"),
        );
        near(tip.target[2], &w["w"], &format!("{what} tip w"));
        near(tip.target[3], &w["h"], &format!("{what} tip h"));
    }
}

#[derive(Clone, Default)]
struct Hover {
    shown: bool,
}

fn hover_view(s: &Hover, _: &mut Cx<Hover>) -> El {
    let t = tokens(ColorScheme::Light, Scale::Desktop);
    let trigger = Tooltip::bind(el().key("trigger").w(80.0).h(30.0), |s: &mut Hover, on| {
        s.shown = on
    });
    let mut root = col().w_full().h_full().padding(40.0).child(trigger);
    if s.shown {
        root = root.child(Tooltip::new("tt", "Label").anchor("trigger").build(t));
    }
    root
}

#[test]
fn the_pointer_resting_on_the_trigger_shows_it_and_leaving_hides_it() {
    let mut rt = Runtime::new(
        Hover::default(),
        hover_view,
        (400.0, 300.0),
        Env::new(false, 1.0),
    );
    rt.frame(0.05);
    let (x, y) = rt.find("trigger").unwrap();
    rt.pointer_move(x, y);
    for _ in 0..10 {
        rt.frame(0.1);
    }
    assert!(!rt.state.shown, "not shown before the delay");
    for _ in 0..10 {
        rt.frame(0.1);
    }
    assert!(rt.state.shown, "shown once the pointer has rested");
    assert!(rt.find("tt/container").is_some());
    rt.pointer_move(300.0, 250.0);
    for _ in 0..5 {
        rt.frame(0.1);
    }
    assert!(!rt.state.shown, "hidden on leaving");
}
