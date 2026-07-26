#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 2 || $# -gt 4 ]]; then
  echo "usage: $0 DATASET_ROOT OUTPUT [MIN_TOKENS] [MIN_LINES]" >&2
  exit 2
fi

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
crate_root="$(cd -- "$script_dir/.." && pwd)"
dataset_root="$(cd -- "$1" && pwd)"
output="$2"
min_tokens="${3:-50}"
min_lines="${4:-6}"

cargo build --release --manifest-path "$crate_root/Cargo.toml"
binary="$crate_root/target/release/weavatrix-clone"
: > "$output"

while IFS= read -r -d '' subset; do
  "$binary" \
    --mode near \
    --min-tokens "$min_tokens" \
    --min-lines "$min_lines" \
    --format java \
    --output-format bigcloneeval \
    "$subset" >> "$output"
done < <(find "$dataset_root" -mindepth 1 -maxdepth 1 -type d -print0 | sort -z)

echo "BigCloneEval import file: $output"
