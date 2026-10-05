#!/usr/bin/env bash
set -euo pipefail

probe=/workspace/rust-crate-proofs/httparse/1.10.1/verification/probes/derived-traits-nativeenv
evidence="$probe/evidence/nativeenv-remaining-proofs"
why3=/workspace/proof-tools/creusot-data/bin/why3
config=/workspace/scratch/httparse-derived-traits-nativeenv/why3.conf
package=/workspace/scratch/httparse-derived-traits-nativeenv/creusot-data/share/why3find/packages/creusot
start_line=${1:-1}
end_line=${2:-18}

line_no=0
while IFS= read -r target; do
  line_no=$((line_no + 1))
  if (( line_no < start_line )); then
    continue
  fi
  if (( line_no > end_line )); then
    break
  fi
  printf -v prefix '%02d' "$line_no"
  log="$evidence/$prefix.log"
  printf 'target[%d]=%s\n' "$line_no" "$target"

  if ! "$why3" prove --json -C "$config" -L "$package" -L "$probe/verif" \
      -P 'Z3,4.15.3' -t 30 -m 1000 "$probe/$target" >"$log" 2>&1; then
    cat "$log"
    printf 'Why3 command failed for selected target %s\n' "$target" >&2
    exit 2
  fi

  python3 - "$log" "$target" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
target = sys.argv[2]
text = path.read_text()
start = text.find('{')
if start < 0:
    print(f'NO_JSON: {target}: {text.strip()}')
    raise SystemExit(2)
decoder = json.JSONDecoder()
offset = start
results = []
while offset < len(text):
    while offset < len(text) and text[offset].isspace():
        offset += 1
    if offset >= len(text):
        break
    try:
        item, offset = decoder.raw_decode(text, offset)
    except json.JSONDecodeError as exc:
        print(f'JSON_PARSE_ERROR: {target}: {exc}')
        raise SystemExit(2)
    result = item.get('prover-result')
    if result is not None:
        results.append((item.get('term', {}).get('goal_name', '<unnamed>'),
                        result.get('answer', '<missing>'),
                        result.get('time', '<missing>'),
                        result.get('step', '<missing>')))
if not results:
    print(f'NO_GOAL_RESULTS: {target}')
    raise SystemExit(2)
for name, answer, elapsed, steps in results:
    print(f'{target}\t{name}\t{answer}\ttime={elapsed}\tsteps={steps}')
if any(answer != 'Valid' for _, answer, _, _ in results):
    raise SystemExit(1)
PY
done < "$probe/proof-nativeenv-remaining.targets"
