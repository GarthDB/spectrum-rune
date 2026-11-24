#!/usr/bin/env bash
# Regenerates radio_group_scene.json from the oracle: the foundation's `radio-group` recipe (data 4.2.0 has one; its
# `options` slot is named `default`), rendered as the `rune` platform by `design-data` (versions in oracle.toml)
# against the pinned foundation release (see regen-radio-button.sh). Coordinates are relative to the container.
# Three unlabelled radio buttons fill the slot (parts options-1..3), so only the gap between items and the item
# height are compared. The first 16 cases carry no label or description (the oracle draws an empty label and help text
# anyway); the `labeled` and `error` cases (a required label, a description, and for `error` the negative help text) add the
# offsets of the label, the options and the help text inside the container.
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/oracle.sh"
out="$(cd "$(dirname "$0")" && pwd)/radio_group_scene.json"
oracle_cd
{
  for scale in desktop mobile; do for size in s m l xl; do for orient in vertical horizontal; do
    design-data render radio-group --size "$size" --option "orientation=$orient" --scale "$scale" --format scene \
      --slots '{"default":[{"component":"radio-button","options":{}},{"component":"radio-button","options":{}},{"component":"radio-button","options":{}}]}' | jq -c \
      --arg scale "$scale" --arg size "$size" --arg orient "$orient" '
      def part($p): .nodes[] | select(.part == $p) | {x, y, w, h};
      {scale: $scale, size: $size, orientation: $orient,
       container: part("container"), items: [part("options-1"), part("options-2"), part("options-3")]}'
  done; done; done
  for scale in desktop mobile; do for size in s m l xl; do for orient in vertical horizontal; do for kind in labeled error; do
    opts=(--option 'label="Pick one"' --option isRequired=true --option 'description="Choose carefully"')
    [ "$kind" = error ] && opts+=(--option isError=true)
    design-data render radio-group --size "$size" --option "orientation=$orient" --scale "$scale" --format scene "${opts[@]}" \
      --slots '{"default":[{"component":"radio-button","options":{}},{"component":"radio-button","options":{}},{"component":"radio-button","options":{}}]}' | jq -c \
      --arg scale "$scale" --arg size "$size" --arg orient "$orient" --arg kind "$kind" '
      (.nodes[] | select(.part == "container")) as $c
      | def part($p): [.nodes[] | select(.part == $p)][0];
        def rel($p): part($p) | {y: (.y - $c.y), h: .h};
        {scale: $scale, size: $size, orientation: $orient, kind: $kind, container: {h: $c.h},
         label: rel("field-label"), options: rel("options"), help: rel("help-text"),
         indicator: (part("field-label.necessity-indicator") != null),
         help_icon: (part("help-text.icon") != null)}'
  done; done; done; done
} | jq -s . > "$out"
echo "wrote $out" >&2
