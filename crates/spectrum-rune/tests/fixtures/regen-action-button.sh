#!/usr/bin/env bash
# Regenerates action_button_scene.json from the oracle: `design-data` v0.20.x run from the
# spectrum-design-data dataset root (default ~/repos/spectrum-design-data). The committed fixtures were
# made at dataset commit 95907289; at a later commit Button defaults and label wrapping differ, so
# check that commit out (e.g. `git worktree add --detach <dir> 95907289`) and pass <dir> as $1.
# Coordinates are relative to the container; the test compares with 1px / 3-per-channel tolerance.
# Only the fields the tests read are kept, rounded to 2 decimals; nulls and false flags are
# dropped and colours written as `r,g,b[,a]`, to keep the file small.
#
# Matrix (the full cross product is far too big; each block answers one question):
#   A. colour table at size M: quiet x selected x emphasized, rest / hover / down, in all four
#      scheme+scale corners; disabled at rest in all four corners
#   B. the same table for static white and black, in two corners (light desktop, dark mobile)
#      plus static x disabled (a14.23: the oracle draws it like a plain disabled button) for
#      quiet x selected x emphasized in all four corners, and its hover in light desktop (no
#      `down`: the oracle scales a disabled button then, which it never receives)
#   C. sizes XS..XL x four corners for the default look, plus selected + emphasized down in two
#      corners (press scale and radius per size)
#   D. flags alone, all sizes, light desktop and dark mobile: hide label, hold icon, hide label +
#      hold icon, pending, pending + hide label, quiet + hold icon selected
#   E. press scale at the perspective clamps: a one-letter and a long label, XS, M and XL
# Combinations an error constraint of the catalog excludes are skipped (constraints.sh); advisory
# ones (emphasized without selected) stay. The oracle always draws the optional icon, so every
# case has one.
set -euo pipefail
root="${1:-$HOME/repos/spectrum-design-data}"
source "$(dirname "$0")/constraints.sh"
out="$(cd "$(dirname "$0")" && pwd)/action_button_scene.json"
cd "$root"

