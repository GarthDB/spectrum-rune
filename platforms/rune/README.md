# spectrum-rune platform manifest

A Layer 2 [platform manifest](https://opensource.adobe.com/spectrum-design-data/spec/manifest/) owned by this
repo, not incubated in spectrum-design-data. It is the worked example of an implementation managing its own
manifest.

- `manifest.json`: identity and the foundation pin (`@adobe/spectrum-design-data@3.10.0`, the commit in `oracle.toml`).
- `extensions/implementations/`: the `spectrum_rune::` type behind each component.
- `extensions/visuals/radio-group.json`: a recipe the foundation does not have yet. It is the oracle for
  `RadioGroup` and a candidate to promote (artifact move, `platforms/README.md` in spectrum-design-data).

`.design-data.toml` at the repo root selects it (`default_platform = "rune"`):

```sh
design-data validate-manifest
design-data platform show rune
```

The fixture scripts (`crates/spectrum-rune/tests/fixtures/`) run the CLI as `rune` through `oracle_cd`. They read the
foundation from an oracle checkout, not from the `github` `[source]`: CLI 0.27.1 fetches no relationships, visuals or
icons from a `github` source, so `render` cannot bind recipes to it.

No token overrides: every value comes from the foundation.

## Known gaps

- A `github` `[source]` cannot drive `render` (adobe/spectrum-design-data#1732) and no CLI command exposes icons
  (#1733), so `cargo xtask gen` still reads `icons/ui.json` from the oracle checkout.
- Where Rune cannot draw what a recipe says (focus-visible stroke colour, hover-vs-focus precedence, busy state, font
  family) the manifest has nowhere to say so; those are tracked in beads (a14.14, wav), not in the manifest.
- The `radio-group` recipe's nested radio buttons carry no label (nested parts get none), so `RadioGroup` is checked
  for gap and item geometry only.
