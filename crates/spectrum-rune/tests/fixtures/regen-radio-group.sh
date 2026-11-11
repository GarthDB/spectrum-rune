#!/usr/bin/env bash
# Regenerates radio_group_scene.json from the oracle: the `radio-group` recipe in platforms/rune/extensions/visuals
# (the foundation has none), rendered as the `rune` platform by `design-data` (versions in oracle.toml) against the
# oracle dataset checkout at $1 (see regen-radio-button.sh). Coordinates are relative to the container.
# The recipe's nested radio buttons carry no label, so only the gap between items and the item height are compared.
set -euo pipefail
root="${1:-$HOME/repos/spectrum-design-data}"
source "$(dirname "$0")/oracle.sh"
out="$(cd "$(dirname "$0")" && pwd)/radio_group_scene.json"
oracle_cd
{
  for scale in desktop mobile; do for size in s m l xl; do for orient in vertical horizontal; do
    design-data render radio-group --size "$size" --option "orientation=$orient" --scale "$scale" --format scene | jq -c \
      --arg scale "$scale" --arg size "$size" --arg orient "$orient" '
      def part($p): .nodes[] | select(.part == $p) | {x, y, w, h};
      {scale: $scale, size: $size, orientation: $orient,
       container: part("container"), items: [part("item-1"), part("item-2"), part("item-3")]}'
  done; done; done
} | jq -s . > "$out"
echo "wrote $out" >&2
