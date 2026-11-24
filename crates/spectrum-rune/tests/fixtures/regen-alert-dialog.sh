#!/usr/bin/env bash
# Regenerates alert_dialog_scene.json from the oracle: the foundation's `alert-dialog` recipe (a surface with a header of a
# status icon and a title, a description and a footer holding the buttons), rendered as the `rune` platform by `design-data`
# (versions in oracle.toml) against the foundation release pinned in .design-data.toml. Optional $1: a spectrum-design-data
# checkout to read instead. Coordinates are relative to the container; the test compares with 1px / 3-per-channel tolerance.
#
# Matrix: scheme x scale x variant (confirmation, warning, error); a title, a one-line description and two buttons. The
# oracle draws the icon for `warning` and `error` only, and no shadow. Not in the matrix: wrapping text, three buttons, the
# stacked button group. Text widths follow the installed font and are not compared.
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/oracle.sh"
out="$(cd "$(dirname "$0")" && pwd)/alert_dialog_scene.json"
oracle_cd

SLOTS='{"button-group":[{"component":"button","options":{"label":"Cancel","variant":"secondary","style":"outline"}},{"component":"button","options":{"label":"Delete","variant":"negative"}}]}'

render() {
  local scheme=$1 scale=$2 variant=$3
  design-data render alert-dialog --color-scheme "$scheme" --scale "$scale" --format scene \
    --option 'title="Delete file"' --option 'description="This will permanently delete the file and its history."' \
    --option "variant=\"$variant\"" --slots "$SLOTS" | jq -c \
    --arg scheme "$scheme" --arg scale "$scale" --arg variant "$variant" '
    def col: if . == null then null else gsub("rgba?\\(|\\)| "; "") end;
    (.nodes[] | select(.part == "container")) as $c
    | def part($p): [.nodes[] | select(.part == $p)][0];
      part("header") as $h | part("icon") as $i | part("title") as $t | part("description") as $d
    | part("footer") as $f | part("button-group") as $g
    | {scheme: $scheme, scale: $scale, variant: $variant,
       container: {w: $c.w, h: $c.h, radius: $c.radius, fill: ($c.fill | col)},
       header: {x: ($h.x - $c.x), y: ($h.y - $c.y), h: $h.h},
       title: {x: ($t.x - $c.x), y: ($t.y - $c.y), h: $t.h, size: $t.font_size, color: ($t.color | col)},
       description: {x: ($d.x - $c.x), y: ($d.y - $c.y), h: $d.h, size: $d.font_size, color: ($d.color | col)},
       footer: {y: ($f.y - $c.y), h: $f.h},
       group_right_inset: (($c.x + $c.w) - ($g.x + $g.w))}
      + (if $i then {icon: {x: ($i.x - $c.x), y: ($i.y - $c.y), w: $i.w, color: ($i.color | col)}} else {} end)'
}

{
  for scheme in light dark; do for scale in desktop mobile; do for variant in confirmation warning error; do
    render $scheme $scale $variant
  done; done; done
} | jq -s . > "$out"
echo "wrote $out"
