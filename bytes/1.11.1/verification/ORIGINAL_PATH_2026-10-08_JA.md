# 元のbytes APIへ戻った後の作業記録

対象: bytes 1.11.1 / bytes-runtime-verification。開始点 c930f472。

## 1. 比較のDeepModel frontend障害: 切り分け済み

`probes/deepmodel-frontier-2026-10-08/RESULTS.md` とmanifestが証拠。
元の参照forwarding implと同じ形の最小診断で、receiverのDeepModel欠落によるICEを再現した。
receiverにモデルを加えるとICEは消えるが、元のgeneric boundsではRHSのDeepModel不足になる。
同じモデル型のRHS制約も加える診断だけは翻訳成功（8 Coma、prover未実行）。
具体的比較bodyはplaceholder、ghost列とmemoryの対応も未確立であり、元APIの証明ではない。

決定: 診断のghost列や追加generic制約をそのまま本体へ移植しない。
元APIの異種比較（byte列と文字列を含む）と標準比較契約の対応、実memoryに結び付くモデルが整った場合に再検討する。
同じICEへのassertion追加や独立代替bufferへの投資を繰り返さない。

## 2–3. 実allocationの仕様と元の生成→freeze→読み出し

作業中。with_capacity→extend_from_slice→freezeは、off=0かつ容量に余裕のある場合も、
Vec再構成と実Shared control blockの生成を含む。元のbodyを検証せずpublic freezeをtrustedにして
所有権移動が証明できたとは扱わない。静的vtableのfrontend障害とphysical capabilityの接続を別々に調査する。
実Clone/自動Dropとrefcount保存はこの限定経路から結論しない。

### 元のextendに接続した局所memory-effect境界

`storage_ops::copy_to_uninit_prefix_raw`は元のnative memcpyをそのまま実行するordinary program leaf。
一時trustedなのはtyped sliceへのcopy効果だけ（入力長のprefixがKnown、suffixと長さを保持）。
allocation identity/ownership/refcountは仮定しない。Astraレビュー済み。
同じ契約のループ実装bodyとcallerを含む5 filesがWhy3で通過。raw-copy bodyは未証明。
元crate native lib/integration 1009、leaf native tests 5が通過。
証拠: `artifacts/component-evidence/storage-raw-copy-2026-10-08/manifest.json`。
trust除去はこのleaf内に限定できるが、BytesMutとB1 capabilityの対応は別途未完了。

### 元のconstructor bodyの無改造診断

`probes/original-constructor-2026-10-08/evidence/diagnostic-result.json` は、
sidecar追加前の実field/bodyをsource hashで固定した診断。native constructor 2 testsは通過。
翻訳はinvalid_ptr内のpointer→usize castで停止し、Comaは0。ManuallyDrop/newとwrapping_addの契約不足も出ている。
この結果は保存sourceだけに適用し、後続instrumentationの結果と混同しない。

### 実allocationに接続した元BytesMutのbody gate: 通過

`original-unique-write-2026-10-08` の最終positive replayは50 Coma / 50 proof JSON / null 0。
actual with_capacity/from_vec、容量内reserve、spare_capacity_mut、extend_from_slice、advance_mutと
live BytesMutを返すcallerを含むsource-gated proof。元の4 native fieldsを保持し、cfg-only sidecarに
実B1のBoundPtrとRecovery/PhysicalRegionを運ぶ。ptrは数値address一致ではなくB1 pointer wordとの等式。
Some以外にauthorityを与えない。constructorのspare Unknown、追記prefixの値、未使用suffixのframeを検証する。
bit tagはbody-proved bitwise lemmaで処理し、bytes固有ownership/refcountのtrusted契約は追加していない。

未書込みbyteをadvance_mutで公開するnegativeは、新intervalのKnown-slot前提だけ失敗（51 JSON中null 1）。
そのunsafe pathはnativeでは実行しない。native extraction 1 test、元test_buf_mut 23 + test_bytes 118も通過。
B1/B4とgeneric typed memcpyのTCB、cfg/native解釈は明示したまま。全crate、Shared、growth、freeze、Dropの証明ではない。
sourceと証拠の一致をrootでも確認した。証拠はprobe/evidence/positive-final-50.tar.gzとaudit.json。

### freeze接続用のgeneric pointer観測とShared branch

数値base_modelだけでは原pointerとの対応を導かず、actual Vec値に対するexact mutable getter observerを追加した。
getter callerは1 file通過、別Vecのpointerを返すnegativeはそのpostconditionだけ失敗。
std observerとB1/B2のexact pointer clauseはgeneric TCBであり、permission/injectivity/liveness/refcountを与えない。
probe `vec-pointer-correspondence-2026-10-08` とstrict installerの新pointer-model.patchが証拠。

Shared branchのsource-sliced distinguishing testは7 files通過。元のBytes ptr/len、実Shared Boxのwardとraw pointer対応、
Shared buf/capを確認。alignmentは既存body-proved bit lemmaを使い元debug assertionを保持した。
vtable getterとAtomic constructorsは型のnormal-returnだけのgeneric abstraction。
Atomic初期値、Bytes.dataの格納pointerとの対応、refcount、callback、Dropは未証明。
full From<Vec>/freeze/callerの証明とは数えない。証拠 `probes/vtable-leaf-2026-10-08/`。
