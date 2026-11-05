# Independent final release compatibility check

Final scanner-probe3bf4a8e17fd937036c55a52cf691bcc07b69a85e07e0f0115bc380b81c8e52fa
was independently executed source-only against the frozen37 fixture. Its raw
37-row output equals the old frozen probe output byte-for-byte. Writer-provided
final37 equals this independent execution. Positive41 preserves those37 rows and
adds four distinct singleton mappings. None configuration remains byte-identical
to the old immutable canonical baseline and omits the optional field.

Exact final binary, source and output hashes are in final-release-comparison.json.
The first attempt lacked Cargo in PATH and exited before scanning; activating
the existing tool environment made the independent source-only execution pass.
No edits, builds, installation or server calls were performed by this reviewer.

This concludes the requested legacy37/None regression check with no blocker.
A full phase2 source snapshot was not frozen by this reviewer; source-before to
final writer patch is a separate provenance/review input and must not be called
an independently verified phase2-to-final source identity. Captured-publish,
negative tests and publication readiness remain the other designated reviewers'
and publisher's gates, outside this result.
