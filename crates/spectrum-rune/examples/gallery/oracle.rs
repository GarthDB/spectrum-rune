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

/// Runs `design-data render <component> <args> --format scene` and lists each part's tokens.
pub fn run(component: &str, args: &[String]) -> Result<Vec<String>, String> {
    let bin = cli();
    let out = Command::new(&bin).arg("--version").output().map_err(|e| {
        format!("cannot run `{bin}`: {e} (set DESIGN_DATA to a binary in the oracle.toml range)")
    })?;
    let shown = String::from_utf8_lossy(&out.stdout);
    let v = shown.trim().trim_start_matches("design-data ");
    let (min, max) = oracle_range();
    if !version(v).is_some_and(|v| min <= v && v < max) {
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
    let mut lines = vec![format!(
        "oracle {v}: {} x {}",
        scene["width"].as_f64().unwrap_or(0.0).round(),
        scene["height"].as_f64().unwrap_or(0.0).round()
    )];
    for n in scene["nodes"].as_array().into_iter().flatten() {
        lines.push(format!(
            "{}: {} x {}",
            n["part"].as_str().unwrap_or("?"),
            n["w"].as_f64().unwrap_or(0.0).round(),
            n["h"].as_f64().unwrap_or(0.0).round()
        ));
        for t in n["tokens"].as_array().into_iter().flatten() {
            lines.push(format!(
                "    {} = {} ({})",
                t["prop"].as_str().unwrap_or("?"),
                t["token"].as_str().unwrap_or("?"),
                t["value"].as_str().unwrap_or("?")
            ));
        }
    }
    Ok(lines)
}
