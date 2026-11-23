//! Pixel assertions for the Progress circle, which `ElementInfo` cannot give: the ring's stroke width and where the arc
//! stops, read from `progress_circle_scene.json` (from `design-data render progress-circle --format scene`), never from our
//! `Tokens`. See `support/pixels.rs`.

#[path = "support/pixels.rs"]
mod pixels;

use pixels::Pixels;
use rune::{col, Cx, El, Env, Runtime};
use serde_json::Value;
use spectrum_rune::progress_circle::ProgressCircle;
use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
use spectrum_rune::Size;

const SIZE: (u32, u32) = (120, 120);
const PAD: f32 = 10.0;

#[derive(Clone)]
struct App {
    case: Value,
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
    let mut p = ProgressCircle::new("pc").size(match c["size"].as_str().unwrap() {
        "s" => Size::S,
        "m" => Size::M,
        _ => Size::L,
    });
    if args.contains("isIndeterminate") {
        p = p.indeterminate(true);
    } else if let Some(v) = args.strip_prefix("--option value=") {
        p = p.value(v.parse().unwrap());
    }
    col().padding(PAD).child(p.build(t))
}

fn rgb(s: &str) -> [i32; 3] {
    let v: Vec<i32> = s.split(',').map(|c| c.parse().unwrap()).collect();
    [v[0], v[1], v[2]]
}

/// Whether `px` is nearer `want` than `other`: antialiasing softens a pixel on a small ring's edge.
fn is(px: [u8; 4], want: [i32; 3], other: [i32; 3]) -> bool {
    let d = |c: [i32; 3]| (0..3).map(|i| (i32::from(px[i]) - c[i]).abs()).sum::<i32>();
    px[3] > 200 && d(want) < d(other)
}

#[test]
fn the_ring_has_the_oracles_stroke_and_the_arc_stops_where_its_sweep_says() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/progress_circle_scene.json")).unwrap();
    for c in &cases {
        let what = format!("{} {} {} {}", c["scheme"], c["scale"], c["size"], c["args"]);
        let mut rt = Runtime::new(
            App { case: c.clone() },
            view,
            (SIZE.0 as f32, SIZE.1 as f32),
            Env::new(c["scheme"] == "dark", 1.0),
        );
        for _ in 0..5 {
            rt.frame(0.05);
        }
        let px = Pixels::of(&rt, SIZE);
        let d = c["track"]["w"].as_f64().unwrap() as f32;
        let thick = c["track"]["stroke_width"].as_f64().unwrap() as f32;
        let (cx, cy) = (PAD + d / 2.0, PAD + d / 2.0);
        let mid = (d - thick) / 2.0;
        let at = |angle_deg: f32, r: f32| {
            let a = angle_deg.to_radians();
            px.at(
                (cx + r * a.sin()).round() as u32,
                (cy - r * a.cos()).round() as u32,
            )
        };
        let (track, fill) = (
            rgb(c["track"]["stroke"].as_str().unwrap()),
            rgb(c["fill"]["color"].as_str().unwrap()),
        );
        // An indeterminate circle has no sweep in the oracle (it spins): only its stroke is checked.
        let Some(sweep) = c["fill"]["sweep"].as_f64().map(|s| s as f32) else {
            continue;
        };
        // Inside the stroke and clear of it: the ring's two edges are `thick` apart.
        let inner = at(200.0, mid - thick * 0.25);
        let outer = at(200.0, mid + thick * 0.25);
        let beyond = at(200.0, mid + thick * 0.5 + 1.5);
        assert!(
            inner[3] > 100 && outer[3] > 100,
            "{what}: the ring is not drawn across its stroke: {inner:?} {outer:?}"
        );
        assert!(
            beyond[3] < 30,
            "{what}: the ring is wider than the stroke ({thick}): {beyond:?}"
        );
        if sweep > 0.0 && sweep < 1.0 {
            let before = sweep * 360.0 - 12.0;
            let after = sweep * 360.0 + 12.0;
            if before > 0.0 {
                assert!(
                    is(at(before, mid), fill, track),
                    "{what}: the arc stops before {before} degrees: {:?}",
                    at(before, mid)
                );
            }
            if after < 360.0 {
                assert!(
                    is(at(after, mid), track, fill),
                    "{what}: the arc runs past {after} degrees: {:?}",
                    at(after, mid)
                );
            }
        } else if sweep >= 1.0 {
            assert!(
                is(at(180.0, mid), fill, track),
                "{what}: a full ring is not all arc"
            );
        } else {
            assert!(
                is(at(90.0, mid), track, fill),
                "{what}: an empty ring has an arc: {:?}",
                at(90.0, mid)
            );
        }
    }
}
