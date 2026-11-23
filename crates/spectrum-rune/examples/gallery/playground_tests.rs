#![cfg(test)]

mod tests {
    use rune::{row, Cx, El};

    fn init() -> Page {
        Page::default()
    }

    fn view(s: &Page, cx: &mut Cx<Page>) -> El {
        row()
            .gap(24.0)
            .child(stage_view(s, cx))
            .child(side_view(s, cx))
    }

    fn poll(s: &mut Page, t: f64, tx: &mut rune::Tx) -> bool {
        spectrum_rune_playground::poll(&mut s.play, t, tx)
    }
    use spectrum_rune::constraints::{COMPONENTS as CATALOG, *};
    use spectrum_rune::tokens::{tokens, ColorScheme, Scale};
    use spectrum_rune_playground::adapters::*;
    use spectrum_rune_playground::*;

    /// The options derived from the catalog (what the controls grey out by) equal what each builder reports,
    /// for every value of every enumerated option, so the adapter and the catalog cannot drift apart.
    #[test]
    fn catalog_options_match_builder_options() {
        let t = tokens(ColorScheme::Light, Scale::Desktop);
        for ad in ADAPTERS {
            let comp = ad.id();
            for def in component(comp)
                .unwrap()
                .options
                .iter()
                .filter(|d| !d.values.is_empty())
            {
                for l in def.values {
                    let mut s = init();
                    s.play
                        .values
                        .insert(format!("{comp}/{}", def.name), lit_str(l));
                    let sorted = |mut o: Opts| {
                        o.sort_by_key(|(n, _)| *n);
                        o
                    };
                    let mut derived = sorted(options(&s.play, comp, None));
                    if errors(comp, &derived).is_empty() {
                        let reported = sorted(preview(&s.play, comp, t).0);
                        // A builder may leave out options with no visual effect (Button's size, isDisabled).
                        derived.retain(|(n, _)| reported.iter().any(|(r, _)| r == n));
                        assert_eq!(derived, reported, "{comp} {} = {}", def.name, lit_str(l));
                    }
                }
            }
        }
    }

    /// The label and icon presets (not catalog values) must reach the builder too.
    #[test]
    fn extras_match_builder_options() {
        let t = tokens(ColorScheme::Light, Scale::Desktop);
        for ad in ADAPTERS {
            let comp = ad.id();
            for (opt, presets) in extras(comp) {
                for (id, _) in presets {
                    let mut s = init();
                    s.play.values.insert(format!("{comp}/{opt}"), id.to_owned());
                    let derived = options(&s.play, comp, None);
                    if !errors(comp, &derived).is_empty() {
                        continue;
                    }
                    let reported = preview(&s.play, comp, t).0;
                    for name in ["label", "icon"] {
                        let has = |o: &Opts| o.iter().any(|(n, _)| *n == name);
                        // Button and Picker builders do not report their label (no visual option of their own).
                        if matches!(comp, "button" | "picker") && name == "label" {
                            continue;
                        }
                        assert_eq!(has(&derived), has(&reported), "{comp} {opt} = {id}: {name}");
                    }
                }
            }
        }
    }

    /// Every combination the controls can reach builds without a panic: the catalog options and the
    /// presets, skipping what `errors()` excludes (the UI shows a message for those instead).
    #[test]
    fn every_reachable_combination_builds() {
        let t = tokens(ColorScheme::Light, Scale::Desktop);
        for ad in ADAPTERS {
            let comp = ad.id();
            let mut controls: Vec<(String, Vec<String>)> = Vec::new();
            for def in component(comp)
                .unwrap()
                .options
                .iter()
                .filter(|d| !d.values.is_empty())
            {
                let unset = def.default.is_none().then(|| "none".to_owned());
                let all = unset.into_iter().chain(def.values.iter().map(lit_str));
                controls.push((def.name.to_owned(), all.collect()));
            }
            for (opt, presets) in extras(comp) {
                controls.push((
                    opt.to_owned(),
                    presets.iter().map(|(id, _)| id.to_string()).collect(),
                ));
            }
            let total: usize = controls.iter().map(|(_, v)| v.len()).product();
            let mut built = 0;
            for mut n in 0..total {
                let mut s = init();
                for (opt, values) in &controls {
                    s.play
                        .values
                        .insert(format!("{comp}/{opt}"), values[n % values.len()].clone());
                    n /= values.len();
                }
                if errors(comp, &options(&s.play, comp, None)).is_empty() {
                    preview(&s.play, comp, t);
                    built += 1;
                }
            }
            assert!(built > 0, "{comp}: no combination is valid");
        }
    }

