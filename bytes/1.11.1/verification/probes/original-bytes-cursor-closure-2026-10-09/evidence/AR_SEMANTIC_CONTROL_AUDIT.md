# AR semantic control archive audit

Audited 16 immutable archives: 2400 targets; 22015 prover, 61 null, and 0 structural leaves. All 61 saved null-task sidecars were replayed from the archived COMA/proof inputs.

## Method

This accumulating audit uses only completed immutable archives, their receipts, and saved task sidecars. It verifies member and target hashes and proof-tree totals. For each null leaf, it derives the full tactic path from that archive’s proof JSON, prints the archived COMA task with the non-prover Why3 task printer, and checks byte equality with the saved sidecar. It does not inspect live proof output or run a build, compiler, prover, or solver.

## Completed captures

| Control | Archive SHA-256 | Target VCs | Null leaves |
|---|---|---:|---:|
| `wrong_offset` | `48512843725d0d08e389a0439cbc906c197f563a448d230bbadca9894b410537` | 150 targets; 1387 prover / 0 structural | 4 |
| `wrong_length` | `ecea594de94cab3f74bcf828546109f6934c4266f04f22d6fdc9b8fda3061293` | 150 targets; 1386 prover / 0 structural | 4 |
| `lose_empty_owner` | `7e0142455bbc5ea268d07b929133907554a1b54bfcdad4a79fbc4ae6582afc99` | 150 targets; 1410 prover / 0 structural | 13 |
| `len_dispatch` | `eba31e058ea38913d78babd685d4ef2905835b60a709e892f99d7770e071d687` | 150 targets; 1346 prover / 0 structural | 5 |
| `base_read` | `c07e3f6bc4b60b8bb0e28427903b9607ac249575205316aa6f2a4963718c9278` | 150 targets; 1365 prover / 0 structural | 2 |
| `zero_without_lease` | `a325a5d5becad9cf38cd1cb05bae51ea603838db9c851e6eb31e9ae1e89bdca6` | 150 targets; 1380 prover / 0 structural | 4 |
| `reset_bound` | `bad7129695f4e36c6f65f4e93c966818505d8d921b034367057b4d450587dec2` | 150 targets; 1386 prover / 0 structural | 4 |
| `wrong_capacity` | `2c211ff8914984173e77b7de1618c73e291607cacfe366d4ee1d8ca9c3264886` | 150 targets; 1388 prover / 0 structural | 4 |
| `change_identity` | `c8b9b8174484eedab89efa007deb4e7374848f48c34418f0164c5b3a7d9bff02` | 150 targets; 1400 prover / 0 structural | 6 |
| `change_fraction` | `956d2f933ec69d6ba9a32dd31df06fa5f465c3fa83cf6f6391d6de6c2f34ab44` | 150 targets; 1403 prover / 0 structural | 6 |
| `spurious_registration` | `8ec74a0f842ebd43dbd10851b5dbb2aa98f5e518e577c342ed081f0626610b9a` | 150 targets; 1390 prover / 0 structural | 1 |
| `omit_owner` | `980648fead1253d455594813dec360e76eda1a1fc7336b062bcb2b74c96b3b9f` | 150 targets; 1337 prover / 0 structural | 1 |
| `omit_value` | `7ccc7d0f0b665e9b4669a81528c618f3290a73c1da90769bb174c01acaf74be7` | 150 targets; 1376 prover / 0 structural | 1 |
| `negative_missing_acquire` | `48e923735641a9c82a40413d364234500d5c1411c672dc4edec9e7c3ad6d38a8` | 150 targets; 1377 prover / 0 structural | 2 |
| `negative_missing_payload_free` | `42b35e72f9baf7c200c97beaef7013c37bc698a6eb522acbe98ca5eaf5e5721a` | 150 targets; 1340 prover / 0 structural | 2 |
| `negative_missing_control_free` | `847cfae798a8f813857552992f9d7f98539299147e500885eb122cb30c27f097` | 150 targets; 1344 prover / 0 structural | 2 |

