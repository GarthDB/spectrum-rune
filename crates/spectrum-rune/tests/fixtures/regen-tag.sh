#!/usr/bin/env bash
# Regenerates tag_scene.json from the oracle: the foundation's `tag` recipe (a container with a label), rendered as the
# `rune` platform by `design-data` (versions in oracle.toml) against the foundation release pinned in .design-data.toml.
# Optional $1: a spectrum-design-data checkout to read instead. Coordinates are relative to the container; the test compares
# with 1px / 3-per-channel tolerance.
#
# Matrix: scheme x scale x size (s, m, l) x look (plain, selected, selected + emphasized, disabled) at rest; then hover, down
# and keyboard-focus at size m for each look, scheme and scale; then the same removable (`isRemovable`, with its Clear button)
# for plain, selected and disabled at rest and plain on hover. Not in the matrix (not built): `hasAvatar` (the Avatar) and
# `icon` (the oracle draws none for it).
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/oracle.sh"
out="$(cd "$(dirname "$0")" && pwd)/tag_scene.json"
oracle_cd

render() {
  local scheme=$1 scale=$2 size=$3 look=$4 state=$5 removable=${6:-false}
  local args=(--color-scheme "$scheme" --scale "$scale" --size "$size" --format scene)
  case $look in
    selected) args+=(--option isSelected=true) ;;
    emphasized) args+=(--option isSelected=true --option isEmphasized=true) ;;
    disabled) args+=(--option isDisabled=true) ;;
  esac
  [ "$removable" = true ] && args+=(--option isRemovable=true)
  [ "$state" != rest ] && args+=(--state "$state")
  design-data render tag "${args[@]}" | jq -c \
    --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" --arg look "$look" --arg state "$state" --argjson removable "$removable" '
    def col: gsub("rgba?\\(|\\)| "; "");
    (.nodes[] | select(.part == "container")) as $c
    | (.nodes[] | select(.part == "label")) as $l
    | ([.nodes[] | select(.part == "focus-indicator")][0]) as $r
    | ([.nodes[] | select(.part == "clear-button")][0]) as $b
    | ([.nodes[] | select(.part == "clear-button.icon")][0]) as $bi
    | {scheme: $scheme, scale: $scale, size: $size, look: $look, state: $state, removable: $removable,
       container: {h: $c.h, radius: $c.radius, fill: ($c.fill | col)},
       label: {x: ($l.x - $c.x), y: ($l.y - $c.y), h: $l.h, size: $l.font_size, color: ($l.color | col)}}
      + (if $b then {clear: {gap: ($b.x - ($l.x + $l.w)), right: (($c.x + $c.w) - ($b.x + $b.w)), y: ($b.y - $c.y), w: $b.w, h: $b.h, icon: {x: ($bi.x - $b.x), y: ($bi.y - $b.y), w: $bi.w, color: ($bi.color | col)}}} else {} end)
      + (if $r then {ring: {x: ($r.x - $c.x), y: ($r.y - $c.y), h: $r.h, radius: $r.radius, stroke_width: $r.stroke_width, color: ($r.stroke | col)}} else {} end)'
}

{
  for scheme in light dark; do for scale in desktop mobile; do
    for size in s m l; do for look in plain selected emphasized disabled; do render $scheme $scale $size $look rest; done; done
    for look in plain selected emphasized; do for state in hover down keyboard-focus; do render $scheme $scale m $look $state; done; done
    for size in s m l; do for look in plain selected disabled; do render $scheme $scale $size $look rest true; done; done
    render $scheme $scale m plain hover true
  done; done
} | jq -s . > "$out"
echo "wrote $out"
