#!/usr/bin/env bash
# Regenerates breadcrumbs_scene.json from the oracle: the foundation's `breadcrumbs` recipe (a row of breadcrumb-items, each
# a title and a chevron separator except the last), rendered as the `rune` platform by `design-data` (versions in
# oracle.toml) against the foundation release pinned in .design-data.toml. Optional $1: a spectrum-design-data checkout to
# read instead. Coordinates are relative to the container; the test compares with 1px / 3-per-channel tolerance.
#
# Matrix: scheme x scale x size (m, l) x separator (chevron, none); four items: a plain one, another, the current page
# (isCurrent) and a disabled one. Item widths follow the text (the installed font's), so the fixture keeps what does not:
# heights, offsets inside an item and the gap after the title. Not in the matrix: `isMultiline` (wrapping) and
# `isTruncated` (a collapsed menu); the recipe's hover, down and keyboard-focus are `notVisual`.
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/oracle.sh"
out="$(cd "$(dirname "$0")" && pwd)/breadcrumbs_scene.json"
oracle_cd

ITEMS='[{"label":"Home"},{"label":"Docs"},{"label":"Current","isCurrent":true},{"label":"Gone","isDisabled":true}]'

render() {
  local scheme=$1 scale=$2 size=$3 sep=$4
  design-data render breadcrumbs --color-scheme "$scheme" --scale "$scale" --size "$size" --format scene \
    --option "items=$ITEMS" --option "separator=\"$sep\"" | jq -c \
    --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" --arg sep "$sep" '
    def col: gsub("rgba?\\(|\\)| "; "");
    (.nodes[] | select(.part == "container")) as $c
    | def part($p): [.nodes[] | select(.part == $p)][0];
      {scheme: $scheme, scale: $scale, size: $size, separator: $sep, container: {h: $c.h},
       items: [range(1; 5) as $i
         | part("breadcrumb-item-\($i)") as $it
         | part("breadcrumb-item-\($i).breadcrumb-title") as $t
         | part("breadcrumb-item-\($i).separator") as $s
         | {item: {y: ($it.y - $c.y), h: $it.h},
            title: {x: ($t.x - $it.x), y: ($t.y - $it.y), h: $t.h, size: $t.font_size, weight: $t.font_weight, color: ($t.color | col)}}
           + (if $s then {separator: {gap: ($s.x - ($t.x + $t.w)), y: ($s.y - $it.y), w: $s.w, h: $s.h, color: ($s.color | col)}} else {} end)]}'
}

{
  for scheme in light dark; do for scale in desktop mobile; do for size in m l; do for sep in chevron none; do
    render $scheme $scale $size $sep
  done; done; done; done
} | jq -s . > "$out"
echo "wrote $out"