    #[test]
    fn outline_is_greyed_out_for_accent_button() {
        let mut s = init();
        s.play
            .values
            .insert("button/variant".into(), "accent".into());
        let why = errors(
            "button",
            &options(&s.play, "button", Some(("style", "outline"))),
        );
        assert_eq!(
            why,
            ["Outline style is only defined for primary and secondary."]
        );
        assert!(errors(
            "button",
            &options(&s.play, "button", Some(("style", "fill")))
        )
        .is_empty());
        // An icon-only button needs an icon: hiding the label is excluded until one is chosen.
        let why = errors(
            "button",
            &options(&s.play, "button", Some(("isLabelHidden", "true"))),
        );
        assert_eq!(why, ["An icon-only button needs an icon."]);
    }

    /// A value that would break an error rule is rendered disabled, pressing it changes nothing, and
    /// the reason is listed in the UI (the test above checks `errors()`, this checks what is rendered).
    #[test]
    fn excluded_value_is_rendered_disabled_with_its_reason() {
        use rune::{Env, Runtime};
        let mut s = init();
        s.play.values.insert("/component".into(), "button".into());
        s.play
            .values
            .insert("button/variant".into(), "accent".into());
        let mut rt = Runtime::new(s, view, (1100.0, 1400.0), Env::new(false, 1.0));
        rt.frame(0.05);
        let tree = rt.accessibility_json().to_string();
        assert!(
            tree.contains(
                "outline: not available, Error · Outline style is only defined for primary and secondary."
            ),
            "{tree}"
        );
        let (x, y) = rt
            .find("playground/button/style/outline/container")
            .unwrap();
        rt.pointer_move(x, y);
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(0.05);
        assert!(!rt.state.play.values.contains_key("button/style"));
    }

    /// Writes the playground in a few states to `target/gallery/playground-*.png` (for PRs). Needs the
    /// oracle CLI (`DESIGN_DATA`) for the oracle panel; run with `--release -- --ignored`.
    #[test]
    #[ignore = "slow unoptimised; needs the oracle CLI"]
    fn render_playground_shots() {
        use rune::{Env, Runtime};
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/gallery");
        std::fs::create_dir_all(&dir).unwrap();
        let shots: [(&str, &[(&str, &str)]); 13] = [
            ("badge", &[]),
            (
                "button-hover",
                &[
                    ("/component", "button"),
                    ("button/variant", "accent"),
                    ("button/state", "hover"),
                    ("button/icon", "check"),
                ],
            ),
            (
                "button-excluded",
                &[
                    ("/component", "button"),
                    ("button/variant", "accent"),
                    ("button/style", "outline"),
                ],
            ),
            ("catalog-only", &[("/component", "tabs")]),
            (
                "selector-open",
                &[("/component", "button"), ("/picker", "open")],
            ),
            (
                "status-light-s",
                &[("/component", "status-light"), ("status-light/size", "s")],
            ),
            (
                "status-light-xl",
                &[("/component", "status-light"), ("status-light/size", "xl")],
            ),
            ("picker", &[("/component", "picker")]),
            (
                "picker-help",
                &[
                    ("/component", "picker"),
                    ("picker/isRequired", "true"),
                    ("picker/description", "help"),
                ],
            ),
            (
                "picker-error",
                &[
                    ("/component", "picker"),
                    ("picker/isError", "true"),
                    ("picker/description", "help"),
                ],
            ),
            (
                "button-compare",
                &[
                    ("/component", "button"),
                    ("button/variant", "accent"),
                    ("/compare", "on"),
                ],
            ),
            (
                "button-static-white",
                &[("/component", "button"), ("button/staticColor", "white")],
            ),
            (
                "button-static-black",
                &[("/component", "button"), ("button/staticColor", "black")],
            ),
        ];
        for (name, play) in shots {
            for dark in [false, true] {
                let mut s = init();
                s.play.scheme = Some(if dark {
                    ColorScheme::Dark
                } else {
                    ColorScheme::Light
                });
                for (k, v) in play {
                    s.play.values.insert((*k).into(), (*v).into());
                }
                let (w, h) = (1100u32, 2500u32);
                let mut rt = Runtime::new(s, view, (w as f32, h as f32), Env::new(dark, 1.0));
                rt.poll = Some(poll);
                for _ in 0..12 {
                    rt.frame(0.1);
                }
                let tag = if dark { "dark" } else { "light" };
                crate::pixels::Pixels::of(&rt, (w, h))
                    .write_png(&dir.join(format!("playground-{name}-{tag}.png")));
            }
        }
    }

