//! Label wrapping at a request width, for the five recipes that wrap since spectrum-design-data
//! #1700 to #1704 (Switch, Checkbox, Radio button, Button, Badge). The fixtures are
//! `<component>_wrap.json` (`design-data render <c> --width W`, see `fixtures/wrap.sh`).
//!
//! Compared per case: the root's width, height (so the number of lines) and, for Button, radius
//! (a pill is half the height), the control or icon's rectangle (centred on the first line), the
//! label's left edge, top and height (one line height per line). Line widths and the break
//! positions depend on the installed font (the oracle measures Source Sans 3), so the line count is
//! Rune's own, read from the label's height: the heights and the pill radius are checked for that
//! count (oracle height plus one line height per line more or fewer), and a case the oracle wraps
//! must wrap here too. With Adobe Clean installed the counts match the oracle's in every case;
//! Rune's fallback sans is wider and breaks some cases earlier.

use rune::{col, Cx, El, Env, Runtime};
use serde_json::Value;
use spectrum_rune::badge::Badge;
use spectrum_rune::button::Button;
use spectrum_rune::checkbox::Checkbox;
use spectrum_rune::icons::ADD_100;
use spectrum_rune::radio_button::RadioButton;
use spectrum_rune::switch::Switch;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

#[derive(Clone)]
struct App {
    component: &'static str,
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
    let (label, w, s) = (
        c["label"].as_str().unwrap(),
        c["width"].as_f64().unwrap() as f32,
        size(c),
    );
    col().child(match a.component {
        "switch" => Switch::new("k").label(label).size(s).width(w).build(t),
        "checkbox" => Checkbox::new("k").label(label).size(s).width(w).build(t),
        "radio_button" => RadioButton::new("k").label(label).size(s).width(w).build(t),
        "button" => Button::new("k")
            .label(label)
            .icon(&ADD_100)
            .size(s)
            .width(w)
            .build(t),
        _ => Badge::new("k").label(label).size(s).width(w).build(t),
    })
}

fn find(rt: &Runtime<App>, name: &str) -> Option<rune::ElementInfo> {
    let key = format!("k/{name}");
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key.as_str()))
}

fn close(got: f32, want: &Value, what: &str) {
    let w = want.as_f64().unwrap() as f32;
    assert!((got - w).abs() <= 1.0, "{what}: got {got}, want {w}");
}

fn check(component: &'static str, json: &str, control: &str) {
    let cases: Vec<Value> = serde_json::from_str(json).unwrap();
    assert!(cases.len() >= 10, "{component}: fixture too small");
    let mut wrapped = 0;
    for case in cases {
        let what = format!(
            "{component} {}",
            case.to_string().chars().take(150).collect::<String>()
        );
        let mut rt = Runtime::new(
            App {
                component,
                case: case.clone(),
            },
            view,
            (500.0, 300.0),
            Env::new(false, 1.0),
        );
        for _ in 0..10 {
            rt.frame(0.05);
        }
        let b = find(&rt, "container").unwrap().target;
        let lines = case["lines"].as_array().unwrap();
        let line_h = lines[0]["h"].as_f64().unwrap() as f32;
        let label = find(&rt, "label").unwrap();
        let n = (label.target[3] / line_h).round();
        assert!(
            lines.len() == 1 || n >= 2.0,
            "{what}: wrapped to {n} lines, the oracle to {}",
            lines.len()
        );
        // Height of this many lines: the oracle's, plus a line height for each line more.
        let h = case["container"]["h"].as_f64().unwrap() as f32 + (n - lines.len() as f32) * line_h;
        close(b[2], &case["container"]["w"], &format!("{what}: width"));
        close(b[3], &h.into(), &format!("{what}: height"));
        if component == "button" {
            close(
                find(&rt, "container").unwrap().radius,
                &(h / 2.0).into(),
                &format!("{what}: radius"),
            );
        }
        let rel = |e: &rune::ElementInfo, k: usize| e.target[k] - b[k];
        let part = if component == "button" {
            "icon"
        } else {
            control
        };
        if !case["control"].is_null() {
            let c = find(&rt, part).unwrap();
            for (k, n) in ["x", "y", "w", "h"].iter().enumerate() {
                let got = if k < 2 { rel(&c, k) } else { c.target[k] };
                close(got, &case["control"][n], &format!("{what}: {part} {n}"));
            }
        }
        close(rel(&label, 0), &lines[0]["x"], &format!("{what}: label x"));
        close(rel(&label, 1), &lines[0]["y"], &format!("{what}: label y"));
        wrapped += (lines.len() > 1) as usize;
    }
    assert!(
        wrapped >= 5,
        "{component}: the fixture must hold wrapped cases"
    );
}

#[test]
fn switch_wraps_like_the_oracle() {
    check("switch", include_str!("fixtures/switch_wrap.json"), "track");
}

#[test]
fn checkbox_wraps_like_the_oracle() {
    check(
        "checkbox",
        include_str!("fixtures/checkbox_wrap.json"),
        "box",
    );
}

#[test]
fn radio_button_wraps_like_the_oracle() {
    check(
        "radio_button",
        include_str!("fixtures/radio_button_wrap.json"),
        "control",
    );
}

#[test]
fn button_wraps_like_the_oracle() {
    check("button", include_str!("fixtures/button_wrap.json"), "icon");
}

#[test]
fn badge_wraps_like_the_oracle() {
    check("badge", include_str!("fixtures/badge_wrap.json"), "-");
}
