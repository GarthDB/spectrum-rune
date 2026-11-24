//! Checks the Standard dialog against the `design-data render standard-dialog --format scene` oracle. Regenerate the fixture with
//! `tests/fixtures/regen-standard-dialog.sh` (its header explains the matrix). Text widths follow the installed font, so only
//! heights, the heights, offsets, sizes and colours are.

use rune::{col, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::button::{Button, Style, Variant as ButtonVariant};
use spectrum_rune::checkbox::Checkbox;
use spectrum_rune::standard_dialog::{DialogSize, StandardDialog};
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};

#[derive(Clone, Default)]
struct App {
    case: Value,
    dismissed: u32,
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
    let mut d = StandardDialog::new("ad", "Rename file")
        .description("Pick a new name for the file.")
        .size(match c["size"].as_str().unwrap() {
            "s" => DialogSize::S,
            "l" => DialogSize::L,
            _ => DialogSize::M,
        })
        .actions([
            Button::new("ad/cancel")
                .label("Cancel")
                .variant(ButtonVariant::Secondary)
                .style(Style::Outline)
                .build(t),
            Button::new("ad/save").label("Save").build(t),
        ])
        .on_dismiss(|s: &mut App| s.dismissed += 1);
    if c["footer_content"] == "checkbox" {
        d = d.footer_content(Checkbox::new("ad/check").label("Don't ask again").build(t));
    }
    col().padding(20.0).child(d.build(t))
}

fn runtime(c: &Value) -> Runtime<App> {
    let mut rt = Runtime::new(
        App {
            case: c.clone(),
            dismissed: 0,
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
    let key = format!("ad/{name}");
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
        serde_json::from_str(include_str!("fixtures/standard_dialog_scene.json")).unwrap();
    assert_eq!(cases.len(), 24);
    for c in &cases {
        let what = format!(
            "{} {} {} footer {}",
            c["scheme"], c["scale"], c["size"], c["footer_content"]
        );
        let rt = runtime(c);
        let ct = part(&rt, "container").unwrap();
        near(ct.target[2], &c["container"]["w"], &format!("{what} w"));
        near(ct.target[3], &c["container"]["h"], &format!("{what} h"));
        near(
            ct.radius,
            &c["container"]["radius"],
            &format!("{what} radius"),
        );
        assert_color(
            ct.fill,
            c["container"]["fill"].as_str().unwrap(),
            &format!("{what} surface"),
        );
        let rel = |e: &ElementInfo| (e.target[0] - ct.target[0], e.target[1] - ct.target[1]);
        let header = part(&rt, "header").unwrap();
        near(
            rel(&header).0,
            &c["header"]["x"],
            &format!("{what} header x"),
        );
        near(
            rel(&header).1,
            &c["header"]["y"],
            &format!("{what} header y"),
        );
        near(
            header.target[3],
            &c["header"]["h"],
            &format!("{what} header h"),
        );
        let title = part(&rt, "title").unwrap();
        near(rel(&title).1, &c["title"]["y"], &format!("{what} title y"));
        near(
            title.target[3],
            &c["title"]["h"],
            &format!("{what} title h"),
        );
        assert_color(
            title.text_color.unwrap(),
            c["title"]["color"].as_str().unwrap(),
            &format!("{what} title"),
        );
        let d = part(&rt, "description").unwrap();
        near(
            rel(&d).0,
            &c["description"]["x"],
            &format!("{what} description x"),
        );
        near(
            rel(&d).1,
            &c["description"]["y"],
            &format!("{what} description y"),
        );
        near(
            d.target[3],
            &c["description"]["h"],
            &format!("{what} description h"),
        );
        assert_color(
            d.text_color.unwrap(),
            c["description"]["color"].as_str().unwrap(),
            &format!("{what} description"),
        );
        let f = part(&rt, "footer").unwrap();
        near(rel(&f).1, &c["footer"]["y"], &format!("{what} footer y"));
        near(f.target[3], &c["footer"]["h"], &format!("{what} footer h"));
        let g = part(&rt, "button-group").unwrap();
        match c["group"]["after_content"].as_f64() {
            // Widths follow the font: only the gap after the footer content is compared there.
            Some(gap) => {
                let fc = part(&rt, "footer-content").unwrap();
                let got = g.target[0] - (fc.target[0] + fc.target[2]);
                assert!(
                    (f64::from(got) - gap).abs() <= 1.0,
                    "{what}: gap after the footer content: got {got}, want {gap}"
                );
            }
            None => near(
                rel(&g).0,
                &c["group"]["x"],
                &format!("{what} button group x"),
            ),
        }
    }
}

#[test]
fn escape_dismisses() {
    let c =
        serde_json::json!({"scheme":"light","scale":"desktop","size":"m","footer_content":"none"});
    let mut rt = runtime(&c);
    rt.key(KeyEvent::from(Key::Tab));
    rt.frame(0.05);
    rt.key(KeyEvent::from(Key::Escape));
    rt.frame(0.05);
    assert_eq!(rt.state.dismissed, 1);
    let tree = rt.accessibility_json().to_string();
    assert!(tree.contains("Rename file"), "{tree}");
}
