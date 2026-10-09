# Printing the exact failed task

The proof tree records its actual tactics. For a null path, walk the archived
proof.json from its goal node, collecting each parent `tactic` and selected
child index, including the initial compute_specified node. Pass the resulting
comma-separated `tactic:index` sequence to print_proof_task with the archived
COMA file and goal name. Repeating split_vc alone is insufficient when another
compute_specified occurs inside the tree.

The included OCaml source uses the pinned Why3 library; with the activated
external toolchain, build via `ocamlfind ocamlopt -linkpkg -package why3,ocamlgraph
-o print_proof_task print_proof_task.ml`. This tool applies transformations and
prints a task; it invokes no solver. Reprinting saved tasks is an independent
check of artifact correspondence, not a fresh proof or a native counterexample.

AX has no failed semantic proof tasks: its selected ten bodies and complete 177-target positive each have zero null leaves. The new raw-free control is an in-memory source-correspondence rejection and does not claim a prover failure. Earlier task-printing diagnostics belong to their separately published ancestor evidence; the printer sources are retained as general inspection tools, not evidence that AX ran those experiments.