### `wrong_offset`

All four retained leaves are inc_start_api postcondition VCs: two pointer-address offset equalities and two returned-view suffix-content equalities. They show proof sensitivity to the mutated offset; they are not native counterexamples.

All 1297 members and 150 target COMA/proof pairs match their hashes. Recomputed proof leaves: 1387 prover, 4 null, 0 structural. All 4 saved tasks reproduce byte-for-byte.

| Goal and tactic/index path | Task sidecar SHA-256 |
|---|---|
| `vc_inc_start_api` `[0, 34, 3, 0, 0]` (`compute_specified:0,split_vc:34,split_vc:3,split_vc:0,split_vc:0`) | `9853c986c382081d09b791e8a18a8b381351a9a3540e0aaa10da8169e1c701ba` |
| `vc_inc_start_api` `[0, 34, 3, 1, 1]` (`compute_specified:0,split_vc:34,split_vc:3,split_vc:1,split_vc:1`) | `bd0f406806a4f1a6136fcb07b11b6f1bfd87ecbc89dc555279ba83d16836c602` |
| `vc_inc_start_api` `[0, 34, 4, 0, 0]` (`compute_specified:0,split_vc:34,split_vc:4,split_vc:0,split_vc:0`) | `7a70a466b0e4f8a674f5a17d54a3dd4665b381866189eb7a5159a328ec3190d5` |
| `vc_inc_start_api` `[0, 34, 4, 1, 1]` (`compute_specified:0,split_vc:34,split_vc:4,split_vc:1,split_vc:1`) | `e97a88fa1312e18d80ae6cdae9a32fa1a2cd15281326d8839dffbed6aab324a8` |

### `wrong_length`

All four retained leaves are inc_start_api postcondition VCs: two updated-length equalities and two returned-view suffix-content equalities. They show proof sensitivity to the mutated length; they are not native counterexamples.

All 1297 members and 150 target COMA/proof pairs match their hashes. Recomputed proof leaves: 1386 prover, 4 null, 0 structural. All 4 saved tasks reproduce byte-for-byte.

| Goal and tactic/index path | Task sidecar SHA-256 |
|---|---|
| `vc_inc_start_api` `[0, 33, 2, 0, 0]` (`compute_specified:0,split_vc:33,split_vc:2,split_vc:0,split_vc:0`) | `7aa3914a5895c062021c1c3fdb21aeca4a87509b5239a2dff2a8e7ccc0d6717a` |
| `vc_inc_start_api` `[0, 33, 2, 1, 1]` (`compute_specified:0,split_vc:33,split_vc:2,split_vc:1,split_vc:1`) | `3ce61849950c5519ef51f6202a280695b4031c1ed4bc39003e24b877f3a82f3c` |
| `vc_inc_start_api` `[0, 33, 4, 0, 0]` (`compute_specified:0,split_vc:33,split_vc:4,split_vc:0,split_vc:0`) | `d5cc06f105f100d79f96c052d1191e8c91cba2c80b43c092748a188641a0a046` |
| `vc_inc_start_api` `[0, 33, 4, 1, 1]` (`compute_specified:0,split_vc:33,split_vc:4,split_vc:1,split_vc:1`) | `87eb29545e72bf9552833f7f213f50d348b215cc47c2e2a0f9310e407ef5631b` |

### `lose_empty_owner`

The 13 retained inc_start_api leaves are postcondition/frame goals for API-view validity, same owner, unchanged data/vtable, and preservation of owned-view identity/fraction/public metadata. They show proof sensitivity to the empty-owner mutation; they are not native counterexamples.

All 1297 members and 150 target COMA/proof pairs match their hashes. Recomputed proof leaves: 1410 prover, 13 null, 0 structural. All 13 saved tasks reproduce byte-for-byte.

