#!/usr/bin/env bash
# Regenerates switch_scene.json from the oracle: `design-data` (the versions in oracle.toml) run from the
# spectrum-design-data dataset root (default ~/repos/spectrum-design-data). The committed fixtures were
# made at the dataset commit in oracle.toml (the script checks the CLI version and prints the commit
# it ran against); check that commit out (`git worktree add --detach <dir> <commit>`) and pass <dir> as $1.
# Also writes switch_wrap.json: the oracle at a request width (wrap.sh), where the label wraps.
# Coordinates are relative to the container; the test compares with 1px / 3-per-channel tolerance.
set -euo pipefail
root="${1:-$HOME/repos/spectrum-design-data}"
source "$(dirname "$0")/constraints.sh"
source "$(dirname "$0")/oracle.sh"
source "$(dirname "$0")/wrap.sh"
out="$(cd "$(dirname "$0")" && pwd)/switch_scene.json"
cd "$root"
{
  for scheme in light dark; do for scale in desktop mobile; do
    for size in s m l xl; do for sel in false true; do for emph in false true; do for dis in false true; do
      for state in rest hover down; do
        # Hover and down are checked at size m only; they need no per-size repeat.
        [ "$state" != rest ] && [ "$size" != m ] && continue
        [ "$state" != rest ] && [ "$dis" = true ] && continue
        excluded switch "{\"isSelected\":$sel,\"isEmphasized\":$emph,\"isDisabled\":$dis}" && continue
        args=(--size "$size" --option "isSelected=$sel" --option "isEmphasized=$emph" --option "isDisabled=$dis"
              --color-scheme "$scheme" --scale "$scale" --format scene)
        [ "$state" != rest ] && args+=(--state "$state")
        design-data render switch "${args[@]}" | jq -c \
          --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" --arg state "$state" \
          --argjson sel "$sel" --argjson emph "$emph" --argjson dis "$dis" '
          (.nodes[] | select(.part == "container")) as $c
          | def part($p): .nodes[] | select(.part == $p)
              | {x: (.x - $c.x), y: (.y - $c.y), w, h, radius, fill, stroke, stroke_width};
          {scheme: $scheme, scale: $scale, size: $size, selected: $sel, emphasized: $emph,
           disabled: $dis, state: $state,
           track: (part("track")), handle: (part("handle")),
           label_color: ((.nodes[] | select(.part == "label") | .color) // null)}'
      done
    done; done; done; done
  done; done
} | jq -s . > "$out"
echo "wrote $out"

# Focus ring (--state keyboard-focus) around the track: switch_rings.json (ring.jq).
ring="$(dirname "$out")/switch_rings.json"
for scheme in light dark; do for scale in desktop mobile; do for size in s m l xl; do for sel in false true; do
  design-data render switch --size "$size" --option "isSelected=$sel" --color-scheme "$scheme" --scale "$scale" \
    --state keyboard-focus --format scene | jq -c --arg target track \
    --argjson meta "{\"scheme\":\"$scheme\",\"scale\":\"$scale\",\"size\":\"$size\",\"selected\":$sel}" \
    -f "$(dirname "$out")/ring.jq"
done; done; done; done | jq -s . > "$ring"
echo "wrote $ring"
regen_wrap switch track "$(dirname "$out")/switch_wrap.json"

# Keyboard-focus look (--state keyboard-focus, also combined with hover): switch_focus.json, read by
# switch_pixels.rs. Per case the track and handle (fill, stroke, stroke width, radius, size) and the
# label colour, colours written as `r,g,b[,a]`: scheme x scale x selected x emphasized (M), hover
# + keyboard-focus on desktop, and disabled + keyboard-focus (the oracle still draws its ring and
# the disabled colours; our disabled switch is not focusable). Sizes S/L/XL on the pill radius.
focus="$(dirname "$out")/switch_focus.json"
focus_case() {
  local scheme=$1 scale=$2 size=$3 sel=$4 emph=$5 dis=$6 state=$7
  local args=(--size "$size" --option "isSelected=$sel" --option "isEmphasized=$emph" --option "isDisabled=$dis"
              --color-scheme "$scheme" --scale "$scale" --format scene)
  for st in ${state//+/ }; do args+=(--state "$st"); done
  design-data render switch "${args[@]}" | jq -c \
    --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" --arg state "$state" \
    --argjson sel "$sel" --argjson emph "$emph" --argjson dis "$dis" '
    def c: if type == "string" then gsub("rgba?\\(|\\)| "; "") else . end;
    def part($p): .nodes[] | select(.part == $p)
      | {w, h, radius, fill: (.fill | c), stroke: (.stroke | c), stroke_width};
    {scheme: $scheme, scale: $scale, size: $size, selected: $sel, emphasized: $emph, disabled: $dis,
     state: $state, track: part("track"), handle: part("handle"),
     label_color: (.nodes[] | select(.part == "label") | .color | c)}'
}
{
  for scheme in light dark; do for scale in desktop mobile; do for sel in false true; do for emph in false true; do
    focus_case $scheme $scale m $sel $emph false keyboard-focus
    [ $scale = desktop ] && focus_case $scheme $scale m $sel $emph false hover+keyboard-focus
  done; done; done
  for sel in false true; do focus_case $scheme desktop m $sel false true keyboard-focus; done; done
  for size in s l xl; do for sel in false true; do focus_case light desktop $size $sel true false keyboard-focus; done; done
} | jq -s . > "$focus"
echo "wrote $focus"
