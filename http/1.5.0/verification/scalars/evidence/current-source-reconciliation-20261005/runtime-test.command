source /workspace/proof-tools/activate.sh
RUSTFLAGS='-Zcrate-attr=feature(hasher_prefixfree_extras,stmt_expr_attributes,proc_macro_hygiene)' cargo test --tests --features status --locked --offline
