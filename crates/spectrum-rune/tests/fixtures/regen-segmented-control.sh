#!/usr/bin/env bash
# Regenerates segmented_control_scene.json from the oracle: the foundation's `segmented-control` recipe (a rounded track
# holding items, each an icon and a label, the selected one on a raised fill), rendered as the `rune` platform by
# `design-data` (versions in oracle.toml) against the foundation release pinned in .design-data.toml. Optional $1: a
# spectrum-design-data checkout to read instead. Coordinates are relative to the container (an item's parts, to the item);
# the test compares with 1px / 3-per-channel tolerance.
#
# Matrix: scheme x scale x size (s, m, l, xl); three items: plain, selected, disabled. Not in the matrix: vertical
# orientation, hideTracker, and hover, down and keyboard-focus (the oracle draws them like rest). The oracle draws the
# selected item's label in the same subdued colour as the others.
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/oracle.sh"
out="$(cd "$(dirname "$0")" && pwd)/segmented_control_scene.json"
oracle_cd

ITEMS='[{"label":"Home","icon":"workflow:home"},{"label":"Docs","icon":"workflow:home","isSelected":true},{"label":"Gone","icon":"workflow:home","isDisabled":true}]'

render() {
  local scheme=$1 scale=$2 size=$3
  design-data render segmented-control --color-scheme "$scheme" --scale "$scale" --size "$size" --format scene \
    --option "items=$ITEMS" | jq -c --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" '
    def col: gsub("rgba?\\(|\\)| "; "");
    (.nodes[] | select(.part == "container")) as $c
    | def part($p): [.nodes[] | select(.part == $p)][0];
      {scheme: $scheme, scale: $scale, size: $size,
       container: {h: $c.h, radius: $c.radius, fill: ($c.fill | col)},
       items: [range(1; 4) as $i
         | part("item-\($i)") as $it
         | part("item-\($i).icon") as $ic
         | part("item-\($i).label") as $l
         | part("item-\($i + 1)") as $nx
         | {item: {x: ($it.x - $c.x), y: ($it.y - $c.y), h: $it.h, radius: $it.radius,
                   gap_after: (if $nx then ($nx.x - ($it.x + $it.w)) else null end)},
            icon: {x: ($ic.x - $it.x), w: $ic.w, color: ($ic.color | col)},
            label: {x: ($l.x - $it.x), size: $l.font_size, color: ($l.color | col)}}
           + (if $it.fill then {fill: ($it.fill | col)} else {} end)]}'
}

{
  for scheme in light dark; do for scale in desktop mobile; do for size in s m l xl; do
    render $scheme $scale $size
  done; done; done
} | jq -s . > "$out"
echo "wrote $out"
