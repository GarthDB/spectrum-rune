//! Checks the Text area against the `design-data render text-area --format scene` oracle. Regenerate the fixture with
//! `tests/fixtures/regen-text-area.sh` (its header explains the matrix). Text widths follow the installed font, so only
//! heights, offsets, sizes and colours are compared; the border and the focus ring are read from pixels.

#[path = "support/pixels.rs"]
mod pixels;
use pixels::{probe, Pixels, Ring, Side, SIDES};
use rune::{col, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime, TextEdit};
use serde_json::Value;
use spectrum_rune::text_field::TextArea;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

#[derive(Clone, Default)]
struct App {
    case: Value,
    edit: TextEdit,
    submitted: u32,
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
    let mut f = TextArea::new("tf", &a.edit)
        .label("Name")
        .help_text("Your full name")
        .size(match c["size"].as_str().unwrap() {
            "s" => Size::S,
            "m" => Size::M,
            "l" => Size::L,
            _ => Size::XL,
        })
        .on_edit(|s: &mut App| &mut s.edit);
    match c["kind"].as_str().unwrap() {
        "valid" => f = f.valid(true),
        "error" => f = f.error(true),
        "disabled" => f = f.disabled(true),
        _ => {}
    }
    col().padding(20.0).child(f.build(t))
}

