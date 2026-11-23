//! Dev-only: ask the design-data CLI (the oracle, the version range in `oracle.toml`) which tokens each
//! part of the current Playground selection reads. Rune does not record that itself (plain `Tokens`).
//! The CLI is `$DESIGN_DATA` or `design-data` on the PATH, run from the repo root (`.design-data.toml`).

use rune_options_playground::adapters::Choice;
use rune_options_playground::model::{Env, OptionValue};
use rune_options_playground::oracle::{Oracle, Part, Tok};
use serde_json::Value;
use std::process::Command;

const ORACLE_TOML: &str = include_str!("../../../oracle.toml");

fn version(s: &str) -> Option<(u32, u32, u32)> {
    let mut p = s.trim().split('.').map(|n| n.parse().ok());
    Some((p.next()??, p.next()??, p.next()??))
}

fn oracle_range() -> ((u32, u32, u32), (u32, u32, u32)) {
    let get = |key: &str| {
        let line = ORACLE_TOML.lines().find(|l| l.starts_with(key)).unwrap();
        version(line.split('"').nth(1).unwrap()).unwrap()
    };
    (get("cli_min"), get("cli_max"))
}

fn cli() -> String {
    std::env::var("DESIGN_DATA").unwrap_or_else(|_| "design-data".to_owned())
}

fn root() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/../..")
}

/// Runs `design-data render <component> <args> --format scene`: the CLI version and the scene's parts.
fn run(component: &str, args: &[String]) -> Result<(String, Vec<Part>), String> {
    let bin = cli();
    let out = Command::new(&bin).arg("--version").output().map_err(|e| {
        format!("cannot run `{bin}`: {e} (set DESIGN_DATA to a binary in the oracle.toml range)")
    })?;
    let shown = String::from_utf8_lossy(&out.stdout);
    let v = shown.trim().trim_start_matches("design-data ").to_owned();
    let (min, max) = oracle_range();
    if !version(&v).is_some_and(|v| min <= v && v < max) {
        return Err(format!(
            "`{bin}` is {v}; oracle.toml accepts {min:?} up to (not including) {max:?}. Set DESIGN_DATA to a binary in that range."
        ));
    }
    let out = Command::new(&bin)
        .current_dir(root())
        .args(["render", component, "--format", "scene"])
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_owned());
    }
    let scene: Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    let f = |n: &Value, k: &str| n[k].as_f64().unwrap_or(0.0) as f32;
    let parts = scene["nodes"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|n| Part {
            name: n["part"].as_str().unwrap_or("?").to_owned(),
            rect: [f(n, "x"), f(n, "y"), f(n, "w"), f(n, "h")],
            radius: f(n, "radius"),
            tokens: n["tokens"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|t| {
                    let s = |k: &str| t[k].as_str().unwrap_or("?").to_owned();
                    Tok {
                        prop: s("prop"),
                        token: s("token"),
                        value: s("value"),
                    }
                })
                .collect(),
        })
        .collect();
    Ok((v, parts))
}

/// The design-data CLI as the playground's oracle.
pub struct DesignData;

impl Oracle for DesignData {
    /// The `design-data render` arguments for the chosen values (the catalog option names, `--size`, scheme, scale),
    /// then what the component's adapter adds (label, icon, state) and the width.
    fn args(&self, comp: &str, c: &Choice, env: Env) -> Vec<String> {
        let mut a: Vec<String> = vec![
            "--color-scheme".into(),
            if env.dark { "dark" } else { "light" }.into(),
            "--scale".into(),
            if env.mobile { "mobile" } else { "desktop" }.into(),
        ];
        let mut size = None;
        for (n, v) in &c.options {
            let mut opt = |kv: String| a.extend(["--option".to_owned(), kv]);
            match (*n, v) {
                ("size", OptionValue::Str(x)) => size = Some(*x),
                // The free-form options the adapter names itself.
                ("icon" | "label", _) => {}
                (n, OptionValue::Str(x)) => opt(format!("{n}=\"{x}\"")),
                (n, OptionValue::Bool(b)) => opt(format!("{n}={b}")),
                (n, OptionValue::Num(x)) => opt(format!("{n}={x}")),
                (_, OptionValue::Present) => {}
            }
        }
        if let Some(ad) = crate::adapters::ADAPTERS.iter().find(|a| a.id() == comp) {
            a.extend(ad.oracle_args(c));
        }
        if let Some(x) = size {
            a.extend(["--size".to_owned(), x.to_owned()]);
        }
        if let Ok(w) = c.extra("width").parse::<f32>() {
            a.extend(["--width".to_owned(), w.to_string()]);
        }
        a
    }
    fn run(&self, comp: &str, args: &[String]) -> Result<(String, Vec<Part>), String> {
        run(comp, args)
    }
}
