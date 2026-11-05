# Four-configuration original NoHashHasher Debug supplement

All four feature/profile combinations have separate successful Kani evidence:
std debug/release copied unchanged from the fixed packet; no_std dependency
debug/release newly executed separately. The harness remains std in all cases.
Metadata/configuration guards confirm the original dependency feature selection;
no_std compilation also rejects the std-only IntMap export as expected.

This is the same limited u8/default-format/fixed64-writer contract throughout,
not generic Debug or inherited API completion. Source manifests/bodies and the
harness are unchanged; each variant carries archive checks and separate logs.
No Aeneas Formatter model or added machine claim is used. The existing fourteen
Aeneas mappings are unaffected. No live publication is performed here.

Keep both sibling variant directories together: the no_std configuration checker
compares its unchanged harness against the copied std variant. The candidate
contains that cross-check dependency under the recorded tool setup.
`verify-all.bash` reruns the two variants sequentially. Forming this candidate
copies existing std evidence; it does not claim a new fresh std verification run.
