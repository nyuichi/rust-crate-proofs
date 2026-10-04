#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "$0")" && pwd)
cd "$script_dir"

if [[ "${HTTPARSE_PROOF_QUEUE_LOCKED:-0}" != 1 ]]; then
    exec "$script_dir/run-proof.bash" \
        env HTTPARSE_PROOF_QUEUE_LOCKED=1 "$0" "$@"
fi

export CARGO_NET_OFFLINE=true
configuration=${1:-default}
case "$configuration" in
    default)
        cargo creusot clean --force
        cargo creusot prove
        ;;
    no-default)
        cargo creusot clean --force
        cargo creusot prove -- --no-default-features
        ;;
    all-features)
        # httparse's only feature is `std`, which is already enabled by default.
        cargo creusot clean --force
        cargo creusot prove -- --all-features
        ;;
    *)
        printf 'Usage: %s [default|no-default|all-features]\n' "$0" >&2
        exit 2
        ;;
esac
