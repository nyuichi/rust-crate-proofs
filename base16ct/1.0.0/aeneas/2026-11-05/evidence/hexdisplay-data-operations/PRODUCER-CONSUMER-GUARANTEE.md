# HexDisplay slice storage: supplemental verification

Constructing `base16ct::HexDisplay(bytes)` stores the supplied byte slice; reading its public `.0` field returns that same slice in the extracted value model. Constructing and then projecting preserves every modeled byte and length, for arbitrary well-typed slice inputs. Three Lean theorems check these data operations through transparent selection drivers, and their axiom audits use only `propext` and `Quot.sound`.

This result concerns public tuple construction and field access. The driver functions are verification entry points, not additional functions exported by base16ct. It does not establish hexadecimal formatting/encoding/decoding, physical pointer identity, aliasing or general Rust memory safety. The Rust compiler checks the original borrowed signatures; the Lean model represents the slice value. Existing three primary API mappings remain unchanged.

The original crate source and generated Lean are unchanged. The portable replay passed after the single documented original-dependency source-prefix relocation; an earlier relocation comparison failure is retained as history. The producer performed this selfcheck, so it is not an independent review. Public adoption remains a separate review/integration gate.
