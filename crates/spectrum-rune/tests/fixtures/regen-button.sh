#!/usr/bin/env bash
# Regenerates button_scene.json from the oracle: `design-data` (the versions in oracle.toml) run from the
# spectrum-design-data dataset root (default ~/repos/spectrum-design-data). The committed fixtures were
# made at the dataset commit in oracle.toml (the script checks the CLI version and prints the commit
# it ran against); check that commit out (`git worktree add --detach <dir> <commit>`) and pass <dir> as $1.
# Coordinates are relative to the container; the test compares with 1px / 3-per-channel tolerance.
# Only the fields the tests read are kept, rounded to 2 decimals; nulls and false flags are
# dropped and colours written as `r,g,b[,a]`, to keep the file small.
#
# Matrix (the full cross product is far too big; each part has one block):
#   A. variant x style x scheme x scale at size M, rest / hover / down      (colour table)
#   B. size S/M/L/XL x scheme x scale for accent fill and primary outline, rest; down in two corners
#   C. each flag alone on accent / primary / secondary x fill / outline, in two scheme+scale corners:
#      disabled, pending, label hidden, pending + hidden, static white, static black,
#      disabled + static white / black
#   D. hover and down for pending, and for static white / black (primary, secondary x fill, outline)
#   E. press scale at the perspective clamps: a one-letter and a long label, M and XL
#   F. focus ring (--state keyboard-focus):
#      colour, thickness, gap and rect: variant x style x staticColor x scheme at M desktop, and
#      sizes S/L/XL x scale for three looks, written to button_focus_ring.json (ring.jq)
# Combinations an error constraint of the catalog excludes (accent/negative outline) are skipped
# (constraints.sh); advisory ones stay.
# The oracle always draws the optional icon, so every case has one. A hidden or pending button
# has no label.
set -euo pipefail
root="${1:-$HOME/repos/spectrum-design-data}"
source "$(dirname "$0")/constraints.sh"
source "$(dirname "$0")/oracle.sh"
source "$(dirname "$0")/wrap.sh"
out="$(cd "$(dirname "$0")" && pwd)/button_scene.json"
oracle_cd

# render variant style size scheme scale label disabled pending hidden static state
render() {
  local variant=$1 style=$2 size=$3 scheme=$4 scale=$5 label=$6 dis=$7 pend=$8 hid=$9 stat=${10} state=${11}
  # Combinations an `error` constraint excludes are undefined in the oracle: skip them. The icon
  # is always drawn, so it is present.
  local opts="{\"variant\":\"$variant\",\"style\":\"$style\",\"isLabelHidden\":$hid,\"icon\":true"
  [ "$stat" != none ] && opts+=",\"staticColor\":\"$stat\""
  excluded button "$opts}" && return 0
  local args=(--size "$size" --label "$label" --color-scheme "$scheme" --scale "$scale" --format scene
    --option "variant=\"$variant\"" --option "style=\"$style\"" --option "isDisabled=$dis"
    --option "isPending=$pend" --option "isLabelHidden=$hid")
  [ "$stat" != none ] && args+=(--option "staticColor=\"$stat\"")
  # "hover+keyboard-focus" renders the combined state.
  [ "$state" != rest ] && for st in ${state//+/ }; do args+=(--state "$st"); done
  design-data render button "${args[@]}" | jq -c \
    --arg scheme "$scheme" --arg scale "$scale" --arg variant "$variant" --arg style "$style" \
    --arg size "$size" --arg state "$state" --arg label "$label" --arg stat "$stat" \
    --argjson dis "$dis" --argjson pend "$pend" --argjson hid "$hid" '
    (.nodes[] | select(.part == "container")) as $c
    | def r: if type == "number" then (. * 100 | round / 100) else . end;
      def part($p; $keys): [.nodes[] | select(.part == $p)][0]
        | if . == null then null
          else ({x: (.x - $c.x), y: (.y - $c.y)}
                + (. | {w, h, radius, fill, stroke, stroke_width, color}
                   | with_entries(select(.value != null and (.key | IN($keys[])))))
               ) | map_values(r) end;
    {scheme: $scheme, scale: $scale, variant: $variant, style: $style, size: $size, label: $label,
     disabled: $dis, pending: $pend, hidden: $hid, static: (if $stat == "none" then null else $stat end),
     state: $state, press: (.press.scale // null | if . then . * 10000 | round / 10000 else . end),
     container: part("container"; ["w","h","radius","fill","stroke","stroke_width"]),
     icon: part("icon"; ["w","h","color"]),
     label_part: part("label"; ["w","color"]),
     progress: part("progress"; ["w","h"])}'
}

# ring_case scheme scale size variant style static: one row of button_focus_ring.json
ring_case() {
  local args=(--size "$3" --color-scheme "$1" --scale "$2" --state keyboard-focus --format scene
    --option "variant=\"$4\"" --option "style=\"$5\"")
  [ "$6" != none ] && args+=(--option "staticColor=\"$6\"")
  design-data render button "${args[@]}" | jq -c --arg target container \
    --argjson meta "{\"scheme\":\"$1\",\"scale\":\"$2\",\"size\":\"$3\",\"variant\":\"$4\",\"style\":\"$5\",\"static\":$([ "$6" = none ] && echo null || echo "\"$6\"")}" \
    -f "$(dirname "$out")/ring.jq"
}

