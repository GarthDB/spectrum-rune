use rune::{col, color::hex, el, Cx, El, Env, Runtime};
use spectrum_rune::interaction::{Interaction, State};

#[derive(Default)]
struct App {
    pressed: u32,
}

fn view(_: &App, _: &mut Cx<App>) -> El {
    let i = Interaction::of("btn");
    let fill = match i.state() {
        State::Rest => hex("#ff0000"),
        State::Hover => hex("#00ff00"),
        State::Down => hex("#0000ff"),
    };
    let button = i.attach(
        el().key("btn")
            .size(100.0, 40.0)
            .fill(fill.unwrap())
            .on_press(|s: &mut App| s.pressed += 1),
    );
    col().w_full().h_full().child(button)
}

fn fill_hex(rt: &Runtime<App>) -> String {
    rt.elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some("btn"))
        .unwrap()
        .fill
        .hex()
}

fn settle(rt: &mut Runtime<App>) {
    for _ in 0..20 {
        rt.frame(0.05);
    }
}

#[test]
fn rest_hover_down_then_press_fires() {
    let mut rt = Runtime::new(App::default(), view, (200.0, 100.0), Env::new(false, 1.0));
    settle(&mut rt);
    assert_eq!(fill_hex(&rt), "#ff0000ff", "rest");
    let (x, y) = rt.find("btn").unwrap();
    rt.pointer_move(x, y);
    settle(&mut rt);
    assert_eq!(fill_hex(&rt), "#00ff00ff", "hover");
    rt.pointer_down(x, y);
    settle(&mut rt);
    assert_eq!(fill_hex(&rt), "#0000ffff", "down");
    rt.pointer_up(x, y);
    settle(&mut rt);
    assert_eq!(
        rt.state.pressed, 1,
        "on_press still fires beside on_drag_any"
    );
    assert_eq!(fill_hex(&rt), "#00ff00ff", "back to hover");
    rt.pointer_move(190.0, 95.0);
    settle(&mut rt);
    assert_eq!(fill_hex(&rt), "#ff0000ff", "left");
}
