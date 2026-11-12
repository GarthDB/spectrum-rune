#!/usr/bin/env bash
# Analysis only, writes nothing: for each component, renders the oracle's single and combined interaction states
# (`--state` repeats) and prints whether combining keyboard-focus with down / hover changes the render:
#   `component | options | scheme | down==down+focus | hover==hover+focus | down vs hover`
# `DIFF` in a focus column marks a combined state that is not the single one; `differ` in the last column marks a
# component whose down look is not its hover look. Used for a14.31. Findings (CLI 0.32.0, foundation 3.10.0):
#   - down+focus == down for Switch, Checkbox, Radio (unselected and emphasized) and Action-button: nothing to add.
#   - down+focus != down for a selected neutral Radio (the control takes the hover fill) and for the Accordion
#     (title and chevron take the focus ink): Rune cannot draw them, a pointer press hides keyboard focus
#     (`focus_visible` is false until a navigation key). Tests: `a_press_on_a_keyboard_focused_*` in
#     radio_button_pixels.rs and accordion_pixels.rs.
#   - hover+focus != hover for Action-button (default, quiet): known (README, a14.14).
# Property set compared: fill, stroke, stroke width and text/icon colour of every part but the focus ring.
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/oracle.sh"
oracle_cd

props='[.nodes[] | select(.part != "focus-indicator") | {p: .part, fill, stroke, stroke_width, color}]'
# render component scheme [render args...]: the compared properties of one render
render() {
  local comp=$1 scheme=$2; shift 2
  design-data render "$comp" --size m --color-scheme "$scheme" --scale desktop --format scene "$@" | jq -c "$props"
}
# cmp component label scheme [render args...]
cmp() {
  local comp=$1 label=$2 scheme=$3; shift 3
  local d df h hf a=same b=same c=differ
  d=$(render "$comp" "$scheme" --state down "$@"); df=$(render "$comp" "$scheme" --state down --state keyboard-focus "$@")
  h=$(render "$comp" "$scheme" --state hover "$@"); hf=$(render "$comp" "$scheme" --state hover --state keyboard-focus "$@")
  [ "$d" = "$df" ] || a=DIFF
  [ "$h" = "$hf" ] || b=DIFF
  [ "$d" = "$h" ] && c=same
  echo "$comp | $label | $scheme | down==down+focus: $a | hover==hover+focus: $b | down vs hover: $c"
}
for s in light dark; do
  cmp switch "unselected" $s
  cmp switch "selected" $s --option isSelected=true
  cmp switch "selected emphasized" $s --option isSelected=true --option isEmphasized=true
  cmp checkbox "unselected" $s
  cmp checkbox "selected" $s --option isSelected=true
  cmp checkbox "selected emphasized" $s --option isSelected=true --option isEmphasized=true
  cmp checkbox "error" $s --option isError=true
  cmp radio-button "unselected" $s
  cmp radio-button "selected" $s --option isSelected=true
  cmp radio-button "selected emphasized" $s --option isSelected=true --option isEmphasized=true
  cmp action-button "default" $s
  cmp action-button "selected" $s --option isSelected=true
  cmp action-button "quiet" $s --option isQuiet=true
  cmp accordion "closed" $s
  cmp accordion "open" $s --option isOpen=true
done
