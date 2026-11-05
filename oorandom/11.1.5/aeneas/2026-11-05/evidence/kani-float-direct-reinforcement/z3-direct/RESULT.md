# Direct joint independentIEEE reinforcement

Sameoriginaljoint32harness with Z3 succeeds:72checks,zero failures,4.647s, all128state/incbits exactIEEEbits/range/publicstate vs one actualwordcall. Direct64harness times out60s (processSIGTERM,-15), no counterexample. OldCadicaltimeouts unchanged; source/assertions/domain/checks unchanged. Solverversion/hash/fullcommand/sourcehash/RSSsampling recorded in runs.json. Peak sampled32/64~74/295MB below3GB. No stubs/models/observer/originalsource edit.

This independently reinforces the same Rand32contract already proved by A+B; it is not an extra API. Rand64's A+B proof in ../../kani-oorandom-float-split remains successful and is not withdrawn by this strongerjoint solver timeout. The completedcontracts are identical logically; direct64 is unfinished alternative evidence. No further solver retry.