| Goal and tactic/index path | Task sidecar SHA-256 |
|---|---|
| `vc_inc_start_api` `[0, 36, 0, 0, 2, 1]` (`compute_specified:0,split_vc:36,split_vc:0,split_vc:0,split_vc:2,split_vc:1`) | `c090068386a6ddc0805de6b46a091bbcd71f1ea714d7209ad216137cb9392736` |
| `vc_inc_start_api` `[0, 36, 0, 1, 2, 1]` (`compute_specified:0,split_vc:36,split_vc:0,split_vc:1,split_vc:2,split_vc:1`) | `2ff77679efa59927e4f21fc2348441ea70009fe3d159364360508267d0c71978` |
| `vc_inc_start_api` `[0, 36, 1, 0, 2, 1]` (`compute_specified:0,split_vc:36,split_vc:1,split_vc:0,split_vc:2,split_vc:1`) | `ffa86ec9aaf02f0fc4884c9c6590d1e1d0645f3ffb3404c0cb14834812e3e591` |
| `vc_inc_start_api` `[0, 36, 1, 1, 2, 1, 0]` (`compute_specified:0,split_vc:36,split_vc:1,split_vc:1,split_vc:2,split_vc:1,split_vc:0`) | `a4dc520b773ca7bc0c1dfd4fa3142f62b06f36001f9d6deed858719e32b12d5e` |
| `vc_inc_start_api` `[0, 36, 1, 1, 2, 1, 1]` (`compute_specified:0,split_vc:36,split_vc:1,split_vc:1,split_vc:2,split_vc:1,split_vc:1`) | `d14c26dcb6f897b0e3e9ac53d8065ba4f2e106a057cdba868287dff74aaf3b6f` |
| `vc_inc_start_api` `[0, 36, 5, 1, 2, 1, 0]` (`compute_specified:0,split_vc:36,split_vc:5,split_vc:1,split_vc:2,split_vc:1,split_vc:0`) | `ad7a1cd05235cbb8bb4641c06899952953356ba593721e308684a2d81a065154` |
| `vc_inc_start_api` `[0, 36, 5, 1, 2, 1, 1]` (`compute_specified:0,split_vc:36,split_vc:5,split_vc:1,split_vc:2,split_vc:1,split_vc:1`) | `9fe63fa715309827920ba7967958f0c67ee6af37daaae1839c220a907b18cf80` |
| `vc_inc_start_api` `[0, 36, 6, 0, 2, 1, 0]` (`compute_specified:0,split_vc:36,split_vc:6,split_vc:0,split_vc:2,split_vc:1,split_vc:0`) | `c73f207efc9d57ec20b648a5c68a431846186726b14e9fe041384f88b0f54746` |
| `vc_inc_start_api` `[0, 36, 6, 0, 2, 1, 1]` (`compute_specified:0,split_vc:36,split_vc:6,split_vc:0,split_vc:2,split_vc:1,split_vc:1`) | `293c1cae309855a7e49a7ca3b87157b06c0a9337020a09f46cb20061179ef3b7` |
| `vc_inc_start_api` `[0, 36, 6, 1, 2, 1, 0]` (`compute_specified:0,split_vc:36,split_vc:6,split_vc:1,split_vc:2,split_vc:1,split_vc:0`) | `9adad65ea0d4e8caa46bf98364639341956a1ebbf322a43675924a26e7ca7f47` |
| `vc_inc_start_api` `[0, 36, 6, 1, 2, 1, 1]` (`compute_specified:0,split_vc:36,split_vc:6,split_vc:1,split_vc:2,split_vc:1,split_vc:1`) | `6ffd67dcf997d6839d845b08344fe5cd7b0a04d40b154cf2a0d4aa4023e3b1bf` |
| `vc_inc_start_api` `[0, 36, 6, 2, 2, 1, 0]` (`compute_specified:0,split_vc:36,split_vc:6,split_vc:2,split_vc:2,split_vc:1,split_vc:0`) | `6e128171c04b37d3ca9fe1a9d6322131390f651b84db265c85a7402e0557d57b` |
| `vc_inc_start_api` `[0, 36, 6, 2, 2, 1, 1]` (`compute_specified:0,split_vc:36,split_vc:6,split_vc:2,split_vc:2,split_vc:1,split_vc:1`) | `d6d9072286af715e0e7e157b5639834d8f6d3d81fe0c83af02267acc0d0ea82e` |

