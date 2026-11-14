#!/usr/bin/env bash
# Regenerates badge_scene.json from the oracle: `design-data` (the versions in oracle.toml) against
# the foundation release pinned in .design-data.toml (fetched by the CLI). Optional $1: a spectrum-design-data
# checkout to read instead (at the oracle.toml dataset commit).
# Also writes badge_wrap.json: the oracle at a request width (wrap.sh), where the label wraps.
# Coordinates are relative to the container; the test compares with 1px / 3-per-channel tolerance.
# Only the fields the tests read are kept, rounded to 2 decimals; nulls and zero strokes are
# dropped and colours written as `r,g,b[,a]`, to keep the file small.
#
# Matrix (the full cross product is far too big; each block answers one question):
#   A. colour table: all 25 variants x 3 styles x the four scheme+scale corners at size M, with a
#      label and an icon (300 cases)
#   B. sizes S..XL x neutral bold / accent outline / informative subtle, light desktop and dark mobile,
#      label only
#   C. sizes x four corners, neutral bold, label only and with an icon (padding, height, type)
#   D. icon-only (no label) at every size, two corners, bold and outline
#   E. label width: a one-letter and a long label at S and XL
# Combinations an error constraint of the catalog excludes are skipped (constraints.sh); there is
# none for the badge today, so the call only keeps the matrix honest when one is added.
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/constraints.sh"
source "$(dirname "$0")/oracle.sh"
source "$(dirname "$0")/wrap.sh"
out="$(cd "$(dirname "$0")" && pwd)/badge_scene.json"
oracle_cd

CORNERS=("light desktop" "light mobile" "dark desktop" "dark mobile")
VARIANTS="neutral informative positive negative indigo yellow magenta fuchsia purple seafoam accent notice gray red orange chartreuse celery green cyan blue pink turquoise brown cinnamon silver"

# render variant style size scheme scale label(or -) icon(true|false)
render() {
  local v=$1 s=$2 size=$3 scheme=$4 scale=$5 label=$6 icon=$7
  local opts="{\"variant\":\"$v\",\"style\":\"$s\",\"size\":\"$size\",\"icon\":$icon"
  [ "$label" != - ] && opts+=",\"label\":true"
  excluded badge "$opts}" && return 0
  local args=(--size "$size" --color-scheme "$scheme" --scale "$scale" --format scene
    --option "variant=\"$v\"" --option "style=\"$s\"")
  [ "$icon" = true ] && args+=(--option icon=true)
  if [ "$label" = - ]; then args+=(--option label=false); else args+=(--label "$label"); fi
  design-data render badge "${args[@]}" | jq -c \
    --arg v "$v" --arg s "$s" --arg size "$size" --arg scheme "$scheme" --arg scale "$scale" \
    --arg label "$label" '
    (.nodes[] | select(.part == "container")) as $c
    | def r: if type == "number" then (. * 100 | round / 100) else . end;
      def col: if type == "string" then gsub("rgba?\\(|\\)| "; "") else . end;
      def part($p; $keys): [.nodes[] | select(.part == $p)][0]
        | if . == null then null
          else ({x: (.x - $c.x), y: (.y - $c.y)}
                + (. | {w, h, radius, radii, fill, stroke, color}
                   | with_entries(select(.value != null and .value != "transparent" and (.key | IN($keys[])))))
               ) | map_values(r) | (if .fill then .fill |= col else . end)
               | (if .stroke then .stroke |= col else . end) | (if .color then .color |= col else . end) end;
    {scheme: $scheme, scale: $scale, size: $size, variant: $v, style: $s,
     label: (if $label == "-" then null else $label end),
     container: part("container"; ["w","h","radius","radii","fill","stroke"]),
     icon: part("icon"; ["x","y","w","h","color"]),
     label_part: part("label"; ["x","w","color"])}
    | del(.container.x, .container.y)
    | with_entries(select(.value != null))'
}

{
  # A
  for corner in "${CORNERS[@]}"; do set -- $corner
    for s in bold subtle outline; do for v in $VARIANTS; do
      render $v $s m $1 $2 Label true
    done; done
  done

  # B
  for corner in "light desktop" "dark mobile"; do set -- $corner
    for size in s m l xl; do
      render neutral bold $size $1 $2 Label false
      render accent outline $size $1 $2 Label false
      render informative subtle $size $1 $2 Label false
    done
  done

  # C
  for corner in "${CORNERS[@]}"; do set -- $corner
    for size in s m l xl; do
      render neutral bold $size $1 $2 Label false
      render neutral bold $size $1 $2 Label true
    done
  done

  # D
  for corner in "light desktop" "dark mobile"; do set -- $corner
    for size in s m l xl; do
      render neutral bold $size $1 $2 - true
      render accent outline $size $1 $2 - true
    done
  done

  # E
  for size in s xl; do
    render neutral bold $size light desktop A true
    render neutral bold $size light desktop "A much longer badge label here" true
  done
} | jq -sc '.' > "$out"
echo "wrote $out"
regen_wrap badge - "$(dirname "$out")/badge_wrap.json"
