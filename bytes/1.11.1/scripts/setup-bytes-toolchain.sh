#!/usr/bin/env bash
# Reproduce the bytes-only Linux toolchain; retain the existing itoa installation.
set -euo pipefail
base_root=${BYTES_BASE_TOOL_ROOT:-/workspace/proof-tools}
tool_root=${BYTES_TOOL_ROOT:-/workspace/bytes-proof-tools}
source "$base_root/activate.sh"
rustup_binary=$CARGO_HOME/bin/rustup
creusot_sha=318615be3b8bbc60d1f6d52469ba5c0bdebed4f1
why3_sha=54c92f96bb0711d6e991c18f10bfbc08d90d028b
why3find_sha=eab37557d3e24e1913a3c4f44bc5528ef497c6c9
if [[ -f "$tool_root/installation-manifest.json" ]]; then
 python3 - "$tool_root/installation-manifest.json" <<'PY'
import hashlib, json, pathlib, sys
m=json.loads(pathlib.Path(sys.argv[1]).read_text())
for name, value in m['binaries'].items():
 p=pathlib.Path(value['path'])
 if not p.is_file() or hashlib.sha256(p.read_bytes()).hexdigest()!=value['sha256']:
  raise SystemExit('Installed binary changed: '+name)
print('Pinned bytes toolchain already installed; binary hashes verified')
PY
 exit 0
fi
mkdir -p "$tool_root/cargo/bin" "$tool_root/vanilla"
if [[ ! -x "$tool_root/cargo/bin/rustup" ]]; then cp "$rustup_binary" "$tool_root/cargo/bin/rustup"; fi
for binary in cargo rustc; do ln -sfn rustup "$tool_root/cargo/bin/$binary"; done
export CARGO_HOME=$tool_root/cargo
export RUSTUP_TOOLCHAIN=nightly-2026-06-22
export CREUSOT_DATA_HOME=$tool_root/creusot-data
export XDG_CONFIG_HOME=$tool_root/config
export XDG_CACHE_HOME=$tool_root/cache
export PATH="$CARGO_HOME/bin:$PATH"
"$CARGO_HOME/bin/rustup" toolchain install "$RUSTUP_TOOLCHAIN" --profile minimal --component rustc-dev --component llvm-tools-preview --component rustfmt --component rust-src
if [[ ! -d "$tool_root/creusot-source" ]]; then
 git clone --depth 1 --branch v0.13.0 https://github.com/creusot-rs/creusot.git "$tool_root/creusot-source"
fi
[[ $(git -C "$tool_root/creusot-source" rev-parse HEAD) == "$creusot_sha" ]]
if [[ -n $(git -C "$tool_root/creusot-source" status --porcelain --untracked-files=no) ]]; then
 printf 'Retain the experimental source edits; use a fresh BYTES_TOOL_ROOT for setup.\n' >&2; exit 1
fi
export CARGO_TARGET_DIR=$tool_root/targets/build
cd "$tool_root/creusot-source"
cargo build --locked --release -p creusot-rustc -p cargo-creusot -p creusot-install -p prelude-generator
cp "$CARGO_TARGET_DIR/release/creusot-rustc" "$tool_root/vanilla/creusot-rustc"
ln -sfn "$CARGO_TARGET_DIR/release/cargo-creusot" "$CARGO_HOME/bin/cargo-creusot"
mkdir -p "$CREUSOT_DATA_HOME/toolchains/$RUSTUP_TOOLCHAIN/bin"
ln -sfn "$tool_root/vanilla/creusot-rustc" "$CREUSOT_DATA_HOME/toolchains/$RUSTUP_TOOLCHAIN/bin/creusot-rustc"
"$CARGO_TARGET_DIR/release/creusot-install" prelude provers why3-conf --provers-parallelism 1
if [[ ! -d "$tool_root/why3-source/.git" ]]; then
 mkdir -p "$tool_root/why3-source"
 git -C "$tool_root/why3-source" init
 git -C "$tool_root/why3-source" fetch --depth 1 https://gitlab.inria.fr/why3/why3.git "$why3_sha"
 git -C "$tool_root/why3-source" checkout --detach FETCH_HEAD
fi
[[ $(git -C "$tool_root/why3-source" rev-parse HEAD) == "$why3_sha" ]]
if [[ ! -d "$tool_root/why3find-source/.git" ]]; then
 git clone https://github.com/creusot-rs/why3find.git "$tool_root/why3find-source"
 git -C "$tool_root/why3find-source" checkout "$why3find_sha"
fi
[[ $(git -C "$tool_root/why3find-source" rev-parse HEAD) == "$why3find_sha" ]]
if [[ ! -d "$tool_root/ocaml/_opam" ]]; then
 opam switch create "$tool_root/ocaml" ocaml-base-compiler.5.3.0 --yes --jobs=4
fi
opam pin add --switch="$tool_root/ocaml" why3.git-54c92f96 "$tool_root/why3-source" --kind=path --yes --no-action
opam pin add --switch="$tool_root/ocaml" why3find.git-eab37557 "$tool_root/why3find-source" --kind=path --yes --no-action
opam install --switch="$tool_root/ocaml" why3 why3find --yes --jobs=4
for binary in why3 why3find; do ln -sfn "$tool_root/ocaml/_opam/bin/$binary" "$CREUSOT_DATA_HOME/bin/$binary"; done
python3 - "$tool_root" "$RUSTUP_HOME" "$base_root" <<'PY'
import hashlib,json,pathlib,shlex,sys
root,rustup,base=map(pathlib.Path,sys.argv[1:]);q=lambda p:shlex.quote(str(p))
config=root/'creusot-data/creusot_why3.conf'
s=config.read_text();config.write_text('[main]\nmagic = 14\nrunning_provers_max = 1\nmemlimit = 1024\ntimelimit = 5.0\n\n'+s)
activation=f'''export RUSTUP_HOME={q(rustup)}
export RUSTUP_TOOLCHAIN=nightly-2026-06-22
export CARGO_HOME={q(root/'cargo')}
export CREUSOT_DATA_HOME={q(root/'creusot-data')}
export XDG_CONFIG_HOME={q(root/'config')}
export XDG_CACHE_HOME={q(root/'cache')}
export OPAMROOT={q(base/'opamroot')}
export OPAMSWITCH={q(root/'ocaml')}
export CARGO_BUILD_JOBS=4
export CARGO_TARGET_DIR={q(root/'targets/bytes')}
export PATH="$CARGO_HOME/bin:$CREUSOT_DATA_HOME/bin:{base}/system/usr/bin:{base}/autoconf/bin:{base}:$PATH"
export LD_LIBRARY_PATH="{base}/system/usr/lib/x86_64-linux-gnu:$RUSTUP_HOME/toolchains/nightly-2026-06-22-x86_64-unknown-linux-gnu/lib${{LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}}"
'''
(root/'activate.sh').write_text(activation)
paths={'creusot-rustc':root/'vanilla/creusot-rustc','cargo-creusot':root/'cargo/bin/cargo-creusot'}
for name in ['why3','why3find','alt-ergo','z3','cvc4','cvc5']:paths[name]=root/'creusot-data/bin'/name
manifest={'creusot_sha':'318615be3b8bbc60d1f6d52469ba5c0bdebed4f1',
          'why3_sha':'54c92f96bb0711d6e991c18f10bfbc08d90d028b',
          'why3find_sha':'eab37557d3e24e1913a3c4f44bc5528ef497c6c9',
          'binaries':{name:{'path':str(p),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for name,p in paths.items()}}
(root/'installation-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
PY
