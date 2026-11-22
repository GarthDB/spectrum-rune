#!/usr/bin/env bash
# Regenerates picker_scene.json from the oracle: the foundation's `picker` recipe (the closed field only), rendered as the
# `rune` platform by `design-data` (versions in oracle.toml) against the foundation release pinned in .design-data.toml.
# Optional $1: a spectrum-design-data checkout to read instead. Coordinates are relative to the container; the test
# compares with 1px / 3-per-channel tolerance.
#
# Matrix: scheme x scale x size x content (value, placeholder) x quiet x error x disabled at rest; hover, down and
# keyboard-focus at size M, desktop and mobile, enabled (a disabled field ignores the pointer and is not focusable).
# The recipe has no `labelPosition: side`, no quiet fill, no disabled fill and no placeholder hover look, so none of
# those are in the matrix; `description`, `isRequired` and `necessityIndicator` are not built (the recipe composes
# help-text and the field-label asterisk). The ring of keyboard-focus goes to picker_rings.json (ring.jq).
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/constraints.sh"
source "$(dirname "$0")/oracle.sh"
out="$(cd "$(dirname "$0")" && pwd)/picker_scene.json"
ring="$(dirname "$out")/picker_rings.json"
oracle_cd

render() {
  local scheme=$1 scale=$2 size=$3 content=$4 quiet=$5 err=$6 dis=$7 state=$8
  local args=(--size "$size" --color-scheme "$scheme" --scale "$scale" --format scene
              --option label=Country --option "isQuiet=$quiet" --option "isError=$err" --option "isDisabled=$dis")
  if [ "$content" = value ]; then args+=(--option value=Canada); else args+=(--option placeholder=Select); fi
  [ "$state" != rest ] && args+=(--state "$state")
  design-data render picker "${args[@]}"
}

{
  for scheme in light dark; do for scale in desktop mobile; do
    for size in s m l xl; do for content in value placeholder; do
      for quiet in false true; do for err in false true; do for dis in false true; do
        for state in rest hover down keyboard-focus; do
          [ "$state" != rest ] && { [ "$size" != m ] || [ "$dis" = true ]; } && continue
          excluded picker "{\"isQuiet\":$quiet,\"isError\":$err,\"isDisabled\":$dis}" && continue
          render $scheme $scale $size $content $quiet $err $dis $state | jq -c \
            --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" --arg content "$content" --arg state "$state" \
            --argjson quiet "$quiet" --argjson err "$err" --argjson dis "$dis" '
            def col: gsub("rgba?\\(|\\)| "; "");
            (.nodes[] | select(.part == "container")) as $c
            | def part($p): [.nodes[] | select(.part == $p)][0]
                | {x: (.x - $c.x), y: (.y - $c.y), w, h, radius, fill, stroke, stroke_width, color};
              {scheme: $scheme, scale: $scale, size: $size, content: $content, quiet: $quiet, error: $err, disabled: $dis,
               state: $state, press: (.press.scale // null),
               container: {w: $c.w, h: $c.h},
               field: part("field"), text: part($content), icon: part("disclosure-icon"),
               label: part("field-label.label")}'
        done
      done; done; done
    done; done
  done; done
} | jq -s . > "$out"
echo "wrote $out"

# Focus ring (--state keyboard-focus) around the field: picker_rings.json (ring.jq).
for scheme in light dark; do for scale in desktop mobile; do for size in s m l xl; do for err in false true; do
  render $scheme $scale $size value false $err false keyboard-focus | jq -c --arg target field \
    --argjson meta "{\"scheme\":\"$scheme\",\"scale\":\"$scale\",\"size\":\"$size\",\"error\":$err}" \
    -f "$(dirname "$out")/ring.jq"
done; done; done; done | jq -s . > "$ring"
echo "wrote $ring"
