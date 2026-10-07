# 元の `BytesMut` 経路: stage 2 の不変条件と証明境界

日付: 2026-10-08

追記: この設計監査の後、上記Some/None方針を元のBytesMutへinstrumentし、
unique/off0/no-growthのsource-gated body proofが50 filesで通過した。
監査の「候補／未証明」は監査時点の区分であり、最新の範囲と証拠は
`ORIGINAL_PATH_2026-10-08_JA.md` と `probes/original-unique-write-2026-10-08/README.md` を参照。
freeze/Shared/Dropは引き続き未証明。

監査基準: commit `738cdeb5`（raw-copy leaf 接続後、original-unique sidecar 導入前）

対象: bytes 1.11.1 の元の `BytesMut` / `Bytes` 実装

## 目的と結論

この文書は、元の `ptr / len / cap / data` フィールドを保ったまま、実際の Vec 割当てに対する物理所有権をどこまで結び付けられるかを記録する。代替 owner 型、任意の `Seq` を実割当てとみなすモデル、または既存 verified API の証明を元の型へ移す案は対象外である。

現在の作業ツリーには、Creusot 専用 sidecar を使う original-unique gate が追加されている。`from_vec` が B1 `detach_vec` で実 Vec を消費し、`spare_capacity_mut` が B4 を呼ぶ変更である。これは具体的な実装候補だが、Astra の現行統合 gate が未完了のため、本書では**未証明の候補**として扱う。stage 2 完了とは扱わない。

stage 2 の判定対象は、実 Vec から始まる `KIND_VEC`, offset 0 の `BytesMut` が spare capacity に追記し、元の `freeze` と `Bytes::from(Vec)` の実際の `Shared` 分岐を通り、呼出元へ生きた `Bytes` を返す経路である。条件は開始時 `len < cap`、追記後も `len < cap` とする。出力を証明関数内で破棄しない。この限定は、自動 Drop、clone、refcount 更新を証明したことを意味しない。

## 証拠の状態

| 区分 | 実装・証拠 | 判定 |
|---|---|---|
| BODY / B1 物理境界 | `ownership_proof::raw_vec::detach_vec` | 信頼境界。実 Vec を消費し、実ポインタ・capacity と Recovery / 全域 PhysicalRegion を対応付ける意味を持つ。具体的な BytesMut 呼出元での統合 proof とは別。 |
| BODY / pointer metadata | `RawAllocation::bound_ptr_at_zero`, `RawAllocation::into_bound_ptr_at_zero`, `BoundPtr::as_non_null` | descriptor から offset-zero `BoundPtr` を作る本体は body-checked metadata 操作。単独では read/write/deallocation authority を作らない。 |
| TRUSTED / B2, B3 | `resume_vec`, `deallocate_vec`, `deallocate_bound_vec` | B2 は完全な Recovery / PhysicalRegion と Known prefix を要求して Vec を再構築する。B3 は完全な authority で明示解放する。元の `BytesMut::freeze` への接続はない。 |
| TRUSTED / B4 | `borrow_spare_preserving_prefix` など | region と BoundPtr を受けて物理 slice を作る汎用アクセス境界。実 BytesMut の live region との対応は caller が証明する。 |
| BODY / 台帳 | `owned_region::{OwnedRegion, PhysicalRegion}`、`unique_reclaim::reclaim` | OwnedRegion の split/join は純粋な RA/slot-map の操作。`reclaim` は B1/B4 に接続した孤立 helper。いずれも、それだけでは元 `BytesMut` の native field を証明しない。 |
| TRUSTED / typed byte-copy effect | commit `738cdeb5`, `storage_ops::copy_to_uninit_prefix_raw` | 同じ native `copy_nonoverlapping` に対し、typed slice の長さ・prefix 初期化・suffix frame だけを仮定する。manifest は5 proof files の成功を記録する。割当て identity、ownership、liveness、refcount は仮定しない。 |
| UNPROVED / current source candidate | `bytes_mut::BytesMut::unique_valid`, `from_vec`, `spare_capacity_mut`, `extend_from_slice`, `advance_mut` の `bytes_original_unique_gate` 部分 | 実 BytesMut fields と B1/B4 sidecar を結ぶ変更が現在の作業ツリーにある。これは active gate の入力であり、この文書時点では統合 proof 成功を主張しない。 |
| UNPROVED / actual freeze path | `BytesMut::freeze`, `rebuild_vec`, `impl From<Vec<u8>> for Bytes` | 実際の `ManuallyDrop` / `Vec::from_raw_parts` / Bytes shared record 組み立ては残る。B2、Shared field relation、vtable の関係は未接続。 |
| HISTORICAL component evidence | bound-vec-constructor, owned-region, bound-pointer-offset, storage-raw-copy probes | component ごとの証拠。異なる representation / extraction の isolated result を現行 BytesMut 統合 proof と数えない。 |

