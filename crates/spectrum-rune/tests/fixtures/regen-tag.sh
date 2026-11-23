#!/usr/bin/env bash
# Regenerates tag_scene.json from the oracle: the foundation's `tag` recipe (a container with a label), rendered as the
# `rune` platform by `design-data` (versions in oracle.toml) against the foundation release pinned in .design-data.toml.
# Optional $1: a spectrum-design-data checkout to read instead. Coordinates are relative to the container; the test compares
# with 1px / 3-per-channel tolerance.
#
# Matrix: scheme x scale x size (s, m, l) x look (plain, selected, selected + emphasized, disabled) at rest; then hover, down
# and keyboard-focus at size m for each look, scheme and scale. Not in the matrix (not built): `isRemovable` (the Clear
# button), `hasAvatar` (the Avatar) and `icon` (the oracle draws none for it).
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/oracle.sh"
out="$(cd "$(dirname "$0")" && pwd)/tag_scene.json"
oracle_cd

render() {
  local scheme=$1 scale=$2 size=$3 look=$4 state=$5
  local args=(--color-scheme "$scheme" --scale "$scale" --size "$size" --format scene)
  case $look in
    selected) args+=(--option isSelected=true) ;;
    emphasized) args+=(--option isSelected=true --option isEmphasized=true) ;;
    disabled) args+=(--option isDisabled=true) ;;
  esac
  [ "$state" != rest ] && args+=(--state "$state")
  design-data render tag "${args[@]}" | jq -c \
    --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" --arg look "$look" --arg state "$state" '
    def col: gsub("rgba?\\(|\\)| "; "");
    (.nodes[] | select(.part == "container")) as $c
    | (.nodes[] | select(.part == "label")) as $l
    | ([.nodes[] | select(.part == "focus-indicator")][0]) as $r
    | {scheme: $scheme, scale: $scale, size: $size, look: $look, state: $state,
       container: {h: $c.h, radius: $c.radius, fill: ($c.fill | col)},
       label: {x: ($l.x - $c.x), y: ($l.y - $c.y), h: $l.h, size: $l.font_size, color: ($l.color | col)}}
      + (if $r then {ring: {x: ($r.x - $c.x), y: ($r.y - $c.y), h: $r.h, radius: $r.radius, stroke_width: $r.stroke_width, color: ($r.stroke | col)}} else {} end)'
}

{
  for scheme in light dark; do for scale in desktop mobile; do
    for size in s m l; do for look in plain selected emphasized disabled; do render $scheme $scale $size $look rest; done; done
    for look in plain selected emphasized; do for state in hover down keyboard-focus; do render $scheme $scale m $look $state; done; done
  done; done
} | jq -s . > "$out"
echo "wrote $out"