# render quiet selected emphasized static disabled pending hidden hold size scheme scale label state
render() {
  local q=$1 sel=$2 emp=$3 stat=$4 dis=$5 pend=$6 hid=$7 hold=$8 size=$9 scheme=${10} scale=${11} label=${12} state=${13}
  local opts="{\"hideLabel\":$hid,\"isQuiet\":$q,\"isSelected\":$sel,\"isEmphasized\":$emp,\"hasHoldIcon\":$hold,\"isPending\":$pend,\"isDisabled\":$dis,\"icon\":true"
  [ "$stat" != none ] && opts+=",\"staticColor\":\"$stat\""
  excluded action-button "$opts}" && return 0
  local args=(--size "$size" --label "$label" --color-scheme "$scheme" --scale "$scale" --format scene
    --option "isQuiet=$q" --option "isSelected=$sel" --option "isEmphasized=$emp" --option "isDisabled=$dis"
    --option "isPending=$pend" --option "hideLabel=$hid" --option "hasHoldIcon=$hold")
  [ "$stat" != none ] && args+=(--option "staticColor=\"$stat\"")
  [ "$state" != rest ] && args+=(--state "$state")
  design-data render action-button "${args[@]}" | jq -c \
    --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" --arg state "$state" --arg label "$label" \
    --arg stat "$stat" --argjson q "$q" --argjson sel "$sel" --argjson emp "$emp" --argjson dis "$dis" \
    --argjson pend "$pend" --argjson hid "$hid" --argjson hold "$hold" '
    (.nodes[] | select(.part == "container")) as $c
    | def r: if type == "number" then (. * 100 | round / 100) else . end;
      def part($p; $keys): [.nodes[] | select(.part == $p)][0]
        | if . == null then null
          else ({x: (.x - $c.x), y: (.y - $c.y)}
                + (. | {w, h, radius, fill, color}
                   | with_entries(select(.value != null and (.key | IN($keys[])))))
               ) | map_values(r) end;
    {scheme: $scheme, scale: $scale, size: $size, label: $label,
     quiet: $q, selected: $sel, emphasized: $emp, disabled: $dis, pending: $pend, hidden: $hid, hold_icon: $hold,
     static: (if $stat == "none" then null else $stat end),
     state: $state, press: (.press.scale // null | if . then . * 10000 | round / 10000 else . end),
     container: part("container"; ["w","h","radius","fill"]),
     icon: part("icon"; ["x","y","w","h","color"]),
     label_part: part("label"; ["x","w","color"]),
     hold: part("hold-icon"; ["x","y","w","h","color"]),
     progress: part("progress"; ["x","y","w","h"])}'
}

table() { # corner list, static, states
  local stat=$1; shift; local states=$1; shift
  for corner in "$@"; do set -- $corner
    for q in false true; do for sel in false true; do for emp in false true; do
      for state in $states; do
        render $q $sel $emp $stat false false false true m $1 $2 Label $state
      done
    done; done; done
  done
}

{
  # A
  table none "rest hover down" "light desktop" "light mobile" "dark desktop" "dark mobile"
  for corner in "light desktop" "light mobile" "dark desktop" "dark mobile"; do set -- $corner
    for q in false true; do for sel in false true; do for emp in false true; do
      render $q $sel $emp none true false false true m $1 $2 Label rest
    done; done; done
  done

  # B
  for stat in white black; do table $stat "rest hover down" "light desktop" "dark mobile"; done
  for stat in white black; do for corner in "light desktop" "dark mobile"; do set -- $corner
    render false false false $stat true false false false m $1 $2 Label rest
    render true false false $stat true false false false m $1 $2 Label rest
  done; done

  # C
  for corner in "light desktop" "light mobile" "dark desktop" "dark mobile"; do set -- $corner
    for size in xs s m l xl; do render false false false none false false false false $size $1 $2 Label rest; done
  done
  for corner in "light desktop" "dark mobile"; do set -- $corner
    for size in xs s m l xl; do
      render false true true none false false false false $size $1 $2 Label down
      render true false false none false false false false $size $1 $2 Label hover
    done
  done

  # D
  for corner in "light desktop" "dark mobile"; do set -- $corner
    for size in xs s m l xl; do
      render false false false none false false true false $size $1 $2 Label rest
      render false false false none false false false true $size $1 $2 Label rest
      render false false false none false false true true $size $1 $2 Label rest
      render false false false none false true false false $size $1 $2 Label rest
      render false false false none false true true false $size $1 $2 Label rest
      render true true false none false false false true $size $1 $2 Label rest
    done
    for state in hover down; do
      render false false false none false true false false m $1 $2 Label $state
      render false false false white false true false false m $1 $2 Label $state
    done
  done

  # E
  for size in xs m xl; do
    render false false false none false false false false $size light desktop A down
    render false false false none false false false false $size light desktop "A much longer label here" down
  done

  # B2: static x disabled
  for stat in white black; do
    for corner in "light desktop" "light mobile" "dark desktop" "dark mobile"; do set -- $corner
      for q in false true; do for sel in false true; do for emp in false true; do
        render $q $sel $emp $stat true false false true m $1 $2 Label rest
      done; done; done
    done
    for state in hover; do render false false false $stat true false false true m light desktop Label $state; done
  done
} | jq -s 'map(
    del(.container.x, .container.y)
    | with_entries(select(.value != null and .value != false))
    | def c: if type == "string" then gsub("rgba?\\(|\\)| "; "") else . end;
      .container |= map_values(c) | (.icon, .label_part, .hold) |= (if . then map_values(c) else . end)) | .[]' \
  | jq -sc '.[]' | sed '1s/^/[/; $!s/$/,/; $s/$/]/' > "$out"
echo "wrote $out"
