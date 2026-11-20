//! Dev-only: ask the design-data CLI (the oracle, the version range in `oracle.toml`) which tokens each
//! part of the current Playground selection reads. Rune does not record that itself (plain `Tokens`).
//! The CLI is `$DESIGN_DATA` or `design-data` on the PATH, run from the repo root (`.design-data.toml`).

use serde_json::Value;
use std::process::Command;

const ORACLE_TOML: &str = include_str!("../../../../oracle.toml");

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

/// One part of the oracle's scene: its box and the tokens it reads (`prop = token (value)`).
pub struct Part {
    pub name: String,
    pub rect: [f32; 4],
    pub tokens: Vec<String>,
}

/// Runs `design-data render <component> <args> --format scene`: the CLI version and the scene's parts.
pub fn run(component: &str, args: &[String]) -> Result<(String, Vec<Part>), String> {
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
            tokens: n["tokens"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|t| {
                    format!(
                        "{} = {} ({})",
                        t["prop"].as_str().unwrap_or("?"),
                        t["token"].as_str().unwrap_or("?"),
                        t["value"].as_str().unwrap_or("?")
                    )
                })
                .collect(),
        })
        .collect();
    Ok((v, parts))
}
