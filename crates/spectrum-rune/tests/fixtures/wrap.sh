# Sourced by the regen scripts of the components whose recipe wraps its label (switch, checkbox,
# radio-button, button, badge; spectrum-design-data #1700 to #1704). Needs $root and `design-data`.
# `wrap_row <component> <control part|-> <size> <scheme> <scale> <width> <label>` prints one JSON row
# of a `*_wrap.json` file: the oracle's render at the request width (`--width`), geometry relative to
# the container. Several `label` lines are the scene's text nodes, one per line. A case where a word
# is wider than the room (the oracle warns) is skipped: it is undefined whatever the font. So is a
# case whose line breaks change with the width 8px narrower or wider: a break that close to the
# edge is not a clear case (Rune lays text out with the installed font, not Source Sans 3).
wrap_label="Wrapped label text goes here"

wrap_row() {
  local component=$1 control=$2 size=$3 scheme=$4 scale=$5 width=$6 label=$7 scene breaks w b
  render_at() {
    design-data render "$component" --size "$size" --color-scheme "$scheme" --scale "$scale" \
      --width "$1" --label "$label" --format scene 2>/dev/null
  }
  scene=$(render_at "$width")
  breaks() { jq -c '[.nodes[] | select(.part == "label") | .text]'; }
  b=$(echo "$scene" | breaks)
  for w in $((width - 8)) $((width + 8)); do
    [ "$(render_at "$w" | breaks)" = "$b" ] || return 0
  done
  echo "$scene" | jq -c --arg c "$component" --arg control "$control" --arg size "$size" \
    --arg scheme "$scheme" --arg scale "$scale" --argjson width "$width" --arg label "$label" '
    select((.diagnostics // []) | length == 0)
    | (.nodes[] | select(.part == "container")) as $k
    | def r: (. * 100 | round) / 100;
      def rel($n): {x: (($n.x - $k.x) | r), y: (($n.y - $k.y) | r), w: ($n.w | r), h: ($n.h | r)};
      {scheme: $scheme, scale: $scale, size: $size, width: $width, label: $label,
       container: {w: ($k.w | r), h: ($k.h | r), radius: ($k.radius | r)},
       control: ([.nodes[] | select(.part == $control)][0] as $n | if $n then rel($n) else null end),
       lines: [.nodes[] | select(.part == "label") | rel(.) + {text}]}'
}

# regen_wrap <component> <control part|-> <file>
regen_wrap() {
  {
    for size in s m l xl; do for width in 110 130 150 170 190 230; do
      wrap_row "$1" "$2" $size light desktop $width "$wrap_label"
    done; done
    for width in 130 170 210 250; do
      wrap_row "$1" "$2" m dark mobile $width "$wrap_label"
      wrap_row "$1" "$2" xl dark mobile $width "$wrap_label"
    done
    if declare -F wrap_extra >/dev/null; then wrap_extra "$1" "$2"; fi
  } | jq -s . > "$3"
  echo "wrote $3"
}