参照先は `verification/RAW_VEC_TRUSTED_BOUNDARY.md`、`verification/OWNERSHIP_FEASIBILITY_RESULTS.md`、および各 probe manifest である。ソース行番号ではなく method 名を基準にする。統合中の編集で行番号が変わるためである。

## stage 2 の original-field schema

### native record と proof-only sidecar

native `BytesMut` の実フィールドは引き続き `ptr: NonNull<u8>, len, cap, data: *mut Shared` である。現在の候補は `cfg(all(creusot, bytes_original_unique_gate))` に限り `unique_proof: Option<OriginalUniqueProof>` を加え、`Some` の payload を `BoundPtr` と `Ghost<(Recovery, PhysicalRegion)>` にする。native build には ghost sidecar がない。`None` は別 constructor / representation であり、physical authority があることを意味しない。

`Some` が表すべき exact unique state は次の通り。

1. **実割当ての起点:** Recovery と PhysicalRegion は、同じ実 Vec を消費した B1 `detach_vec` から出る。両 resource は同じ namespace/capacity に属し、region は `[0, cap)` 全体、Recovery は一致する allocation の唯一の回収 marker である。空 region のみから liveness を推論しない。
2. **native pointer の一致:** sidecar BoundPtr は B1 descriptor から得た offset-zero metadata であり、`self.ptr` はその stored pointer word と論理的に等しい。現在の候補は `self.ptr@ == proof.base.raw_pointer()` を要求し、`from_vec` で `proof.base.as_non_null()` を実フィールドへコピーする。数値 address の一致で置き換えない。
3. **元の packed data field:** `KIND_MASK == 1`, `KIND_VEC == 1`, vector offset bits は0、`len <= cap`。完全な field relation では capacity-class bits も `original_capacity_to_repr(cap)` に一致させる。現行候補の `data.addr_logic() < 32 && data.addr_logic() % 4 == 1` は KIND_VEC と offset 0 を区別するための粗い条件であり、capacity-class と `cap` の exact relation までは述べない。
4. **初期化 byte relation:** すべての `i < len` は Known で、slot 値は当該実 BytesMut の表示 byte と等しい。`i >= len` の spare slot は、B4 write 後に限り Known としてよい。任意の Seq を physical allocation の代用品にしない。現行 predicate は prefix が Known であることを要求し、`from_vec` / append 契約が値 relation を加える構成だが、これらはまだ proof 済みとしない。
5. **限定された append:** offset 0、`n <= cap-len` の no-grow append のみ。開始時 `len < cap` かつ `n < cap-len` とすれば、出力も `len < cap` となり、後の `Bytes::from(Vec)` が Shared 分岐を選ぶ。B4 spare borrow が `[len, cap)` を借り、typed raw-copy が source prefix だけを初期化し、`advance_mut` がその prefix を可視 length に含める順に関係付ける。

