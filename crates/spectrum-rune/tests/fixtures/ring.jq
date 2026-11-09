# The focus ring of a scene (the `focus-indicator` part) and the part it surrounds ($target), as
# rects relative to the container, for tests/*_pixels.rs, added to the object $meta. `stroke` is
# `r,g,b`; `gap` is the ring's gapToParent token. Used by the regen-*.sh scripts:
#   design-data render ... | jq -c --arg target box --argjson meta '{"size":"m"}' -f ring.jq
(.nodes[] | select(.part == "container")) as $c
| (.nodes[] | select(.part == $target)) as $t
| (.nodes[] | select(.part == "focus-indicator")) as $r
| def rect($n): {x: ($n.x - $c.x), y: ($n.y - $c.y), w: $n.w, h: $n.h};
  $meta + {target: rect($t),
           ring: (rect($r) + {radius: $r.radius, stroke: ($r.stroke | gsub("rgba?\\(|\\)| "; "")),
                              width: $r.stroke_width,
                              gap: ([$r.tokens[] | select(.prop == "gapToParent") | .value | rtrimstr("px") | tonumber] | first)})}
