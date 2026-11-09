#!/usr/bin/env bash
# Regenerates switch_scene.json from the oracle: `design-data` v0.20.x run from the
# spectrum-design-data dataset root (default ~/repos/spectrum-design-data).
# Coordinates are relative to the container; the test compares with 1px / 3-per-channel tolerance.
set -euo pipefail
root="${1:-$HOME/repos/spectrum-design-data}"
out="$(cd "$(dirname "$0")" && pwd)/switch_scene.json"
cd "$root"
{
  for scheme in light dark; do for scale in desktop mobile; do
    for size in s m l xl; do for sel in false true; do for emph in false true; do for dis in false true; do
      for state in rest hover down; do
        # Hover and down are checked at size m only; they need no per-size repeat.
        [ "$state" != rest ] && [ "$size" != m ] && continue
        [ "$state" != rest ] && [ "$dis" = true ] && continue
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
