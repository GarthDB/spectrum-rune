//! Checks the Progress bar and the Meter against the `design-data render progress-bar|meter --format scene` oracle.
//! Regenerate the fixtures with `tests/fixtures/regen-progress.sh` (its header explains the matrix). Text widths depend on
//! the installed font, so a text is compared by its height, its colour and (for the value) its right edge.

use rune::{col, Cx, El, ElementInfo, Env, Runtime};
use serde_json::Value;
use spectrum_rune::meter::{Meter, Variant};
use spectrum_rune::progress_bar::ProgressBar;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

/// The value of `--option name=value` in the fixture's recorded arguments, unquoted.
fn arg(args: &str, name: &str) -> Option<String> {
    args.split("--option ").find_map(|a| {
        let a = a.trim();
        a.strip_prefix(name)
            .and_then(|r| r.strip_prefix('='))
            .map(|v| v.trim_matches('"').to_owned())
    })
}

#[derive(Clone)]
struct App {
    case: Value,
    meter: bool,
}

fn size_of(c: &Value) -> Size {
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
    let args = c["args"].as_str().unwrap();
    let value: f32 = arg(args, "value").map_or(0.0, |v| v.parse().unwrap());
    let hide = arg(args, "hideLabel").is_some();
    let value_label = arg(args, "valueLabel");
    col().child(if a.meter {
        let mut m = Meter::new("pb")
            .size(size_of(c))
            .label("Loading")
            .value(value)
            .width(200.0)
            .variant(match arg(args, "variant").as_deref() {
                Some("positive") => Variant::Positive,
                Some("notice") => Variant::Notice,
                Some("negative") => Variant::Negative,
                _ => Variant::Informative,
            });
        if let Some(h) = arg(args, "helpText") {
            m = m.help_text(h);
        }
        m.build(t)
    } else {
        let mut p = ProgressBar::new("pb")
            .size(size_of(c))
            .label("Loading")
            .width(200.0)
            .hide_label(hide)
            .indeterminate(arg(args, "isIndeterminate").is_some());
        if arg(args, "value").is_some() {
            p = p.value(value);
        }
        if let Some(l) = value_label {
            p = p.value_label(l);
        }
        p.build(t)
    })
}

fn runtime(a: App) -> Runtime<App> {
    let dark = a.case["scheme"] == "dark";
    let mut rt = Runtime::new(a, view, (400.0, 200.0), Env::new(dark, 1.0));
    for _ in 0..20 {
        rt.frame(0.05);
    }
    rt
}

fn part(rt: &Runtime<App>, name: &str) -> Option<ElementInfo> {
    let key = format!("pb/{name}");
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

fn check(cases: &[Value], meter: bool) {
    for c in cases {
        let what = format!(
            "{} {} {} {} {}",
            c["comp"], c["scheme"], c["scale"], c["size"], c["args"]
        );
        let rt = runtime(App {
            case: c.clone(),
            meter,
        });
        let ct = part(&rt, "container").unwrap();
        near(ct.target[2], &c["container"]["w"], &format!("{what} w"));
        near(ct.target[3], &c["container"]["h"], &format!("{what} h"));
        for name in ["label", "value", "help-text"] {
            let key = if name == "help-text" { "help" } else { name };
            match (part(&rt, name), c.get(key)) {
                (None, None) => {}
                (Some(e), Some(w)) => {
                    near(
                        e.target[1] - ct.target[1],
                        &w["y"],
                        &format!("{what} {name} y"),
                    );
                    near(e.target[3], &w["h"], &format!("{what} {name} h"));
                    assert_color(
                        e.text_color.unwrap(),
                        w["color"].as_str().unwrap(),
                        &format!("{what} {name}"),
                    );
                    match name {
                        "value" => {
                            // Right-aligned to the container: the oracle's right gap, not its text width.
                            let want = c["container"]["w"].as_f64().unwrap()
                                - (w["x"].as_f64().unwrap() + w["w"].as_f64().unwrap());
                            let got = (ct.target[0] + ct.target[2]) - (e.target[0] + e.target[2]);
                            assert!((f64::from(got) - want).abs() <= 1.0, "{what} value right");
                        }
                        _ => near(
                            e.target[0] - ct.target[0],
                            &w["x"],
                            &format!("{what} {name} x"),
                        ),
                    }
                }
                (got, want) => panic!(
                    "{what}: {name} drawn {}, oracle {}",
                    got.is_some(),
                    want.is_some()
                ),
            }
        }
        let tr = part(&rt, "track").unwrap();
        let w = &c["track"];
        near(
            tr.target[1] - ct.target[1],
            &w["y"],
            &format!("{what} track y"),
        );
        near(tr.target[3], &w["h"], &format!("{what} track h"));
        near(tr.radius, &w["radius"], &format!("{what} track radius"));
        assert_color(
            tr.fill,
            w["fill"].as_str().unwrap(),
            &format!("{what} track"),
        );
        let f = part(&rt, "fill").unwrap();
        let w = &c["fill"];
        near(
            f.target[0] - tr.target[0],
            &w["x"],
            &format!("{what} fill x"),
        );
        near(f.target[2], &w["w"], &format!("{what} fill w"));
        near(f.target[3], &w["h"], &format!("{what} fill h"));
        // The oracle draws an empty fill square (radius 0); an empty fill has no width to round.
        if f.target[2] > 0.5 {
            near(f.radius, &w["radius"], &format!("{what} fill radius"));
        }
        if f.target[2] > 0.5 {
            assert_color(f.fill, w["fill"].as_str().unwrap(), &format!("{what} fill"));
        }
    }
}

#[test]
fn progress_bar_matches_oracle_scene() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/progress_bar_scene.json")).unwrap();
    assert_eq!(cases.len(), 60);
    check(&cases, false);
}

#[test]
fn meter_matches_oracle_scene() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/meter_scene.json")).unwrap();
    assert_eq!(cases.len(), 68);
    check(&cases, true);
}

#[test]
fn it_is_a_named_progress_indicator() {
    let c = serde_json::json!({"comp":"progress-bar","scheme":"light","scale":"desktop","size":"m","args":"--option value=3"});
    let mut rt = runtime(App {
        case: c,
        meter: false,
    });
    rt.frame(0.05);
    let tree = rt.accessibility_json().to_string();
    let at = tree.find("\"key\":\"pb/container\"").expect(&tree);
    let node = &tree[at.saturating_sub(300)..(at + 200).min(tree.len())];
    assert!(node.contains("\"role\":\"progress-indicator\""), "{node}");
    assert!(node.contains("\"name\":\"Loading\""), "{node}");
}
