#!/usr/bin/env bash
# Regenerates status_light_scene.json from the oracle: the `status-light` recipe in platforms/rune/extensions/visuals
# (with its component in the sibling extension; the foundation has neither yet), rendered as the `rune` platform by `design-data` (versions in oracle.toml) against the pinned foundation
# release (see regen-radio-button.sh). Coordinates are relative to the container.
# Also writes status_light_wrap.json: the oracle at a request width (wrap.sh), where the label wraps.
#
# Matrix: every variant of the catalog component x the four scheme+scale corners at size M, then sizes S, L, XL x the
# four corners for one semantic and one named colour. The recipe has no minimum height and no block padding: the
# component is as tall as its text, as in Figma and React Spectrum S2 (see the recipe's README note).
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/oracle.sh"
source "$(dirname "$0")/wrap.sh"
out="$(cd "$(dirname "$0")" && pwd)/status_light_scene.json"
oracle_cd

CORNERS=("light desktop" "light mobile" "dark desktop" "dark mobile")
VARIANTS="informative neutral positive notice negative indigo celery chartreuse yellow magenta fuchsia purple seafoam pink turquoise cinnamon brown silver cyan gray red orange green blue accent"

render() {
  local v=$1 size=$2 scheme=$3 scale=$4
  design-data render status-light --size "$size" --color-scheme "$scheme" --scale "$scale" --format scene \
    --option "variant=\"$v\"" --label Label | jq -c \
    --arg v "$v" --arg size "$size" --arg scheme "$scheme" --arg scale "$scale" '
    select((.diagnostics // []) | length == 0)
    | (.nodes[] | select(.part == "container")) as $c
    | def r: (. * 100 | round) / 100;
      def col: gsub("rgba?\\(|\\)| "; "");
      def part($p): [.nodes[] | select(.part == $p)][0];
      (part("dot")) as $d | (part("label")) as $l
      | {scheme: $scheme, scale: $scale, size: $size, variant: $v,
         container: {w: ($c.w | r), h: ($c.h | r)},
         dot: {x: (($d.x - $c.x) | r), y: (($d.y - $c.y) | r), w: ($d.w | r), h: ($d.h | r), radius: ($d.radius | r), fill: ($d.fill | col)},
         label: {x: (($l.x - $c.x) | r), color: ($l.color | col)}}'
}

{
  for corner in "${CORNERS[@]}"; do set -- $corner
    for v in $VARIANTS; do render $v m $1 $2; done
  done
  for corner in "${CORNERS[@]}"; do set -- $corner
    for size in s l xl; do render informative $size $1 $2; render celery $size $1 $2; done
  done
} | jq -sc '.' > "$out"
echo "wrote $out"
regen_wrap status-light dot "$(dirname "$out")/status_light_wrap.json"