    fn tree_of(play: &[(&str, &str)]) -> String {
        use rune::{Env, Runtime};
        let mut s = init();
        for (k, v) in play {
            s.play.values.insert((*k).into(), (*v).into());
        }
        let mut rt = Runtime::new(s, view, (1100.0, 2500.0), Env::new(false, 1.0));
        rt.frame(0.05);
        rt.accessibility_json().to_string()
    }

    /// The catalog default is marked, an option without one reads "unset", and the options panel shows
    /// the real label text.
    #[test]
    fn defaults_are_marked_and_the_panel_shows_real_values() {
        let tree = tree_of(&[("/component", "button")]);
        assert!(tree.contains("fill (default)"), "{tree}");
        assert!(tree.contains("unset (default)"), "{tree}");
        assert!(tree.contains("label = Label"), "{tree}");
        assert!(tree.contains("Builder options"), "{tree}");
    }

    /// When the current selection is itself excluded, the reason is said once (on the stage), not under
    /// every value of every control.
    #[test]
    fn an_excluded_selection_does_not_repeat_its_reason() {
        let tree = tree_of(&[
            ("/component", "button"),
            ("button/variant", "accent"),
            ("button/style", "outline"),
        ]);
        assert_eq!(
            tree.matches("Not defined: Outline style").count(),
            1,
            "{tree}"
        );
        assert_eq!(
            tree.matches("not available, Error · Outline style").count(),
            0
        );
    }

    #[test]
    fn oracle_colours_compare_within_three_per_channel() {
        let rune = ("#3b63fb".to_owned(), 1.0);
        assert!(same_color("rgb(59, 99, 251)", &rune));
        assert!(same_color("rgb(61, 97, 250)", &rune));
        assert!(!same_color("rgb(59, 99, 240)", &rune));
        // A translucent text colour: Rune's hex carries the alpha, the oracle's rgba its own.
        let text = "#000000ed".to_owned();
        assert!(same_color(
            "rgba(0, 0, 0, 0.93)",
            &(text.clone(), hex_alpha(&text))
        ));
        assert!(same_color("transparent", &("#000000".to_owned(), 0.0)));
        assert!(!same_color("transparent", &rune));
    }

    /// A static white or black Button is previewed on a surface that contrasts with it.
    #[test]
    fn static_colour_buttons_get_a_contrasting_backdrop() {
        use rune::{Env, Runtime};
        for (static_color, dark) in [
            ("white", false),
            ("white", true),
            ("black", false),
            ("black", true),
        ] {
            let mut s = init();
            s.play.scheme = Some(if dark {
                ColorScheme::Dark
            } else {
                ColorScheme::Light
            });
            s.play.values.insert("/component".into(), "button".into());
            s.play
                .values
                .insert("button/staticColor".into(), static_color.into());
            let mut rt = Runtime::new(s, view, (1100.0, 1500.0), Env::new(dark, 1.0));
            for _ in 0..10 {
                rt.frame(0.05);
            }
            let fill = |key: &str| {
                rt.elements()
                    .iter()
                    .find(|e| e.key.as_deref() == Some(key))
                    .unwrap()
                    .fill
            };
            let back = fill("playground/stage/backdrop");
            let lum = |c: rune::Color| c.r + c.g + c.b;
            if static_color == "white" {
                assert!(lum(back) < 1.5, "white needs a dark backdrop: {back:?}");
            } else {
                assert!(lum(back) > 1.5, "black needs a light backdrop: {back:?}");
            }
        }
    }

