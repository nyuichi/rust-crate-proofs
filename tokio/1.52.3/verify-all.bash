#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$script_dir/../.." && pwd)

export CARGO_NET_OFFLINE=true
rust_target_dir="${CARGO_TARGET_DIR:-$repo_root/target}"
verus_target_dir="${VERUS_CARGO_TARGET_DIR:-$repo_root/target/verus/tokio-1.52.3}"
oneshot_probe_target_dir="${VERUS_ONESHOT_PROBE_TARGET_DIR:-$repo_root/target/verus/tokio-1.52.3-oneshot-poll-probe}"
verification_test_target_dir="${VERIFICATION_TEST_TARGET_DIR:-$repo_root/target/tokio-1.52.3-verification-tests}"

# Keep runtime regressions focused on the verified sync primitives.
CARGO_TARGET_DIR="$rust_target_dir" cargo test \
  --manifest-path "$script_dir/Cargo.toml" \
  --locked \
  --features full \
  --test sync_set_once

CARGO_TARGET_DIR="$rust_target_dir" cargo test \
  --manifest-path "$script_dir/Cargo.toml" \
  --locked \
  --features full \
  --test sync_oneshot

CARGO_TARGET_DIR="$rust_target_dir" cargo test \
  --manifest-path "$script_dir/Cargo.toml" \
  --locked \
  --features full \
  --test sync_watch

CARGO_TARGET_DIR="$rust_target_dir" cargo test \
  --manifest-path "$script_dir/Cargo.toml" \
  --locked \
  --features full \
  --test sync_broadcast

CARGO_TARGET_DIR="$rust_target_dir" cargo test \
  --manifest-path "$script_dir/Cargo.toml" \
  --locked \
  --features full,test-util \
  --test sync_mpsc

CARGO_TARGET_DIR="$rust_target_dir" cargo test \
  --manifest-path "$script_dir/Cargo.toml" \
  --locked \
  --features full,test-util \
  --test sync_mpsc_weak

CARGO_TARGET_DIR="$rust_target_dir" cargo test \
  --manifest-path "$script_dir/Cargo.toml" \
  --locked \
  --features full,test-util \
  --test sync_semaphore

CARGO_TARGET_DIR="$rust_target_dir" cargo test \
  --manifest-path "$script_dir/Cargo.toml" \
  --locked \
  --features full,test-util \
  --test sync_semaphore_owned

CARGO_TARGET_DIR="$rust_target_dir" cargo test \
  --manifest-path "$script_dir/Cargo.toml" \
  --locked \
  --features full,test-util \
  --test sync_notify

CARGO_TARGET_DIR="$rust_target_dir" cargo test \
  --manifest-path "$script_dir/Cargo.toml" \
  --locked \
  --features full,test-util \
  --test sync_notify_owned

CARGO_TARGET_DIR="$rust_target_dir" cargo test \
  --manifest-path "$script_dir/Cargo.toml" \
  --locked \
  --features full,test-util \
  --test sync_barrier

CARGO_TARGET_DIR="$rust_target_dir" cargo test \
  --manifest-path "$script_dir/Cargo.toml" \
  --locked \
  --features full,test-util \
  --lib \
  sync::tests::atomic_waker

# Exercise only the SetOnce and oneshot loom modules. `test-util` is needed
# because Tokio's cfg(loom) lib-test module also compiles paused-time helpers.
RUSTFLAGS="--cfg=loom" CARGO_TARGET_DIR="$rust_target_dir/loom-set-once" cargo test \
  --manifest-path "$script_dir/Cargo.toml" \
  --locked \
  --features full,test-util \
  --lib \
  loom_set_once

RUSTFLAGS="--cfg=loom" CARGO_TARGET_DIR="$rust_target_dir/loom-oneshot" cargo test \
  --manifest-path "$script_dir/Cargo.toml" \
  --locked \
  --features full,test-util \
  --lib \
  loom_oneshot

# The full upstream watch and mpsc loom modules have deliberately explosive
# scheduler searches. Verus proves their protocol state spaces; these bounded
# exact cases connect the important production races without making the
# integrated run unbounded in practice.
run_loom_exact() {
  local target_name="$1"
  local test_name="$2"
  RUSTFLAGS="--cfg=loom" LOOM_MAX_PREEMPTIONS=2 \
    CARGO_TARGET_DIR="$rust_target_dir/$target_name" cargo test \
    --manifest-path "$script_dir/Cargo.toml" \
    --locked \
    --features full,test-util \
    --lib \
    "$test_name" \
    -- \
    --exact
}

run_loom_exact loom-watch sync::tests::loom_watch::smoke
run_loom_exact loom-watch sync::tests::loom_watch::multiple_sender_drop_concurrently
run_loom_exact loom-watch sync::tests::loom_watch::wait_for_returns_correct_value

run_loom_exact loom-broadcast sync::tests::loom_broadcast::broadcast_wrap
run_loom_exact loom-broadcast sync::tests::loom_broadcast::broadcast_two
run_loom_exact loom-broadcast sync::tests::loom_broadcast::drop_rx

run_loom_exact loom-mpsc sync::tests::loom_mpsc::closing_tx
run_loom_exact loom-mpsc sync::tests::loom_mpsc::closing_unbounded_tx
run_loom_exact loom-mpsc sync::tests::loom_mpsc::closing_and_sending

run_loom_exact loom-semaphore sync::tests::loom_semaphore_batch::basic_usage
run_loom_exact loom-semaphore sync::tests::loom_semaphore_batch::concurrent_cancel
run_loom_exact loom-semaphore sync::tests::loom_semaphore_batch::batch

run_loom_exact loom-notify sync::tests::loom_notify::notify_one
run_loom_exact loom-notify sync::tests::loom_notify::notify_waiters
run_loom_exact loom-notify sync::tests::loom_notify::notify_drop

run_loom_exact loom-atomic-waker sync::tests::loom_atomic_waker::basic_notification
run_loom_exact loom-atomic-waker sync::tests::loom_atomic_waker::test_panicky_waker

# Compile the existing positive and negative Send/Sync/Unpin assertions for
# Sender, Receiver, and Sender::closed without running unrelated tests.
CARGO_TARGET_DIR="$rust_target_dir" cargo test \
  --manifest-path "$script_dir/Cargo.toml" \
  --locked \
  --features full,test-util \
  --test async_send_sync \
  --no-run

expected_verus="0.2026.07.27.31579f0"
actual_verus=$(verus --version)
if [[ "$actual_verus" != *"$expected_verus"* ]]; then
  echo "expected Verus $expected_verus" >&2
  echo "$actual_verus" >&2
  exit 1
fi

CARGO_TARGET_DIR="$verus_target_dir" cargo verus verify \
  --manifest-path "$script_dir/verification/Cargo.toml" \
  --locked \
  --offline

# Check the erased proof-view representation against the standard
# UnsafeCell<MaybeUninit<T>> used by Tokio's transparent non-loom wrapper.
CARGO_TARGET_DIR="$verification_test_target_dir" cargo test \
  --manifest-path "$script_dir/verification/Cargo.toml" \
  --locked \
  --offline \
  tokio_loom_cell::layout_tests::erased_layout_matches_tokio_value_field

# This is a connection/translation probe for the production-shaped
# `Future::poll` signature. Its body is an explicit external boundary, so
# `0 verified, 0 errors` means the signature connected successfully; it is not
# a proof of the oneshot poll body or its wake-up behavior.
CARGO_TARGET_DIR="$oneshot_probe_target_dir" cargo verus verify \
  --manifest-path "$script_dir/verification-probes/oneshot-poll/Cargo.toml" \
  --locked \
  --offline
