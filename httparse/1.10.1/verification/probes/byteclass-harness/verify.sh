#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source /workspace/proof-tools/activate.sh

manifest="$script_dir/Cargo.toml"
package_name=httparse-byteclass-harness
cargo_target_dir="$script_dir/target"
proof_runner="$script_dir/../../../run-proof.bash"
coma_dir="$script_dir/verif/httparse_byteclass_harness_rlib"
proof_target="verif/httparse_byteclass_harness_rlib"
data_home=/workspace/proof-tools/creusot-data
why3find="$data_home/bin/why3find"
why3_config=/workspace/proof-tools/config/creusot/why3.conf
why3find_lib="$data_home/share/why3find"
dune_locations="why3find:lib:$why3find_lib"
scratch_root=/workspace/scratch/httparse-byteclass-harness

check_tools() {
  [[ -r "$manifest" ]] || { printf 'missing manifest: %s\n' "$manifest" >&2; exit 2; }
  [[ -x "$why3find" ]] || { printf 'missing why3find: %s\n' "$why3find" >&2; exit 2; }
  [[ -r "$why3_config" ]] || { printf 'missing Why3 config: %s\n' "$why3_config" >&2; exit 2; }
  [[ -d "$why3find_lib/packages/creusot" ]] || {
    printf 'missing Creusot Why3 package under: %s\n' "$why3find_lib" >&2
    exit 2
  }
  [[ -x "$proof_runner" ]] || { printf 'missing proof runner: %s\n' "$proof_runner" >&2; exit 2; }
}

translate() {
  mkdir -p "$scratch_root/translation-config" "$scratch_root/translation-cache" "$coma_dir"
  cd "$script_dir"
  CARGO_TARGET_DIR="$cargo_target_dir" cargo clean --manifest-path "$manifest" --package "$package_name"
  find "$coma_dir" -type f -name '*.coma' -delete
  env -u WHY3CONFIG \
    CREUSOT_DATA_HOME="$data_home" \
    DUNE_DIR_LOCATIONS="$dune_locations" \
    XDG_CONFIG_HOME="$scratch_root/translation-config" \
    XDG_CACHE_HOME="$scratch_root/translation-cache" \
    CARGO_TARGET_DIR="$cargo_target_dir" \
    CARGO_NET_OFFLINE=true \
    cargo creusot --simple-triggers=false -- --manifest-path "$manifest"
}

check_coma_targets() {
  local -a mandatory generated
  local target
  # Creusot emits included source bodies as nested Coma files; require each
  # body used by this checkpoint, while allowing additional generated output.
  mandatory=(
    "$coma_dir/byteclass/HEADER_VALUE_MAP/make_map.coma"
    "$coma_dir/byteclass/TOKEN_MAP/make_map.coma"
    "$coma_dir/byteclass/URI_MAP/make_map.coma"
    "$coma_dir/byteclass/is_header_name_token.coma"
    "$coma_dir/byteclass/is_header_value_token.coma"
    "$coma_dir/byteclass/is_method_token.coma"
    "$coma_dir/byteclass/is_uri_token.coma"
    "$coma_dir/classify_byte.coma"
    "$coma_dir/verification_model/accepted_prefix_span.coma"
    "$coma_dir/verification_model/cursor_slice_skip.coma"
    "$coma_dir/verification_model/impl_Clone_for_ByteClass/clone.coma"
    "$coma_dir/verification_model/impl_Clone_for_CursorModel/clone.coma"
    "$coma_dir/verification_model/impl_Clone_for_Span/clone.coma"
    "$coma_dir/verification_model/impl_Clone_for_TokenOutcome/clone.coma"
    "$coma_dir/verification_model/impl_Clone_for_TokenResult/clone.coma"
    "$coma_dir/verification_model/maximal_prefix_end.coma"
    "$coma_dir/verification_model/parse_token_model.coma"
  )
  for target in "${mandatory[@]}"; do
    [[ -s "$target" ]] || {
      printf 'translation did not produce a nonempty mandatory target: %s\n' "$target" >&2
      exit 2
    }
  done
  mapfile -d '' -t generated < <(find "$coma_dir" -type f -name '*.coma' -print0 | sort -z)
  if [[ "${#generated[@]}" -eq 0 ]]; then
    printf 'translation produced no Coma targets under %s\n' "$coma_dir" >&2
    exit 2
  fi
}

prove() {
  mkdir -p "$scratch_root/proof-config" "$scratch_root/proof-cache"
  cd "$script_dir"
  env \
    CREUSOT_DATA_HOME="$data_home" \
    WHY3CONFIG="$why3_config" \
    DUNE_DIR_LOCATIONS="$dune_locations" \
    XDG_CONFIG_HOME="$scratch_root/proof-config" \
    XDG_CACHE_HOME="$scratch_root/proof-cache" \
    "$why3find" query creusot

  WHY3CONFIG="$why3_config" "$proof_runner" env \
    CREUSOT_DATA_HOME="$data_home" \
    WHY3CONFIG="$why3_config" \
    DUNE_DIR_LOCATIONS="$dune_locations" \
    XDG_CONFIG_HOME="$scratch_root/proof-config" \
    XDG_CACHE_HOME="$scratch_root/proof-cache" \
    "$why3find" prove --no-cache -s -j 1 "$proof_target"
}

check_tools

case "${1:-translate}" in
  translate)
    translate
    check_coma_targets
    ;;
  prove)
    translate
    check_coma_targets
    prove
    ;;
  *)
    echo "usage: $0 [translate|prove]" >&2
    exit 2
    ;;
esac
