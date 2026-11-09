//! The button-like roots hug their content in a column and centre in a row (a14.22).

use rune::{col, row, text, Cx, El, Env, Runtime};
use spectrum_rune::action_button::ActionButton;
use spectrum_rune::badge::Badge;
use spectrum_rune::button::Button;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};

fn items() -> Vec<(&'static str, El)> {
    let t = tokens(ColorScheme::Light, Scale::Desktop);
    vec![
        ("b", Button::new("b").label("Hi").build(t)),
        ("a", ActionButton::new("a").label("Hi").build(t)),
        ("g", Badge::new("g").label("Hi").build(t)),
    ]
}

fn in_row(_: &(), _: &mut Cx<()>) -> El {
    let tall = text("tall").h(100.0);
    row()
        .key("r")
        .w(400.0)
        .h(100.0)
        .center_y()
        .children(items().into_iter().map(|(_, e)| e))
        .child(tall)
}

fn in_col(_: &(), _: &mut Cx<()>) -> El {
    col().w(400.0).children(items().into_iter().map(|(_, e)| e))
}

fn rects(view: fn(&(), &mut Cx<()>) -> El) -> Vec<(String, [f32; 4])> {
    let mut rt = Runtime::new((), view, (400.0, 400.0), Env::new(false, 1.0));
    rt.frame(0.05);
    rt.elements()
        .into_iter()
        .filter_map(|e| {
            Some((
                e.key.filter(|k| k == "r" || k.ends_with("/container"))?,
                e.rect,
            ))
        })
        .collect()
}

#[test]
fn centred_in_a_row_next_to_a_taller_sibling() {
    let r = rects(in_row);
    assert_eq!(r.len(), 4);
    let mid = r
        .iter()
        .find(|(k, _)| k == "r")
        .map(|(_, [_, y, _, h])| y + h / 2.0)
        .unwrap();
    for (k, [_, y, _, h]) in r.iter().filter(|(k, _)| k != "r") {
        assert!(
            (y + h / 2.0 - mid).abs() < 0.5,
            "{k}: y={y} h={h} row mid={mid}"
        );
    }
}

#[test]
fn keeps_its_content_width_in_a_column() {
    let r = rects(in_col);
    assert_eq!(r.len(), 3);
    for (k, [x, _, w, _]) in r {
        assert!(x < 0.5 && w < 200.0, "{k}: x={x} w={w}");
    }
}

fn narrow(_: &(), _: &mut Cx<()>) -> El {
    let t = tokens(ColorScheme::Light, Scale::Desktop);
    let b = |k: &str| Button::new(k).label("A rather long label").build(t);
    col()
        .w(400.0)
        .child(row().key("r").w(60.0).child(b("rb")))
        .child(col().key("c").w(60.0).child(b("cb")))
}

/// The wrapper must not squeeze the root: in a parent narrower than the content it overflows
/// exactly as the bare root did (width 142 on main too).
#[test]
fn does_not_shrink_in_a_narrow_parent() {
    let wide = rects(|_, _| {
        let t = tokens(ColorScheme::Light, Scale::Desktop);
        col()
            .w(400.0)
            .child(Button::new("w").label("A rather long label").build(t))
    });
    let want = wide[0].1[2];
    for (k, [_, _, w, _]) in rects(narrow)
        .iter()
        .filter(|(k, _)| k.ends_with("/container"))
    {
        assert!((w - want).abs() < 0.5, "{k}: w={w} want {want}");
    }
}
