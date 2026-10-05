#!/usr/bin/env bash
set -euo pipefail
probe_root=$(cd "$(dirname "$0")" && pwd)
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
cd "$probe_root"
mkdir -p evidence
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9
printf 'Bytes adapter metadata subset: one prover; 1024 MiB; no sc-drf\n'
feature_tree=$(cargo tree --locked -e features --edges normal,build)
if [[ "$feature_tree" == *'creusot-std feature "sc-drf"'* ]]; then exit 2; fi
native_log=evidence/native-test.log
proof_log=evidence/creusot-proof.log
if ! cargo test --locked --offline 2>&1 | tee "$native_log"; then
    exit 1
fi
cargo clean --package adapter-metadata-subset
rm -rf -- verif
if ! cargo creusot --only=coma -- --locked 2>&1 | tee evidence/creusot-translation.log; then
    exit 1
fi
cargo creusot clean --force
if ! cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1 2>&1 | tee "$proof_log"; then
    exit 1
fi

archive_tmp=evidence/adapter-metadata-subset-proof.tmp.tar.gz
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
mkdir -p "$stage/source" "$stage/proof"
cp README.md Cargo.toml Cargo.lock build.rs run.sh why3find.json "$stage/source/"
cp -R src "$stage/source/"
cp evidence/native-test.log evidence/creusot-translation.log evidence/creusot-proof.log "$stage/"
cp -R evidence/attempts "$stage/"
cp -R verif "$stage/proof/"
build_out=$(find "$CARGO_TARGET_DIR/debug/build" -type f -path '*/adapter-metadata-subset-*/out/actual_adapter_metadata.rs' -printf '%T@ %h\n' | sort -nr | head -n1 | cut -d' ' -f2-)
mkdir -p "$stage/extraction"
cp "$build_out"/actual_adapter_metadata.rs "$build_out"/*_source_snapshot.rs "$build_out"/source_fragments.txt "$stage/extraction/"
(
    cd "$stage"
    find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS
)
tar -C "$stage" -czf "$archive_tmp" .
archive_hash=$(sha256sum "$archive_tmp" | cut -d' ' -f1)
archive=evidence/adapter-metadata-subset-proof-${archive_hash:0:16}.tar.gz
mv "$archive_tmp" "$archive"
printf '%s  %s\n' "$archive_hash" "$(basename "$archive")" > evidence/adapter-metadata-subset-proof-${archive_hash:0:16}.sha256
{
    printf 'crate=bytes 1.11.1 source-sliced adapter metadata subset\n'
    printf 'native=cargo test --locked --offline (passed, 5 tests)\n'
    printf 'translation=cargo creusot --only=coma -- --locked (passed)\n'
    printf 'proof=cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1 (passed)\n'
    printf 'source=src/buf/{take.rs,limit.rs,chain.rs,reader.rs,writer.rs}; exact selected fragments and full snapshots are archived\n'
    cat "evidence/adapter-metadata-subset-proof-${archive_hash:0:16}.sha256"
} > evidence/receipt.txt
chmod a-w "$archive" "evidence/adapter-metadata-subset-proof-${archive_hash:0:16}.sha256" evidence/receipt.txt
printf 'Evidence archive: %s\n' "$archive"
