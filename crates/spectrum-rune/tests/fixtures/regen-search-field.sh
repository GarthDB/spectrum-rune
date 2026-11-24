#!/usr/bin/env bash
# Regenerates search_field_scene.json from the oracle: the foundation's `search-field` recipe (a field label, a pill field
# holding a search icon and the value, and help text), rendered as the `rune` platform by `design-data` (versions in
# oracle.toml) against the foundation release pinned in .design-data.toml. Optional $1: a spectrum-design-data checkout to
# read instead. Coordinates are relative to the container; the test compares with 1px / 3-per-channel tolerance, and reads
# the border and the focus ring from pixels.
#
# Matrix: scheme x scale x size (s, m, l, xl) x case (rest, disabled, hover, focus); a label, the value "Ada" and help text.
# The recipe has no clear button, valid or error state. Not in the matrix: the placeholder (Rune draws it inside the editor,
# so it has no part to compare), no label, `width`. Text widths follow the installed font and are not compared.
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/oracle.sh"
out="$(cd "$(dirname "$0")" && pwd)/search_field_scene.json"
oracle_cd

render() {
  local scheme=$1 scale=$2 size=$3 kind=$4 opts=()
  case $kind in
    disabled) opts=(--option isDisabled=true) ;;
    hover) opts=(--state hover) ;;
    focus) opts=(--state keyboard-focus) ;;
  esac
  design-data render search-field --color-scheme "$scheme" --scale "$scale" --size "$size" --format scene \
    --option 'label="Search"' --option 'value="Ada"' --option 'helpText="Find things"' ${opts[@]+"${opts[@]}"} | jq -c \
    --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" --arg kind "$kind" '
    def col: if . == null then null else gsub("rgba?\\(|\\)| "; "") end;
    (.nodes[] | select(.part == "container")) as $c
    | def part($p): [.nodes[] | select(.part == $p)][0];
      part("field") as $f | part("value") as $v | part("leading-icon") as $i | part("focus-indicator") as $r
    | part("field-label") as $fl | part("field-label.label") as $ll | part("help-text") as $ht
    | part("help-text.text") as $ht_text
    | {scheme: $scheme, scale: $scale, size: $size, kind: $kind,
       container: {w: $c.w, h: $c.h},
       label: {y: ($fl.y - $c.y), h: $fl.h, text_y: ($ll.y - $c.y), color: ($ll.color | col)},
       field: {x: ($f.x - $c.x), y: ($f.y - $c.y), w: $f.w, h: $f.h, radius: $f.radius, fill: ($f.fill | col), stroke: ($f.stroke | col)},
       icon: {x: ($i.x - $f.x), y: ($i.y - $f.y), w: $i.w, color: ($i.color | col), gap_after: ($v.x - ($i.x + $i.w))},
       value: {x: ($v.x - $f.x), y: ($v.y - $f.y), h: $v.h, color: ($v.color | col)},
       help: {y: ($ht.y - $c.y), h: $ht.h, color: ($ht_text.color | col)}}
      + (if $r then {ring: {stroke: ($r.stroke | col)}} else {} end)'
}

{
  for scheme in light dark; do for scale in desktop mobile; do for size in s m l xl; do for kind in rest disabled hover focus; do
    render $scheme $scale $size $kind
  done; done; done; done
} | jq -s . > "$out"
echo "wrote $out"
