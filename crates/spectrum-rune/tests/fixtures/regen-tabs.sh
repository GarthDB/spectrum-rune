#!/usr/bin/env bash
# Regenerates tabs_scene.json from the oracle: the foundation's `tabs` recipe (a row of tab items, each an icon and a label with
# a selection indicator under the selected one), rendered as the `rune` platform by `design-data` (versions in oracle.toml)
# against the foundation release pinned in .design-data.toml. Optional $1: a spectrum-design-data checkout to read instead.
# Coordinates are relative to the container (an item's parts, to the item); the test compares with 1px / 3-per-channel
# tolerance.
#
# Matrix: scheme x scale x size (s, m, l, xl) x density (regular, compact); four items: plain, selected, disabled, plain.
# Not in the matrix: `orientation: vertical` (the oracle draws the selected item's indicator with no height there) and the
# tabs' hover, down and keyboard-focus (the oracle draws them like rest). Items carry an icon (the oracle draws one anyway).
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/oracle.sh"
out="$(cd "$(dirname "$0")" && pwd)/tabs_scene.json"
oracle_cd

ITEMS='[{"label":"Home","icon":"workflow:home"},{"label":"Docs","icon":"workflow:home","isSelected":true},{"label":"Gone","icon":"workflow:home","isDisabled":true},{"label":"More","icon":"workflow:home"}]'

render() {
  local scheme=$1 scale=$2 size=$3 density=$4
  design-data render tabs --color-scheme "$scheme" --scale "$scale" --size "$size" --format scene \
    --option "items=$ITEMS" --option "density=\"$density\"" | jq -c \
    --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" --arg density "$density" '
    def col: gsub("rgba?\\(|\\)| "; "");
    (.nodes[] | select(.part == "container")) as $c
    | def part($p): [.nodes[] | select(.part == $p)][0];
      {scheme: $scheme, scale: $scale, size: $size, density: $density, container: {h: $c.h},
       items: [range(1; 5) as $i
         | part("tab-item-\($i)") as $it
         | part("tab-item-\($i).icon") as $ic
         | part("tab-item-\($i).label") as $l
         | part("tab-item-\($i).selection-indicator") as $si
         | part("tab-item-\($i + 1)") as $nx
         | {item: {x: ($it.x - $c.x), y: ($it.y - $c.y), h: $it.h, gap_after: (if $nx then ($nx.x - ($it.x + $it.w)) else null end)},
            icon: {x: ($ic.x - $it.x), y: ($ic.y - $it.y), w: $ic.w, color: ($ic.color | col)},
            label: {x: ($l.x - $it.x), y: ($l.y - $it.y), h: $l.h, size: $l.font_size, color: ($l.color | col)}}
           + (if $si then {indicator: {x: ($si.x - $it.x), y: ($si.y - $it.y), w_matches_item: (($si.w - $it.w) | fabs < 0.01), h: $si.h, fill: ($si.fill | col)}} else {} end)]}'
}

{
  for scheme in light dark; do for scale in desktop mobile; do for size in s m l xl; do for density in regular compact; do
    render $scheme $scale $size $density
  done; done; done; done
} | jq -s . > "$out"
echo "wrote $out"
