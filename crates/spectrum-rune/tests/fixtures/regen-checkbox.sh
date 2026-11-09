#!/usr/bin/env bash
# Regenerates checkbox_scene.json from the oracle: `design-data` v0.20.x run from the
# spectrum-design-data dataset root (default ~/repos/spectrum-design-data).
# Coordinates are relative to the container; the test compares with 1px / 3-per-channel tolerance.
set -euo pipefail
root="${1:-$HOME/repos/spectrum-design-data}"
source "$(dirname "$0")/constraints.sh"
out="$(cd "$(dirname "$0")" && pwd)/checkbox_scene.json"
cd "$root"
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
