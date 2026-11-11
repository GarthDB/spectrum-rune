//! Several switches and checkboxes in one parent must not share sibling keys. Rune repairs a
//! duplicate by renaming the later key `<key>#dupN` (and prints a notice once), which the headless
//! runtime does not expose as a warning list, so the renamed keys are what this checks.

use rune::{col, Cx, El, Env, Runtime};
use spectrum_rune::accordion::AccordionItem;
use spectrum_rune::checkbox::Checkbox;
use spectrum_rune::radio_group::RadioGroup;
use spectrum_rune::switch::Switch;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};

fn view(_: &(), _: &mut Cx<()>) -> El {
    let t = tokens(ColorScheme::Light, Scale::Desktop);
    col()
        .child(Switch::new("sw1").label("One").build(t))
        .child(Switch::new("sw2").label("Two").build(t))
        .child(Checkbox::new("cb1").label("One").build(t))
        .child(Checkbox::new("cb2").label("Two").selected(true).build(t))
        .child(AccordionItem::new("ac1").title("One").open(true).build(t))
        .child(AccordionItem::new("ac2").title("Two").build(t))
        .child(
            RadioGroup::new("rg1")
                .item("a", "A")
                .item("b", "B")
                .build(t),
        )
        .child(
            RadioGroup::new("rg2")
                .item("a", "A")
                .item("b", "B")
                .build(t),
        )
}

#[test]
fn two_switches_and_two_checkboxes_share_a_parent() {
    let mut rt = Runtime::new((), view, (300.0, 300.0), Env::new(false, 1.0));
    rt.frame(0.05);
    let keys: Vec<String> = rt.elements().into_iter().filter_map(|e| e.key).collect();
    assert!(
        keys.iter().all(|k| !k.contains("#dup")),
        "duplicate sibling key repaired: {keys:?}"
    );
    for k in [
        "sw1/track",
        "sw2/track",
        "cb1/box",
        "cb2/box",
        "cb2/label",
        "ac1/header",
        "ac2/header",
        "ac2/divider",
        "rg1/a/control",
        "rg2/a/control",
    ] {
        assert!(keys.iter().any(|x| x == k), "{k} missing in {keys:?}");
    }
}