### `len_dispatch`

The five retained bytes_cursor_terminal_drop leaves include a callback-registration fact, an explicit function-pointer precondition, and postcondition goals for ledger removal, non-ownership after the drop, and the reclaimed effect. They show proof sensitivity to length-based dispatch; they are not native counterexamples.

All 1297 members and 150 target COMA/proof pairs match their hashes. Recomputed proof leaves: 1346 prover, 5 null, 0 structural. All 5 saved tasks reproduce byte-for-byte.

| Goal and tactic/index path | Task sidecar SHA-256 |
|---|---|
| `vc_bytes_cursor_terminal_drop` `[0, 2, 2, 1]` (`compute_specified:0,split_vc:2,split_vc:2,split_vc:1`) | `d67fd6516ae4d14d5e57197389270e01d2b6319c344de0f3f3068301be659ec9` |
| `vc_bytes_cursor_terminal_drop` `[0, 2, 3, 1]` (`compute_specified:0,split_vc:2,split_vc:3,split_vc:1`) | `4e9589f1b2b6a67634e6f592be342e42bf6c592299b4f993b7109825914e85bd` |
| `vc_bytes_cursor_terminal_drop` `[0, 3, 2, 0, 0, 1]` (`compute_specified:0,split_vc:3,split_vc:2,split_vc:0,compute_specified:0,split_vc:1`) | `27de64d0ead3aec40c8b61b0e9f9c04035186f7028b506c36563324a869ea2a4` |
| `vc_bytes_cursor_terminal_drop` `[0, 3, 4, 0, 1]` (`compute_specified:0,split_vc:3,split_vc:4,split_vc:0,split_vc:1`) | `6d2a1d9fb86114e80da3bb74c74ac13756c7bc1e683d6cf8d900e7b4c68d4bfd` |
| `vc_bytes_cursor_terminal_drop` `[0, 3, 5, 2, 1]` (`compute_specified:0,split_vc:3,split_vc:5,split_vc:2,split_vc:1`) | `d68bb492a5fb8868e038d376512c757556c69b942778c2a3e0827191fad71e96` |

The earlier printer attempt failed in tooling with `Failure("nth")`; its stderr sidecar SHA-256 is `f7cf2c5840bc580227686adc7d3801212569230210cc0fb6982a4659a31a386d`. This is distinct from the five null proof leaves, all of which replay with the tactic-aware printer.

### `base_read`

Both retained read_api leaves are postcondition goals: the returned pointer matches the view's bound pointer, and the returned logical view equals the input Bytes content. They show proof sensitivity to the base-read mutation; they are not native counterexamples.

All 1299 members and 150 target COMA/proof pairs match their hashes. Recomputed proof leaves: 1365 prover, 2 null, 0 structural. All 2 saved tasks reproduce byte-for-byte.

| Goal and tactic/index path | Task sidecar SHA-256 |
|---|---|
| `vc_read_api` `[0, 27, 3, 1]` (`compute_specified:0,split_vc:27,split_vc:3,split_vc:1`) | `c6bd95ec7b89905608136860554f9d2af1e9bd94f81a7c1b05c0a85ac34c5773` |
| `vc_read_api` `[0, 28, 1]` (`compute_specified:0,split_vc:28,split_vc:1`) | `a36418f1c71e57c464d010cf38802bade5da4594dd142784fcaae22f08baf0c3` |

### `zero_without_lease`

