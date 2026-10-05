# Independent nested arity replay

PASS: all seven recorded nested `split_vc` nodes reproduce their exact expected
child counts, for 15 nested child tasks in total. No solver was started.

`why3 -D why3` output is printer output, not a serialization intended to be
parsed as a fresh source theory. Reparsing it can redefine builtin types and
symbols. This audit instead uses the installed Why3 OCaml API on the frozen
original COMA:

1. `Env.read_file` parses the COMA and `Task.split_theory` obtains its roots.
2. Select the exact original goal name and apply `Trans.apply_transform
   "split_vc"` once.
3. Select the recorded child index and apply `split_vc` only to that task.
   No already successful sibling is transformed again.
4. Save every produced task with `Driver.print_task`, retaining the complete
   context. The original API root is byte-identical to independent CLI output
   from `why3 prove -D why3 input.coma -T Coma -G <goal>`.
5. Check that the nested parent bytes equal the selected initial child bytes,
   and both arities equal the stored complete proof-tree arities.

| Target | Initial arity | Selected child (zero based) | Nested arity |
| --- | ---: | ---: | ---: |
| `InlineExtension::eq` | 6 | 2 | 2 |
| `From for Method::from` | 2 | 1 | 3 |
| `PartialEq for Method::eq` | 3 | 1 | 2 |
| `PartialEq for Method_1::eq` | 3 | 1 | 2 |
| `PartialEq for Method_2::eq` | 3 | 1 | 2 |
| `PartialEq for &str::eq` | 3 | 1 | 2 |
| `PartialEq for str::eq` | 3 | 1 | 2 |

`report.json` pins each frozen input COMA and proof JSON, all full task streams,
commands, and parent/child identities. It supplements the surrounding Method
current/historical full-context comparison; it does not itself establish that
historical proofs apply to current contexts. That comparison remains recorded
by the surrounding reconciliation audit.

Reproduce with:

```sh
source /workspace/proof-tools/activate.sh
python3 http/1.5.0/verification/method/evidence/current-source-reconciliation-20261005/independent-astra-nested/audit.py
```

The helper is built in a temporary directory from `replay.ml`. Its CLI is:

```text
replay INPUT.coma ORIGINAL_GOAL CHILD_PATH_OR_DASH OUTPUT_DIR
```

Use `-` to split only the original goal. Use `1` to split the root and then its
second child. Longer comma-separated indices walk only that exact nested path.
No printed `.why` file is ever fed back into the parser.
