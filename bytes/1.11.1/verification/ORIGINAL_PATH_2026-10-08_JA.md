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

以下の限定経路まで通過。with_capacity→extend_from_slice→freezeは、off=0かつ容量に余裕のある場合も、
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
trust除去はこのleaf内に限定できる。BytesMutとB1 capabilityの対応は後述のbody gateで接続済み。

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

### 元の生成→追記→freeze→実byte読み出し: 限定経路で通過

`original-freeze-read-2026-10-08` は61 Coma / 61 proof JSON / null 0。
元の with_capacity → extend_from_slice → freeze → Bytes slice → first byte loadを接続した。
unique/off0、入力長より容量が大きい経路に限定し、live Bytesと入力全byte列の一致、
実ロード値（空入力ならNone）を証明する。実Shared Boxのbuf/capとpointer対応も含む。
B1 descriptorを保持しB2再構成へ消費する。元rebuild_vecは変更せずcfg-only helperで解釈する。

actual Bytes.ptrだけをnullへ変える負例はas_sliceのvalidity前提だけ失敗。
同じ最終source/stdでunique-write回帰は51 files通過。archive内sourceと現source一致を監査済み。
元crate native library/integration全1009 tests、no-default-features library checkも通過。
最終証拠: `probes/original-freeze-read-2026-10-08/evidence/audit.json` と `final-native/manifest.json`。

これは元bodyのsource-gated concrete proofであり、全crate・open trait refinementではない。
元のDropはformal gateから除外。Clone、複数reader、growth、off>0、len==cap、KIND_ARC、
Atomic値とBytes.dataの対応、refcount、vtable/callback、unwind cleanupは未完了。
local atomic constructors/static getterはnormal-return/type-validityだけのtrusted tool境界。
generic B1/B2/B4、typed memcpy、std observersは明示TCB。bytes protocolはtrustedにしていない。
exact pointer観測はconstructor由来の直接copy対応であり、任意の同address pointerのprovenance証明ではない。

決定: DeepModel比較ICEへの同じ試行、static/atomic materializationの迂回の追加投資は固定保留。
前者は実memory modelと元generic APIを保つ比較契約、後者は適切なfrontend/std契約という
新しい前提が得られた場合のみ再開する。local trusted除去とrefcount protocol証明は別課題として維持する。

### 強い最終契約と再利用監査（依頼stage 1–2）

`ownership-design-2026-10-08/STRONG_SPEC_JA.md`に、全original representation、
clone/slice/split/freeze/release/transfer/Dropの強い契約を設計した。Astraレビュー済み。
61件のphysical/content部分は有用だが、actual data/count/vtableが不足しているため
最終singleton protocolへのembeddingは未証明。現exclusive sidecarは共有用に配置変更が必要。
「既存証明が全てそのまま使える」「共有まで小変更で済む」は保証しない。
shared ghost access/readonly lifetime/token/weak atomic/Drop境界のadmissionを先に確認し、
新前提なしに同じ失敗を再試行しない。今回production変更・新API body証明はない。
archive全member hash、現source一致、active tool factをread-only auditで再確認した。

### 元Clone(&self)の共有更新関門: 方法未確立として固定

stock GhostSharedで実B1 capabilityを永久保持し、&selfから共有witnessを複製して
二つの実B4 readを証明した（34 files）。しかしB3向け回収はE0507で拒否、Coma0。
読み出しだけの永久保持はcomplete lifecycleには採用しない。Lunaがstock全関連API、
Astraがresource adequacyとcommitter opening案を検討。新generic ruleなしに
原Cloneの共有更新+回収を両立する方法は見つかっていない。productionは無変更。
新opening案は同invariantへのalternate Tokens再入を排除できず採用しなかった。
再開条件はD2026-10-08-Tで固定。証拠とコード:
`probes/tokenless-sharing-2026-10-08/RESULTS_JA.md`、`audit.json`。

### 新しい汎用trusted境界で登録更新callerが通過

ユーザーの方針変更をD-U、限定結果をD-Vに記録した。StdのMutex例・AtomicInvariant・
Committerの契約を参照し、別入口のないEventAtomicを明示TCBにした。
追加引数なしのduplicate(&self)から、元登録を保存し別の登録を生成するbody proofが通過
（最終8 files、native2 tests）。bytes固有の登録法則はtrustedにしていない。
原Bytesへの接続、physical共有・回収・自動Dropは依然未完了。
詳細: [trusted境界と証拠](TRUSTED_ATOMIC_EVENT_2026-10-08_JA.md)。

### 回収可能な共有read component: fixed2でexactly-onceまで通過

FullBorrow<PhysicalRegion>のdescriptorを共有し、別のEndBorrowを最後に回収する
構成で41 filesが通過。二つの実readの全byte列がinputと一致し、peer退役後の
readも保存。Release減算、実Acquire load、token全量のjoin/end、B3を接続した。
各退役receiptと追加の診断Acquireで最後の結果がちょうど1回と証明した。
Acquire省略・token不足はVC失敗、重複・live slice中の退役はRust型検査で拒否。
parentがsubjective回収権限を保持するfixed2 componentであり、原count1からの
clone/control/data/vtable/任意last-thread回収/自動Dropはまだ未接続。

原release_sharedは最後のbuffer/control解放を小さなfree_sharedに明示化した。
原orderingとlayoutは保持。native1011/noStd/portable targeted2が通過したが、
このコード変更のformal refcount接続は次の原Shared leaf gateで行う。
純粋なfraction-map補題15 bodiesも保存した。詳しい証拠はSTATUSのリンク先。

### 原Shared生成leaf: 実ref_cnt fieldとtyped control権限を接続

len<capのFrom<Vec>分岐に対応するconstructorが34 files/110 leavesで通過。
実core AtomicUsizeを生成し、そのままBox<Shared>へ移した後、そのfieldを
atomic権限が指すことをbodyで証明。Box由来のtyped Permも同じcontrol allocationを
指す。constructor単体の9 leavesも全通過。native/model object identityは
明示的なgeneric TCBであり、bytesのrefcount/last-owner法則はtrustedにしていない。
原Bytes.data/vtable、cloneから最後のcleanupまでの接続は引き続き作業中。
証拠: probes/original-shared-lifecycle-2026-10-08、commit 3ba052a6。
