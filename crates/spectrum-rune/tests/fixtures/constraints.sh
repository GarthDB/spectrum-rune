# Sourced by the regen-*.sh scripts. `excluded <component> '<options json>'` succeeds if an `error`
# constraint of the component's catalog excludes the combination (matrices skip those), fails if it
# is allowed, and aborts the script if the catalog or filter is broken. $root is the dataset root.
constraints_jq="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/constraints.jq"
excluded() {
  local r
  r=$(jq -r --argjson o "$2" -f "$constraints_jq" \
        "$root/packages/design-data/components/$1.json") || exit 1
  [ "$r" = true ]
}
