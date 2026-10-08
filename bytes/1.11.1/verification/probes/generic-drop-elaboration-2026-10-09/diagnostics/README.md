# Interface diagnostics, not native Drop correspondence evidence

The first two scratch helper/caller runs used only
`#[ensures(*(^guard).0 == true)]`. The first scratch retained a Drop implementation;
the second removed it. The helper proved, but the caller did not: the summary
left the original nested borrow's future unconstrained. These are development
logs, not immutable exact-task archives or accepted native effect proofs.

`final-interface-scratch.rs` adds the body-proved relational loan frame and has
no Drop implementation. Both helper and caller then prove. The final archive
for the actual gate must independently connect native MIR to its generated
shadow; scratch success alone supplies no automatic-Drop correspondence claim.

The stock diagnostic under drop-feasibility and its original archives remain
unchanged. Its implicit destructor call is still erased by stock Creusot.
