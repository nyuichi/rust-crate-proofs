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

An initial split-only printer failed on len_dispatch's path
[0,3,2,0,0,1]. That tool diagnostic is preserved separately. The immutable
control archive still contained all150 targets and five null leaves; allfive
exact tasks were subsequently recovered by replaying their recorded tactics.
