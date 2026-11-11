# Sourced by the regen-*.sh scripts (after $root is set). Checks the `design-data` version against
# oracle.toml at the repo root. $root is optional: empty, the foundation is the release pinned by the repo's
# .design-data.toml (a github source, fetched and cached by the CLI); set, it is a dataset checkout to read instead.
# $root may be relative; the scripts change directory, so make it absolute once.
[ -z "$root" ] || root="$(cd "$root" && pwd)"
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
if [ -n "$root" ]; then
  dataset_head=$(git -C "$root" rev-parse --short HEAD)
  echo "oracle: $cli_version, dataset $dataset_head (oracle.toml: $oracle_dataset)" >&2
  case "$dataset_head" in
    "$oracle_dataset"*) ;;
    *) echo "warning: dataset $dataset_head is not the commit oracle.toml records ($oracle_dataset)" >&2 ;;
  esac
else
  echo "oracle: $cli_version, foundation from .design-data.toml (oracle.toml dataset: $oracle_dataset)" >&2
fi

# Runs the CLI as the `rune` platform: from the repo root, whose .design-data.toml pins the foundation release
# and the manifest; or, with $root set, from a scratch dir that reads the foundation from that checkout instead.
oracle_cd() {
  local repo
  repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
  if [ -z "$root" ]; then
    cd "$repo"
    return
  fi
  oracle_scratch="$(mktemp -d)"
  trap 'rm -rf "$oracle_scratch"' EXIT
  printf 'default_platform = "rune"\n[source]\ntype = "path"\nroot = "%s"\n[platforms]\nrune = "%s/platforms/rune/manifest.json"\n' \
    "$root" "$repo" > "$oracle_scratch/.design-data.toml"
  cd "$oracle_scratch"
}

# The manifest's foundationVersion and the .design-data.toml source tag pin the same release: they must agree.
repo_root="$(dirname "$oracle_toml")"
pin=$(sed -n 's/.*"foundationVersion": *"\([^"]*\)".*/\1/p' "$repo_root/platforms/rune/manifest.json")
tag=$(sed -n 's/^tag *= *"\([^"]*\)".*/\1/p' "$repo_root/.design-data.toml")
[ "${pin#@adobe/spectrum-design-data@}" = "${tag#@adobe/spectrum-design-data@}" ] ||
  echo "warning: manifest foundationVersion '$pin' differs from the .design-data.toml tag '$tag'" >&2
if [ -n "$root" ]; then
  pin_commit=$(git -C "$root" rev-parse --short "$pin^{commit}" 2>/dev/null || true)
  case "$pin_commit" in
    "$oracle_dataset"*) ;;
    *) echo "warning: foundationVersion $pin is commit '${pin_commit:-unknown}', not oracle.toml's $oracle_dataset (unknown: the checkout may lack tags, \`git fetch --tags\`)" >&2 ;;
  esac
fi
