#!/usr/bin/env bash
# Regenerates tooltip_scene.json from the oracle: the foundation's `tooltip` recipe (a container with a label and a tip on
# the side that faces the trigger), rendered as the `rune` platform by `design-data` (versions in oracle.toml) against the
# foundation release pinned in .design-data.toml. Optional $1: a spectrum-design-data checkout to read instead.
# Coordinates are relative to the container; the test compares with 1px / 3-per-channel tolerance.
#
# Matrix: scheme x scale x variant (neutral, informative, negative) x placement (top, bottom, left, right), label "Label".
# Not in the matrix: `maxWidth` (not visual in the recipe: the oracle does not wrap), `shouldFlip`, `offset`.
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/oracle.sh"
out="$(cd "$(dirname "$0")" && pwd)/tooltip_scene.json"
oracle_cd

render() {
  local scheme=$1 scale=$2 variant=$3 placement=$4
  design-data render tooltip --color-scheme "$scheme" --scale "$scale" --format scene \
    --option "variant=\"$variant\"" --option "placement=\"$placement\"" | jq -c \
    --arg scheme "$scheme" --arg scale "$scale" --arg variant "$variant" --arg placement "$placement" '
    def col: gsub("rgba?\\(|\\)| "; "");
    (.nodes[] | select(.part == "container")) as $c
    | (.nodes[] | select(.part == "label")) as $l
    | ([.nodes[] | select(.part | startswith("tip-"))][0]) as $t
    | {scheme: $scheme, scale: $scale, variant: $variant, placement: $placement,
       container: {w: $c.w, h: $c.h, radius: $c.radius, fill: ($c.fill | col)},
       label: {x: ($l.x - $c.x), y: ($l.y - $c.y), h: $l.h, size: $l.font_size, color: ($l.color | col)},
       tip: {part: $t.part, x: ($t.x - $c.x), y: ($t.y - $c.y), w: $t.w, h: $t.h, fill: ($t.fill | col)}}'
}

{
  for scheme in light dark; do for scale in desktop mobile; do
    for variant in neutral informative negative; do
      for placement in top bottom left right; do render $scheme $scale $variant $placement; done
    done
  done; done
} | jq -s . > "$out"
echo "wrote $out"
