#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/proofs"
lake build Proofs EstablishedRange
printf "import Proofs\nimport EstablishedRange\n#print axioms EstablishedRange.rand32_contract\n#print axioms EstablishedRange.rand64_contract\n" | lake env lean --stdin
