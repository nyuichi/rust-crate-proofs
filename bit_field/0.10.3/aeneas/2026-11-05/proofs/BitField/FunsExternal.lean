-- Manual u8-only external model, based on the retained official Aeneas template.
-- [bit_field]: external functions.
-- This is a template file: rename it to "FunsExternal.lean" and fill the holes.
module
public import Aeneas
public import BitField.Types
@[expose] public section
open Aeneas Aeneas.Std Result ControlFlow Error
set_option linter.dupNamespace false
set_option linter.hashCommand false
set_option linter.unusedVariables false
set_option linter.style.whitespace false
set_option linter.style.setOption false
set_option linter.style.longLine false
set_option linter.internalConstructors false

/- You can set the `maxHeartbeats` value with the `-max-heartbeats` CLI option -/
set_option maxHeartbeats 1000000

/- You can set the `maxRecDepth` value with the `-max-recdepth` CLI option -/
set_option maxRecDepth 2048
open bit_field

/-- [core::mem::size_of]:
    Source: '/rustc/library/core/src/mem/mod.rs', lines 375:0-375:34
    Name pattern: [core::mem::size_of]
    Visibility: public -/
/- This is a manually supplied, executable model ONLY for the one
    size_of::<u8>() call in the guarded extracted program. The generic signature
    comes from Aeneas; it does NOT model size_of for arbitrary types. Do not reuse
    it for other instantiations. The guard above checks every serialized LLBC
    function and every generated Lean module and rejects non-U8 applications.
    Kani independently checks Rust size_of::<u8>() = 1 on x86_64. -/
@[rust_fun "core::mem::size_of"]
def core.mem.size_of (_T : Type _) : Result Std.Usize := .ok 1#usize

