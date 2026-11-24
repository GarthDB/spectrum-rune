//! Checks the Help text against the `design-data render help-text --format scene` oracle. Regenerate the fixture with
//! `tests/fixtures/regen-help-text.sh` (its header explains the matrix). Text widths follow the installed font, so only heights,
//! offsets, gaps, sizes and colours are compared.

use rune::{col, Cx, El, ElementInfo, Env, Runtime};
use serde_json::Value;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

#[derive(Clone, Default)]
struct App {
    case: Value,
}

fn size(c: &Value) -> Size {
    match c["size"].as_str().unwrap() {
        "s" => Size::S,
        "m" => Size::M,
        "l" => Size::L,
        _ => Size::XL,
    }
}

fn view(a: &App, _: &mut Cx<App>) -> El {
    let c = &a.case;
    let t = tokens(
        if c["scheme"] == "dark" {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        },
        if c["scale"] == "mobile" {
            Scale::Mobile
        } else {
            Scale::Desktop
        },
    );
    use spectrum_rune::help_text::{HelpText, Variant};
    let mut h = HelpText::new("ht", "Enter a name").size(size(c));
    match c["kind"].as_str().unwrap() {
        "negative" => h = h.variant(Variant::Negative),
        "hidden" => h = h.variant(Variant::Negative).hide_icon(true),
        "disabled" => h = h.disabled(true),
        _ => {}
    }
    col().padding(20.0).child(h.build(t))
}

fn runtime(c: &Value) -> Runtime<App> {
    let mut rt = Runtime::new(
        App { case: c.clone() },
        view,
        (600.0, 300.0),
        Env::new(c["scheme"] == "dark", 1.0),
    );
    for _ in 0..20 {
        rt.frame(0.05);
    }
    rt
}

fn part(rt: &Runtime<App>, name: &str) -> Option<ElementInfo> {
    let key = format!("ht/{name}");
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
        serde_json::from_str(include_str!("fixtures/help_text_scene.json")).unwrap();
    assert_eq!(cases.len(), 64);
    for c in &cases {
        let what = format!("{} {} {} {}", c["scheme"], c["scale"], c["size"], c["kind"]);
        let rt = runtime(c);
        let ct = part(&rt, "container").unwrap();
        near(ct.target[3], &c["container"]["h"], &format!("{what} h"));
        let t = part(&rt, "text").unwrap();
        near(
            t.target[0] - ct.target[0],
            &c["text"]["x"],
            &format!("{what} text x"),
        );
        near(
            t.target[1] - ct.target[1],
            &c["text"]["y"],
            &format!("{what} text y"),
        );
        near(t.target[3], &c["text"]["h"], &format!("{what} text h"));
        assert_color(
            t.text_color.unwrap(),
            c["text"]["color"].as_str().unwrap(),
            &format!("{what} text"),
        );
        match (part(&rt, "icon"), c.get("icon")) {
            (None, None) => {}
            (Some(i), Some(w)) => {
                near(
                    i.target[0] - ct.target[0],
                    &w["x"],
                    &format!("{what} icon x"),
                );
                near(
                    i.target[1] - ct.target[1],
                    &w["y"],
                    &format!("{what} icon y"),
                );
                near(i.target[2], &w["w"], &format!("{what} icon w"));
                assert_color(
                    i.text_color.unwrap(),
                    w["color"].as_str().unwrap(),
                    &format!("{what} icon"),
                );
            }
            (got, want) => panic!(
                "{what}: icon drawn {}, oracle {}",
                got.is_some(),
                want.is_some()
            ),
        }
    }
}
