#!/usr/bin/env bash
# Regenerates progress_bar_scene.json, meter_scene.json and progress_circle_scene.json from the oracle: the foundation's
# `progress-bar`, `meter` and `progress-circle` recipes, rendered as the `rune` platform by `design-data` (versions in
# oracle.toml) against the foundation release pinned in .design-data.toml. Optional $1: a spectrum-design-data checkout to
# read instead. Coordinates are relative to the container; the test compares with 1px / 3-per-channel tolerance.
#
# Bars and meter are rendered at a fixed width of 200 with the label "Loading". Matrix (progress bar): scheme x scale x size x
# value (0, 40, 100); then, at size m, hideLabel with a value label, a custom value label, and indeterminate. Meter: scheme x
# scale x size x variant at value 40 with help text, and one without help text. Circle: scheme x scale x size (s, m, l) x
# value (0, 25, 50, 100), and indeterminate. Not in the matrix: `staticColor`, `labelPosition=side` (the oracle draws it like
# top), and the indeterminate animation (the oracle gives one frame).
set -euo pipefail
root="${1:-}"
source "$(dirname "$0")/oracle.sh"
dir="$(cd "$(dirname "$0")" && pwd)"
oracle_cd

JQ_COMMON='
  def col: gsub("rgba?\\(|\\)| "; "");
  def part($p): [.nodes[] | select(.part == $p)][0];
  def text($p; $c): part($p) as $t | if $t then {x: ($t.x - $c.x), y: ($t.y - $c.y), w: $t.w, h: $t.h, size: $t.font_size, color: ($t.color | col), text: $t.text} else null end;
'

bar() { # comp scheme scale size args...
  local comp=$1 scheme=$2 scale=$3 size=$4; shift 4
  design-data render "$comp" --color-scheme "$scheme" --scale "$scale" --size "$size" --format scene --width 200 --label Loading "$@" | jq -c \
    --arg comp "$comp" --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" --arg args "$*" "$JQ_COMMON"'
    (part("container")) as $c
    | (part("track")) as $tr | (part("fill")) as $f
    | {comp: $comp, scheme: $scheme, scale: $scale, size: $size, args: $args, container: {w: $c.w, h: $c.h},
       label: text("label"; $c), value: text("value"; $c), help: text("help-text"; $c),
       track: {y: ($tr.y - $c.y), h: $tr.h, radius: $tr.radius, fill: ($tr.fill | col)},
       fill: {x: ($f.x - $tr.x), w: $f.w, h: $f.h, radius: $f.radius, fill: ($f.fill | col)}}
    | with_entries(select(.value != null))'
}

circle() {
  local scheme=$1 scale=$2 size=$3; shift 3
  design-data render progress-circle --color-scheme "$scheme" --scale "$scale" --size "$size" --format scene "$@" | jq -c \
    --arg scheme "$scheme" --arg scale "$scale" --arg size "$size" --arg args "$*" "$JQ_COMMON"'
    (part("track")) as $t | (part("fill")) as $f
    | {scheme: $scheme, scale: $scale, size: $size, args: $args,
       track: {w: $t.w, h: $t.h, stroke: ($t.stroke | col), stroke_width: $t.stroke_width},
       fill: {sweep: $f.sweep, stroke_width: $f.stroke_width, color: ($f.color | col)}}'
}

{
  for scheme in light dark; do for scale in desktop mobile; do
    for size in s m l xl; do for v in 0 40 100; do bar progress-bar $scheme $scale $size --option "value=$v"; done; done
    bar progress-bar $scheme $scale m --option value=40 --option hideLabel=true --option 'valueLabel="2 of 5"'
    bar progress-bar $scheme $scale m --option value=40 --option 'valueLabel="2 of 5"'
    bar progress-bar $scheme $scale m --option isIndeterminate=true
  done; done
} | jq -s . > "$dir/progress_bar_scene.json"

{
  for scheme in light dark; do for scale in desktop mobile; do
    for size in s m l xl; do for variant in informative positive notice negative; do
      bar meter $scheme $scale $size --option value=40 --option "variant=\"$variant\"" --option 'helpText="Help"'
    done; done
    bar meter $scheme $scale m --option value=40 --option 'variant="informative"'
  done; done
} | jq -s . > "$dir/meter_scene.json"

{
  for scheme in light dark; do for scale in desktop mobile; do
    for size in s m l; do
      for v in 0 25 50 100; do circle $scheme $scale $size --option "value=$v"; done
      circle $scheme $scale $size --option isIndeterminate=true
    done
  done; done
} | jq -s . > "$dir/progress_circle_scene.json"
echo "wrote $dir/progress_bar_scene.json, meter_scene.json and progress_circle_scene.json"
