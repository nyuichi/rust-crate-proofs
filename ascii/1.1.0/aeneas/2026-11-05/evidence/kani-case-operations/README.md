# Original ASCII case and character body checks

Seven Kani harnesses directly verify unchanged ascii 1.1.0: to_ascii_uppercase/lowercase, make_ascii_uppercase/lowercase, as_char, eq_ignore_ascii_case and as_printable_char. Uppercase maps a-z to A-Z; lowercase maps A-Z to a-z; other codes are preserved. The mutators write that exact result back. Equality matches an independent mathematical lowercase-code comparison. as_char preserves the ASCII code; printable pictures map 0–31 to U+2400–U+241F and DEL to U+2421, preserving other characters.

Every one of the 128 valid AsciiChar values is represented by a nondeterministic byte below 128. The original safe constructor builds the enum; an assertion checks its public as_byte observation equals that byte before calling the target API. Thus the domain is the complete type, not a sampled/bounded subset. Equality checks all 128×128 pairs. No invalid enum, copied implementation, private-field accessor or raw-memory observer is used.

The separate harness package depends on the artifact root via `path = "../.."`. Source and manifest are archive-compared before the run. There is no loop/sample bound or unwind override. The constructor is additionally checked only for these ASCII inputs; its non-ASCII panic domain is not covered, so this does not complete the new API.

Reproduce with verify-kani-case-operations.bash at the artifact root. Kani 0.68.0, CBMC 6.11.0 and nightly-2026-08-21 check the original bodies under default features on x86_64. Trust includes compiler and Kani/CBMC standard/intrinsic semantics. This remains separate from the twenty-one Aeneas machine claims and does not resolve Aeneas's Char-cast backend limitation.
