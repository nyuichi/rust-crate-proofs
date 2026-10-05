source /workspace/proof-tools/activate.sh
RUSTFLAGS='-Zcrate-attr=feature(hasher_prefixfree_extras)' cargo test --locked --offline