The four retained inc_start_api leaves are zero-branch obligations: cursor_by is zero and the returned BoundPtr has no physical bound. These are proof-level branch/API conditions, not evidence that native execution omitted a lease or event.

All 1299 members and 150 target COMA/proof pairs match their hashes. Recomputed proof leaves: 1380 prover, 4 null, 0 structural. All 4 saved tasks reproduce byte-for-byte.

| Goal and tactic/index path | Task sidecar SHA-256 |
|---|---|
| `vc_inc_start_api` `[0, 19, 4, 0, 0]` (`compute_specified:0,split_vc:19,split_vc:4,split_vc:0,split_vc:0`) | `8390a8ae4437dc68f24384e266e2d65fc5d9c26bed7581b311c4acfc5ee23e6d` |
| `vc_inc_start_api` `[0, 19, 4, 0, 1]` (`compute_specified:0,split_vc:19,split_vc:4,split_vc:0,split_vc:1`) | `c6565a92127f6577e2991afb0d4e97e1677bb9f8a33e1d206afcbf3780d664b2` |
| `vc_inc_start_api` `[0, 19, 4, 1, 0]` (`compute_specified:0,split_vc:19,split_vc:4,split_vc:1,split_vc:0`) | `7af32f9a6f1c11772e96f1d1d2bbdb661f0c12264b888094c5c733f71f7f9fdc` |
| `vc_inc_start_api` `[0, 19, 4, 1, 1]` (`compute_specified:0,split_vc:19,split_vc:4,split_vc:1,split_vc:1`) | `1cf6e594d76e1e970b0f14ac7066708736c22c3e4f2d60148f2ac0ea4f2f4d3a` |

### `reset_bound`

All four retained inc_start_api leaves are two api_view_valid postconditions and two returned-view suffix-content postconditions. They show proof sensitivity to the reset-bound mutation; they are not native counterexamples.

All 1299 members and 150 target COMA/proof pairs match their hashes. Recomputed proof leaves: 1386 prover, 4 null, 0 structural. All 4 saved tasks reproduce byte-for-byte.

| Goal and tactic/index path | Task sidecar SHA-256 |
|---|---|
| `vc_inc_start_api` `[0, 32, 0, 0, 0, 0]` (`compute_specified:0,split_vc:32,split_vc:0,split_vc:0,split_vc:0,split_vc:0`) | `d6a3255c204ed162b3cb2b472a5c411950d321381cf11e31d07ef86a879cb58f` |
| `vc_inc_start_api` `[0, 32, 0, 0, 1, 1]` (`compute_specified:0,split_vc:32,split_vc:0,split_vc:0,split_vc:1,split_vc:1`) | `ff8904a47341a7d1b86da36a0d3ad47dd98ab09c6d9bc05e97d2d43878b5e3a7` |
| `vc_inc_start_api` `[0, 32, 4, 0, 0]` (`compute_specified:0,split_vc:32,split_vc:4,split_vc:0,split_vc:0`) | `0bf0ed323973b4652077d2ab4012429046ece4503165b825cb31b4b3694eb324` |
| `vc_inc_start_api` `[0, 32, 4, 1, 1]` (`compute_specified:0,split_vc:32,split_vc:4,split_vc:1,split_vc:1`) | `d503c182c99b0ac628cb965e1c041d7fe7bf1087e217c3b503239f3952df64d6` |

### `wrong_capacity`

All four retained inc_start_api leaves are two api_view_valid postconditions and two same_api_owner postconditions. They show proof sensitivity to the capacity mutation; they are not native counterexamples.

All 1299 members and 150 target COMA/proof pairs match their hashes. Recomputed proof leaves: 1388 prover, 4 null, 0 structural. All 4 saved tasks reproduce byte-for-byte.

