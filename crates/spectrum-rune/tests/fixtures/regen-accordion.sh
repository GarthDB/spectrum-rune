#!/usr/bin/env bash
# Regenerates accordion_scene.json from the oracle: `design-data` (the versions in oracle.toml) run from the
# spectrum-design-data dataset root (default ~/repos/spectrum-design-data).
# Coordinates are relative to the `item`; the tests compare with 1px / 3-per-channel tolerance.
# Only the fields the tests read are kept, rounded to 2 decimals, colours written as `r,g,b[,a]`.
# `content_pad` is the content's own padding (top, bottom) from the scene's measurements: the
# content's height is the CLI's placeholder, so only its padding and position are compared.
#
# Matrix (each block answers one question; the full cross product is far too big):
#   A. geometry and colour at rest: sizes x densities x {closed, open} x the four scheme+scale
#      corners (96)
#   B. quiet (no divider): sizes x {closed, open} x two corners (16)
#   C. hover: sizes x {closed, open} x four corners at regular density, plus M at compact and
#      spacious, and quiet M (56)
#   D. disabled: sizes x {closed, open} x four corners at rest, plus M hovered (the oracle still
#      fills a disabled item on hover; the component does not, and the tests skip that fill) (40)
#   E. catalog defaults: size M and density omitted (the CLI default) at two sizes
# `down` is `notVisual`: the oracle draws it like rest, so it has no case here.
# Combinations an error constraint of the catalog excludes are skipped (constraints.sh); there is
# none for the accordion today, so the call only keeps the matrix honest when one is added.
set -euo pipefail
root="${1:-$HOME/repos/spectrum-design-data}"
source "$(dirname "$0")/constraints.sh"
source "$(dirname "$0")/oracle.sh"
out="$(cd "$(dirname "$0")" && pwd)/accordion_scene.json"
oracle_cd

CORNERS=("light desktop" "light mobile" "dark desktop" "dark mobile")
SIZES="s m l xl"

# render size density(or -) open quiet disabled state scheme scale
render() {
  local size=$1 density=$2 open=$3 quiet=$4 disabled=$5 state=$6 scheme=$7 scale=$8
  local opts="{\"size\":\"$size\",\"isOpen\":$open,\"isQuiet\":$quiet,\"isDisabled\":$disabled"
  local args=(--size "$size" --state "$state" --color-scheme "$scheme" --scale "$scale" --format scene
    --option isOpen=$open --option isQuiet=$quiet --option isDisabled=$disabled)
  if [ "$density" != - ]; then
    opts+=",\"density\":\"$density\""
    args+=(--option "density=\"$density\"")
  fi
  excluded accordion "$opts}" && return 0
  design-data render accordion "${args[@]}" | jq -c \
    --arg size "$size" --arg density "$density" --arg state "$state" --arg scheme "$scheme" --arg scale "$scale" \
    --argjson open "$open" --argjson quiet "$quiet" --argjson disabled "$disabled" '
    (.nodes[] | select(.part == "item")) as $i
    | def r: if type == "number" then (. * 100 | round / 100) else . end;
      def col: if type == "string" then gsub("rgba?\\(|\\)| "; "") else . end;
      def part($p; $keys): [.nodes[] | select(.part == $p)][0]
        | if . == null then null
          else ({x: (.x - $i.x), y: (.y - $i.y)}
                + (. | {w, h, fill, color, rotation}
                   | with_entries(select(.value != null and .value != "transparent" and (.key | IN($keys[])))))
               ) | map_values(r) | (if .fill then .fill |= col else . end)
               | (if .color then .color |= col else . end) end;
      def pad($k): [.measurements[] | select(.part == "content" and .prop == $k)][0].h;
    {scheme: $scheme, scale: $scale, size: $size, density: (if $density == "-" then null else $density end),
     open: $open, quiet: $quiet, disabled: $disabled, state: $state,
     item: part("item"; ["h", "fill"]),
     header: part("header"; ["h"]),
     icon: part("disclosure-icon"; ["x","y","w","h","color","rotation"]),
     title: part("title"; ["x","y","h","color"]),
     content: part("content"; ["y","h"]),
     content_pad: (if $open then [pad("paddingBlockStart"), pad("paddingBlockEnd")] else null end),
     divider: part("divider"; ["y","h","fill"])}
    | del(.item.x, .item.y, .header.x, .header.y, .content.x, .divider.x)
    | with_entries(select(.value != null))'
}

{
  # A
  for corner in "${CORNERS[@]}"; do set -- $corner
    for size in $SIZES; do for d in compact regular spacious; do for o in false true; do
      render $size $d $o false false rest $1 $2
    done; done; done
  done

  # B
  for corner in "light desktop" "dark mobile"; do set -- $corner
    for size in $SIZES; do for o in false true; do
      render $size regular $o true false rest $1 $2
    done; done
  done

  # C
  for corner in "${CORNERS[@]}"; do set -- $corner
    for size in $SIZES; do for o in false true; do
      render $size regular $o false false hover $1 $2
    done; done
    for d in compact spacious; do for o in false true; do
      render m $d $o false false hover $1 $2
    done; done
    render m regular true true false hover $1 $2
    render m regular false true false hover $1 $2
  done

  # D
  for corner in "${CORNERS[@]}"; do set -- $corner
    for size in $SIZES; do for o in false true; do
      render $size regular $o false true rest $1 $2
    done; done
    for o in false true; do render m regular $o false true hover $1 $2; done
  done

  # E
  for size in m xl; do
    render $size - false false false rest light desktop
    render $size - true false false rest light desktop
  done
} | jq -sc '.' > "$out"
echo "wrote $out"

# Keyboard-focus look (--state keyboard-focus, also combined with hover): accordion_focus.json, read by
# accordion_pixels.rs. Per case the item fill and the title and chevron colours as `r,g,b[,a]` (null
# when none): scheme x open x quiet at M and desktop for keyboard-focus, hover and hover +
# keyboard-focus (the oracle: the hover wash and ink), and disabled
# + keyboard-focus, each with its rest case (the oracle draws the disabled ink; ours is not focusable).
focus="$(dirname "$out")/accordion_focus.json"
focus_case() {
  local scheme=$1 open=$2 quiet=$3 dis=$4 state=$5
  local args=(--size m --color-scheme "$scheme" --scale desktop --format scene
    --option isOpen=$open --option isQuiet=$quiet --option isDisabled=$dis)
  for st in ${state//+/ }; do args+=(--state "$st"); done
  design-data render accordion "${args[@]}" | jq -c \
    --arg scheme "$scheme" --arg state "$state" --argjson open "$open" --argjson quiet "$quiet" --argjson dis "$dis" '
    def col: if type == "string" then gsub("rgba?\\(|\\)| "; "") else . end;
    def node($p): [.nodes[] | select(.part == $p)][0];
    {scheme: $scheme, open: $open, quiet: $quiet, disabled: $dis, state: $state,
     item_fill: (node("item").fill | col), title: (node("title").color | col),
     icon: (node("disclosure-icon").color | col)}'
}
{
  for scheme in light dark; do for open in false true; do for quiet in false true; do
    for state in rest hover keyboard-focus hover+keyboard-focus; do
      focus_case $scheme $open $quiet false $state
    done
  done; done; done
  for scheme in light dark; do
    focus_case $scheme false false true rest
    focus_case $scheme false false true keyboard-focus
  done
} | jq -s . > "$focus"
echo "wrote $focus"