    /// "Compare light and dark" shows a second copy under the other scheme's tokens.
    #[test]
    fn compare_shows_a_copy_under_the_other_scheme() {
        use rune::{Env, Runtime};
        for dark in [false, true] {
            let mut s = init();
            s.play.scheme = Some(if dark {
                ColorScheme::Dark
            } else {
                ColorScheme::Light
            });
            s.play.values.insert("/component".into(), "button".into());
            s.play.values.insert("/compare".into(), "on".into());
            let mut rt = Runtime::new(s, view, (1100.0, 1500.0), Env::new(dark, 1.0));
            for _ in 0..10 {
                rt.frame(0.05);
            }
            let fill = |key: &str| {
                rt.elements()
                    .iter()
                    .find(|e| e.key.as_deref() == Some(key))
                    .unwrap()
                    .fill
            };
            let (here, there) = (
                fill("playground/preview/container"),
                fill("playground/alt/container"),
            );
            assert_ne!(here, there, "dark={dark}: both copies look the same");
        }
    }

    /// A rule the current options break is listed as active, the others as inactive.
    #[test]
    fn catalog_rules_show_their_live_state() {
        use rune::{Env, Runtime};
        let text_for = |variant: &str| {
            let mut s = init();
            s.play.values.insert("/component".into(), "button".into());
            s.play
                .values
                .insert("button/variant".into(), variant.into());
            s.play
                .values
                .insert("button/style".into(), "outline".into());
            let mut rt = Runtime::new(s, view, (1100.0, 1400.0), Env::new(false, 1.0));
            rt.frame(0.05);
            rt.accessibility_json().to_string()
        };
        assert!(text_for("accent").contains("active · Error · Outline style"));
        assert!(text_for("primary").contains("inactive · Error · Outline style"));
    }

    /// Needs the oracle CLI in the oracle.toml range (`DESIGN_DATA=... cargo test -- --ignored`).
    #[test]
    #[ignore]
    fn oracle_runs_every_component_at_its_defaults() {
        for ad in ADAPTERS {
            let comp = ad.id();
            let s = init();
            let args = oracle_args(&s.play, comp, false);
            let r = compare(&s.play, comp, false, &args).expect(comp);
            assert!(r.parts.iter().any(|p| p.name == "container"), "{comp}");
            assert!(
                r.parts.iter().any(|p| !p.rows.is_empty()),
                "{comp}: no tokens"
            );
            let bad: Vec<_> = r
                .parts
                .iter()
                .filter(|p| !matches!(p.verdict, Verdict::Matches))
                .map(|p| &p.name)
                .collect();
            assert!(bad.is_empty(), "{comp}: {bad:?}");
        }
    }

    /// Static white and black Buttons match the oracle too (translucent fills and text colours).
    #[test]
    #[ignore]
    fn oracle_matches_static_colour_buttons() {
        for c in ["white", "black"] {
            let mut s = init();
            s.play.values.insert("/component".into(), "button".into());
            s.play.values.insert("button/staticColor".into(), c.into());
            let args = oracle_args(&s.play, "button", false);
            let r = compare(&s.play, "button", false, &args).expect(c);
            let bad: Vec<_> = r
                .parts
                .iter()
                .filter(|p| !matches!(p.verdict, Verdict::Matches))
                .map(|p| &p.name)
                .collect();
            assert!(bad.is_empty(), "{c}: {bad:?}");
        }
    }

