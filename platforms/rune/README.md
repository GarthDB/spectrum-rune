# spectrum-rune platform manifest

A Layer 2 [platform manifest](https://opensource.adobe.com/spectrum-design-data/spec/manifest/) owned by this
repo, not incubated in spectrum-design-data. It is the worked example of an implementation managing its own
manifest.

- `manifest.json`: identity and the foundation pin (`@adobe/spectrum-design-data@3.12.0`, the commit in `oracle.toml`).
- `extensions/implementations/`: the `spectrum_rune::` type behind each component.
- `extensions/components/status-light.json` and `extensions/visuals/status-light.json`: Status light as Rune defines it
  before the foundation does: the component with its anatomy fixed (`container`, `dot`, `label`; a whole-file copy of the
  foundation component, so re-sync it when the pin moves) and a recipe bound only through relationships (plus a literal
  `0` block padding, because the renderer sizes a row with neither `minHeight` nor block padding at 32px). It is as tall as
  its text with the dot centred on the first line, as Figma and React Spectrum S2 draw it. The oracle for `StatusLight`;
  the workflow trial for promoting a component and recipe upstream (bead 4qa.7, adobe/spectrum-design-data#1761).
- `extensions/visuals/radio-group.json`: a recipe the foundation does not have yet. It is the oracle for
  `RadioGroup` and a candidate to promote (artifact move, `platforms/README.md` in spectrum-design-data).

`.design-data.toml` at the repo root selects it (`default_platform = "rune"`):

```sh
design-data validate-manifest
design-data platform show rune
```

The fixture scripts (`crates/spectrum-rune/tests/fixtures/`) run the CLI as `rune` from the repo root, so they read the
pinned foundation release through the `github` `[source]` and need no checkout. This needs CLI 0.31.0 or later: 0.27.1
fetched no relationships, visuals or icons from a `github` source, so `render` could not bind recipes to it.

No token overrides: every value comes from the foundation.

## Known gaps

- Where Rune cannot draw what a recipe says (focus-visible stroke colour, hover-vs-focus precedence, busy state, font
  family) the manifest has nowhere to say so; those are tracked in beads (a14.14, wav), not in the manifest.
- The `radio-group` recipe's nested radio buttons carry no label (nested parts get none), so `RadioGroup` is checked
  for gap and item geometry only.
