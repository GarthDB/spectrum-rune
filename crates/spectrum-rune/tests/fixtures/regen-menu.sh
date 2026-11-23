#!/usr/bin/env bash
# Regenerates menu_scene.json from the oracle: the foundation's `menu` recipe (menu-items repeated over `items`),
# rendered as the `rune` platform by `design-data` (versions in oracle.toml) against the foundation release pinned in
# .design-data.toml. Optional $1: a spectrum-design-data checkout to read instead. Coordinates are relative to the
# container; the test compares with 1px / 3-per-channel tolerance.
#
# Matrix: scheme x scale x size x variant at rest, then hover, down and keyboard-focus at size M for each variant,
# scheme and scale. Variants: `plain` (no selection mode) and `single` (selection mode single, item 2 selected); both
# have a value on item 2 and a disabled item 3. In the oracle a state applies to every item; the test compares each
# enabled item with it. Not in the matrix: multiple selection (checkbox or switch), icons, thumbnails, section header.
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/oracle.sh"
out="$(cd "$(dirname "$0")" && pwd)/menu_scene.json"
oracle_cd

ITEMS='[{"label":"One"},{"label":"Two","value":"V","isSelected":true},{"label":"Three","isDisabled":true}]'
PLAIN_ITEMS='[{"label":"One"},{"label":"Two","value":"V"},{"label":"Three","isDisabled":true}]'

render() {
  local scheme=$1 scale=$2 size=$3 variant=$4 state=$5
  local args=(--size "$size" --color-scheme "$scheme" --scale "$scale" --format scene)
  if [ "$variant" = single ]; then
    args+=(--option "items=$ITEMS" --option 'selectionMode="single"')
  else
    args+=(--option "items=$PLAIN_ITEMS")
  fi
  [ "$state" != rest ] && args+=(--state "$state")
  design-data render menu "${args[@]}" | jq -c \
    --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" --arg variant "$variant" --arg state "$state" '
    def col: gsub("rgba?\\(|\\)| "; "");
    (.nodes[] | select(.part == "container")) as $c
    | def rel($n): {x: ($n.x - $c.x), y: ($n.y - $c.y), w: $n.w, h: $n.h};
      def part($p): [.nodes[] | select(.part == $p)][0];
      {scheme: $scheme, scale: $scale, size: $size, variant: $variant, state: $state,
       container: {w: $c.w, h: $c.h},
       items: [range(1; 4) as $i
         | part("menu-item-\($i)") as $it
         | part("menu-item-\($i).label") as $l
         | part("menu-item-\($i).value") as $v
         | part("menu-item-\($i).checkmark") as $k
         | {item: (rel($it) + {radius: $it.radius, fill: ($it.fill // null | if . then col else . end)}),
            label: {x: ($l.x - $it.x), y: ($l.y - $it.y), h: $l.h, color: ($l.color | col)}}
           + (if $v then {value: {right: (($it.x + $it.w) - ($v.x + $v.w)), y: ($v.y - $it.y), color: ($v.color | col)}} else {} end)
           + (if $k then {checkmark: {x: ($k.x - $it.x), y: ($k.y - $it.y), w: $k.w, h: $k.h, color: ($k.color | col)}} else {} end)
         | with_entries(select(.value != null))]}'
}

{
  for scheme in light dark; do for scale in desktop mobile; do
    for size in s m l xl; do for variant in plain single; do render $scheme $scale $size $variant rest; done; done
    for variant in plain single; do for state in hover down keyboard-focus; do render $scheme $scale m $variant $state; done; done
  done; done
} | jq -s . > "$out"
echo "wrote $out"