    /// The state control reaches the oracle: a hovered Button reads different tokens than one at rest.
    #[test]
    #[ignore]
    fn oracle_state_changes_the_tokens() {
        let tokens_at = |state: &str| {
            let mut s = init();
            s.play.values.insert("/component".into(), "button".into());
            s.play.values.insert("button/icon".into(), "check".into());
            s.play.values.insert("button/state".into(), state.into());
            let args = oracle_args(&s.play, "button", false);
            let r = compare(&s.play, "button", false, &args).expect(state);
            r.parts
                .iter()
                .flat_map(|p| p.rows.iter().map(|r| (r.prop.clone(), r.token.clone())))
                .collect::<Vec<_>>()
        };
        assert_ne!(tokens_at("rest"), tokens_at("hover"));
    }

    /// Pressing Run lists the oracle's tokens in the UI (needs the CLI, like the test above).
    #[test]
    #[ignore]
    fn run_button_lists_the_oracle_tokens() {
        use rune::{Env, Runtime};
        let s = init();
        let mut rt = Runtime::new(s, view, (1100.0, 1400.0), Env::new(false, 1.0));
        rt.frame(0.05);
        let (x, y) = rt.find("playground/oracle/run/container").unwrap();
        rt.pointer_move(x, y);
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(0.05);
        let tree = rt.accessibility_json().to_string();
        assert!(tree.contains("base-height-"), "{tree}");
    }

    /// The oracle runs by itself once the options have been still for the delay, and not before.
    /// It needs no CLI: a missing or out-of-range one is an error result, still a run.
    #[test]
    fn oracle_runs_after_the_options_settle() {
        use rune::{Env, Runtime};
        let s = init();
        let mut rt = Runtime::new(s, view, (1100.0, 900.0), Env::new(false, 1.0));
        rt.poll = Some(poll);
        rt.frame(0.05);
        assert!(rt.state.play.oracle.is_none(), "ran before the delay");
        for _ in 0..4 {
            rt.frame(0.1);
        }
        assert!(
            rt.state.play.oracle.is_some(),
            "did not run after the delay"
        );
    }

    /// The state control drives the Button preview: hover and down fill differently from rest.
    #[test]
    fn the_state_control_drives_the_button_preview() {
        use rune::{Env, Runtime};
        let fill_at = |state: &str| {
            let mut s = init();
            s.play.values.insert("/component".into(), "button".into());
            s.play.values.insert("button/state".into(), state.into());
            let mut rt = Runtime::new(s, view, (1100.0, 900.0), Env::new(false, 1.0));
            for _ in 0..10 {
                rt.frame(0.05);
            }
            let e = rt.elements();
            e.iter()
                .find(|e| e.key.as_deref() == Some("playground/preview/container"))
                .unwrap()
                .fill
        };
        assert_ne!(fill_at("rest"), fill_at("hover"));
        assert_ne!(fill_at("rest"), fill_at("down"));
        assert_eq!(fill_at("keyboard-focus"), fill_at("rest"));
    }

    /// Every catalog component builds in the playground, previewed or not, without the oracle.
    #[test]
    fn every_catalog_component_builds() {
        use rune::{Env, Runtime};
        for c in CATALOG {
            let mut s = init();
            s.play.values.insert("/component".into(), c.name.into());
            let mut rt = Runtime::new(s, view, (1100.0, 900.0), Env::new(false, 1.0));
            rt.poll = Some(poll);
            for _ in 0..10 {
                rt.frame(0.1);
            }
            let tree = rt.accessibility_json().to_string();
            assert!(tree.contains("Catalog rules"), "{}", c.name);
            assert_eq!(
                tree.contains("Not previewed"),
                !previewed(c.name),
                "{}",
                c.name
            );
        }
    }

    #[test]
    fn pressing_a_value_selects_it() {
        use rune::{Env, Runtime};
        let s = init();
        let mut rt = Runtime::new(s, view, (1100.0, 900.0), Env::new(false, 1.0));
        rt.frame(0.05);
        let (x, y) = rt.find("playground/badge/style/subtle/container").unwrap();
        rt.pointer_move(x, y);
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(0.05);
        assert_eq!(rt.state.play.values["badge/style"], "subtle");
    }

    /// One story per catalog component, each under its own path and opening on its own component.
    #[test]
    fn there_is_one_story_per_catalog_component() {
        assert_eq!(stories().len(), menu_order().len());
        assert_eq!(Page::of("button").play.values["/component"], "button");
    }
}
