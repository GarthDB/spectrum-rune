# Is the option combination `$o` excluded by an `error` constraint of this component catalog?
# Implements option-constraints.md (when ANDed, a literal or any-of; require an array or
# {present: true}); same semantics as src/constraints/mod.rs `check`. Prints true or false.
#   jq -e --argjson o '{"variant":"accent","style":"outline"}' -f constraints.jq components/button.json
# `$o` names options as the catalog does; a free-form option (icon) is `true` when set. An option
# left out takes its catalog default (null: unset).
(.options // {}) as $decl
| def val($k): if $o | has($k) then $o[$k] else $decl[$k].default end;
  def any_of($v; $w): ($w | if type == "array" then . else [.] end) | any(. == $v);
  def violated: .require | to_entries
    | any(if (.value | type) == "object" then val(.key) as $v | ($v == null or $v == false)
          else (val(.key) as $v | any_of($v; .value) | not) end);
  [(.constraints // [])[] | select(.severity == "error")
   | select(.when | to_entries | all(val(.key) as $v | any_of($v; .value)))
   | select(violated)] | length > 0
