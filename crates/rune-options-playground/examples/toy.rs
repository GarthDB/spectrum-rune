//! A component library that is not Spectrum, played with: two toy components (a chip and a toggle), a catalog with a few
//! options and rules, an adapter for each, and no oracle. This is all a library needs to give `rune-options-playground`.
//!
//! ```sh
//! cargo run -p rune-options-playground --example toy -- --playground "Components/Chip"
//! ```
//! (⌘⇧D opens the playground in a window too.)

use rune::{col, el, row, text, App, AppSpec, Cx, El, Playground};
use rune_kit::{Kit, Switch};
use rune_options_playground::adapters::{Adapter, Choice, ExtraKind, Extras, Opts};
use rune_options_playground::model::{
    Env, Library, Lit, OptionDef, OptionValue, Rule, Severity, Violation,
};

struct Toy;

const CHIP_RULES: [Rule; 1] = [Rule {
    severity: Severity::Error,
    description: Some("A danger chip needs room: it is not defined at size s."),
}];
const TOGGLE_RULES: [Rule; 1] = [Rule {
    severity: Severity::Advisory,
    description: Some("A disabled toggle that is on looks the same as a disabled one that is off."),
}];

fn is(o: &[(&str, OptionValue)], name: &str, v: OptionValue) -> bool {
    o.iter()
        .rev()
        .find(|(n, _)| *n == name)
        .is_some_and(|(_, x)| *x == v)
}

impl Library for Toy {
    fn components(&self) -> Vec<&'static str> {
        vec!["chip", "toggle"]
    }
    fn options(&self, comp: &str) -> Vec<OptionDef> {
        let str_option = |name, values: &[&'static str], default| OptionDef {
            name,
            values: values.iter().map(|v| Lit::Str(v)).collect(),
            default: Some(Lit::Str(default)),
        };
        let bool_option = |name| OptionDef {
            name,
            values: vec![Lit::Bool(false), Lit::Bool(true)],
            default: Some(Lit::Bool(false)),
        };
        match comp {
            "chip" => vec![
                str_option("size", &["s", "m"], "m"),
                str_option("tone", &["neutral", "accent", "danger"], "neutral"),
            ],
            "toggle" => vec![bool_option("isOn"), bool_option("isDisabled")],
            _ => Vec::new(),
        }
    }
    fn rules(&self, comp: &str) -> Vec<Rule> {
        match comp {
            "chip" => CHIP_RULES.to_vec(),
            "toggle" => TOGGLE_RULES.to_vec(),
            _ => Vec::new(),
        }
    }
    fn check(&self, comp: &str, o: &[(&str, OptionValue)]) -> Vec<Violation> {
        let hit = |rule: usize, severity, description| Violation {
            rule,
            severity,
            description,
        };
        let mut out = Vec::new();
        match comp {
            "chip" => {
                if is(o, "size", OptionValue::Str("s")) && is(o, "tone", OptionValue::Str("danger"))
                {
                    out.push(hit(0, Severity::Error, CHIP_RULES[0].description));
                }
            }
            "toggle"
                if is(o, "isDisabled", OptionValue::Bool(true))
                    && is(o, "isOn", OptionValue::Bool(true)) =>
            {
                out.push(hit(0, Severity::Advisory, TOGGLE_RULES[0].description));
            }
            _ => {}
        }
        out
    }
    fn adapters(&self) -> &'static [&'static dyn Adapter] {
        &[&ChipA, &ToggleA]
    }
}

