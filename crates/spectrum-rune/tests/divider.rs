//! Checks the Divider against the `design-data render divider --format scene` oracle. Regenerate the fixture with
//! `tests/fixtures/regen-divider-link.sh`.

use rune::{col, Cx, El, Env, Runtime};
use serde_json::Value;
use spectrum_rune::divider::{Divider, Orientation};
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

#[derive(Clone)]
struct App {
    case: Value,
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
    col().padding(20.0).child(
        Divider::new("dv")
            .size(match c["size"].as_str().unwrap() {
                "s" => Size::S,
                "m" => Size::M,
                _ => Size::L,
            })
            .orientation(if c["orientation"] == "vertical" {
                Orientation::Vertical
            } else {
                Orientation::Horizontal
            })
            .length(16.0)
            .build(t),
    )
}

fn rgb(s: &str) -> [f32; 3] {
    let v: Vec<f32> = s.split(',').map(|c| c.parse().unwrap()).collect();
    [v[0], v[1], v[2]]
}

#[test]
fn matches_oracle_scene() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/divider_scene.json")).unwrap();
    assert_eq!(cases.len(), 24);
    for c in &cases {
        let what = format!(
            "{} {} {} {}",
            c["scheme"], c["scale"], c["size"], c["orientation"]
        );
        let mut rt = Runtime::new(
            App { case: c.clone() },
            view,
            (200.0, 200.0),
            Env::new(c["scheme"] == "dark", 1.0),
        );
        rt.frame(0.05);
        let line = rt
            .elements()
            .into_iter()
            .find(|e| e.key.as_deref() == Some("dv"))
            .unwrap();
        let w = &c["line"];
        let near = |got: f32, want: &Value, n: &str| {
            let want = want.as_f64().unwrap() as f32;
            assert!(
                (got - want).abs() <= 1.0,
                "{what} {n}: got {got}, want {want}"
            );
        };
        near(line.target[2], &w["w"], "w");
        near(line.target[3], &w["h"], "h");
        near(line.radius, &w["radius"], "radius");
        let hex = line.fill.hex();
        let want = rgb(w["fill"].as_str().unwrap());
        for i in 0..3 {
            let g = i32::from_str_radix(&hex[1 + 2 * i..3 + 2 * i], 16).unwrap() as f32;
            assert!(
                (g - want[i]).abs() <= 3.0,
                "{what} fill: got {hex}, want {want:?}"
            );
        }
    }
}