fn runtime(c: &Value) -> Runtime<App> {
    let mut rt = Runtime::new(
        App {
            case: c.clone(),
            edit: TextEdit::new("Ada"),
            submitted: 0,
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
    let key = format!("tf/{name}");
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

const SIZE: (u32, u32) = (600, 300);

fn field_rect(rt: &Runtime<App>) -> [f32; 4] {
    part(rt, "field").unwrap().target
}

/// Whether the border, read one pixel in from the field's left edge, is `want` (the stroke is two points wide).
fn border_is(rt: &Runtime<App>, want: &str, what: &str) {
    let px = Pixels::of(rt, SIZE);
    let r = field_rect(rt);
    pixels::near(
        probe(&px, r, Side::Left, 1),
        rgb(want),
        &format!("{what}: border"),
    );
}

#[test]
fn matches_oracle_scene() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/text_area_scene.json")).unwrap();
    assert_eq!(cases.len(), 96);
    for c in &cases {
        if c["kind"] == "hover" || c["kind"] == "focus" {
            continue;
        }
        let what = format!("{} {} {} {}", c["scheme"], c["scale"], c["size"], c["kind"]);
        let rt = runtime(c);
        let ct = part(&rt, "container").unwrap();
        near(ct.target[2], &c["container"]["w"], &format!("{what} w"));
        near(ct.target[3], &c["container"]["h"], &format!("{what} h"));
        let rel = |e: &ElementInfo| (e.target[0] - ct.target[0], e.target[1] - ct.target[1]);
        let lab = part(&rt, "field-label/container").unwrap();
        near(rel(&lab).1, &c["label"]["y"], &format!("{what} label y"));
        near(lab.target[3], &c["label"]["h"], &format!("{what} label h"));
        let lt = part(&rt, "field-label/label").unwrap();
        near(
            rel(&lt).1,
            &c["label"]["text_y"],
            &format!("{what} label text y"),
        );
        assert_color(
            lt.text_color.unwrap(),
            c["label"]["color"].as_str().unwrap(),
            &format!("{what} label"),
        );
        let f = part(&rt, "field").unwrap();
        near(rel(&f).0, &c["field"]["x"], &format!("{what} field x"));
        near(rel(&f).1, &c["field"]["y"], &format!("{what} field y"));
        near(f.target[2], &c["field"]["w"], &format!("{what} field w"));
        near(f.target[3], &c["field"]["h"], &format!("{what} field h"));
        near(
            f.radius,
            &c["field"]["radius"],
            &format!("{what} field radius"),
        );
        assert_color(
            f.fill,
            c["field"]["fill"].as_str().unwrap(),
            &format!("{what} field fill"),
        );
        border_is(&rt, c["field"]["stroke"].as_str().unwrap(), &what);
        let v = part(&rt, "value").unwrap();
        near(
            v.target[0] - f.target[0],
            &c["value"]["x"],
            &format!("{what} value x"),
        );
        near(
            v.target[1] - f.target[1],
            &c["value"]["y"],
            &format!("{what} value y"),
        );
        near(v.target[3], &c["value"]["h"], &format!("{what} value h"));
        assert_color(
            v.text_color.unwrap(),
            c["value"]["color"].as_str().unwrap(),
            &format!("{what} value"),
        );
        match (part(&rt, "validation-marker"), c.get("marker")) {
            (None, None) => {}
            (Some(m), Some(w)) => {
                near(
                    (f.target[0] + f.target[2]) - (m.target[0] + m.target[2]),
                    &w["x_from_right"],
                    &format!("{what} marker x"),
                );
                near(
                    m.target[1] - f.target[1],
                    &w["y"],
                    &format!("{what} marker y"),
                );
                near(m.target[2], &w["w"], &format!("{what} marker w"));
                assert_color(
                    m.text_color.unwrap(),
                    w["color"].as_str().unwrap(),
                    &format!("{what} marker"),
                );
            }
            (got, want) => panic!(
                "{what}: marker drawn {}, oracle {}",
                got.is_some(),
                want.is_some()
            ),
        }
        let h = part(&rt, "help-text/container").unwrap();
        near(rel(&h).1, &c["help"]["y"], &format!("{what} help y"));
        near(h.target[3], &c["help"]["h"], &format!("{what} help h"));
        let ht = part(&rt, "help-text/text").unwrap();
        assert_color(
            ht.text_color.unwrap(),
            c["help"]["color"].as_str().unwrap(),
            &format!("{what} help"),
        );
        assert_eq!(
            part(&rt, "help-text/icon").is_some(),
            c.get("help_icon").is_some(),
            "{what}: help icon"
        );
    }
}

#[test]
fn hover_takes_the_stronger_border_and_keyboard_focus_the_ring() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/text_area_scene.json")).unwrap();
    for c in cases
        .iter()
        .filter(|c| c["kind"] == "hover" || c["kind"] == "focus")
    {
        let what = format!("{} {} {} {}", c["scheme"], c["scale"], c["size"], c["kind"]);
        let mut rt = runtime(c);
        let r = field_rect(&rt);
        if c["kind"] == "hover" {
            rt.pointer_move(r[0] + 6.0, r[1] + r[3] / 2.0);
            for _ in 0..5 {
                rt.frame(0.05);
            }
            border_is(&rt, c["field"]["stroke"].as_str().unwrap(), &what);
            let v = part(&rt, "value").unwrap();
            assert_color(
                v.text_color.unwrap(),
                c["value"]["color"].as_str().unwrap(),
                &format!("{what} value"),
            );
        } else {
            rt.pointer_move(590.0, 290.0);
            rt.key(KeyEvent::from(Key::Tab));
            for _ in 0..5 {
                rt.frame(0.05);
            }
            border_is(&rt, c["field"]["stroke"].as_str().unwrap(), &what);
            let ring = &c["ring"];
            let want = rgb(ring["stroke"].as_str().unwrap());
            let r = field_rect(&rt);
            let px = Pixels::of(&rt, SIZE);
            // The field sits a hair off the pixel grid (the label's line height); Rune snaps its border to the nearest pixel.
            let r = [r[0].round(), r[1].round(), r[2].round(), r[3].round()];
            Ring {
                stroke: want,
                width: 2,
                gap: 2,
            }
            .assert_around(&px, r, &SIDES, &what);
        }
    }
}

#[test]
fn enter_breaks_the_line_and_the_field_grows_with_the_text() {
    let c = serde_json::json!({"scheme":"light","scale":"desktop","size":"m","kind":"rest"});
    let mut rt = runtime(&c);
    let one = part(&rt, "field").unwrap().target[3];
    rt.key(KeyEvent::from(Key::Tab));
    rt.frame(0.05);
    rt.key(KeyEvent::from(Key::Enter));
    rt.key(KeyEvent::from(Key::Text("more".into())));
    for _ in 0..5 {
        rt.frame(0.05);
    }
    assert_eq!(rt.state.edit.text, "Ada\nmore");
    assert_eq!(rt.state.submitted, 0, "Enter is a line break, not a submit");
    let two = part(&rt, "field").unwrap().target[3];
    assert!(
        two > one + 10.0,
        "the field grows with a second line: {one} -> {two}"
    );
    let d = serde_json::json!({"scheme":"light","scale":"desktop","size":"m","kind":"disabled"});
    let mut rt = runtime(&d);
    rt.key(KeyEvent::from(Key::Tab));
    rt.key(KeyEvent::from(Key::Text("x".into())));
    rt.frame(0.05);
    assert_eq!(rt.state.edit.text, "Ada", "a disabled area is not editable");
}
