#!/usr/bin/env bash
# Regenerates radio_group_scene.json from the oracle: the foundation's `radio-group` recipe (data 4.2.0 has one; its
# `options` slot is named `default`), rendered as the `rune` platform by `design-data` (versions in oracle.toml)
# against the pinned foundation release (see regen-radio-button.sh). Coordinates are relative to the container.
# Three unlabelled radio buttons fill the slot (parts options-1..3), so only the gap between items and the item
# height are compared.
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
} | jq -s . > "$out"
echo "wrote $out" >&2
