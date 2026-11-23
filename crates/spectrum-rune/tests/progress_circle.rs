//! Checks the Progress circle against the `design-data render progress-circle --format scene` oracle. Regenerate the
//! fixture with `tests/fixtures/regen-progress.sh`. The ring's stroke and the arc's sweep are read from pixels in
//! `progress_circle_pixels.rs`; here the size and the colours.

use rune::{col, Cx, El, ElementInfo, Env, Runtime};
use serde_json::Value;
use spectrum_rune::progress_circle::ProgressCircle;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

#[derive(Clone)]
struct App {
    case: Value,
}

pub fn build(c: &Value) -> El {
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
    let args = c["args"].as_str().unwrap();
    let mut p = ProgressCircle::new("pc").size(match c["size"].as_str().unwrap() {
        "s" => Size::S,
        "m" => Size::M,
        _ => Size::L,
    });
    if args.contains("isIndeterminate") {
        p = p.indeterminate(true);
    } else if let Some(v) = args.strip_prefix("--option value=") {
        p = p.value(v.parse().unwrap());
    }
    p.build(t)
}

fn view(a: &App, _: &mut Cx<App>) -> El {
    col().padding(10.0).child(build(&a.case))
}

fn part(rt: &Runtime<App>, name: &str) -> Option<ElementInfo> {
    let key = format!("pc/{name}");
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

#[test]
fn matches_oracle_scene() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/progress_circle_scene.json")).unwrap();
    assert_eq!(cases.len(), 60);
    for c in &cases {
        let what = format!("{} {} {} {}", c["scheme"], c["scale"], c["size"], c["args"]);
        let mut rt = Runtime::new(
            App { case: c.clone() },
            view,
            (200.0, 200.0),
            Env::new(c["scheme"] == "dark", 1.0),
        );
        for _ in 0..5 {
            rt.frame(0.05);
        }
        let track = part(&rt, "track").unwrap();
        let w = c["track"]["w"].as_f64().unwrap() as f32;
        assert!((track.target[2] - w).abs() <= 1.0, "{what} track w");
        assert!(
            (track.target[3] - c["track"]["h"].as_f64().unwrap() as f32).abs() <= 1.0,
            "{what} track h"
        );
        assert_color(
            track.text_color.unwrap(),
            c["track"]["stroke"].as_str().unwrap(),
            &format!("{what} track"),
        );
        let fill = part(&rt, "fill").unwrap();
        assert!((fill.target[2] - w).abs() <= 1.0, "{what} fill w");
        assert_color(
            fill.text_color.unwrap(),
            c["fill"]["color"].as_str().unwrap(),
            &format!("{what} fill"),
        );
    }
}
