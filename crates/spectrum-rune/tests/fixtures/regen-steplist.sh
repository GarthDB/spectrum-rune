#!/usr/bin/env bash
# Regenerates steplist_scene.json from the oracle: the foundation's `steplist` recipe (steps side by side, each a label over a
# row of a circle and a track to the next step), rendered as the `rune` platform by `design-data` (versions in oracle.toml)
# against the foundation release pinned in .design-data.toml. Optional $1: a spectrum-design-data checkout to read instead.
# Coordinates are relative to the container (the label and the row to their step, the circle and track to the row); the
# test compares with 1px / 3-per-channel tolerance.
#
# Matrix: scheme x scale x size (s, m, l, xl); six steps: completed, current, completed disabled, current disabled,
# incomplete, incomplete disabled. Not in the matrix: `completed hover` (the oracle's own hover state; the pointer's here),
# orientation. The oracle sizes a step to its label and lets the track fill what is left; the test compares that shape (the
# track runs from the circle to the step's right edge), not the widths, which follow the installed font.
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/oracle.sh"
out="$(cd "$(dirname "$0")" && pwd)/steplist_scene.json"
oracle_cd

ITEMS='[{"label":"First","stepState":"completed default"},{"label":"Second","stepState":"current default"},{"label":"Third","stepState":"completed default","isDisabled":true},{"label":"Fourth","stepState":"current default","isDisabled":true},{"label":"Fifth","stepState":"incomplete default"},{"label":"Sixth","stepState":"incomplete default","isDisabled":true}]'

render() {
  local scheme=$1 scale=$2 size=$3
  design-data render steplist --color-scheme "$scheme" --scale "$scale" --size "$size" --format scene \
    --option "items=$ITEMS" | jq -c --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" '
    def col: if . == null then null else gsub("rgba?\\(|\\)| "; "") end;
    (.nodes[] | select(.part == "container")) as $c
    | def part($p): [.nodes[] | select(.part == $p)][0];
      {scheme: $scheme, scale: $scale, size: $size, container: {h: $c.h},
       steps: [range(1; 7) as $i
         | part("step-\($i)") as $s
         | part("step-\($i + 1)") as $nx
         | part("step-\($i).label") as $l
         | part("step-\($i).row") as $r
         | part("step-\($i).visual") as $v
         | part("step-\($i).track") as $t
         | {step: {x: ($s.x - $c.x), y: ($s.y - $c.y), h: $s.h,
                   gap_after: (if $nx then ($nx.x - ($s.x + $s.w)) else null end)},
            label: {y: ($l.y - $s.y), h: $l.h, size: $l.font_size, color: ($l.color | col)},
            row: {y: ($r.y - $s.y), h: $r.h},
            visual: {x: ($v.x - $r.x), y: ($v.y - $r.y), w: $v.w, h: $v.h, radius: $v.radius,
                     fill: ($v.fill | col), stroke: ($v.stroke | col)}}
           + (if $t then {track: {y: ($t.y - $r.y), h: $t.h, fill: ($t.fill | col)}} else {} end)]}'
}

{
  for scheme in light dark; do for scale in desktop mobile; do for size in s m l xl; do
    render $scheme $scale $size
  done; done; done
} | jq -s . > "$out"
echo "wrote $out"
