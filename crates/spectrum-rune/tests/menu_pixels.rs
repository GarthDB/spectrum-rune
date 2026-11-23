//! Pixel assertions for the Menu, which `ElementInfo` cannot give: the fill an item shows while it has keyboard
//! focus (`El::focus_fill` is laid over the item, never in its own fill), read from `menu_scene.json` (from
//! `design-data render menu --format scene`), never from our `Tokens`. See `support/pixels.rs`.

#[path = "support/pixels.rs"]
mod pixels;

use pixels::{near, Pixels};
use rune::{col, Cx, El, Env, Key, KeyEvent, Runtime};
use serde_json::Value;
use spectrum_rune::menu::{Menu, MenuItem, Selection};
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

const SIZE: (u32, u32) = (300, 200);

#[derive(Clone, Default)]
struct App {
    dark: bool,
    mobile: bool,
    single: bool,
}

fn view(a: &App, _: &mut Cx<App>) -> El {
    let t = tokens(
        if a.dark {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        },
        if a.mobile {
            Scale::Mobile
        } else {
            Scale::Desktop
        },
    );
    col().child(
        Menu::new("mn")
            .size(Size::M)
            .selection(if a.single {
                Selection::Single
            } else {
                Selection::None
            })
            .item(MenuItem::new("one", "One"))
            .item(MenuItem::new("two", "Two").value("V").selected(a.single))
            .item(MenuItem::new("three", "Three").disabled(true))
            .build(t),
    )
}

fn rgb(s: &str) -> [f32; 3] {
    let v: Vec<f32> = s.split(',').map(|c| c.parse().unwrap()).collect();
    [v[0], v[1], v[2]]
}

#[test]
fn the_focused_item_shows_the_key_focus_fill_and_the_others_none() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("fixtures/menu_scene.json")).unwrap();
    let mut n = 0;
    for c in cases.iter().filter(|c| c["state"] == "keyboard-focus") {
        let want = rgb(c["items"][0]["item"]["fill"].as_str().unwrap());
        for focused in 1..=2usize {
            n += 1;
            let what = format!(
                "{} {} {} item {focused}",
                c["scheme"], c["scale"], c["variant"]
            );
            let mut rt = Runtime::new(
                App {
                    dark: c["scheme"] == "dark",
                    mobile: c["scale"] == "mobile",
                    single: c["variant"] == "single",
                },
                view,
                (SIZE.0 as f32, SIZE.1 as f32),
                Env::new(c["scheme"] == "dark", 1.0),
            );
            for _ in 0..40 {
                rt.frame(0.05);
            }
            // Tab enters the menu on its first item; the arrow keys move within it.
            rt.key(KeyEvent::from(Key::Tab));
            for _ in 1..focused {
                rt.key(KeyEvent::from(Key::Down));
            }
            for _ in 0..40 {
                rt.frame(0.05);
            }
            let px = Pixels::of(&rt, SIZE);
            // A point inside each item's left edge, clear of the corner radius and the content.
            for i in 1..=3usize {
                let item = rt
                    .elements()
                    .into_iter()
                    .find(|e| e.key.as_deref() == Some(format!("mn/menu-item-{i}").as_str()))
                    .unwrap();
                let (x, y) = (item.target[0] + 3.0, item.target[1] + item.target[3] / 2.0);
                let got = px.at(x as u32, y as u32);
                if i == focused {
                    near(got, want, &format!("{what}: item {i} fill"));
                } else {
                    assert!(got[3] <= 8, "{what}: item {i} has a fill: {got:?}");
                }
            }
        }
    }
    assert_eq!(n, 16);
}
