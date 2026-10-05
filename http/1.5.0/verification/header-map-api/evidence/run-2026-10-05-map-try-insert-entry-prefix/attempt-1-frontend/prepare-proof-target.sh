#!/usr/bin/env bash
set -euo pipefail
script_dir=$(cd "$(dirname "$0")" && pwd)
package_dir=$(cd "$script_dir/../../.." && pwd)
cd "$package_dir"
python3 - "$script_dir" <<'PYCODE'
import hashlib, shutil, sys
from pathlib import Path
run=Path(sys.argv[1]); package=Path.cwd()
source=run/'emitted-fresh/verif/http_header_map_api_proof_rlib/header/map/impl_HeaderMap_T/try_insert_entry.coma'
expected='83b7989409c5a9d8d20c1059e6d281c52cf4c4d54250004bb853d140da1cd976'
actual=hashlib.sha256(source.read_bytes()).hexdigest()
if actual!=expected: raise SystemExit(f'unexpected archived COMA hash: {actual}')
target=package/'verif/http_header_map_api_proof_rlib/attempt1_try_insert_entry_prefix/header/map/try_insert_entry_resume.coma'
target.parent.mkdir(parents=True,exist_ok=True); shutil.copyfile(source,target)
(run/'prepared-target.txt').write_text(str(target.relative_to(package))+'\n')
print(f'prepared frozen COMA {target} ({actual})')
PYCODE
