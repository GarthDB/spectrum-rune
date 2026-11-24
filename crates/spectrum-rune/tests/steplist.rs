//! Checks the Steplist against the `design-data render steplist --format scene` oracle. Regenerate the fixture with
//! `tests/fixtures/regen-steplist.sh` (its header explains the matrix). Text widths follow the installed font, so a step's width is
//! not compared: the gaps between the steps, the heights, offsets, sizes and colours are.

#[path = "support/pixels.rs"]
mod pixels;
use pixels::Pixels;
use rune::{col, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::steplist::{Step, StepState, Steplist};
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

#[derive(Clone, Default)]
struct App {
    case: Value,
    went: Vec<usize>,
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
    use StepState::*;
    col().padding(20.0).child(
        Steplist::new("sl")
            .size(match c["size"].as_str().unwrap() {
                "s" => Size::S,
                "m" => Size::M,
                "l" => Size::L,
                _ => Size::XL,
            })
            .step(Step::new("First", Completed))
            .step(Step::new("Second", Current))
            .step(Step::new("Third", Completed).disabled(true))
            .step(Step::new("Fourth", Current).disabled(true))
            .step(Step::new("Fifth", Incomplete))
            .step(Step::new("Sixth", Incomplete).disabled(true))
            .on_select(|s: &mut App, i| s.went.push(i))
            .build(t),
    )
}

fn runtime(c: &Value) -> Runtime<App> {
    let mut rt = Runtime::new(
        App {
            case: c.clone(),
            went: Vec::new(),
        },
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
    let key = format!("sl/{name}");
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
        serde_json::from_str(include_str!("fixtures/steplist_scene.json")).unwrap();
    assert_eq!(cases.len(), 16);
    for c in &cases {
        let what = format!("{} {} {}", c["scheme"], c["scale"], c["size"]);
        let rt = runtime(c);
        let px = Pixels::of(&rt, (600, 300));
        let ct = part(&rt, "container").unwrap();
        near(ct.target[3], &c["container"]["h"], &format!("{what} h"));
        for (i, want) in c["steps"].as_array().unwrap().iter().enumerate() {
            let n = i + 1;
            let what = format!("{what} step {n}");
            let step = part(&rt, &format!("step-{n}")).unwrap();
            near(
                step.target[1] - ct.target[1],
                &want["step"]["y"],
                &format!("{what} y"),
            );
            near(step.target[3], &want["step"]["h"], &format!("{what} h"));
            if n == 1 {
                near(
                    step.target[0] - ct.target[0],
                    &want["step"]["x"],
                    &format!("{what} x"),
                );
            }
            if let Some(gap) = want["step"]["gap_after"].as_f64() {
                let next = part(&rt, &format!("step-{}", n + 1)).unwrap();
                let got = next.target[0] - (step.target[0] + step.target[2]);
                assert!(
                    (f64::from(got) - gap).abs() <= 1.0,
                    "{what} gap to the next step: got {got}, want {gap}"
                );
            }
            let l = part(&rt, &format!("step-{n}.label")).unwrap();
            near(
                l.target[1] - step.target[1],
                &want["label"]["y"],
                &format!("{what} label y"),
            );
            near(l.target[3], &want["label"]["h"], &format!("{what} label h"));
            assert_color(
                l.text_color.unwrap(),
                want["label"]["color"].as_str().unwrap(),
                &format!("{what} label"),
            );
            let r = part(&rt, &format!("step-{n}.row")).unwrap();
            near(
                r.target[1] - step.target[1],
                &want["row"]["y"],
                &format!("{what} row y"),
            );
            near(r.target[3], &want["row"]["h"], &format!("{what} row h"));
            let v = part(&rt, &format!("step-{n}.visual")).unwrap();
            near(
                v.target[0] - r.target[0],
                &want["visual"]["x"],
                &format!("{what} circle x"),
            );
            near(
                v.target[1] - r.target[1],
                &want["visual"]["y"],
                &format!("{what} circle y"),
            );
            near(
                v.target[2],
                &want["visual"]["w"],
                &format!("{what} circle w"),
            );
            near(
                v.target[3],
                &want["visual"]["h"],
                &format!("{what} circle h"),
            );
            near(
                v.radius,
                &want["visual"]["radius"],
                &format!("{what} circle radius"),
            );
            match want["visual"]["fill"].as_str() {
                Some(f) => assert_color(v.fill, f, &format!("{what} circle fill")),
                None => assert_eq!(v.fill.a, 0.0, "{what}: the outlined circle has no fill"),
            }
            // The border is not in `ElementInfo`: the outlined circle's ring is read from pixels, one pixel in from
            // its left edge at mid height.
            if let Some(s) = want["visual"]["stroke"].as_str() {
                let (x, y) = (
                    v.target[0] as u32 + 1,
                    (v.target[1] + v.target[3] / 2.0) as u32,
                );
                let w = rgb(s);
                pixels::near(px.at(x, y), w, &format!("{what} circle stroke"));
            }
            match (part(&rt, &format!("step-{n}.track")), want.get("track")) {
                (None, None) => {}
                (Some(tr), Some(w)) => {
                    near(
                        tr.target[1] - r.target[1],
                        &w["y"],
                        &format!("{what} track y"),
                    );
                    near(tr.target[3], &w["h"], &format!("{what} track h"));
                    assert_color(
                        tr.fill,
                        w["fill"].as_str().unwrap(),
                        &format!("{what} track"),
                    );
                    assert!(
                        (tr.target[0] - (v.target[0] + v.target[2])).abs() <= 1.0,
                        "{what}: the track starts at the circle"
                    );
                    assert!(
                        (tr.target[0] + tr.target[2] - (step.target[0] + step.target[2])).abs()
                            <= 1.0,
                        "{what}: the track runs to the step's edge"
                    );
                }
                (got, want) => panic!(
                    "{what}: track drawn {}, oracle {}",
                    got.is_some(),
                    want.is_some()
                ),
            }
        }
    }
}

#[test]
fn press_enter_and_arrows_select_by_index_and_a_disabled_step_is_inert() {
    let c = serde_json::json!({"scheme":"light","scale":"desktop","size":"m"});
    let mut rt = runtime(&c);
    for n in [2, 3, 1] {
        let (x, y) = rt.find(&format!("sl/step-{n}")).unwrap();
        rt.pointer_move(x, y);
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(0.05);
    }
    assert_eq!(rt.state.went, [1, 0], "Second and First; Third is disabled");
    rt.pointer_move(590.0, 290.0);
    rt.key(KeyEvent::from(Key::Tab));
    rt.frame(0.05);
    rt.key(KeyEvent::from(Key::Right));
    rt.frame(0.05);
    assert_eq!(rt.state.went, [1, 0], "arrows only move focus");
    rt.key(KeyEvent::from(Key::Enter));
    rt.frame(0.05);
    assert_eq!(
        rt.state.went,
        [1, 0, 1],
        "Tab to First, Right to Second, Enter"
    );
    assert!(!part(&rt, "step-3").unwrap().focusable);
}
