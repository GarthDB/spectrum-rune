#!/usr/bin/env bash
# Regenerates help_text_scene.json from the oracle: the foundation's `help-text` recipe (a line of text, with an alert icon
# before it for the negative variant), rendered as the `rune` platform by `design-data` (versions in oracle.toml) against
# the foundation release pinned in .design-data.toml. Optional $1: a spectrum-design-data checkout to read instead.
# Coordinates are relative to the container; the test compares with 1px / 3-per-channel tolerance.
#
# Matrix: scheme x scale x size (s, m, l, xl) x case (neutral, negative, negative with the icon hidden, disabled). Text
# widths follow the installed font and are not compared.
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/oracle.sh"
out="$(cd "$(dirname "$0")" && pwd)/help_text_scene.json"
oracle_cd

render() {
  local scheme=$1 scale=$2 size=$3 kind=$4 opts=()
  case $kind in
    negative) opts=(--option 'variant="negative"') ;;
    hidden) opts=(--option 'variant="negative"' --option hideIcon=true) ;;
    disabled) opts=(--option isDisabled=true) ;;
  esac
  design-data render help-text --color-scheme "$scheme" --scale "$scale" --size "$size" --format scene \
    --option 'text="Enter a name"' ${opts[@]+"${opts[@]}"} | jq -c \
    --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" --arg kind "$kind" '
    def col: if . == null then null else gsub("rgba?\\(|\\)| "; "") end;
    (.nodes[] | select(.part == "container")) as $c
    | def part($p): [.nodes[] | select(.part == $p)][0];
      part("text") as $t | part("icon") as $i
    | {scheme: $scheme, scale: $scale, size: $size, kind: $kind, container: {h: $c.h},
       text: {x: ($t.x - $c.x), y: ($t.y - $c.y), h: $t.h, size: $t.font_size, color: ($t.color | col)}}
      + (if $i then {icon: {x: ($i.x - $c.x), y: ($i.y - $c.y), w: $i.w, gap_after: ($t.x - ($i.x + $i.w)), color: ($i.color | col)}} else {} end)'
}

{
  for scheme in light dark; do for scale in desktop mobile; do for size in s m l xl; do for kind in neutral negative hidden disabled; do
    render $scheme $scale $size $kind
  done; done; done; done
} | jq -s . > "$out"
echo "wrote $out"
