#!/usr/bin/env bash
# Regenerates standard_dialog_scene.json from the oracle: the foundation's `standard-dialog` recipe (a surface with a title,
# a description and a footer of optional content and the buttons), rendered as the `rune` platform by `design-data`
# (versions in oracle.toml) against the foundation release pinned in .design-data.toml. Optional $1: a spectrum-design-data
# checkout to read instead. Coordinates are relative to the container; the test compares with 1px / 3-per-channel tolerance.
#
# Matrix: scheme x scale x size (s, m, l) x footer content (none, a checkbox); a title, a one-line description and two
# buttons. The oracle draws the buttons at the footer's start, or 24 pt after the footer content (the footer has no
# justification), the title in the regular weight (the recipe sets none) and no shadow, image or close button. Not in the
# matrix: wrapping text, three buttons. Text widths follow the installed font and are not compared.
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/oracle.sh"
out="$(cd "$(dirname "$0")" && pwd)/standard_dialog_scene.json"
oracle_cd

BUTTONS='{"component":"button","options":{"label":"Cancel","variant":"secondary","style":"outline"}},{"component":"button","options":{"label":"Save"}}'

render() {
  local scheme=$1 scale=$2 size=$3 footer=$4 slots
  if [ "$footer" = checkbox ]; then
    slots="{\"button-group\":[$BUTTONS],\"footer-content\":[{\"component\":\"checkbox\",\"options\":{\"label\":\"Don't ask again\"}}]}"
  else
    slots="{\"button-group\":[$BUTTONS]}"
  fi
  design-data render standard-dialog --color-scheme "$scheme" --scale "$scale" --size "$size" --format scene \
    --option 'title="Rename file"' --option 'description="Pick a new name for the file."' --slots "$slots" | jq -c \
    --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" --arg footer "$footer" '
    def col: if . == null then null else gsub("rgba?\\(|\\)| "; "") end;
    (.nodes[] | select(.part == "container")) as $c
    | def part($p): [.nodes[] | select(.part == $p)][0];
      part("header") as $h | part("title") as $t | part("description") as $d
    | part("footer") as $f | part("footer-content") as $fc | part("button-group") as $g
    | {scheme: $scheme, scale: $scale, size: $size, footer_content: $footer,
       container: {w: $c.w, h: $c.h, radius: $c.radius, fill: ($c.fill | col)},
       header: {x: ($h.x - $c.x), y: ($h.y - $c.y), h: $h.h},
       title: {x: ($t.x - $c.x), y: ($t.y - $c.y), h: $t.h, size: $t.font_size, color: ($t.color | col)},
       description: {x: ($d.x - $c.x), y: ($d.y - $c.y), h: $d.h, size: $d.font_size, color: ($d.color | col)},
       footer: {y: ($f.y - $c.y), h: $f.h},
       group: {x: ($g.x - $c.x), after_content: (if $footer == "checkbox" then ($g.x - ($fc.x + $fc.w)) else null end)}}'
}

{
  for scheme in light dark; do for scale in desktop mobile; do for size in s m l; do for footer in none checkbox; do
    render $scheme $scale $size $footer
  done; done; done; done
} | jq -s . > "$out"
echo "wrote $out"
