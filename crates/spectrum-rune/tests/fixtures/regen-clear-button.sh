#!/usr/bin/env bash
# Regenerates clear_button_scene.json from the oracle: the foundation's `clear-button` recipe (a circular container with a
# cross icon), rendered as the `rune` platform by `design-data` (versions in oracle.toml) against the foundation release
# pinned in .design-data.toml. Optional $1: a spectrum-design-data checkout to read instead. Coordinates are relative to
# the container; the test compares with 1px / 3-per-channel tolerance.
#
# Matrix: scheme x scale x size (s, m, l, xl) at rest, then disabled; hover, down and keyboard-focus at size m. Not in the
# matrix: `staticColor`.
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/oracle.sh"
out="$(cd "$(dirname "$0")" && pwd)/clear_button_scene.json"
oracle_cd

render() {
  local scheme=$1 scale=$2 size=$3 look=$4 state=$5
  local args=(--color-scheme "$scheme" --scale "$scale" --size "$size" --format scene)
  [ "$look" = disabled ] && args+=(--option isDisabled=true)
  [ "$state" != rest ] && args+=(--state "$state")
  design-data render clear-button "${args[@]}" | jq -c \
    --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" --arg look "$look" --arg state "$state" '
    def col: gsub("rgba?\\(|\\)| "; "");
    (.nodes[] | select(.part == "container")) as $c
    | (.nodes[] | select(.part == "icon")) as $i
    | ([.nodes[] | select(.part == "focus-indicator")][0]) as $r
    | {scheme: $scheme, scale: $scale, size: $size, look: $look, state: $state,
       container: {w: $c.w, h: $c.h, radius: $c.radius},
       icon: {x: ($i.x - $c.x), y: ($i.y - $c.y), w: $i.w, h: $i.h, color: ($i.color | col)}}
      + (if $r then {ring: {x: ($r.x - $c.x), y: ($r.y - $c.y), w: $r.w, radius: $r.radius, stroke_width: $r.stroke_width, color: ($r.stroke | col)}} else {} end)'
}

{
  for scheme in light dark; do for scale in desktop mobile; do
    for size in s m l xl; do render $scheme $scale $size plain rest; render $scheme $scale $size disabled rest; done
    for state in hover down keyboard-focus; do render $scheme $scale m plain $state; done
  done; done
} | jq -s . > "$out"
echo "wrote $out"
