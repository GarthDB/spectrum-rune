//! Checks the StatusLight against the `design-data render status-light --format scene` oracle: the `status-light`
//! recipe in platforms/rune/extensions/visuals. Regenerate the fixture with `tests/fixtures/regen-status-light.sh`.
//! Label wrapping is in `wrap.rs`. Not asserted: the font family (not observable) and the container width (it is the
//! label's measured width plus the dot and gap, so it follows the installed font; CI has a narrower or wider fallback).

use rune::{col, Cx, El, ElementInfo, Env, Runtime};
use serde_json::Value;
use spectrum_rune::status_light::{StatusLight, Variant};
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

#[derive(Clone)]
struct App(Value);

const VARIANTS: [(&str, Variant); 25] = [
    ("informative", Variant::Informative),
    ("accent", Variant::Accent),
    ("neutral", Variant::Neutral),
    ("positive", Variant::Positive),
    ("notice", Variant::Notice),
    ("negative", Variant::Negative),
    ("indigo", Variant::Indigo),
    ("celery", Variant::Celery),
    ("chartreuse", Variant::Chartreuse),
    ("yellow", Variant::Yellow),
    ("magenta", Variant::Magenta),
    ("fuchsia", Variant::Fuchsia),
    ("purple", Variant::Purple),
    ("seafoam", Variant::Seafoam),
    ("pink", Variant::Pink),
    ("turquoise", Variant::Turquoise),
    ("cinnamon", Variant::Cinnamon),
    ("brown", Variant::Brown),
    ("silver", Variant::Silver),
    ("gray", Variant::Gray),
    ("red", Variant::Red),
    ("orange", Variant::Orange),
    ("green", Variant::Green),
    ("cyan", Variant::Cyan),
    ("blue", Variant::Blue),
];

fn view(a: &App, _: &mut Cx<App>) -> El {
    let c = &a.0;
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
    let size = match c["size"].as_str().unwrap() {
        "s" => Size::S,
        "m" => Size::M,
        "l" => Size::L,
        _ => Size::XL,
    };
    let variant = VARIANTS.iter().find(|(n, _)| c["variant"] == *n).unwrap().1;
    col().child(
        StatusLight::new("k", "Label")
            .variant(variant)
            .size(size)
            .build(t),
    )
}

fn part(rt: &Runtime<App>, name: &str) -> ElementInfo {
    let key = format!("k/{name}");
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key.as_str()))
        .unwrap()
}

fn assert_color(got: rune::Color, want: &str, what: &str) {
    let hex = got.hex();
    for (i, w) in want.split(',').take(3).enumerate() {
        let g = i32::from_str_radix(&hex[1 + 2 * i..3 + 2 * i], 16).unwrap();
        assert!(
            (g - w.parse::<i32>().unwrap()).abs() <= 3,
            "{what}: got {hex}, want {want}"
        );
    }
}

fn close(got: f32, want: &Value, what: &str) {
    let w = want.as_f64().unwrap() as f32;
    assert!((got - w).abs() <= 1.0, "{what}: got {got}, want {w}");
}

#[test]
fn matches_oracle_scene() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/status_light_scene.json")).unwrap();
    assert_eq!(cases.len(), 124);
    for c in cases {
        let what = format!(
            "{} {} {} {}",
            c["scheme"], c["scale"], c["size"], c["variant"]
        );
        let mut rt = Runtime::new(
            App(c.clone()),
            view,
            (300.0, 100.0),
            Env::new(c["scheme"] == "dark", 1.0),
        );
        for _ in 0..10 {
            rt.frame(0.05);
        }
        let k = part(&rt, "container");
        close(
            k.target[3],
            &c["container"]["h"],
            &format!("{what}: container h"),
        );
        let d = part(&rt, "dot");
        for (i, n) in ["x", "y"].iter().enumerate() {
            close(
                d.target[i] - k.target[i],
                &c["dot"][n],
                &format!("{what}: dot {n}"),
            );
        }
        close(d.target[2], &c["dot"]["w"], &format!("{what}: dot w"));
        close(d.target[3], &c["dot"]["h"], &format!("{what}: dot h"));
        close(
            d.radius,
            &c["dot"]["radius"],
            &format!("{what}: dot radius"),
        );
        assert_color(
            d.fill,
            c["dot"]["fill"].as_str().unwrap(),
            &format!("{what}: dot fill"),
        );
        let l = part(&rt, "label");
        close(
            l.target[0] - k.target[0],
            &c["label"]["x"],
            &format!("{what}: label x"),
        );
        assert_color(
            l.text_color.unwrap(),
            c["label"]["color"].as_str().unwrap(),
            &format!("{what}: label"),
        );
    }
}
