#!/usr/bin/env bash
# Regenerates checkbox_scene.json from the oracle: `design-data` (the versions in oracle.toml) against
# the foundation release pinned in .design-data.toml (fetched by the CLI). Optional $1: a spectrum-design-data
# checkout to read instead (at the oracle.toml dataset commit; `git worktree add --detach <dir> <commit>`).
# Also writes checkbox_wrap.json: the oracle at a request width (wrap.sh), where the label wraps.
# Coordinates are relative to the container; the test compares with 1px / 3-per-channel tolerance.
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/constraints.sh"
source "$(dirname "$0")/oracle.sh"
source "$(dirname "$0")/wrap.sh"
out="$(cd "$(dirname "$0")" && pwd)/checkbox_scene.json"
oracle_cd
{
  for scheme in light dark; do for scale in desktop mobile; do
    for size in s m l xl; do for value in unselected selected indeterminate; do
      for emph in false true; do for err in false true; do for dis in false true; do
        for state in rest hover down; do
          # Hover and down are checked at size m only, and a disabled box ignores the pointer.
          [ "$state" != rest ] && [ "$size" != m ] && continue
          [ "$state" != rest ] && [ "$dis" = true ] && continue
          opts="{\"isSelected\":$([ "$value" = selected ] && echo true || echo false),\"isIndeterminate\":$([ "$value" = indeterminate ] && echo true || echo false)}"
          excluded checkbox "$opts" && continue
          args=(--size "$size" --option "isEmphasized=$emph" --option "isError=$err" --option "isDisabled=$dis"
                --color-scheme "$scheme" --scale "$scale" --format scene)
          [ "$value" = selected ] && args+=(--option isSelected=true)
          [ "$value" = indeterminate ] && args+=(--option isIndeterminate=true)
          [ "$state" != rest ] && args+=(--state "$state")
          design-data render checkbox "${args[@]}" | jq -c \
            --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" --arg state "$state" --arg value "$value" \
            --argjson emph "$emph" --argjson err "$err" --argjson dis "$dis" '
            (.nodes[] | select(.part == "container")) as $c
            | ([.nodes[] | select(.part == "checkmark" or .part == "dash") | .part] | first) as $m
            | def part($p): .nodes[] | select(.part == $p)
                | {part, x: (.x - $c.x), y: (.y - $c.y), w, h, radius, fill, stroke, stroke_width, color};
            {scheme: $scheme, scale: $scale, size: $size, value: $value, emphasized: $emph, error: $err,
             disabled: $dis, state: $state, press: (.press.scale // null),
             box: (part("box")), label: (part("label")),
             mark: (if $m then part($m) else null end)}'
        done
      done; done; done
    done; done
  done; done
} | jq -s . > "$out"
echo "wrote $out"

# Focus ring (--state keyboard-focus) around the box: checkbox_rings.json (ring.jq).
ring="$(dirname "$out")/checkbox_rings.json"
for scheme in light dark; do for scale in desktop mobile; do for size in s m l xl; do for value in unselected selected indeterminate; do
  for err in false true; do
    args=(--size "$size" --option "isError=$err" --color-scheme "$scheme" --scale "$scale" --state keyboard-focus --format scene)
    [ "$value" = selected ] && args+=(--option isSelected=true)
    [ "$value" = indeterminate ] && args+=(--option isIndeterminate=true)
    design-data render checkbox "${args[@]}" | jq -c --arg target box \
      --argjson meta "{\"scheme\":\"$scheme\",\"scale\":\"$scale\",\"size\":\"$size\",\"value\":\"$value\",\"error\":$err}" \
      -f "$(dirname "$out")/ring.jq"
  done
done; done; done; done | jq -s . > "$ring"
echo "wrote $ring"
regen_wrap checkbox box "$(dirname "$out")/checkbox_wrap.json"

# Keyboard-focus look (--state keyboard-focus, also combined with hover): checkbox_focus.json, read by
# checkbox_pixels.rs. Per case the box (fill, stroke, stroke width, radius, size), the mark's colour
# and the label colour, colours written as `r,g,b[,a]`: scheme x scale x value x emphasized x error
# (M), hover + keyboard-focus on desktop, disabled + keyboard-focus (the oracle still draws its ring
# and the disabled colours; our disabled checkbox is not focusable), and sizes S/L/XL (the radius).
focus="$(dirname "$out")/checkbox_focus.json"
focus_case() {
  local scheme=$1 scale=$2 size=$3 value=$4 emph=$5 err=$6 dis=$7 state=$8
  local args=(--size "$size" --option "isEmphasized=$emph" --option "isError=$err" --option "isDisabled=$dis"
              --color-scheme "$scheme" --scale "$scale" --format scene)
  [ "$value" = selected ] && args+=(--option isSelected=true)
  [ "$value" = indeterminate ] && args+=(--option isIndeterminate=true)
  for st in ${state//+/ }; do args+=(--state "$st"); done
  design-data render checkbox "${args[@]}" | jq -c \
    --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" --arg state "$state" --arg value "$value" \
    --argjson emph "$emph" --argjson err "$err" --argjson dis "$dis" '
    def c: if type == "string" then gsub("rgba?\\(|\\)| "; "") else . end;
    ([.nodes[] | select(.part == "checkmark" or .part == "dash")] | first) as $m
    | (.nodes[] | select(.part == "box")) as $b
    | {scheme: $scheme, scale: $scale, size: $size, value: $value, emphasized: $emph, error: $err,
       disabled: $dis, state: $state,
       box: ($b | {w, h, radius, fill: (.fill | c), stroke: (.stroke | c), stroke_width}),
       mark: (if $m then {part: $m.part, color: ($m.color | c)} else null end),
       label_color: (.nodes[] | select(.part == "label") | .color | c)}'
}
{
  for scheme in light dark; do for scale in desktop mobile; do for value in unselected selected indeterminate; do
    for emph in false true; do for err in false true; do
      focus_case $scheme $scale m $value $emph $err false keyboard-focus
      [ $scale = desktop ] && focus_case $scheme $scale m $value $emph $err false hover+keyboard-focus
    done; done
  done; done; done
  for value in unselected selected; do focus_case light desktop m $value false false true keyboard-focus; done
  for size in s l xl; do for value in unselected selected indeterminate; do
    focus_case light desktop $size $value true false false keyboard-focus
  done; done
} | jq -s . > "$focus"
echo "wrote $focus"