struct ChipA;
impl Adapter for ChipA {
    fn id(&self) -> &'static str {
        "chip"
    }
    fn name(&self) -> &'static str {
        "Chip"
    }
    fn extra_kind(&self, _opt: &str) -> ExtraKind {
        ExtraKind::FreeForm
    }
    fn extras(&self) -> Extras {
        vec![(
            "label",
            vec![("short", "Label"), ("long", "A much longer label")],
        )]
    }
    fn build(&self, c: &Choice, env: Env, key: &str) -> (Opts, El) {
        let k = Kit::neutral(env.dark);
        let (fill, ink) = match c.v("tone").as_str() {
            "accent" => (k.c.info.bg, k.c.info.fg),
            "danger" => (k.c.danger.bg, k.c.danger.fg),
            _ => (k.c.surface_raised_base, k.c.text_base),
        };
        let (pad, style) = match c.v("size").as_str() {
            "s" => (6.0, k.ty.small),
            _ => (10.0, k.ty.ui),
        };
        let chip = row()
            .key(format!("{key}/container"))
            .py(pad / 2.0)
            .pl(pad)
            .pr(pad)
            .radius(pad * 2.0)
            .fill(fill)
            .child(
                text(match c.extra("label") {
                    "long" => "A much longer label",
                    _ => "Label",
                })
                .font(style)
                .color(ink),
            );
        // A row around it keeps the chip at its own width instead of the stage's.
        (c.options.clone(), row().child(chip))
    }
}

struct ToggleA;
impl Adapter for ToggleA {
    fn id(&self) -> &'static str {
        "toggle"
    }
    fn name(&self) -> &'static str {
        "Toggle"
    }
    fn extras(&self) -> Extras {
        Vec::new()
    }
    fn build(&self, c: &Choice, env: Env, key: &str) -> (Opts, El) {
        let k = Kit::neutral(env.dark);
        let s = Switch::new(c.on("isOn"))
            .label("Toggle")
            .disabled(c.on("isDisabled"))
            .build(&k);
        (
            c.options.clone(),
            col().key(format!("{key}/container")).child(s),
        )
    }
}

struct App0;

fn init() -> App0 {
    App0
}

fn view(_: &App0, _: &mut Cx<App0>) -> El {
    col().child(el().child(text("Toy library: open the playground (⌘⇧D)".to_owned())))
}

fn main() {
    App::new(AppSpec::new("Toy library", (900.0, 600.0), init, view))
        .playground(
            rune_options_playground::stories(&Toy)
                .into_iter()
                .fold(Playground::<App0>::new().enabled(true), |p, s| p.story(s)),
        )
        .run();
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune::{Env as RuneEnv, Runtime};
    use rune_options_playground::{side_view, stage_view, Page};

    fn view2(s: &Page, cx: &mut Cx<Page>) -> El {
        row().child(stage_view(s, cx)).child(side_view(s, cx))
    }

    fn page(comp: &str, set: &[(&str, &str)]) -> Runtime<Page> {
        let mut p = Page::of(&Toy, comp);
        for (k, v) in set {
            p.play.values.insert((*k).into(), (*v).into());
        }
        let mut rt = Runtime::new(p, view2, (1000.0, 800.0), RuneEnv::new(false, 1.0));
        rt.frame(0.05);
        rt
    }

    #[test]
    fn one_story_per_component_with_no_oracle_panel() {
        assert_eq!(rune_options_playground::stories(&Toy).len(), 2);
        let tree = page("chip", &[]).accessibility_json().to_string();
        assert!(tree.contains("Catalog rules"), "{tree}");
        assert!(!tree.contains("Oracle"), "{tree}");
        // The toy has no touch scale, so no switch for one.
        assert!(!tree.contains("Mobile scale"), "{tree}");
    }

    #[test]
    fn a_rule_greys_out_the_value_and_says_why() {
        let rt = page("chip", &[("chip/size", "s")]);
        let tree = rt.accessibility_json().to_string();
        assert!(
            tree.contains("danger: not available, Error · A danger chip needs room"),
            "{tree}"
        );
        let danger = rt
            .elements()
            .into_iter()
            .find(|e| e.key.as_deref() == Some("playground/chip/tone/danger"))
            .unwrap();
        assert!(!danger.focusable);
    }

    #[test]
    fn the_other_component_builds_too() {
        let tree = page(
            "toggle",
            &[("toggle/isDisabled", "true"), ("toggle/isOn", "true")],
        )
        .accessibility_json()
        .to_string();
        assert!(tree.contains("advisory"), "{tree}");
    }
}
