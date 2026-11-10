# Sourced by the regen-*.sh scripts (after $root is set). Checks the `design-data` version against
# oracle.toml at the repo root and prints the dataset commit the script runs against.
oracle_toml="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)/oracle.toml"
oracle_get() {
  local v
  v=$(sed -n "s/^$1 *= *\"\([^\"]*\)\".*/\1/p" "$oracle_toml")
  [ -n "$v" ] || { echo "oracle.toml has no non-empty '$1'" >&2; exit 1; }
  echo "$v"
}
oracle_min=$(oracle_get cli_min)
oracle_max=$(oracle_get cli_max)
oracle_dataset=$(oracle_get dataset)
cli_version=$(design-data --version)
v=${cli_version#design-data }
if [ "$(printf '%s\n%s\n' "$oracle_min" "$v" | sort -V | head -n1)" != "$oracle_min" ] ||
   [ "$(printf '%s\n%s\n' "$v" "$oracle_max" | sort -V | head -n1)" != "$v" ] || [ "$v" = "$oracle_max" ]; then
  echo "oracle.toml accepts design-data $oracle_min up to (not including) $oracle_max, found '$cli_version'" >&2
  exit 1
fi
dataset_head=$(git -C "$root" rev-parse --short HEAD)
echo "oracle: $cli_version, dataset $dataset_head (oracle.toml: $oracle_dataset)" >&2
case "$dataset_head" in
  "$oracle_dataset"*) ;;
  *) echo "warning: dataset $dataset_head is not the commit oracle.toml records ($oracle_dataset)" >&2 ;;
esac
