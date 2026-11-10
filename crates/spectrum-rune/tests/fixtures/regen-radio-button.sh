#!/usr/bin/env bash
# Regenerates radio_button_scene.json from the oracle: `design-data` (the versions in oracle.toml) run from the
# spectrum-design-data dataset root (default ~/repos/spectrum-design-data). The committed fixtures were
# made at the dataset commit in oracle.toml (the script checks the CLI version and prints the commit
# it ran against); check that commit out (`git worktree add --detach <dir> <commit>`) and pass <dir> as $1.
# Coordinates are relative to the container; the test compares with 1px / 3-per-channel tolerance.
# Only the fields the test reads are kept.
set -euo pipefail
root="${1:-$HOME/repos/spectrum-design-data}"
source "$(dirname "$0")/constraints.sh"
source "$(dirname "$0")/oracle.sh"
out="$(cd "$(dirname "$0")" && pwd)/radio_button_scene.json"
cd "$root"
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
