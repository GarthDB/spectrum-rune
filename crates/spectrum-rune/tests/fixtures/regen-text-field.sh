#!/usr/bin/env bash
# Regenerates text_field_scene.json from the oracle: the foundation's `text-field` recipe (a field label, a bordered field
# holding the value and, when valid, a checkmark, and help text), rendered as the `rune` platform by `design-data`
# (versions in oracle.toml) against the foundation release pinned in .design-data.toml. Optional $1: a
# spectrum-design-data checkout to read instead. Coordinates are relative to the container; the test compares with 1px /
# 3-per-channel tolerance, and reads the border and the focus ring from pixels.
#
# Matrix: scheme x scale x size (s, m, l, xl) x case (rest, valid, error, disabled, hover, focus); a label, the value "Ada"
# and help text. Not in the matrix: no label (the oracle draws an empty label's height anyway), `width` (the oracle ignores
# it), character count, side labels, a required label. Text widths follow the installed font and are not compared.
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/oracle.sh"
out="$(cd "$(dirname "$0")" && pwd)/text_field_scene.json"
oracle_cd

render() {
  local scheme=$1 scale=$2 size=$3 kind=$4 opts=()
  case $kind in
    valid) opts=(--option showValidIcon=true) ;;
    error) opts=(--option isError=true) ;;
    disabled) opts=(--option isDisabled=true) ;;
    hover) opts=(--state hover) ;;
    focus) opts=(--state keyboard-focus) ;;
  esac
  design-data render text-field --color-scheme "$scheme" --scale "$scale" --size "$size" --format scene \
    --option 'label="Name"' --option 'value="Ada"' --option 'helpText="Your full name"' ${opts[@]+"${opts[@]}"} | jq -c \
    --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" --arg kind "$kind" '
    def col: if . == null then null else gsub("rgba?\\(|\\)| "; "") end;
    (.nodes[] | select(.part == "container")) as $c
    | def part($p): [.nodes[] | select(.part == $p)][0];
      part("field") as $f | part("value") as $v | part("validation-marker") as $m | part("focus-indicator") as $r
    | part("field-label") as $fl | part("field-label.label") as $ll | part("help-text") as $ht
    | part("help-text.text") as $ht_text | part("help-text.icon") as $ht_icon
    | {scheme: $scheme, scale: $scale, size: $size, kind: $kind,
       container: {w: $c.w, h: $c.h},
       label: {y: ($fl.y - $c.y), h: $fl.h, text_y: ($ll.y - $c.y), text_h: $ll.h, color: ($ll.color | col)},
       field: {x: ($f.x - $c.x), y: ($f.y - $c.y), w: $f.w, h: $f.h, radius: $f.radius, fill: ($f.fill | col), stroke: ($f.stroke | col)},
       value: {x: ($v.x - $f.x), y: ($v.y - $f.y), h: $v.h, size: $v.font_size, color: ($v.color | col)},
       help: {y: ($ht.y - $c.y), h: $ht.h, color: ($ht_text.color | col)}}
      + (if $m then {marker: {x_from_right: (($f.x + $f.w) - ($m.x + $m.w)), y: ($m.y - $f.y), w: $m.w, color: ($m.color | col)}} else {} end)
      + (if $ht_icon then {help_icon: {x: ($ht_icon.x - $ht.x), w: $ht_icon.w, color: ($ht_icon.color | col)}} else {} end)
      + (if $r then {ring: {x: ($r.x - $f.x), y: ($r.y - $f.y), w: $r.w, h: $r.h, radius: $r.radius, stroke: ($r.stroke | col)}} else {} end)'
}

{
  for scheme in light dark; do for scale in desktop mobile; do for size in s m l xl; do for kind in rest valid error disabled hover focus; do
    render $scheme $scale $size $kind
  done; done; done; done
} | jq -s . > "$out"
echo "wrote $out"
