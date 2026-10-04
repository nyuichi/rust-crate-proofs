# Unbound cleanup negative control

This run erases the sealed binding while retaining the same native non-null
pointer. It has 18 Coma files and 18 `proof.json` files under
`proof-artifacts/`: all supporting goals pass, while the intended negative
goal fails because B3 requires a matching `Some(namespace, capacity, 0)`
binding. The exact failed goal and local context are in
`negative_vc_detail.log`.