| Goal and tactic/index path | Task sidecar SHA-256 |
|---|---|
| `vc_inc_start_api` `[0, 34, 0, 0, 0, 0]` (`compute_specified:0,split_vc:34,split_vc:0,split_vc:0,split_vc:0,split_vc:0`) | `176a5e05fb133307afec26181c90fc9573f055f42a20af62a14d21923c9f6984` |
| `vc_inc_start_api` `[0, 34, 0, 0, 1, 1]` (`compute_specified:0,split_vc:34,split_vc:0,split_vc:0,split_vc:1,split_vc:1`) | `e069ce204698d656d75eb41284617ba89564b87a938e1f0bf7fe5fa63e7e2c2e` |
| `vc_inc_start_api` `[0, 34, 0, 1, 0, 0]` (`compute_specified:0,split_vc:34,split_vc:0,split_vc:1,split_vc:0,split_vc:0`) | `34e09f4dbb9a26132e8936781013cb3c146907910c309d10e1e15cdea7c588d5` |
| `vc_inc_start_api` `[0, 34, 0, 1, 1, 1]` (`compute_specified:0,split_vc:34,split_vc:0,split_vc:1,split_vc:1,split_vc:1`) | `8b82e9badf4951b6da6864350d37ba933ba55fe33932376025328f820c9cf206` |

### `change_identity`

All six retained inc_start_api leaves are postconditions for api_view_valid, same_api_owner, and preserved view_id. They show proof sensitivity to the identity mutation; they are not native counterexamples.

All 1299 members and 150 target COMA/proof pairs match their hashes. Recomputed proof leaves: 1400 prover, 6 null, 0 structural. All 6 saved tasks reproduce byte-for-byte.

| Goal and tactic/index path | Task sidecar SHA-256 |
|---|---|
| `vc_inc_start_api` `[0, 36, 0, 0, 1, 0, 0]` (`compute_specified:0,split_vc:36,split_vc:0,split_vc:0,split_vc:1,split_vc:0,split_vc:0`) | `a1af6bd4dce80b6dc76d30ea191900ea612b088edbedd7de174b90cb5a49899f` |
| `vc_inc_start_api` `[0, 36, 0, 0, 1, 1, 1]` (`compute_specified:0,split_vc:36,split_vc:0,split_vc:0,split_vc:1,split_vc:1,split_vc:1`) | `c6fa86590758490262ec64b7dd1a0c7b5bc9750ecc88f7e15ee2448422acb059` |
| `vc_inc_start_api` `[0, 36, 0, 1, 1, 0, 0]` (`compute_specified:0,split_vc:36,split_vc:0,split_vc:1,split_vc:1,split_vc:0,split_vc:0`) | `d871fcbe0fe8c804ecc317f111a1e8fc3684d9b2000116fa29870871f0ff34b3` |
| `vc_inc_start_api` `[0, 36, 0, 1, 1, 1, 1]` (`compute_specified:0,split_vc:36,split_vc:0,split_vc:1,split_vc:1,split_vc:1,split_vc:1`) | `b2075369de0000bbdd27c6f74bde8f4442805bf0bcaeddb1a67bdefaa9020ed7` |
| `vc_inc_start_api` `[0, 36, 6, 0, 1, 0, 0]` (`compute_specified:0,split_vc:36,split_vc:6,split_vc:0,split_vc:1,split_vc:0,split_vc:0`) | `f38d75a7367c0585f42d189d21f49abd511cc50bbcf83592e65c930823e191fb` |
| `vc_inc_start_api` `[0, 36, 6, 0, 1, 1, 1]` (`compute_specified:0,split_vc:36,split_vc:6,split_vc:0,split_vc:1,split_vc:1,split_vc:1`) | `77470ed0d897cde576443c666bef86cc5fff5a5583195f017dcfe1a9705397b3` |

### `change_fraction`

All six retained inc_start_api leaves are postconditions for api_view_valid, same_api_owner, and preserved view_fraction. They show proof sensitivity to the fraction mutation; they are not native counterexamples.

