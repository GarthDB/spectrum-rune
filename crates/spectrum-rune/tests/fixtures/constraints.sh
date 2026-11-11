# Sourced by the regen-*.sh scripts. `excluded <component> '<options json>'` succeeds if an `error`
# constraint of the component's catalog excludes the combination (matrices skip those), fails if it
# is allowed, and aborts the script if the catalog or filter is broken. Reads the component with `design-data component`.
constraints_jq="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/constraints.jq"
excluded() {
  local r err
  err=$(mktemp)
  # The CLI's stderr is shown only when it fails (a fetch error, an unknown component).
  r=$(design-data component "$1" 2>"$err" | jq -r --argjson o "$2" -f "$constraints_jq") || { cat "$err" >&2; rm -f "$err"; exit 1; }
  rm -f "$err"
  [ "$r" = true ]
}
