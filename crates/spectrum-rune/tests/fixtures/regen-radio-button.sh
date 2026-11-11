#!/usr/bin/env bash
# Regenerates radio_button_scene.json from the oracle: `design-data` (the versions in oracle.toml) against
# the foundation release pinned in .design-data.toml (fetched by the CLI). Optional $1: a spectrum-design-data
# checkout to read instead (at the oracle.toml dataset commit; `git worktree add --detach <dir> <commit>`).
# Also writes radio_button_wrap.json: the oracle at a request width (wrap.sh), where the label wraps.
# Coordinates are relative to the container; the test compares with 1px / 3-per-channel tolerance.
# Only the fields the test reads are kept.
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/constraints.sh"
source "$(dirname "$0")/oracle.sh"
source "$(dirname "$0")/wrap.sh"
out="$(cd "$(dirname "$0")" && pwd)/radio_button_scene.json"
oracle_cd
{
  for scheme in light dark; do for scale in desktop mobile; do
    for size in s m l xl; do for sel in false true; do for emph in false true; do for dis in false true; do
      for state in rest hover down; do
        # Hover and down are checked at size m only, and a disabled radio ignores the pointer.
        [ "$state" != rest ] && [ "$size" != m ] && continue
        [ "$state" != rest ] && [ "$dis" = true ] && continue
        excluded radio-button "{\"isSelected\":$sel,\"isEmphasized\":$emph,\"isDisabled\":$dis}" && continue
        args=(--size "$size" --option "isSelected=$sel" --option "isEmphasized=$emph" --option "isDisabled=$dis"
              --color-scheme "$scheme" --scale "$scale" --format scene)
        [ "$state" != rest ] && args+=(--state "$state")
        design-data render radio-button "${args[@]}" | jq -c \
          --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" --arg state "$state" \
          --argjson sel "$sel" --argjson emph "$emph" --argjson dis "$dis" '
          (.nodes[] | select(.part == "container")) as $c
          | def part($p; $keys): [.nodes[] | select(.part == $p)][0]
              | if . == null then null else ({x: (.x - $c.x), y: (.y - $c.y)} + (. | {w, h, radius, fill, color, stroke, stroke_width} | with_entries(select(.value != null and (.key | IN($keys[]))))) ) end;
          {scheme: $scheme, scale: $scale, size: $size, selected: $sel, emphasized: $emph, disabled: $dis,
           state: $state, press: (.press.scale // null),
           control: part("control"; ["w","h","radius","fill","stroke","stroke_width"]),
           indicator: part("selection-indicator"; ["w","h","radius","fill"]),
           label: part("label"; ["color"])}'
      done
    done; done; done
  done; done
done; } | jq -s . > "$out"
echo "wrote $out"

# Focus ring (--state keyboard-focus) around the control: radio_button_rings.json (ring.jq).
ring="$(dirname "$out")/radio_button_rings.json"
for scheme in light dark; do for scale in desktop mobile; do for size in s m l xl; do for sel in false true; do
  design-data render radio-button --size "$size" --option "isSelected=$sel" --color-scheme "$scheme" --scale "$scale" \
    --state keyboard-focus --format scene | jq -c --arg target control \
    --argjson meta "{\"scheme\":\"$scheme\",\"scale\":\"$scale\",\"size\":\"$size\",\"selected\":$sel}" \
    -f "$(dirname "$out")/ring.jq"
done; done; done; done | jq -s . > "$ring"
echo "wrote $ring"
regen_wrap radio-button control "$(dirname "$out")/radio_button_wrap.json"

# Keyboard-focus look (--state keyboard-focus, also combined with hover): radio_button_focus.json, read
# by radio_button_pixels.rs. Per case the control (fill, stroke, stroke width, radius, size), the
# indicator fill and the label colour, colours written as `r,g,b[,a]`: scheme x scale x selected x
# emphasized (M), hover + keyboard-focus on desktop, disabled + keyboard-focus (the oracle still draws
# its ring and the disabled colours; our disabled radio is not focusable), and sizes S/L/XL.
focus="$(dirname "$out")/radio_button_focus.json"
focus_case() {
  local scheme=$1 scale=$2 size=$3 sel=$4 emph=$5 dis=$6 state=$7
  local args=(--size "$size" --option "isSelected=$sel" --option "isEmphasized=$emph" --option "isDisabled=$dis"
              --color-scheme "$scheme" --scale "$scale" --format scene)
  for st in ${state//+/ }; do args+=(--state "$st"); done
  design-data render radio-button "${args[@]}" | jq -c \
    --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" --arg state "$state" \
    --argjson sel "$sel" --argjson emph "$emph" --argjson dis "$dis" '
    def c: if type == "string" then gsub("rgba?\\(|\\)| "; "") else . end;
    {scheme: $scheme, scale: $scale, size: $size, selected: $sel, emphasized: $emph, disabled: $dis,
     state: $state,
     control: (.nodes[] | select(.part == "control") | {w, h, radius, fill: (.fill | c), stroke: (.stroke | c), stroke_width}),
     indicator: ([.nodes[] | select(.part == "selection-indicator") | .fill | c] | first),
     label_color: (.nodes[] | select(.part == "label") | .color | c)}'
}
{
  for scheme in light dark; do for scale in desktop mobile; do for sel in false true; do for emph in false true; do
    focus_case $scheme $scale m $sel $emph false keyboard-focus
    [ $scale = desktop ] && focus_case $scheme $scale m $sel $emph false hover+keyboard-focus
  done; done; done; done
  for scheme in light dark; do for sel in false true; do focus_case $scheme desktop m $sel false true keyboard-focus; done; done
  for size in s l xl; do for sel in false true; do focus_case light desktop $size $sel true false keyboard-focus; done; done
} | jq -s . > "$focus"
echo "wrote $focus"
