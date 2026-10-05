# Prior debug-profile Why3 sessions

These XML sessions and shape files are copies of the `.bak` artifacts that
Why3 retained when the later release-profile run replaced the active
`step_chunk_size` and `parse_chunk_size` sessions. They are preserved as
generated; no proof JSON was reconstructed from them.

Immediately before the release run, the elevated no-cache debug-profile batch
reported all selected chunk targets green: 61/61 total, including
`step_chunk_size` (36/36) and `parse_chunk_size` (9/9). Both copied XML
sessions mark the file and every goal as proved. The `step_chunk_size` XML has
one split parent node in addition to its generated child goals; the command's
reported VC count is 36. The later release-profile proof JSON and Coma files
are preserved separately under `../chunk-release/`.

The generated debug Coma and `proof.json` files for these two targets were
overwritten by the release run. The preserved Why3 sessions are evidence of
the completed debug proof; the exact debug Coma/JSON can be regenerated and
re-proved in a later authorized proof slot if needed.
