#!/usr/bin/env bash
# Regenerates popover_scene.json from the oracle: the foundation's `popover` recipe (a container with a content slot and
# a tip on the side that faces the trigger), rendered as the `rune` platform by `design-data` (versions in oracle.toml)
# against the foundation release pinned in .design-data.toml. Optional $1: a spectrum-design-data checkout to read
# instead. Coordinates are relative to the container; the test compares with 1px / 3-per-channel tolerance.
#
# Matrix: scheme x placement (top, bottom, left, right, start, end) x tip shown or hidden, at 120x60; then two other
# sizes (200x100, 80x40) for `top`, with the tip. Not in the matrix: the aligned placements (`top left`, ...), whose tip
# follows the trigger's centre, which a scene has no trigger for.
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/oracle.sh"
out="$(cd "$(dirname "$0")" && pwd)/popover_scene.json"
oracle_cd

render() {
  local scheme=$1 placement=$2 hide=$3 w=$4 h=$5
  design-data render popover --color-scheme "$scheme" --format scene \
    --option "width=$w" --option "height=$h" --option "placement=\"$placement\"" --option "hideTip=$hide" | jq -c \
    --arg scheme "$scheme" --arg placement "$placement" --argjson hide "$hide" '
    def col: gsub("rgba?\\(|\\)| "; "");
    (.nodes[] | select(.part == "container")) as $c
    | ([.nodes[] | select(.part | startswith("tip-"))][0]) as $t
    | {scheme: $scheme, placement: $placement, hideTip: $hide,
       container: {w: $c.w, h: $c.h, radius: $c.radius, fill: ($c.fill | col),
                   shadow: [$c.shadow[] | {x, y, blur, spread, color: (.color | col)}]}}
      + (if $t then {tip: {part: $t.part, x: ($t.x - $c.x), y: ($t.y - $c.y), w: $t.w, h: $t.h, fill: ($t.fill | col)}} else {} end)'
}

{
  for scheme in light dark; do
    for placement in top bottom left right start end; do
      for hide in false true; do render $scheme $placement $hide 120 60; done
    done
    render $scheme top false 200 100
    render $scheme top false 80 40
  done
} | jq -s . > "$out"
echo "wrote $out"
