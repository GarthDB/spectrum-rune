//! Checks the Field label against the `design-data render field-label --format scene` oracle. Regenerate the fixture with
//! `tests/fixtures/regen-field-label.sh` (its header explains the matrix). Text widths follow the installed font, so only heights,
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
    use spectrum_rune::field_label::{FieldLabel, NecessityIndicator};
    let mut l = FieldLabel::new("fl", "Name").size(size(c));
    match c["kind"].as_str().unwrap() {
        "required" => l = l.required(true),
        "text" => {
            l = l
                .required(true)
                .necessity_indicator(NecessityIndicator::Text)
        }
        "disabled" => l = l.disabled(true),
        _ => {}
    }
    col().padding(20.0).child(l.build(t))
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
    let key = format!("fl/{name}");
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
        serde_json::from_str(include_str!("fixtures/field_label_scene.json")).unwrap();
    assert_eq!(cases.len(), 64);
    for c in &cases {
        let what = format!("{} {} {} {}", c["scheme"], c["scale"], c["size"], c["kind"]);
        let rt = runtime(c);
        let ct = part(&rt, "container").unwrap();
        near(ct.target[3], &c["container"]["h"], &format!("{what} h"));
        let l = part(&rt, "label").unwrap();
        near(
            l.target[0] - ct.target[0],
            &c["label"]["x"],
            &format!("{what} label x"),
        );
        near(
            l.target[1] - ct.target[1],
            &c["label"]["y"],
            &format!("{what} label y"),
        );
        near(l.target[3], &c["label"]["h"], &format!("{what} label h"));
        assert_color(
            l.text_color.unwrap(),
            c["label"]["color"].as_str().unwrap(),
            &format!("{what} label"),
        );
        match (part(&rt, "necessity-indicator"), c.get("indicator")) {
            (None, None) => {}
            (Some(i), Some(w)) => {
                let gap = i.target[0] - (l.target[0] + l.target[2]);
                near(gap, &w["gap"], &format!("{what} gap to the asterisk"));
                near(
                    i.target[1] - ct.target[1],
                    &w["y"],
                    &format!("{what} asterisk y"),
                );
                near(i.target[2], &w["w"], &format!("{what} asterisk w"));
                assert_color(
                    i.text_color.unwrap(),
                    w["color"].as_str().unwrap(),
                    &format!("{what} asterisk"),
                );
            }
            (got, want) => panic!(
                "{what}: asterisk drawn {}, oracle {}",
                got.is_some(),
                want.is_some()
            ),
        }
    }
}
