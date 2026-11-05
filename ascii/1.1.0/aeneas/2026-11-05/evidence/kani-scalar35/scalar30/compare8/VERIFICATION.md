# Original ASCII scalar comparisons8 — full input domains

The original manual macro PartialEq and PartialOrd implementations for AsciiChar against u8 and char, in both operand directions, return exactly numeric code equality and order. All eight original fully qualified API calls passed Kani with zero failed checks. Every valid AsciiChar code and every u8 / every valid Unicode scalar value is quantified; no string length, buffer shape or sampled input subset is involved.

Safe original AsciiChar construction and its public code observation cover all128 valid enum variants. Unicode construction excludes only values outside Rust char's valid scalar domain. The expected ordering is an independent integer branch returning Less/Equal/Greater, rather than a duplicate call to the target character comparison.

These are manually implemented macro bodies, distinct from excluded derived AsciiChar-to-AsciiChar comparisons. The unchanged original crate and manifest remain a path dependency. No source, model or verification check was changed. Kani0.68/CBMC6.11/nightly2026-08-21 and default x86_64 compiler semantics are trusted. Other targets, const evaluation, invalid enum/char representations and inherited ne/lt/le/gt/ge methods are not claimed by these eight checks. Publication/source binding remains separate; primary API count changes by zero here.