All 1299 members and 150 target COMA/proof pairs match their hashes. Recomputed proof leaves: 1403 prover, 6 null, 0 structural. All 6 saved tasks reproduce byte-for-byte.

| Goal and tactic/index path | Task sidecar SHA-256 |
|---|---|
| `vc_inc_start_api` `[0, 39, 0, 0, 1, 0, 0]` (`compute_specified:0,split_vc:39,split_vc:0,split_vc:0,split_vc:1,split_vc:0,split_vc:0`) | `75c6b48879ccb3297e571bd72ee15d48f5a4c434c5b479e167ecdc740e64057b` |
| `vc_inc_start_api` `[0, 39, 0, 0, 1, 1, 1]` (`compute_specified:0,split_vc:39,split_vc:0,split_vc:0,split_vc:1,split_vc:1,split_vc:1`) | `416d0ec8def573ff613777def82c61fc78b7e5c15c651aea1c1ec9776f6ef342` |
| `vc_inc_start_api` `[0, 39, 0, 1, 1, 0, 0]` (`compute_specified:0,split_vc:39,split_vc:0,split_vc:1,split_vc:1,split_vc:0,split_vc:0`) | `7379d5163aecb75b1a6d4364a6b9ad031180d83d893636d25df4266a6102de26` |
| `vc_inc_start_api` `[0, 39, 0, 1, 1, 1, 1]` (`compute_specified:0,split_vc:39,split_vc:0,split_vc:1,split_vc:1,split_vc:1,split_vc:1`) | `ff3b39ea88838dcf93add7a6b858a4548c5725373b523e5949eb2a6b7701701d` |
| `vc_inc_start_api` `[0, 39, 6, 1, 1, 0, 0]` (`compute_specified:0,split_vc:39,split_vc:6,split_vc:1,split_vc:1,split_vc:0,split_vc:0`) | `05b9e166b2afe7c8b2c9ba4db8e32fad4d2794b1adb14551f3e8cdf0b6e57a73` |
| `vc_inc_start_api` `[0, 39, 6, 1, 1, 1, 1]` (`compute_specified:0,split_vc:39,split_vc:6,split_vc:1,split_vc:1,split_vc:1,split_vc:1`) | `3f8643a17820d986d82b86f54ded38ade29d040c27617ef53c02deeaccf95405` |

### `spurious_registration`

The one retained vc_cursor_scope leaf requires the observation-map cardinality to equal one. It shows proof sensitivity to this registration mutation; it is not a native counterexample.

All 1299 members and 150 target COMA/proof pairs match their hashes. Recomputed proof leaves: 1390 prover, 1 null, 0 structural. All 1 saved tasks reproduce byte-for-byte.

| Goal and tactic/index path | Task sidecar SHA-256 |
|---|---|
| `vc_cursor_scope` `[0, 16, 0, 1]` (`compute_specified:0,split_vc:16,split_vc:0,split_vc:1`) | `2d9b32dcd908b587570fb851fcc7ef8bebeab647e622ea1a8a7e7081a3fc48f3` |

### `omit_owner`

The single retained vc_cursor_scope leaf is a bundled postcondition over returned-view invariants/content, ownership and public metadata, event-ledger insertion, and observational-frame preservation. It shows proof sensitivity to the removed owner component; it is not a direct runtime event-absence theorem.

All 1299 members and 150 target COMA/proof pairs match their hashes. Recomputed proof leaves: 1337 prover, 1 null, 0 structural. All 1 saved tasks reproduce byte-for-byte.

| Goal and tactic/index path | Task sidecar SHA-256 |
|---|---|
| `vc_cursor_scope` `[0, 9]` (`compute_specified:0,split_vc:9`) | `ef1c9130e0ecb4d6bf39e1262ac6b49f3fba8a0bd71d3a47005a9a4f5dec0fb9` |

### `omit_value`

The single retained vc_cursor_scope leaf is the archived Boolean result goal `not result4 = True`. It shows proof sensitivity to the omitted value component; no broader native behavior is inferred.

