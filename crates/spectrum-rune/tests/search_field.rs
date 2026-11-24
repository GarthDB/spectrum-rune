//! Checks the Search field against the `design-data render search-field --format scene` oracle. Regenerate the fixture with
//! `tests/fixtures/regen-search-field.sh` (its header explains the matrix). Text widths follow the installed font, so only
//! heights, offsets, sizes and colours are compared; the border and the focus ring are read from pixels.

#[path = "support/pixels.rs"]
mod pixels;
use pixels::{probe, Pixels, Ring, Side, SIDES};
use rune::{col, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime, TextEdit};
use serde_json::Value;
use spectrum_rune::text_field::SearchField;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

#[derive(Clone, Default)]
struct App {
    case: Value,
    edit: TextEdit,
    submitted: u32,
    clear_button: bool,
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
    let mut f = SearchField::new("tf", &a.edit)
        .label("Search")
        .help_text("Find things")
        .size(match c["size"].as_str().unwrap() {
            "s" => Size::S,
            "m" => Size::M,
            "l" => Size::L,
            _ => Size::XL,
        })
        .clear_button(a.clear_button)
        .on_edit(|s: &mut App| &mut s.edit)
        .on_submit(|s: &mut App, _| s.submitted += 1);
    if c["kind"] == "disabled" {
        f = f.disabled(true);
    }
    col().padding(20.0).child(f.build(t))
}

fn runtime(c: &Value) -> Runtime<App> {
    let mut rt = Runtime::new(
        App {
            case: c.clone(),
            edit: TextEdit::new("Ada"),
            submitted: 0,
            clear_button: false,
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
        serde_json::from_str(include_str!("fixtures/search_field_scene.json")).unwrap();
    assert_eq!(cases.len(), 64);
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
        let i = part(&rt, "leading-icon").unwrap();
        near(
            i.target[0] - f.target[0],
            &c["icon"]["x"],
            &format!("{what} icon x"),
        );
        near(
            i.target[1] - f.target[1],
            &c["icon"]["y"],
            &format!("{what} icon y"),
        );
        near(i.target[2], &c["icon"]["w"], &format!("{what} icon w"));
        assert_color(
            i.text_color.unwrap(),
            c["icon"]["color"].as_str().unwrap(),
            &format!("{what} icon"),
        );
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
        let h = part(&rt, "help-text/container").unwrap();
        near(rel(&h).1, &c["help"]["y"], &format!("{what} help y"));
        near(h.target[3], &c["help"]["h"], &format!("{what} help h"));
        assert_color(
            part(&rt, "help-text/text").unwrap().text_color.unwrap(),
            c["help"]["color"].as_str().unwrap(),
            &format!("{what} help"),
        );
    }
}

#[test]
fn hover_takes_the_stronger_border_and_keyboard_focus_the_ring() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/search_field_scene.json")).unwrap();
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
fn the_clear_button_shows_with_text_and_empties_the_field() {
    let c = serde_json::json!({"scheme":"light","scale":"desktop","size":"m","kind":"rest"});
    let mut rt = Runtime::new(
        App {
            case: c.clone(),
            edit: TextEdit::new("Ada"),
            submitted: 0,
            clear_button: true,
        },
        view,
        (600.0, 300.0),
        Env::new(false, 1.0),
    );
    for _ in 0..20 {
        rt.frame(0.05);
    }
    let (x, y) = rt
        .find("tf/clear-button/container")
        .expect("a clear button while there is text");
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    for _ in 0..5 {
        rt.frame(0.05);
    }
    assert_eq!(rt.state.edit.text, "");
    for _ in 0..40 {
        rt.frame(0.05);
    }
    assert!(
        rt.elements()
            .iter()
            .all(|e| e.exiting || e.key.as_deref() != Some("tf/clear-button/container")),
        "no clear button when empty"
    );
    rt.key(KeyEvent::from(Key::Tab));
    rt.key(KeyEvent::from(Key::Enter));
    rt.frame(0.05);
    assert_eq!(rt.state.submitted, 1);
}
