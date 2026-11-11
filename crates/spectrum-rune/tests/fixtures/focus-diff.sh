#!/usr/bin/env bash
# Analysis only, writes nothing: for each component, renders rest and --state keyboard-focus with the
# oracle (versions in oracle.toml, run from the dataset root like the regen-*.sh scripts) and prints
# every part property the keyboard-focus state changes (fill, stroke, stroke width, text/icon colour;
# the focus-indicator ring itself is not listed): `component | options | scheme | part.prop | rest -> focus`.
# Used for the keyboard-focus analysis in the PR that added Button's focus fill (a14.29); the
# Switch, Checkbox, Radio-button, Action-button and Accordion looks (a14.32 to a14.36) follow it.
# Note: a selected / indeterminate Checkbox box and a selected Radio control have stroke_width 0 in
# the oracle, so their `stroke` lines here are not painted; only the unselected ones are.
set -euo pipefail
root="${1:-$HOME/repos/spectrum-design-data}"
source "$(dirname "$0")/oracle.sh"
oracle_cd

props='[.nodes[] | select(.part != "focus-indicator") | {p: .part, fill, stroke, stroke_width, color}]'
# diff component "options" scheme [render args...]
diff_states() {
  local comp=$1 label=$2 scheme=$3; shift 3
  local rest focus
  rest=$(design-data render "$comp" --size m --color-scheme "$scheme" --scale desktop --format scene "$@" | jq -c "$props")
  focus=$(design-data render "$comp" --size m --color-scheme "$scheme" --scale desktop --format scene --state keyboard-focus "$@" | jq -c "$props")
  jq -rn --argjson a "$rest" --argjson b "$focus" --arg comp "$comp" --arg label "$label" --arg scheme "$scheme" '
    [range(0; $a | length)] | map(. as $i | $a[$i] as $x | $b[$i] as $y
      | ["fill", "stroke", "stroke_width", "color"][] as $k
      | select($x[$k] != $y[$k]) | "\($comp) | \($label) | \($scheme) | \($x.p).\($k) | \($x[$k]) -> \($y[$k])") | .[]'
}
for scheme in light dark; do
  for sel in false true; do for emph in false true; do
    diff_states switch "selected=$sel emphasized=$emph" $scheme --option isSelected=$sel --option isEmphasized=$emph
    diff_states radio-button "selected=$sel emphasized=$emph" $scheme --option isSelected=$sel --option isEmphasized=$emph
  done; done
  for val in none selected indeterminate; do for err in false true; do for emph in false true; do
    o=(--option isError=$err --option isEmphasized=$emph)
    [ $val = selected ] && o+=(--option isSelected=true)
    [ $val = indeterminate ] && o+=(--option isIndeterminate=true)
    diff_states checkbox "value=$val error=$err emphasized=$emph" $scheme "${o[@]}"
  done; done; done
  for q in false true; do for sel in false true; do for emph in false true; do
    diff_states action-button "quiet=$q selected=$sel emphasized=$emph" $scheme --option isQuiet=$q --option isSelected=$sel --option isEmphasized=$emph --option icon=true
  done; done; done
  for v in accent informative negative neutral positive notice; do for st in bold subtle outline; do
    diff_states badge "variant=$v style=$st" $scheme --option "variant=\"$v\"" --option "style=\"$st\""
  done; done
  for open in false true; do for quiet in false true; do
    diff_states accordion "open=$open quiet=$quiet" $scheme --option isOpen=$open --option isQuiet=$quiet
  done; done
done | sort -u
