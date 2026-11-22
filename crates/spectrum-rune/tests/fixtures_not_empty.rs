//! A regen script run against data it does not understand writes `[]` (a part renamed, an option dropped,
//! a relationship that no longer resolves) and every test reading it then passes over zero cases.
//! Fail on that instead.

#[test]
fn no_oracle_fixture_is_empty() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let empty = match &v {
            serde_json::Value::Array(a) => a.is_empty(),
            serde_json::Value::Object(o) => o.is_empty(),
            _ => false,
        };
        assert!(
            !empty,
            "{} is empty: re-run its regen script and check it against the oracle",
            path.display()
        );
    }
}
