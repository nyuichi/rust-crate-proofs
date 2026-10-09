# AQ development diagnostics

The first frontend candidate failed before proof translation: two unwrapped
Pearlite expressions and an extra mutable Ghost-reference layer (E0308).
No failed semantic VC or native counterexample is claimed. Frozen input/log
archive aq-frontend-diagnostic-v1.tar.gz SHA256
837b39edaaad77517bbc2c39ca5fe0d3fc39b720a2f34c8637f6be392cf5c428
has 806 regular unique safe members, all hashes root-rechecked, and excludes
proof outputs/solver caches. Source repair follows immutable capture.

Native capture selection initially matched a call site as well as the
new_empty_with_ptr definition, then omitted the bytes namespace in the
without_provenance definition needle. Final v5 uses exact definition signatures
and captures 25 bodies. These were capture plumbing errors, not semantic
interface failures. Scratch logs v1–v5 preserve the observed diagnostics.

Frontend v2 passes ordinary Rust syntax/types but Creusot rejects program
null_mut/is_null calls in logic. Exact logical pointer predicates replace only
specification expressions; actual native calls remain. Immutable frontend
archive SHA256 0f8031230cf9599e1860da0be52317e3295b749cb1e5ebadd886d11c86275d8f.
This is a translator classification diagnostic, not a failed body proof.

Frontend v3 translates all inherited/new bodies successfully. The first
semantic run uses explicit diagnostic/checker-skip mode while correspondence
work is unfinished; it cannot be presented as an admission result even if all
body conditions prove. Source/generator inputs stay frozen through capture.

The first prover launch encounters a Why3 parser error before semantic VCs at
slice_view.coma line1817 (`UInt64.le begin end''0`). Source/log capture SHA256
86cc50f82f0d343567d8f44b2d7f5da22dc3c82594cdfc226ce3d40108c26be1;
separate immutable aq-coma-backend-diagnostic-v1 archives all 139 translated
Coma tasks with hashes. Investigate identifier escaping/alpha-renaming; no
failed semantic interface, body proof result or admission is claimed.

First completed semantic diagnostic: 139 files / 1261 prover leaves / three
nulls / zero structural leaves, only vc_slice_view. Archive SHA256
aecd8c7011c932b7c4746d25e48daae9899fbeb2e4560cbdb10acb75f72ea973.
Exact failed task sidecars were printed with exit zero at paths
[0,21,1,0,1,0], [0,21,1,1,1,0] and [0,40,1,1,0]. The first two are
view-valid/content obligations; the third is an impossible Child branch's
Resolve obligation. This is the first failure of the new semantic interface.
Read-only diagnosis finds opaque shifted metadata and a wrapper postcondition
that permits Child despite its actual View-only body. Repair must expose
body definitions and exact generic pointer facts and prove the narrower
returned variant, without weakening allocation/content or assuming Resolve.
