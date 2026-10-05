#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source /workspace/proof-tools/activate.sh
cd "$script_dir"

case "${1:-translate}" in
  translate)
    cargo creusot --simple-triggers=false
    ;;
  prove)
    mapfile -t target_paths < <(
      tail -n +2 "$script_dir/evidence/current-after-next-contract/TARGETS.tsv" |
        cut -f1 |
        sed 's#^#verif/httparse_code_string_harness_rlib/#; s#$#.coma#'
    )
    "$script_dir/../../../run-proof.bash" bash -c \
      'set -euo pipefail
       source /workspace/scratch/httparse-string-model/creusot-env.sh
       cd "$1/string"
       shift
       if ! grep -Eq "^running_provers_max = 1$" "$WHY3CONFIG" ||
          ! grep -Eq "^memlimit = 1000$" "$WHY3CONFIG"; then
         printf "Expected isolated Why3 profile (one prover, 1000 MiB): %s\n" "$WHY3CONFIG" >&2
         exit 2
       fi
       package_path=$(why3find query creusot | sed -n "s/^[[:space:]]*path: //p")
       expected_package=/workspace/scratch/httparse-string-model/creusot-data/share/why3find/packages/creusot
       if [[ "$package_path" != "$expected_package" ]]; then
         printf "Unexpected Creusot theory package: %s\n" "$package_path" >&2
         exit 2
       fi
       why3find prove --no-cache -s -j 1 "$@"' \
      _ "$script_dir" "${target_paths[@]}"
    ;;
  *)
    echo "usage: $0 [translate|prove]" >&2
    exit 2
    ;;
esac
