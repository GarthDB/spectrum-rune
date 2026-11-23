#!/usr/bin/env bash
# Regenerates divider_scene.json and link_scene.json from the oracle: the foundation's `divider` and `link` recipes,
# rendered as the `rune` platform by `design-data` (versions in oracle.toml) against the foundation release pinned in
# .design-data.toml. Optional $1: a spectrum-design-data checkout to read instead. Coordinates are relative to the
# container (the line, for a divider); the test compares with 1px / 3-per-channel tolerance.
#
# Divider: scheme x scale x size (s, m, l) x orientation (horizontal, vertical), 16 long. Link: scheme x scale x variant
# (primary, secondary) x quiet x standalone x state (rest, hover, down, keyboard-focus). Not in the matrix: `staticColor`.
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/oracle.sh"
dir="$(cd "$(dirname "$0")" && pwd)"
oracle_cd

JQ_COMMON='
  def col: gsub("rgba?\\(|\\)| "; "");
  def part($p): [.nodes[] | select(.part == $p)][0];
'

divider() {
  local scheme=$1 scale=$2 size=$3 orientation=$4
  design-data render divider --color-scheme "$scheme" --scale "$scale" --size "$size" --format scene \
    --option "orientation=\"$orientation\"" | jq -c \
    --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" --arg orientation "$orientation" "$JQ_COMMON"'
    part("line") as $l
    | {scheme: $scheme, scale: $scale, size: $size, orientation: $orientation,
       line: {w: $l.w, h: $l.h, radius: $l.radius, fill: ($l.fill | col)}}'
}

link() {
  local scheme=$1 scale=$2 variant=$3 quiet=$4 standalone=$5 state=$6
  local args=(--color-scheme "$scheme" --scale "$scale" --format scene --option "variant=\"$variant\"" --option "isQuiet=$quiet" --option "isStandalone=$standalone")
  [ "$state" != rest ] && args+=(--state "$state")
  design-data render link "${args[@]}" | jq -c \
    --arg scheme "$scheme" --arg scale "$scale" --arg variant "$variant" --argjson quiet "$quiet" --argjson standalone "$standalone" --arg state "$state" "$JQ_COMMON"'
    part("container") as $c | part("text") as $t | part("focus-indicator") as $r
    | {scheme: $scheme, scale: $scale, variant: $variant, quiet: $quiet, standalone: $standalone, state: $state,
       container: {h: $c.h, radius: $c.radius},
       text: {h: $t.h, size: $t.font_size, color: ($t.color | col), underline: ($t.underline != null)}}
      + (if $r then {ring: {x: ($r.x - $c.x), y: ($r.y - $c.y), grow_w: ($r.w - $c.w), h: $r.h, radius: $r.radius, stroke_width: $r.stroke_width, color: ($r.stroke | col)}} else {} end)'
}

{
  for scheme in light dark; do for scale in desktop mobile; do for size in s m l; do for o in horizontal vertical; do
    divider $scheme $scale $size $o
  done; done; done; done
} | jq -s . > "$dir/divider_scene.json"

{
  for scheme in light dark; do for scale in desktop mobile; do for variant in primary secondary; do
    for quiet in false true; do for standalone in true false; do for state in rest hover down keyboard-focus; do
      link $scheme $scale $variant $quiet $standalone $state
    done; done; done
  done; done; done
} | jq -s . > "$dir/link_scene.json"
echo "wrote $dir/divider_scene.json and link_scene.json"
