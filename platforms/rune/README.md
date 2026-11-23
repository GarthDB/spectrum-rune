# spectrum-rune platform manifest

A Layer 2 [platform manifest](https://opensource.adobe.com/spectrum-design-data/spec/manifest/) owned by this
repo, not incubated in spectrum-design-data. It is the worked example of an implementation managing its own
manifest.

- `manifest.json`: identity and the foundation pin (`@adobe/spectrum-design-data@4.4.0`, the commit in `oracle.toml`).
- `extensions/implementations/`: the `spectrum_rune::` type behind each component.
- Status light has no extension any more: foundation 4.4.0 ships its own component and recipe (the promotion
  trial of bead 4qa.7, adobe/spectrum-design-data#1761, landed), and `StatusLight` is checked against that.
- Radio group has no extension any more either: foundation 4.4.0 ships its own recipe (adobe/spectrum-design-data #2067, an
  `options` slot named `default`, vertical and horizontal), and `RadioGroup` is checked against that.

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
- `RadioGroup` is checked for gap and item geometry only: the fixture's radio buttons carry no label (data 4.4.0's
  `with.label` could add them, not done), and the recipe's field label and help text are not built.
