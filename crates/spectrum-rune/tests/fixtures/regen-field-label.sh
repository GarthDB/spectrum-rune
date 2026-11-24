#!/usr/bin/env bash
# Regenerates field_label_scene.json from the oracle: the foundation's `field-label` recipe (a label and, when required, an
# asterisk after it), rendered as the `rune` platform by `design-data` (versions in oracle.toml) against the foundation
# release pinned in .design-data.toml. Optional $1: a spectrum-design-data checkout to read instead. Coordinates are
# relative to the container; the test compares with 1px / 3-per-channel tolerance.
#
# Matrix: scheme x scale x size (s, m, l, xl) x case (plain, required, required with a text indicator, disabled). The oracle
# draws no necessity indicator for `necessityIndicator: text` (Spectrum shows "(required)"). Not in the matrix: the side
# label position (the oracle draws it like the top one), static colours, wrapping. Text widths follow the installed font,
# so the label's width is not compared, only the gap to the asterisk.
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/oracle.sh"
out="$(cd "$(dirname "$0")" && pwd)/field_label_scene.json"
oracle_cd

render() {
  local scheme=$1 scale=$2 size=$3 kind=$4 opts=()
  case $kind in
    required) opts=(--option isRequired=true) ;;
    text) opts=(--option isRequired=true --option 'necessityIndicator="text"') ;;
    disabled) opts=(--option isDisabled=true) ;;
  esac
  design-data render field-label --color-scheme "$scheme" --scale "$scale" --size "$size" --format scene \
    --option 'label="Name"' ${opts[@]+"${opts[@]}"} | jq -c \
    --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" --arg kind "$kind" '
    def col: if . == null then null else gsub("rgba?\\(|\\)| "; "") end;
    (.nodes[] | select(.part == "container")) as $c
    | def part($p): [.nodes[] | select(.part == $p)][0];
      part("label") as $l | part("necessity-indicator") as $n
    | {scheme: $scheme, scale: $scale, size: $size, kind: $kind, container: {h: $c.h},
       label: {x: ($l.x - $c.x), y: ($l.y - $c.y), h: $l.h, size: $l.font_size, color: ($l.color | col)}}
      + (if $n then {indicator: {gap: ($n.x - ($l.x + $l.w)), y: ($n.y - $c.y), w: $n.w, h: $n.h, color: ($n.color | col)}} else {} end)'
}

{
  for scheme in light dark; do for scale in desktop mobile; do for size in s m l xl; do for kind in plain required text disabled; do
    render $scheme $scale $size $kind
  done; done; done; done
} | jq -s . > "$out"
echo "wrote $out"