# Drops what the tests do not read (shared by both scene files).
tidy='map(
    del(.container.x, .container.y)
    | with_entries(select(.value != null and .value != false))
    | def c: if type == "string" then gsub("rgba?\\(|\\)| "; "") else . end;
      .container |= map_values(c) | (.icon, .label_part) |= (if . then map_values(c) else . end))'

{
  # A
  for scheme in light dark; do for scale in desktop mobile; do
    for variant in accent negative primary secondary; do for style in fill outline; do
      for state in rest hover down; do
        render $variant $style m $scheme $scale Label false false false none $state
      done
    done; done
  done; done

  # B
  for scheme in light dark; do for scale in desktop mobile; do
    for size in s m l xl; do
      render accent fill $size $scheme $scale Label false false false none rest
      render primary outline $size $scheme $scale Label false false false none rest
    done
  done; done
  for corner in "light desktop" "dark mobile"; do set -- $corner
    for size in s m l xl; do
      render accent fill $size $1 $2 Label false false false none down
      render primary outline $size $1 $2 Label false false false none down
    done
  done

  # C
  for corner in "light desktop" "dark mobile"; do set -- $corner
    for variant in accent primary secondary; do for style in fill outline; do
      for flags in "true false false none" "false true false none" "false false true none" \
                   "false true true none" "false false false white" "false false false black" \
                   "true false false white" "true false false black"; do
        render $variant $style m $1 $2 Label $flags rest
      done
    done; done
  done

  # D
  for state in hover down; do
    render accent fill m light desktop Label false true false none $state
    render primary outline m dark mobile Label false true false none $state
    for variant in primary secondary; do for style in fill outline; do for stat in white black; do
      render $variant $style m light desktop Label false false false $stat $state
    done; done; done
  done

  # E
  for size in m xl; do
    render accent fill $size light desktop A false false false none down
    render accent fill $size light desktop "A much longer label here" false false false none down
  done
} | jq -s "$tidy" > "$out"
echo "wrote $out"

# F
ring="$(dirname "$out")/button_focus_ring.json"
{
  for scheme in light dark; do for variant in accent negative primary secondary; do
    for style in fill outline; do for stat in none white black; do
      [ "$style" = outline ] && excluded button "{\"variant\":\"$variant\",\"style\":\"outline\"}" && continue
      ring_case "$scheme" desktop m "$variant" "$style" "$stat"
    done; done
  done; done
  for scheme in light dark; do for scale in desktop mobile; do for size in s l xl; do
    ring_case "$scheme" "$scale" "$size" accent fill none
    ring_case "$scheme" "$scale" "$size" primary outline none
    ring_case "$scheme" "$scale" "$size" secondary outline white
  done; done; done
} | jq -s . > "$ring"
echo "wrote $ring"

# H. keyboard-focus fill (button_focus_fill.json, read by button_pixels.rs); a "+" renders states combined:
#    keyboard-focus alone for variant x style x scheme x scale at M, for static white / black in two
#    corners, and for sizes S/L/XL (the pill radius) on accent and secondary fills; hover+keyboard-focus
#    and down+keyboard-focus for every variant x style in two corners; disabled and pending (with
#    hover) with keyboard-focus on accent / primary / secondary x fill / outline in two corners.
fill="$(dirname "$out")/button_focus_fill.json"
{
  for scheme in light dark; do for scale in desktop mobile; do
    for variant in accent negative primary secondary; do for style in fill outline; do
      render $variant $style m $scheme $scale Label false false false none keyboard-focus
    done; done
    for size in s l xl; do
      render accent fill $size $scheme $scale Label false false false none keyboard-focus
      render secondary fill $size $scheme $scale Label false false false none keyboard-focus
    done
  done; done
  for corner in "light desktop" "dark mobile"; do set -- $corner
    for variant in primary secondary; do for style in fill outline; do for stat in white black; do
      render $variant $style m $1 $2 Label false false false $stat keyboard-focus
    done; done; done
    for variant in accent negative primary secondary; do for style in fill outline; do
      for state in hover+keyboard-focus down+keyboard-focus; do
        render $variant $style m $1 $2 Label false false false none $state
      done
    done; done
    for variant in accent primary secondary; do for style in fill outline; do
      render $variant $style m $1 $2 Label true false false none keyboard-focus
      render $variant $style m $1 $2 Label false true false none keyboard-focus
      render $variant $style m $1 $2 Label false true false none hover+keyboard-focus
    done; done
  done
} | jq -s "$tidy" > "$fill"
echo "wrote $fill"

# G. request width (wrap.sh): the label wraps, the button grows a line height per extra line; a width
#    below the minimum width (2.25 x height) is lifted to it. One case has a one-letter label.
wrap_extra() { wrap_row button icon m light desktop 40 A; wrap_row button icon m light desktop 120 "Label"; }
regen_wrap button icon "$(dirname "$out")/button_wrap.json"
