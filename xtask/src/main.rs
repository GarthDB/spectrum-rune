//! `cargo xtask gen`: writes the `generated.rs` files under `crates/spectrum-rune/src/` from the
//! `design-data` CLI, which reads the foundation release pinned in `.design-data.toml`
//! (a github source, fetched and cached by the CLI). No spectrum-design-data checkout needed.

use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

type Res<T> = Result<T, String>;

const SCHEMES: [&str; 2] = ["light", "dark"];
const SCALES: [&str; 2] = ["desktop", "mobile"];
const KEYWORDS: [&str; 8] = ["type", "match", "ref", "fn", "in", "as", "mod", "use"];

fn main() {
    if std::env::args().nth(1).as_deref() != Some("gen") {
        eprintln!("usage: cargo xtask gen");
        std::process::exit(2);
    }
    if let Err(e) = generate() {
        eprintln!("xtask gen failed: {e}");
        std::process::exit(1);
    }
}

fn run(cmd: &mut Command) -> Res<String> {
    let out = cmd.output().map_err(|e| format!("{cmd:?}: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "{cmd:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    String::from_utf8(out.stdout).map_err(|e| e.to_string())
}

fn json(cmd: &mut Command) -> Res<Value> {
    serde_json::from_str(&run(cmd)?).map_err(|e| format!("{cmd:?}: bad JSON: {e}"))
}

/// The repo root: the CLI runs here so it picks up `.design-data.toml`.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// A `key = "value"` line of `oracle.toml` at the repo root.
fn oracle(key: &str) -> Res<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../oracle.toml");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    text.lines()
        .find_map(|l| {
            let (k, v) = l.split_once('=')?;
            // value is a quoted string; anything after the closing quote (a comment) is ignored
            let v = v.trim().strip_prefix('"')?.split('"').next()?;
            (k.trim() == key).then(|| v.to_string())
        })
        .filter(|v| !v.is_empty())
        .ok_or_else(|| format!("oracle.toml has no non-empty `{key}`"))
}

fn semver(v: &str) -> Res<(u32, u32, u32)> {
    let p: Vec<u32> = v
        .split(|c: char| !c.is_ascii_digit())
        .take(3)
        .map(|n| n.parse().map_err(|_| format!("bad version `{v}`")))
        .collect::<Res<_>>()?;
    match p[..] {
        [a, b, c] => Ok((a, b, c)),
        _ => Err(format!("bad version `{v}`")),
    }
}

fn check_inputs() -> Res<(String, String)> {
    let version = run(Command::new("design-data").arg("--version"))?
        .trim()
        .to_string();
    let (min, max) = (oracle("cli_min")?, oracle("cli_max")?);
    let v = version.strip_prefix("design-data ").unwrap_or(&version);
    if semver(v)? < semver(&min)? || semver(v)? >= semver(&max)? {
        return Err(format!(
            "oracle.toml accepts design-data {min} up to (not including) {max}, found `{version}`"
        ));
    }
    // The CLI fetches the tag `.design-data.toml` pins; the headers record oracle.toml's `dataset`, so the
    // pin and the manifest's foundationVersion must agree (a bump of one without the other is an error).
    let toml =
        std::fs::read_to_string(root().join(".design-data.toml")).map_err(|e| e.to_string())?;
    let tag = toml
        .lines()
        .find_map(|l| {
            l.trim()
                .strip_prefix("tag")?
                .trim()
                .strip_prefix('=')?
                .trim()
                .strip_prefix('"')?
                .split('"')
                .next()
                .map(String::from)
        })
        .ok_or(".design-data.toml has no `tag`")?;
    let manifest: Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("platforms/rune/manifest.json"))
            .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if manifest["foundationVersion"] != tag.as_str() {
        return Err(format!(
            ".design-data.toml pins `{tag}` but platforms/rune/manifest.json foundationVersion is {}; \
             bump both and oracle.toml's `dataset` together",
            manifest["foundationVersion"]
        ));
    }
    Ok((version, oracle("dataset")?))
}

/// `design-data catalog export` of the pinned release with the rune platform: `components/`,
/// `icons/` and `visuals/`, which the generators read instead of a checkout.
fn catalog_export() -> Res<PathBuf> {
    let out = std::env::temp_dir().join(format!("spectrum-rune-catalog-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    run(Command::new("design-data")
        .current_dir(root())
        .args(["catalog", "export", "--platform", "rune", "--out"])
        .arg(&out))?;
    Ok(out)
}

fn export(scheme: &str, scale: &str, contrast: &str) -> Res<Map<String, Value>> {
    let v = json(
        Command::new("design-data")
            .current_dir(root())
            .arg("export")
            .args([
                "--color-scheme",
                scheme,
                "--scale",
                scale,
                "--contrast",
                contrast,
            ]),
    )?;
    match v {
        Value::Object(m) => Ok(m),
        _ => Err("export is not an object".into()),
    }
}

/// Drops `desktop|mobile|light|dark` segments and a leading `color`, how scene token names
/// relate to export names.
fn normalize(name: &str) -> String {
    let segs: Vec<&str> = name
        .split('-')
        .filter(|s| !SCHEMES.contains(s) && !SCALES.contains(s))
        .collect();
    let segs = if segs.first() == Some(&"color") {
        &segs[1..]
    } else {
        &segs[..]
    };
    segs.join("-")
}

fn parse_px(v: &str) -> Res<f64> {
    v.strip_suffix("px")
        .unwrap_or(v)
        .parse()
        .map_err(|_| format!("unsupported scoped value `{v}`"))
}

/// Values bound by recipes that `export` omits, per scale: normalized name -> scale -> px.
fn scoped_values(
    dir: &Path,
    exported: &BTreeSet<String>,
) -> Res<BTreeMap<String, BTreeMap<String, f64>>> {
    let mut states = Vec::new();
    let mut recipes: Vec<_> = std::fs::read_dir(dir.join("visuals"))
        .map_err(|e| e.to_string())?
        .flatten()
        .collect();
    recipes.sort_by_key(|e| e.file_name());
    for entry in recipes {
        let path = entry.path();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("bad recipe name")?
            .to_string();
        let read = |p: PathBuf| -> Res<Value> {
            serde_json::from_str(
                &std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?,
            )
            .map_err(|e| e.to_string())
        };
        let recipe = read(path)?;
        let catalog = read(dir.join(format!("components/{name}.json")))?;
        let sizes: Vec<String> = recipe["sizeScale"]
            .as_object()
            .map_or_else(|| vec!["m".into()], |m| m.keys().cloned().collect());
        let mut variants = vec![Map::new()];
        for (opt, def) in catalog["options"].as_object().into_iter().flatten() {
            if opt == "size" {
                continue;
            }
            let mut one = |v: Value| variants.push(Map::from_iter([(opt.clone(), v)]));
            if def["type"] == "boolean" {
                one(Value::Bool(true));
            }
            for v in def["values"].as_array().into_iter().flatten() {
                one(v["value"].clone());
            }
        }
        for scale in SCALES {
            for size in &sizes {
                for (i, options) in variants.iter().enumerate() {
                    let id = format!("{name}.{scale}.{size}.{i}");
                    states.push(serde_json::json!({"id": id, "component": name, "size": size, "options": options, "colorScheme": "light", "scale": scale}));
                }
            }
        }
    }
    let tmp = std::env::temp_dir().join(format!("spectrum-rune-xtask-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let batch = tmp.join("batch.json");
    std::fs::write(
        &batch,
        serde_json::to_string(&serde_json::json!({ "states": states })).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    run(Command::new("design-data")
        .current_dir(root())
        .arg("render")
        .arg("--batch")
        .arg(&batch)
        .arg("--out")
        .arg(tmp.join("out")))?;
    let index: Value = serde_json::from_str(
        &std::fs::read_to_string(tmp.join("out/index.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    let mut out: BTreeMap<String, BTreeMap<String, f64>> = BTreeMap::new();
    for state in index["states"]
        .as_array()
        .ok_or("index.json has no states")?
    {
        if let Some(err) = state.get("error") {
            return Err(format!("render {} failed: {err}", state["id"]));
        }
        let id = state["id"].as_str().ok_or("state without id")?;
        let scale = id.split('.').nth(1).ok_or("bad state id")?;
        let scene_file = state["scene"].as_str().ok_or("state without scene")?;
        let scene_path = if Path::new(scene_file).is_absolute() {
            PathBuf::from(scene_file)
        } else {
            tmp.join("out").join(scene_file)
        };
        let scene: Value = serde_json::from_str(
            &std::fs::read_to_string(&scene_path)
                .map_err(|e| format!("{}: {e}", scene_path.display()))?,
        )
        .map_err(|e| e.to_string())?;
        for node in scene["nodes"].as_array().into_iter().flatten() {
            for t in node["tokens"].as_array().into_iter().flatten() {
                let raw = t["token"].as_str().ok_or("token without name")?;
                let norm = normalize(raw);
                if exported.contains(raw) || exported.contains(&norm) {
                    continue;
                }
                let value = parse_px(t["value"].as_str().ok_or("token without value")?)?;
                let slot = out.entry(norm.clone()).or_default();
                if let Some(old) = slot.insert(scale.to_string(), value) {
                    if old != value {
                        return Err(format!("`{norm}` ({scale}) is {old} and {value}"));
                    }
                }
            }
        }
    }
    for (name, by_scale) in &out {
        if by_scale.len() != SCALES.len() {
            return Err(format!(
                "scoped token `{name}` has values for {:?} only",
                by_scale.keys()
            ));
        }
    }
    Ok(out)
}

fn ident(key: &str) -> Res<String> {
    let id = key.replace('-', "_");
    let ok = id.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if !ok || KEYWORDS.contains(&id.as_str()) {
        return Err(format!("token `{key}` is not a usable field name"));
    }
    Ok(id)
}

fn f(v: f64) -> String {
    format!("{:?}", v as f32)
}

fn dim(v: &Value) -> Res<f64> {
    v["value"]
        .as_f64()
        .ok_or_else(|| format!("bad dimension {v}"))
}

fn srgb_decode(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn color(hex: &str) -> Res<String> {
    let h = hex
        .strip_prefix('#')
        .ok_or_else(|| format!("bad color {hex}"))?;
    let byte = |i: usize| {
        u8::from_str_radix(h.get(i..i + 2).unwrap_or(""), 16)
            .map(f64::from)
            .map_err(|_| format!("bad color {hex}"))
    };
    let a = match h.len() {
        6 => 1.0,
        8 => byte(6)? / 255.0,
        _ => return Err(format!("bad color {hex}")),
    };
    let ch = |i| byte(i).map(|b| f(srgb_decode(b / 255.0)));
    Ok(format!(
        "Color {{ r: {}, g: {}, b: {}, a: {} }}",
        ch(0)?,
        ch(2)?,
        ch(4)?,
        f(a)
    ))
}

fn weight(name: &str) -> Res<u16> {
    Ok(match name {
        "light" => 300,
        "regular" => 400,
        "medium" => 500,
        "bold" => 700,
        "extra-bold" => 800,
        "black" => 900,
        _ => return Err(format!("unknown font weight `{name}`")),
    })
}

fn typography(v: &Value) -> Res<String> {
    Ok(format!(
        "Typography {{ family: {:?}, size: {}, weight: {}, line_height: {}, letter_spacing_em: {} }}",
        v["fontFamily"].as_str().ok_or("typography without family")?,
        f(dim(&v["fontSize"])?),
        weight(v["fontWeight"].as_str().ok_or("typography without weight")?)?,
        f(dim(&v["lineHeight"])?),
        f(dim(&v["letterSpacing"])?),
    ))
}

/// Rust type and expression for one token, or None for untyped values (gradient stops, text-align).
fn emit(token: &Value) -> Res<Option<(&'static str, String, Option<String>)>> {
    let v = &token["$value"];
    Ok(Some(match token["$type"].as_str() {
        Some("color") => (
            "Color",
            color(v.as_str().ok_or("color is not a string")?)?,
            None,
        ),
        Some("dimension") => {
            let unit = v["unit"].as_str().unwrap_or("px");
            ("f32", f(dim(v)?), (unit != "px").then(|| unit.to_string()))
        }
        Some("number") => ("f32", f(v.as_f64().ok_or("bad number")?), None),
        Some("fontFamily") => (
            "&'static str",
            format!("{:?}", v.as_str().ok_or("bad fontFamily")?),
            None,
        ),
        Some("fontWeight") => (
            "u16",
            weight(v.as_str().ok_or("bad fontWeight")?)?.to_string(),
            None,
        ),
        Some("typography") => ("Typography", typography(v)?, None),
        Some("shadow") => {
            let layers: Res<Vec<String>> = v
                .as_array()
                .ok_or("bad shadow")?
                .iter()
                .map(|s| {
                    Ok(format!(
                        "Shadow {{ x: {}, y: {}, blur: {}, spread: {}, color: {} }}",
                        f(dim(&s["offsetX"])?),
                        f(dim(&s["offsetY"])?),
                        f(dim(&s["blur"])?),
                        f(dim(&s["spread"])?),
                        color(s["color"].as_str().ok_or("shadow without color")?)?
                    ))
                })
                .collect();
            (
                "&'static [Shadow]",
                format!("&[{}]", layers?.join(", ")),
                None,
            )
        }
        None => return Ok(None),
        Some(other) => return Err(format!("unsupported token type `{other}`")),
    }))
}

fn generate() -> Res<()> {
    let (version, commit) = check_inputs()?;
    let dir = catalog_export()?;

    let mut modes: Vec<(String, Map<String, Value>)> = Vec::new();
    for scheme in SCHEMES {
        for scale in SCALES {
            let regular = export(scheme, scale, "regular")?;
            if regular != export(scheme, scale, "high")? {
                return Err(format!("contrast=high now differs from regular for {scheme}/{scale}: add a high-contrast token set"));
            }
            modes.push((
                format!("{}_{}", scheme.to_uppercase(), scale.to_uppercase()),
                regular,
            ));
        }
    }
    let keys: BTreeSet<String> = modes[0].1.keys().cloned().collect();
    for (name, m) in &modes {
        if m.keys().cloned().collect::<BTreeSet<_>>() != keys {
            return Err(format!("{name} exports a different token set"));
        }
    }
    let scoped = scoped_values(&dir, &keys)?;
    if let Some(clash) = scoped.keys().find(|k| keys.contains(*k)) {
        return Err(format!(
            "scoped token `{clash}` collides with an exported token"
        ));
    }

    let mut types: BTreeMap<&str, (&'static str, Option<String>)> = BTreeMap::new();
    let mut skipped = Vec::new();
    for key in &keys {
        match emit(&modes[0].1[key])? {
            Some((ty, _, unit)) => {
                types.insert(key, (ty, unit));
            }
            None => skipped.push(key.as_str()),
        }
    }

    let mut out = String::new();
    writeln!(out, "// @generated by `cargo xtask gen`. Do not edit.\n// Source: {version}, spectrum-design-data {commit}.").unwrap();
    writeln!(
        out,
        "// Skipped untyped tokens ({}): {}.\n",
        skipped.len(),
        skipped.join(", ")
    )
    .unwrap();
    writeln!(
        out,
        "use super::{{ColorScheme, Scale, Shadow, Typography}};\nuse rune::Color;\n"
    )
    .unwrap();
    writeln!(
        out,
        "#[derive(Clone, Debug, PartialEq)]\npub struct Tokens {{"
    )
    .unwrap();
    for (key, (ty, unit)) in &types {
        if let Some(u) = unit {
            writeln!(out, "    /// Unit: {u}.").unwrap();
        }
        writeln!(out, "    pub {}: {ty},", ident(key)?).unwrap();
    }
    writeln!(out, "    // Recipe-scoped values `export` omits (px).").unwrap();
    for key in scoped.keys() {
        writeln!(out, "    pub {}: f32,", ident(key)?).unwrap();
    }
    writeln!(out, "}}\n").unwrap();
    for (name, map) in &modes {
        let scale = name.split('_').nth(1).unwrap().to_lowercase();
        writeln!(out, "pub static {name}: Tokens = Tokens {{").unwrap();
        for key in types.keys() {
            let (_, expr, _) = emit(&map[*key])?.ok_or("token lost its type")?;
            writeln!(out, "    {}: {expr},", ident(key)?).unwrap();
        }
        for (key, by_scale) in &scoped {
            writeln!(out, "    {}: {},", ident(key)?, f(by_scale[&scale])).unwrap();
        }
        writeln!(out, "}};\n").unwrap();
    }
    writeln!(out, "pub fn tokens(scheme: ColorScheme, scale: Scale) -> &'static Tokens {{\n    match (scheme, scale) {{").unwrap();
    for (a, b) in [
        ("Light", "Desktop"),
        ("Light", "Mobile"),
        ("Dark", "Desktop"),
        ("Dark", "Mobile"),
    ] {
        writeln!(
            out,
            "        (ColorScheme::{a}, Scale::{b}) => &{}_{},",
            a.to_uppercase(),
            b.to_uppercase()
        )
        .unwrap();
    }
    writeln!(out, "    }}\n}}").unwrap();

    let target = write_rs("tokens", out)?;
    println!(
        "wrote {} ({} tokens, {} scoped, {} skipped)",
        target.display(),
        types.len(),
        scoped.len(),
        skipped.len()
    );
    generate_icons(&dir, &version, &commit)
}

fn write_rs(module: &str, out: String) -> Res<PathBuf> {
    let target = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../crates/spectrum-rune/src/{module}/generated.rs"));
    std::fs::write(&target, out).map_err(|e| format!("{}: {e}", target.display()))?;
    run(Command::new("rustfmt")
        .arg("--edition")
        .arg("2021")
        .arg(&target))?;
    Ok(target)
}

/// `icons/generated.rs` from `icons/ui.json`: one `static Icon` per (name, size) plus `icon(name, size)`.
/// Several paths are joined into one body. A non-square viewBox cannot be a Rune `Icon` (one `view`
/// number), so it is skipped and listed in the header; any other odd shape is an error.
fn generate_icons(dir: &Path, version: &str, commit: &str) -> Res<()> {
    let src = dir.join("icons/ui.json");
    let ui: Value = serde_json::from_str(
        &std::fs::read_to_string(&src).map_err(|e| format!("{}: {e}", src.display()))?,
    )
    .map_err(|e| e.to_string())?;
    let mut statics = String::new();
    let mut lookup = String::new();
    let mut skipped = Vec::new();
    // serde_json's map is sorted (no preserve_order), so output order is deterministic.
    for (name, icon) in ui["icons"].as_object().ok_or("ui.json has no icons")? {
        for (size, shape) in icon["sizes"]
            .as_object()
            .ok_or_else(|| format!("icon `{name}` has no sizes"))?
        {
            let id = format!("{name}-{size}");
            let n: u32 = size
                .parse()
                .map_err(|_| format!("icon `{id}`: size is not a number"))?;
            let vb: Vec<f64> = shape["viewBox"]
                .as_str()
                .unwrap_or("")
                .split(' ')
                .map(|v| v.parse().unwrap_or(f64::NAN))
                .collect();
            if vb.len() != 4
                || vb[0] != 0.0
                || vb[1] != 0.0
                || vb[2..].iter().any(|v| !v.is_finite() || *v <= 0.0)
            {
                return Err(format!(
                    "icon `{id}`: unsupported viewBox {}",
                    shape["viewBox"]
                ));
            }
            if vb[2] != vb[3] {
                skipped.push(format!("{id} ({}x{})", vb[2], vb[3]));
                continue;
            }
            let paths = shape["paths"].as_array().filter(|p| !p.is_empty());
            let body: Res<String> = paths
                .ok_or_else(|| format!("icon `{id}` has no paths"))?
                .iter()
                .map(|p| match p["d"].as_str() {
                    _ if p.as_object().is_some_and(|o| o.len() != 1) => {
                        Err(format!("icon `{id}`: path has keys besides `d`: {p}"))
                    }
                    Some(d) if !d.contains('"') => {
                        Ok(format!("<path d=\"{d}\" fill=\"currentColor\"/>"))
                    }
                    _ => Err(format!("icon `{id}`: path without usable `d`")),
                })
                .collect();
            let st = ident(&id)
                .map_err(|_| format!("icon `{id}` is not a usable static name"))?
                .to_uppercase();
            writeln!(
                statics,
                "pub static {st}: Icon = Icon {{ name: {id:?}, view: {}, body: {:?} }};",
                f(vb[2]),
                body?
            )
            .unwrap();
            writeln!(lookup, "        ({name:?}, {n}) => &{st},").unwrap();
        }
    }
    let mut out = String::new();
    writeln!(out, "// @generated by `cargo xtask gen`. Do not edit.\n// Source: {version}, spectrum-design-data {commit}, icons/ui.json.\n// Skipped non-square icons ({}): {}.\n", skipped.len(), skipped.join(", ")).unwrap();
    writeln!(out, "use rune::Icon;\n\n{statics}").unwrap();
    writeln!(out, "/// The UI icon `name` at `size` (the `ui.json` size key: 50, 75, 100, ...).\npub fn icon(name: &str, size: u32) -> Option<&'static Icon> {{\n    Some(match (name, size) {{\n{lookup}        _ => return None,\n    }})\n}}").unwrap();
    let target = write_rs("icons", out)?;
    println!("wrote {}", target.display());
    generate_constraints(dir, version, commit)
}

fn lit(v: &Value, ctx: &str) -> Res<String> {
    match v {
        Value::String(s) => Ok(format!("Lit::Str({s:?})")),
        Value::Bool(b) => Ok(format!("Lit::Bool({b})")),
        _ => Err(format!(
            "{ctx}: literal is neither a string nor a boolean: {v}"
        )),
    }
}

fn lits(v: &Value, ctx: &str) -> Res<Vec<Value>> {
    match v {
        Value::Array(a) if !a.is_empty() => Ok(a.clone()),
        Value::Array(_) => Err(format!("{ctx}: empty array")),
        one => Ok(vec![one.clone()]),
    }
}

/// SPEC-064 for one literal: the option is declared, and the literal is one of its declared values
/// (or a boolean, for a boolean option). Value checks are skipped when the option has no `values`.
fn check_lit(opt: &Value, v: &Value, ctx: &str) -> Res<()> {
    if let Some(values) = opt["values"].as_array() {
        if let Some(bad) = values.iter().find(|d| d.get("value").is_none()) {
            return Err(format!(
                "{ctx}: option `values` entry is not {{\"value\": ..}}: {bad}"
            ));
        }
        if !values.iter().any(|d| &d["value"] == v) {
            return Err(format!(
                "{ctx}: value {v} is not a declared value of the option"
            ));
        }
    } else if opt["type"] == "boolean" && !v.is_boolean() {
        return Err(format!("{ctx}: value {v} is not a boolean"));
    }
    Ok(())
}

/// `constraints/generated.rs` from every `components/*.json` `constraints[]` (spec
/// `option-constraints.md`), including components without rules. Unknown shapes and SPEC-064
/// violations (a name or value the component's `options` does not declare) are errors.
fn generate_constraints(dir: &Path, version: &str, commit: &str) -> Res<()> {
    let components = dir.join("components");
    let mut files: Vec<_> = std::fs::read_dir(&components)
        .map_err(|e| format!("{}: {e}", components.display()))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    let (mut body, mut n_rules) = (String::new(), 0);
    for path in &files {
        let file = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("?");
        let catalog: Value = serde_json::from_str(
            &std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?,
        )
        .map_err(|e| format!("{file}: {e}"))?;
        let rules = match &catalog["constraints"] {
            Value::Null => &Vec::new(),
            Value::Array(a) => a,
            other => return Err(format!("{file}: `constraints` is not an array: {other}")),
        };
        let mut defaults: BTreeMap<String, String> = BTreeMap::new();
        let mut rules_rs = String::new();
        for (i, rule) in rules.iter().enumerate() {
            let at = format!("{file} rule {i}");
            let obj = rule
                .as_object()
                .ok_or_else(|| format!("{at}: not an object"))?;
            if let Some(k) = obj.keys().find(|k| {
                !["when", "require", "severity", "description", "source"].contains(&k.as_str())
            }) {
                return Err(format!("{at}: unknown key `{k}`"));
            }
            let severity = match rule["severity"].as_str() {
                Some("error") => "Error",
                Some("advisory") => "Advisory",
                other => return Err(format!("{at}: unknown severity {other:?}")),
            };
            let description = match &rule["description"] {
                Value::Null => "None".to_string(),
                Value::String(s) => format!("Some({s:?})"),
                other => return Err(format!("{at}: description is not a string: {other}")),
            };
            let declared = |opt: &str| -> Res<&Value> {
                catalog["options"]
                    .get(opt)
                    .filter(|o| o.is_object())
                    .ok_or_else(|| {
                        format!("{at}: SPEC-064: option `{opt}` is not declared in `options`")
                    })
            };
            let mut note_default = |opt: &str, decl: &Value| -> Res<()> {
                match &decl["default"] {
                    Value::Null => {}
                    d => {
                        defaults.insert(
                            opt.to_string(),
                            lit(d, &format!("{at}: default of `{opt}`"))?,
                        );
                    }
                }
                Ok(())
            };
            let when = rule["when"]
                .as_object()
                .filter(|m| !m.is_empty())
                .ok_or_else(|| format!("{at}: `when` is not a non-empty object"))?;
            let mut when_rs = Vec::new();
            for (opt, v) in when {
                let decl = declared(opt)?;
                note_default(opt, decl)?;
                let ls: Res<Vec<String>> = lits(v, &format!("{at} when.{opt}"))?
                    .iter()
                    .map(|l| {
                        check_lit(decl, l, &format!("{at} when.{opt}"))?;
                        lit(l, &format!("{at} when.{opt}"))
                    })
                    .collect();
                when_rs.push(format!("({opt:?}, &[{}])", ls?.join(", ")));
            }
            let require = rule["require"]
                .as_object()
                .filter(|m| !m.is_empty())
                .ok_or_else(|| format!("{at}: `require` is not a non-empty object"))?;
            let mut require_rs = Vec::new();
            for (opt, v) in require {
                let decl = declared(opt)?;
                note_default(opt, decl)?;
                let req = match v {
                    Value::Array(a) if !a.is_empty() => {
                        let ls: Res<Vec<String>> = a
                            .iter()
                            .map(|l| {
                                check_lit(decl, l, &format!("{at} require.{opt}"))?;
                                lit(l, &format!("{at} require.{opt}"))
                            })
                            .collect();
                        format!("Require::OneOf(&[{}])", ls?.join(", "))
                    }
                    Value::Object(m) if m.len() == 1 && m.get("present") == Some(&Value::Bool(true)) => {
                        "Require::Present".to_string()
                    }
                    other => {
                        return Err(format!(
                            "{at}: require.{opt} is neither a non-empty array nor {{\"present\": true}}: {other}"
                        ))
                    }
                };
                require_rs.push(format!("({opt:?}, {req})"));
            }
            writeln!(
                rules_rs,
                "        Rule {{ when: &[{}], require: &[{}], severity: Severity::{severity}, description: {description} }},",
                when_rs.join(", "),
                require_rs.join(", ")
            )
            .unwrap();
            n_rules += 1;
        }
        let defaults_rs: Vec<String> = defaults
            .iter()
            .map(|(k, v)| format!("({k:?}, {v})"))
            .collect();
        writeln!(
            body,
            "    ComponentConstraints {{ name: {name:?}, defaults: &[{}], rules: &[{}] }},",
            defaults_rs.join(", "),
            if rules_rs.is_empty() {
                String::new()
            } else {
                format!("\n{rules_rs}    ")
            }
        )
        .unwrap();
    }
    let mut out = String::new();
    writeln!(out, "// @generated by `cargo xtask gen`. Do not edit.\n// Source: {version}, spectrum-design-data {commit}, components/*.json `constraints`.\n").unwrap();
    writeln!(
        out,
        "use super::{{ComponentConstraints, Lit, Require, Rule, Severity}};\n"
    )
    .unwrap();
    writeln!(out, "/// Every catalog under `components/`, sorted by name; most have no rules yet.\npub static COMPONENTS: &[ComponentConstraints] = &[\n{body}];\n").unwrap();
    writeln!(out, "pub fn component(name: &str) -> Option<&'static ComponentConstraints> {{\n    COMPONENTS.iter().find(|c| c.name == name)\n}}").unwrap();
    let target = write_rs("constraints", out)?;
    println!(
        "wrote {} ({} components, {n_rules} rules)",
        target.display(),
        files.len()
    );
    Ok(())
}
