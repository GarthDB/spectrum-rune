#!/usr/bin/env bash
# Regenerates side_navigation_scene.json from the oracle: the foundation's `side-navigation` recipe (a list of items, each a
# row of icon and label plus a selection indicator, under an optional section header), rendered as the `rune` platform by
# `design-data` (versions in oracle.toml) against the foundation release pinned in .design-data.toml. Optional $1: a
# spectrum-design-data checkout to read instead. Coordinates are relative to the container (an item's parts, to the item);
# the test compares with 1px / 3-per-channel tolerance.
#
# Matrix: scheme x scale x section title (with, without); four items: plain, selected, disabled, plain. Not in the matrix:
# the items' `state` (the oracle draws hover, down and keyboard-focus like rest), `level` (it does not indent) and items
# without an icon (the oracle draws a default icon for every item).
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/oracle.sh"
out="$(cd "$(dirname "$0")" && pwd)/side_navigation_scene.json"
oracle_cd

ITEMS='[{"label":"Home","icon":"workflow:home"},{"label":"Docs","icon":"workflow:home","isSelected":true},{"label":"Gone","icon":"workflow:home","isDisabled":true},{"label":"Settings","icon":"workflow:home"}]'

render() {
  local scheme=$1 scale=$2 title=$3
  local args=(--color-scheme "$scheme" --scale "$scale" --format scene --option "items=$ITEMS")
  [ "$title" = with ] && args+=(--option 'sectionTitle="Section"')
  design-data render side-navigation "${args[@]}" | jq -c \
    --arg scheme "$scheme" --arg scale "$scale" --arg title "$title" '
    def col: gsub("rgba?\\(|\\)| "; "");
    (.nodes[] | select(.part == "container")) as $c
    | def part($p): [.nodes[] | select(.part == $p)][0];
      ([.nodes[] | select(.part == "section-header")][0]) as $s
      | {scheme: $scheme, scale: $scale, title: $title, container: {w: $c.w, h: $c.h}}
        + (if $s then {header: {x: ($s.x - $c.x), y: ($s.y - $c.y), h: $s.h, size: $s.font_size, color: ($s.color | col)}} else {} end)
        + {items: [range(1; 5) as $i
          | part("item-\($i)") as $it
          | part("item-\($i).icon") as $ic
          | part("item-\($i).label") as $l
          | part("item-\($i).selection-indicator") as $si
          | {item: {y: ($it.y - $c.y), h: $it.h, w: $it.w},
             icon: {x: ($ic.x - $it.x), y: ($ic.y - $it.y), w: $ic.w, color: ($ic.color | col)},
             label: {x: ($l.x - $it.x), y: ($l.y - $it.y), h: $l.h, size: $l.font_size, color: ($l.color | col)}}
            + (if $si then {indicator: {x: ($si.x - $it.x), y: ($si.y - $it.y), w: $si.w, h: $si.h, fill: ($si.fill | col)}} else {} end)]}'
}

{
  for scheme in light dark; do for scale in desktop mobile; do for title in with without; do
    render $scheme $scale $title
  done; done; done
} | jq -s . > "$out"
echo "wrote $out"
