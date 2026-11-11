# Sourced by the regen-*.sh scripts (after $root is set). Checks the `design-data` version against
# oracle.toml at the repo root and prints the dataset commit the script runs against.
# $root may be relative; the scripts change directory, so make it absolute once.
root="$(cd "$root" && pwd)"
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

# Runs the CLI as the `rune` platform: from a scratch dir whose .design-data.toml reads the foundation
# from $root (the pinned oracle checkout) and the manifest from platforms/rune. A github [source]
# cannot be used here: the CLI (0.27.1) fetches no relationships, visuals or icons from it.
oracle_cd() {
  local repo
  repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
  oracle_scratch="$(mktemp -d)"
  trap 'rm -rf "$oracle_scratch"' EXIT
  printf 'default_platform = "rune"\n[source]\ntype = "path"\nroot = "%s"\n[platforms]\nrune = "%s/platforms/rune/manifest.json"\n' \
    "$root" "$repo" > "$oracle_scratch/.design-data.toml"
  cd "$oracle_scratch"
}

# The manifest pins the foundation by release tag, oracle.toml by commit: they must be the same commit.
pin=$(sed -n 's/.*"foundationVersion": *"\([^"]*\)".*/\1/p' "$(dirname "$oracle_toml")/platforms/rune/manifest.json")
pin_commit=$(git -C "$root" rev-parse --short "$pin^{commit}" 2>/dev/null || true)
case "$pin_commit" in
  "$oracle_dataset"*) ;;
  *) echo "warning: foundationVersion $pin is commit '${pin_commit:-unknown}', not oracle.toml's $oracle_dataset (unknown: the checkout may lack tags, \`git fetch --tags\`)" >&2 ;;
esac
