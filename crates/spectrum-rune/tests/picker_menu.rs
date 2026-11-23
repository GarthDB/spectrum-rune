//! The Picker's menu: the Menu in a modal Popover under the field. Behaviour only (geometry of the pieces is checked
//! against the oracle in `popover.rs` and `menu.rs`).

use rune::{col, el, Cx, El, ElementInfo, Env, Key, KeyEvent, Runtime};
use spectrum_rune::picker::Picker;
use spectrum_rune::picker_menu::PickerMenu;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};

#[derive(Clone, Default)]
struct App {
    open: bool,
    chosen: Option<String>,
}

fn view(a: &App, _: &mut Cx<App>) -> El {
    let t = tokens(ColorScheme::Light, Scale::Desktop);
    let mut root = col()
        .key("root")
        .w_full()
        .h_full()
        .padding(40.0)
        .gap(20.0)
        .child(
            Picker::new("pm")
                .label("Country")
                .value(a.chosen.clone().unwrap_or_else(|| "Canada".into()))
                .open(a.open)
                .on_open(|s: &mut App| s.open = true)
                .build(t),
        )
        .child(el().w(100.0).h(40.0));
    if a.open {
        root = root.child(
            PickerMenu::new("pm")
                .label("Country")
                .item("ca", "Canada")
                .item("mx", "Mexico")
                .item("us", "United States")
                .selected("mx")
                .on_choose(|s: &mut App, id: &str| {
                    s.chosen = Some(id.to_owned());
                    s.open = false;
                })
                .on_close(|s: &mut App| s.open = false)
                .build(t),
        );
    }
    root
}

fn settle(rt: &mut Runtime<App>) {
    for _ in 0..20 {
        rt.frame(0.05);
    }
}

fn part(rt: &Runtime<App>, key: &str) -> Option<ElementInfo> {
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some(key))
}

fn opened() -> Runtime<App> {
    let mut rt = Runtime::new(App::default(), view, (500.0, 400.0), Env::new(false, 1.0));
    settle(&mut rt);
    let (x, y) = rt.find("pm/field").unwrap();
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    settle(&mut rt);
    rt
}

#[test]
fn pressing_the_field_shows_the_menu_under_it_lined_up_with_its_start() {
    let rt = opened();
    assert!(rt.state.open);
    let field = part(&rt, "pm/field").unwrap().target;
    let pop = part(&rt, "pm/popover/container").unwrap().target;
    assert!(
        (pop[0] - field[0]).abs() <= 1.0,
        "popover x {} vs field x {}",
        pop[0],
        field[0]
    );
    let gap = pop[1] - (field[1] + field[3]);
    assert!((gap - 4.0).abs() <= 1.0, "popover is {gap} below the field");
    assert!(
        part(&rt, "pm/popover/tip").is_none(),
        "a picker's popover has no tip"
    );
    assert_eq!(
        part(&rt, "pm/menu/menu-item-2").unwrap().label.as_deref(),
        Some("Mexico")
    );
}

#[test]
fn focus_starts_on_the_selected_item_and_arrows_and_enter_choose() {
    let mut rt = opened();
    assert!(
        part(&rt, "pm/menu/menu-item-2").unwrap().focused,
        "the selected item has focus"
    );
    rt.key(KeyEvent::from(Key::Down));
    settle(&mut rt);
    assert!(part(&rt, "pm/menu/menu-item-3").unwrap().focused);
    rt.key(KeyEvent::from(Key::Enter));
    settle(&mut rt);
    assert_eq!(rt.state.chosen.as_deref(), Some("us"));
    assert!(!rt.state.open);
    assert!(
        part(&rt, "pm/popover/container").is_none(),
        "the menu is gone"
    );
}

#[test]
fn escape_and_a_press_outside_close_it_and_focus_goes_back_to_the_field() {
    let mut rt = opened();
    rt.key(KeyEvent::from(Key::Escape));
    settle(&mut rt);
    assert!(!rt.state.open);
    assert!(
        part(&rt, "pm/field").unwrap().focused,
        "focus returns to the field"
    );
    let mut rt = opened();
    // Well clear of the field and the menu under it.
    let (x, y) = (450.0, 350.0);
    rt.pointer_move(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    settle(&mut rt);
    assert!(!rt.state.open);
    assert_eq!(rt.state.chosen, None);
}