All 1299 members and 150 target COMA/proof pairs match their hashes. Recomputed proof leaves: 1376 prover, 1 null, 0 structural. All 1 saved tasks reproduce byte-for-byte.

| Goal and tactic/index path | Task sidecar SHA-256 |
|---|---|
| `vc_cursor_scope` `[0, 46]` (`compute_specified:0,split_vc:46`) | `8efdfc37a98a9704117395204950552fcd9be97fdab420238a2eba89071c3b41` |

### `negative_missing_acquire`

The two retained leaves are `acquired_Payload` postconditions in `release_core` and `shared_drop_checked`. They show proof sensitivity to removing the generic acquire feature from these retained callback paths; they are not standalone native counterexamples.

All 1299 members and 150 target COMA/proof pairs match their hashes. Recomputed proof leaves: 1377 prover, 2 null, 0 structural. All 2 saved tasks reproduce byte-for-byte.

| Goal and tactic/index path | Task sidecar SHA-256 |
|---|---|
| `vc_release_core` `[0, 26, 1, 1]` (`compute_specified:0,split_vc:26,split_vc:1,split_vc:1`) | `1362b1634c9313b699582650592ef1a8512fa5ef45261c5c57bb74410b5d5922` |
| `vc_shared_drop_checked` `[0, 29, 1, 1]` (`compute_specified:0,split_vc:29,split_vc:1,split_vc:1`) | `f5901ba6d59c8605ca513fa6fd619d720e47cbf17bc6287f97232f6bf610b9f3` |

### `negative_missing_payload_free`

Both retained `free_recovered` leaves require `not inv_Atomic_usize(ref_cnt)` at the ghost free-recovery boundary. The duplicated goal shows sensitivity to the uninhabitable GhostConjure(false) receipt condition, not that native payload deallocation was omitted.

All 1299 members and 150 target COMA/proof pairs match their hashes. Recomputed proof leaves: 1340 prover, 2 null, 0 structural. All 2 saved tasks reproduce byte-for-byte.

| Goal and tactic/index path | Task sidecar SHA-256 |
|---|---|
| `vc_free_recovered` `[0, 11]` (`compute_specified:0,split_vc:11`) | `c61655df5df09ce6fee2a4fc517b68ebeb71f3d1b9a6b00c3e313a51904c43fa` |
| `vc_free_recovered` `[0, 11]` (`compute_specified:0,split_vc:11`) | `c61655df5df09ce6fee2a4fc517b68ebeb71f3d1b9a6b00c3e313a51904c43fa` |

### `negative_missing_control_free`

Both retained `free_recovered` leaves require the negation of a conjunction covering namespace, pointer, size, alignment, and allocated-state facts at the ghost free-recovery boundary. The duplicated goal shows sensitivity to the uninhabitable GhostConjure(false) receipt condition, not that native control deallocation was omitted.

All 1299 members and 150 target COMA/proof pairs match their hashes. Recomputed proof leaves: 1344 prover, 2 null, 0 structural. All 2 saved tasks reproduce byte-for-byte.

| Goal and tactic/index path | Task sidecar SHA-256 |
|---|---|
| `vc_free_recovered` `[0, 12]` (`compute_specified:0,split_vc:12`) | `e064e035dea1ade3ff79ccabda21e08054e22e614a884b909f09b4c8a033f461` |
| `vc_free_recovered` `[0, 12]` (`compute_specified:0,split_vc:12`) | `e064e035dea1ade3ff79ccabda21e08054e22e614a884b909f09b4c8a033f461` |

## Limits

Each capture is diagnostic with correspondence status 2. VC failure establishes proof sensitivity only; it is not a native counterexample. The generic acquire mutation affects retained callback obligations across paths, so those are not independent native defects. The free controls exercise GhostConjure(false) recovery conditions; their remaining obligations do not directly establish that native payload or control deallocation was omitted.