offset 0 は必要な gate 条件である。`BoundPtr::advance_within` の native pointer arithmetic の由来と、Creusot 側 `provenance_specs::wrapping_offset` の numeric-address contract は区別する。非零 offset の pointer equality はこの schema から導かない。

### 現在の候補 source がしていること・していないこと

- `from_vec` の proof cfg branch は、同じ入力 Vec を `detach_vec` に渡し、返る BoundPtr から実 `ptr` を設定して sidecar に capability を保存する。native cfg は従来の ManuallyDrop/as_mut_ptr body を維持する。
- `spare_capacity_mut` の proof cfg branch は sidecar capability の PhysicalRegion を `borrow_spare_preserving_prefix` に渡す。native cfg は従来の raw slice body を維持する。
- `extend_from_slice` は範囲 precondition 付きで reserve 後に spare を取得し、`copy_to_uninit_prefix_raw` を実行してから `advance_mut` する。現在の selected gate は reserve が capacity を増やさない範囲に制限する。
- `advance_mut` は length を進め、slot map を保持する。precondition は advance される全 slot が Known であること。
- これらは source-level correspondence の設計として合理的だが、active exact-source gate の結果が保存・監査されるまでは proof result と書かない。

## stage 2: 実際の `Bytes::Shared` へ渡す条件

元 `BytesMut::freeze` は `self` を ManuallyDrop にし、KIND_VEC branch で `rebuild_vec(ptr, len, cap, off)` を呼ぶ。`rebuild_vec` は `ptr.sub(off)` と `len+off`, `cap+off` を使って `Vec::from_raw_parts` を行う。その Vec が元の `Bytes::from(Vec<u8>)` に渡る。

この `From<Vec<u8>>` は `len == cap` なら boxed-slice path へ行き、`len != cap` なら実 `Shared { buf: ptr, cap, ref_cnt: AtomicUsize::new(1) }` を Box 化する。Bytes record は `ptr`, `len`, `data`, `vtable: &SHARED_VTABLE` を持つ。ここでの `bytes::Shared` は `bytes_mut.rs` の KIND_ARC Shared 型とは別である。`off == 0` と `len < cap` を固定し、実 Bytes Shared branch を選ぶ。

stage 2 の post-state は単に `Bytes` の Seq view では不十分である。少なくとも `Bytes.ptr ==` 元 B1 base pointer、Bytes length と Known prefix、`Shared.buf` と cap、refcount 初期値1、Shared data pointer、選択 vtable を actual record fields に対応付け、最後の Bytes を live return として残す必要がある。Bytes 型にこの経路のための verifier View/Invariant はまだない。

### 現時点での最小 missing bridge

現在の original-unique candidate は `RawAllocation::into_bound_ptr_at_zero` で B1 descriptor を消費し、BoundPtr と Recovery / PhysicalRegion を sidecar に残す。一方、既存 `resume_vec` (B2) は `RawAllocation`、Recovery、全 PhysicalRegion を受け取る。このため、今の state shape のままでは既存 B2 をそのまま呼べない。数値 address から RawAllocation を再生成してはならない。

一番小さい B2 接続案は、B1 の RawAllocation を sidecar に affine に保持し、B4 用 BoundPtr は既存 `RawAllocation::bound_ptr_at_zero(&self)` から借りて作ること。そのうえで freeze proof path が exact same descriptor と full capability を既存 B2 に渡す。代案として BoundPtr 版 B2 を加えるなら、B1 sealed binding から元 descriptor と同じ allocation を復元することをその汎用 primitive boundary が明示しなければならない。単なる address/capacity equality の contract では足りない。

さらに現在の B1/B2 contracts が明示する Vec base relation は `base_model` という numeric address model である。B1/B2 trusted prose は実 pointer preservation を説明するが、source contract surface の numeric model と exact pointer-word equality は別命題である。Bytes Shared field relation を証明する段階で、trusted B1/B2 boundary が exact consumed/returned allocation pointer identity をどう表すかを明示する。existing generic primitive contract の真偽を bytes-specific axiom で補ってはならない。

B2 の後にも、actual `Bytes::from(Vec)` Shared record body への bridge が残る。そこで Vec pointer から `Shared.buf` / `Bytes.ptr` への field transfer、Box / Atomic constructor contracts、および vtable field が必要となる。

## vtable / record translation の現状

不変な旧 D05 blocker を単に再試行しない。2026-10-08 の新しい source-sliced diagnostic は premise を変えた小実験であり、次の限定結果を持つ (`verification/probes/vtable-leaf-2026-10-08/`)。

- `&SHARED_VTABLE` を含む proof source は unsupported static definition kind で拒否された。
- proof cfg から static を除外し、`requires(true), ensures(true)` の trusted local getter を field expression に使うと translation は通る。native cfg は static initializer と getter を維持する。ただし cfg 間の correspondence は証明していない。
- 元の shared branch source-slice は、直接 `shared as usize` を使うと PointerExposeProvenance で拒否される。generic `pointer_addr` numeric helper に置換した branch は translation でき、requested ptr/len postcondition を COMA に出すが、返却 Bytes と Box/atomic intermediates は `Any.any_l()` のままで postcondition は未証明。
- `AtomicUsize::new(1)` / `AtomicPtr::new` は contractless external warning を出し、proof caller の precondition を作る。proof phase はこの diagnostic では実行されていない。

これは vtable/Bytes Shared API の成功証明ではない。実 static table と native getter の同一性、callback 値・意味、Box/atomic field relation が解決したことにもならない。ただし source-sliced translation という限定目的では変更した premise の結果なので、古い unchanged-frontend 再試行とは区別して記録する。

## 次に行う body proof と受入条件

最初の現実的な gate は、現行 candidate の**実際の元 BytesMut body**を選択し、`from_vec` → B4 `spare_capacity_mut` → typed copy → `advance_mut` を通して、no-grow append 後の BytesMut を live return すること。条件は入力実 Vec、off0、`0 < n < cap-len` とし、returned slot values が input Vec prefix と append source prefix の連結と等しく、suffix・ptr・cap・data field を frame すること。proof関数で BytesMut を破棄しない。この connected gate が通れば、B1/B4/append の exact source correspondence を得るが、freeze/Bytes/Drop はまだ未証明である。

次の stage 2 gate は、descriptor を保持したまま既存 B2 で Vec を再構成し、実 `Bytes::from(Vec)` Shared branch に入る exact source caller である。受け入れには次が必要。

1. 統合 proof の source snapshot が production `BytesMut::freeze` / `Bytes::from(Vec)` body と field layout に一致する。
2. B1 namespace, exact base pointer, full Recovery / PhysicalRegion と Known bytes が B2 に affine に移る。
3. B2 output Vec から actual Shared.buf / Bytes.ptr の pointer identity, capacity, length が説明される。
4. `Bytes.data` と `SHARED_VTABLE` の actual field relation が COMA / VCs で表現され、未証明の callback/Atomic/Box precondition に隠れていない。
5. 最終 Bytes を live caller result として返し、自動 Drop や clone/refcount を含めない。

もし vtable field materialization がなお記録値 relation を作れないなら、その点を exact frontend/VC blocker として保存する。vtable field を true/true trusted getter の identity contract で埋めたり、Bytes Shared construction 全体を trusted 化して stage completion と数えない。

## 非主張

- `BytesMut` / `Bytes` の automatic Drop、Shared clone、last-owner release、atomic/refcount-to-token correspondence は証明しない。
- reserve/reallocation、KIND_ARC、split/unsplit、nonzero offset、全 constructor はこの gate 外である。
- address equality や opaque Vec address observer は provenance、liveness、permission の根拠にならない。
- B1/B2/B3/B4 と raw-copy effect は明示 generic TCB であり、bytes-specific ownership theorem ではない。
- isolated probes、COMA translation、native tests、historical field-substituted extraction を original path integrated proof と呼ばない。
