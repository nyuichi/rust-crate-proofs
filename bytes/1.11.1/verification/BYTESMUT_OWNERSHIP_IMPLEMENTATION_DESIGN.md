# bytes 1.11.1 所有権検証・実装設計案

2026-10-04。対象source checkpoint: `27cc729c35c1dafffb38562491c6759f6d74cc75`。
ユーザー提示のGPT6Pro案を起点に、実装とCreusot 0.13のAPIを再確認した設計案。
**この文書は実装・証明の完了報告ではない。新しいtrusted契約もまだ導入しない。**
前段の実験結果は [所有権境界の診断](BYTESMUT_OWNERSHIP_DESIGN.md) にある。

## 1. 採用する方針と修正点

既存の `Vec<T> -> Seq<T>` モデルを維持する。bytes内部の物理メモリbridge、
その上の領域管理、参照数protocolを分ける。bytes固有のsplit、promotion、
refcount、last-owner判定、Dropをtrustedにしない。

提示案からの重要な変更は次の通り。

| 論点 | 推奨設計 |
|---|---|
| OwnedRegionのsplit/join | 既存Resource algebraを使って証明する。ただし物理メモリとの解釈は別の明示的TCBであり、台帳の証明だけではアクセス権にならない |
| Shared内のSuspendedVec | 通常buildにも小さな内部型変更を入れ、Vecを保持しないprivate `RawBuffer` を第一候補にする。単にcfg専用の別storageに差し替えない |
| park_vec | RawBuffer案では実Vecを再構築しない。descriptorとRecoveryをSharedへ移す通常操作にでき、独立したtrusted parkを減らせる |
| 最後の破棄 | 保存していた昔のVec.lenの初期化を仮定しない。全領域・Recoveryを集めてraw allocationを解放する |
| 参照数 | 現在手元にあるghost値の個数ではなく、未返却の登録の集合に対応させる。forgetは登録を消さない |
| weak memory | native fetch_sub追加とrelease-sequence伝播を別の判定条件にする |
| 実Drop | 明示的release関数の本体証明と、Rustの自動Dropからの接続を別々に確認する |

bytes内へ実装を置くことは、TCBのsoundness要件を軽くしない。初期段階では
局所的なmemory-model拡張として監査し、安定後に汎用部分のcreusot-std移管を検討する。

## 2. 資源と物理メモリの対応

以下は設計記法であり、コンパイル可能なRust署名ではない。

- `AllocDesc<A>`: allocationの世代を含むidentity、provenance、base、capacity、
  layout、allocator。コピー可能な記述だけで、read/write/free権限を持たない。
- `Recovery<A>`: allocationを復元・解放する唯一のaffine権限。byte access権はない。
- `OwnedRegion<A>[lo,hi)`: 各byte位置への排他的な物理アクセス権と現在のslot状態。
- `Slot = Known(u8) | Unknown`: Unknownは「初期化を証明していない」。値0でも、
  必ず未初期化という意味でもない。
- `RefTicket<S,id>`: Shared allocation Sに対する未返却登録を表す。
- `PendingHandle`: descriptor複製中の内部状態。完成済みBytesMut invariantを要求しない。
- `Finalizer<S>`: 最後の登録を除去した操作が得る一意なprotocol権限。
  これだけでは他threadのmemory resourceを使えない。

Aはbyte buffer、SはShared control blockであり、別のallocationである。
両方のlayout、所有権、破棄を個別に扱う。AのRecoveryでSを解放しない。

### 領域algebra

候補は `Resource<FMap<Int, Excl<Slot>>>`。mapのdomainを `[lo,hi)` とし、
同じkeyに二つのExclが重なる合成を不正にする。既存のFMap/Exclの法則から、
intervalのrestriction、disjoint union、split/join、内容保存を証明する。

ただし、公開された任意の `Resource::alloc` はghost台帳しか作らない。
実allocationとresource idを結ぶsealedな対応を、実Vecを消費するbridgeだけが作る。
任意のdescriptorと新しいresourceを組み合わせてもOwnedRegionを作れない構造にする。

OwnedRegionのconstructor、内部Resource、任意の `ExclUpdate` を外へ公開しない。
ghostの値更新だけで「実byteを書き換えた」ことにしてはならない。公開split/joinは
内容を保存する操作だけに限定し、内容変更は実writeまたは終了したmutable borrowに対応する。

物理アクセス権は **non-objective** として扱う。objectiveなResource mapを包んだだけで
threadを越えた観測権限を得ないよう、`NotObjective` と既存の `AtView` 規律を維持する。
必要なSend/Syncの成立条件もここで監査する。

### 借用

`borrow_slice` は同じallocation/provenance、region内のbounds、全要素Knownを要求する。
`borrow_slice_mut` はregionの排他的借用に結び付いた `&mut [u8]` を返す。
借用終了時の内容をregionへ戻すprophetic postconditionが必要で、返すreferenceを
regionの借用より長生きさせない。借用中は同じregionのsplit/join/transfer/recoveryを禁止する。

`borrow_uninit_slice` は `&mut [MaybeUninit<u8>]` を介し、終了時の各slotを反映する。
Unknownをread可能なu8とみなさない。可能なら既存Permの**借用**へ接続してbridgeを減らすが、
interior regionから所有する `Box<Perm>` を作らない。

## 3. runtime表現: RawBufferを推奨

現状は `Shared { vec: Vec<u8>, original_capacity_repr, ref_count }`。
実Vecを置いたまま、ghost側だけで通常APIを禁止しても、既存のVec契約と自動field Dropが残る。
`ManuallyDrop<Vec<u8>>` もdestructorを抑止するだけで、Vecの通常APIや論理モデルを封じない。

推奨候補は、全build共通のprivate型 `RawBuffer { base, capacity, ... }` へ置き換えること。
必要なlength情報は純粋なdescriptor情報として扱い、常に初期化済みであるという意味を持たせない。
RawBufferはCopy/Clone、Deref<Target=Vec<_>>、通常のVecを返すaccessorを持たず、
自動でallocationを解放するDropも持たない。所有者のverified release pathだけが解放する。
`ManuallyDrop<Vec>` はdetach時の一時的な実装手段としてのみ使う。

これは内部表現の実変更であり、上流sourceと同一だとは報告しない。公開API、byte内容、
capacity/reuseの振舞い、panic/abort、native Orderingの保存を確認する。Sharedのalignmentと
pointer tag条件、既存のunsafe Send/Syncも再監査する。Vecの未規定なfield配置には依存しない。

この変更はpromotion/dropだけでなく、現在の次のVec利用箇所も通す必要がある。

- `reserve_inner` のunique reclaim、copy、set_len、reserve。
- `From<BytesMut> for Vec<u8>` のallocation引き取り。
- `shared_v_to_vec` と `shared_v_to_mut` のdescriptor利用と所有権引き取り。
- `release_shared` とKIND_VEC側Dropの実deallocation。

これらをproof buildから削除して完了扱いにしない。初回end-to-endの範囲外のbodyは
未証明と明示し、表現変更による通常buildへの影響はテストとsource reviewで確認する。

特に、`split_to(len)` で得た左側をclearし、spareへ `MaybeUninit::uninit()` を書いてdropすると、
残った右側から見たallocation prefixはUnknownになり得る。現在のunique reserve経路は
`set_len(offset + len)` 後にVec::reserveを呼ぶため、そのままB2を挿すには初期化証拠が足りない。
ここで確認したのは通常のVec→Seqモデルへの復元条件が成立しないことまでであり、
このsource調査だけで上流runtimeのUBを判定したものではない。

この経路の将来の証明には、Unknownを読むことなく保存できるraw realloc/copy、または
Knownなvisible部分を先頭へ移すことを証明した別のreclaim処理が必要。前者を第一候補とし、
必要ならgenericなB5 raw reallocation契約を追加する。全allocation資源を消費し、実allocatorの
reallocation、更新されたprovenance/capacity、既存slotの保存、新規spareのUnknownを規定する。
古いpointer/権限を再利用できないことと失敗時の所有権も必要である。B5は初回split/dropの
TCBへ先取りせず、reserveを対象にするとき別途監査する。通常Vecのreserveがspare内容を
保存するという未規定の保証を仮定しない。

実変更が想定以上に広い場合の代案は、private opaque wrapperにVecを完全に封じる案。
その場合、Vecの保持・露出・自動Dropを含むrepresentation interpretation自体が追加TCBとなる。
代案では内部Vec.lenを0に保つことで古い初期化prefixへの依存を減らせるが、allocation ownershipの
分離、API封じ込め、実Drop接続は依然として監査が必要。take/recover時は空ownerへの置換等で
二重破棄を避ける。単なるcfg aliasや「使わない」という規約では代用しない。
RawBuffer案を第一候補とし、両案を同時に実装しない。

## 4. 最小の追加trusted境界

既存のCreusot/solver/Rust/allocator/Resource algebraのTCBに加える候補を明示する。
「Vecの表現bridgeだけ」という呼び方では、physical accessとdeallocationの仮定を隠してしまう。

| ID | 入出力・要求 | trustedに含めないもの |
|---|---|---|
| B1 detach | 実Vec所有権を消費し、実base/capacity、Recovery、全capacity regionを得る。旧lenまでKnownで旧内容に一致、spareはUnknownへ弱めてよい | packed metadata、BytesMut構築、promotion |
| B2 resume | 同一allocationのRecoveryと完全なregionを消費し、要求した新lenまでKnown、正しいlayout/base/allocator、未終了borrowなしで通常Vecへ戻す | regionの回収・join、unique判定、内容移動 |
| B3 deallocate | 完全なallocation権限を消費し、対応する実native deallocationを行う。正しいbase/layout/allocator、borrow/readersなし。初期化済みprefixは不要 | 最後の所有者判定、全regionが戻ったという定理 |
| B4 physical access | regionと実pointer/reference、初期化、借用終了後の内容を結ぶ | split/join、ghostだけのwrite、BytesMutのbounds計算 |
| A1 native atomic operation | 必要なら実fetch_subのordering-parametric契約と正確なword演算 | ticket保存、last-owner uniqueness |
| A2 RMW message forwarding | 必要なら実RMWのrelease-sequence伝播を表すgeneric規則 | 最後のdecrementで全権限が現れるというbytes専用公理 |

B1/B2/B4は分離論理的なphysical interpretation、frame、provenance、初期化、借用寿命と
weak-memory viewの整合性を説明するsoundness noteを伴う。negative probeが通るだけでは
これらのprimitive自体のsoundness証明にはならない。

B3は実deallocation命令、または意味を監査した標準destructorへの接続が必要。
ghost resourceを捨てるだけではFreedとは扱わない。buffer Aのpartial initializationを
無視して `Box<[u8]>` に変換することは禁止する。既存free_boxed_slice証明は、条件の合う
完全初期化済みBoxに限って再利用し、raw storageにはlayout等の補題を再利用する。

Shared allocation Sには既存のtyped Perm/Boxを第一候補にする。ただし並行するatomic参照、
immutable descriptor参照と最終的なBox回収の両立は別の判定条件である。通常の
`&mut Shared` を全thread共存中に作ることは許さない。Sのfield accessやdeallocationに
追加primitiveが必要なら、一般的な物理メモリ境界として別途記録する。`release_shared`全体を
trusted wrapperにして隠さない。

### Sの共有と回収に使う既存API候補

Sについては `FullBorrow<Box<Perm<*const Shared>>>` を `GhostShared` に置き、
各handleに分割した `LifetimeToken` を持たせる構成を先に試す。scopedな
`FullBorrow::borrow` でimmutable descriptorとatomic objectへの参照を得る。
最後に全fractionを回収してlifetimeを終了し、保持していた `EndBorrow::get` から
元のfull typed permissionを取り戻す。`GhostShared` 自体から中身を取り出せるとは仮定しない。

これは未検証の候補である。atomic fieldへの実アクセス、非atomic fieldの不変性、
両drop順、fractionが一つ残る場合の回収拒否、AtViewの同期をG0で確認する。
Aのownedな可変領域をこの共有借用で代用することはしない。

この構成はSの非atomic fieldを凍結する。unique reserveやdescriptor置換では、
全S lifetime fractionを集めて現在の借用を終了してから変更し、Sが存続するなら新しい
synthetic lifetimeを設定する。`rc == 1` のloadだけで `&mut Shared` を得ることはできない。
実atomic履歴と登録・fraction回収を接続するこの判定条件を、reserve対応前に確認する。

Sの解放にB3を使うなら、一般的なtyped-allocation primitiveとしてfull owned Perm、
正しいallocator/layout、未終了borrowなしを要求する。さらに `needs_drop::<T>() == false`
か、別途証明したfield破棄済み状態が必要。RawBuffer descriptor、metadata、atomicだけの
Sharedでは前者を第一候補にするが、論理的なfield資源の処理も省略しない。
「Sharedと中のbufferをまとめてfreeする」というbytes専用trusted primitiveは作らない。

## 5. BytesMut invariantと操作

KIND_VECのhandleは、allocation A、offset o、visible `[o,C)`、退避prefix `[0,o)`、
Recoveryを保持する。`cap = C-o`、`len <= cap`、visible先頭lenはKnownで、logical contentsは
その値列に等しい。実pointerはprovenanceを保った `base+o` であり、packed offsetとoも一致する。
数値アドレスが等しいだけでは十分ではない。

KIND_ARCのhandleは、Sへのticket、visible interval、stashを持つ。Shared protocolには
RawBuffer/Recovery、未返却登録、retired pool、control blockの権限がある。全allocationは
handle/pending/retiredの責任へ分割される。verified操作では実regionを重複なく移動させる。
忘れられたaffine資源まで常に取り戻せるとは主張せず、未返却の責任を登録へ残す。
finalizeは実際に保持・回収した資源による完全coverageを別途要求する。

| 操作 | 領域と内容 |
|---|---|
| split_to(at), at<=len | return `[o,o+at)`, len=cap=at。self `[o+at,o+c)`, len=l-at, cap=c-at |
| split_off(at), at<=cap | self `[o,o+at)`, len=min(l,at), cap=at。return `[o+at,o+c)`, len=max(l-at,0), cap=c-at |
| advance(n) | visible前方n bytesをstashへ移す。初期化・値を保存し、allocation coverageを失わない |
| truncate/clear | logical lenのみ縮める。直ちにKnownをUnknownへ変更しない |
| spare capacity borrow | 実MaybeUninit更新に合わせてslot状態を変更する。以前Knownだった位置もUnknownになり得る |

split_toの内部でadvanceを使う場合、前方regionを「selfのstash」と「returnのvisible」に
二重に割り当てない。split専用の資源移動、またはstashからreturnへの明示的transferを行う。
既存stashの割り振り規則を固定し、通常は元handleへ残す。

古いShared.vec.lenに対応した位置が後でUnknownになる可能性があるため、破棄時に
その昔のlenでVecをresumeしない。通常Vecへ戻す場合も、新しい要求lenまでのKnownを証明する。
例えばclear後にspareを再未初期化したbufferの破棄は、byte readなしのB3で扱える。

## 6. promotionとPendingHandle

実経路は `from_vec -> shallow_clone -> promote_to_shared(2) -> ptr::read(self)`。
KIND_VECのまま二つの所有handleを作るモデルは使わない。

1. unique handleのinvariantを開き、descriptorと全資源を得る。
2. RawBuffer/RecoveryをSharedへ移し、実AtomicUsize(2)と二つのpending登録を同時に対応させる。
3. コピーするのはruntime descriptorだけ。regionとticketは複製しない。
4. regionをsplitし、実ptr/len/cap/dataを更新する。
5. 二つの登録をvalidなhandleへ対応させ、両方のinvariantを閉じる。

proof用ghost fieldをBytesMutへ付けた場合、`ptr::read(self)` にghost fieldまでコピーする契約を
与えない。推奨する小変更は、private shallow_cloneをdescriptor/PendingHandleを返す処理へ
分け、split callerで完成したBytesMutを組み立てること。完成済みinvariantを持つBytesMutを
一時的に二つ作ってから「後で直す」方式は採らない。

bounds panicはpending状態へ入る前に処理する。pending中の実命令にpanic/unwindがないかを
調べ、あるならreservation/regionを戻すcleanupを証明する。allocator failure、abortと
通常returnも区別する。pendingのghost値を忘れてcleanup済みとはしない。

## 7. 参照数・資源保存・forget

Shared protocolは未返却登録集合 `Registered(S)` を持つ。登録にはlive/pending等の状態と
region責任を対応させる。実RMWのlinearization時点でcountをこの集合の大きさに接続する。
単なるloadが返した値を「今あるRust変数の数」と同一視しない。

- incrementでは、既存の有効な登録を前提に新しい登録を作り、実RMWと対応させる。
- decrementでは、ticketとそのhandleが担うvisible/stashを提出して登録を除去する。
- `mem::forget(handle)` やghost tokenの破棄は登録を除去しない。リークはあり得るが、
  それを使って最後の所有者になったことにはできない。
- empty regionにも独立したticketが必要。regionが空という理由でcountを減らせない。
- 整数overflow/abort条件は実 `increment_shared` から証明し、無限精度countをそのまま
  usizeへ置き換えない。

集合が空になったという純粋な事実だけでは、物理regionは得られない。
retired poolが保持する**実資源**と完全coverageを回収時に消費する。
最初のclosed harnessで両方のdropを実行した場合の完全回収と、一般利用でforgetを許す
memory safetyを区別する。任意の利用者に対するno-leakは主張しない。

## 8. native weak memoryと最後のdrop

releaseはvisible/stashを、そのthreadのviewに結び付いたretired resourceとして提出する。
非最後の `fetch_sub(1, Release)` は登録を除去して戻る。最後のRMWはFinalizerを一つ得るが、
他threadのnon-objectiveな資源はまだ `AtView` の内側に置く。
実装どおりの `load(Acquire)` の後に、必要なviewが現在のSyncView以下であることを証明し、
そこで初めて全region/Recovery/control block権限を取り出す。

0.13の `committer.rs` ではRelaxed loadから前段のAcquireSyncViewを得られるが、それを
現在のthreadが取得したことにはならない。Release shoot_storeは現在のSyncViewをpublishし、
前段のrelease messageを継承するpostconditionを明示していない。
したがってA1だけで十分とは見積もらない。

最初に3参加者のprobeを作る。異なるthreadが資源を返却し、Release RMW、途中のRelaxed RMWも
含むrelease-sequence、最後のAcquireを通じて両方の資源が回収できることを確認する。
必要なA2は、実RMWがmodification orderの直前のwriteを読むことと、そのrelease messageを
次へ渡すことを扱う。Release RMWでは自身のpublicationも含める。途中のRelaxed readerへ
資源の使用権を与えず、plain storeへ無条件で同じ伝播規則を適用しない。

最後のAcquireが自分のzeroへのRMWより前の値へ戻らないこと、zero後に新規登録できないことも
必要。弱いmemory orderをSeqCstへ変えたり、sc-drfを有効にしたりして通さない。

単一thread harnessは自threadのviewで先に進められる可能性がある。しかしその成功を
並行last-owner回収の証明へ一般化しない。

## 9. Dropと二つのallocationの破棄

`mem::drop` の既存契約はResolveであり、deallocationのpostconditionではない。
次の三点を別々に証明・記録する。

1. actual release helperの本体: 非最後の分岐と、Acquire後の全region回収。
2. 最後の分岐から実buffer Aのdeallocation、続くcontrol block Sのdeallocationまで。
3. `Drop for BytesMut` と自動drop glueが、このhelperを正しい資源で一度呼ぶこと。

RawBufferには自動deallocationを持たせず、Aの解放を明示する。SのBox回収/破棄では、
残るfield dropの効果も確認する。drop呼出しのVCやghost tokenの消費だけで
A/Sの実解放を報告しない。通常dropからの接続をツールが表現できない場合は、
helperの証明成功と自動Drop未接続を分け、第一段階の完了条件を未達とする。

## 10. zero capacity・空領域

capacity=0のVecには実buffer allocationがない場合がある。NonNullなdangling pointerを
実allocationが存在する証拠にしない。descriptorを `Empty` と `Allocated` に分け、Emptyには
bufferをfreeしない規則を持たせる。論理的なinstance identityは持てても実allocationとは区別する。

空intervalの台帳はunitで、物理byte権限の複製禁止を空集合にそのまま要求する意味はない。
RecoveryとRefTicketは別に一意性を持つ。Aが空でもShared allocation Sは存在し得る。
`at=0`、`at=cap`、one-past pointer、alignment、length-zero sliceの要件を個別に扱う。

## 11. 実装順と合格条件

| 段階 | 実装・証明 | 合格条件／止める条件 |
|---|---|---|
| G0 feasibility | region借用write-back、non-objective転送、Sの共有参照と回収、PendingHandle/type invariant、Drop接続、vtable翻訳の小probe | 実現できない点をbytes専用のtrusted定理で埋めない。必要なtool変更を具体化して設計へ戻す |
| G1 memory kernel | B1–B4のsoundness note、sealed algebra、実Vec len<capからowned split・独立mutate・join・復元/解放 | helperから二つのowned regionを返せる。borrow-only probeで代用しない。期待反例を拒否 |
| G2 unique handle | actual from_vec/as_slice_mut/truncate/clear、KIND_VECに留まるadvance分岐と破棄 | 実fieldとresourceの関係、値と初期化、offset/base回収を証明。大offsetによるShared promotionは後続段階 |
| G3 shared sequential | A1のnative fetch_sub契約を用意し、RawBufferを使う実promotion、pending split_to、二つのmutation、両drop順 | A/Sの実破棄へ接続。count=2専用の偽Sharedモデルで代用しない |
| G4 boundary cases | split_off(at>len)、0/cap、zero capacity、Unknown化、忘れられたhandle、3以上の登録 | 領域/登録保存、リーク時の早期free禁止、初期化を必要としない破棄 |
| G5 native concurrency | A1/A2の小probe、実increment/release、cross-thread mutationとlast-owner回収 | 実Orderingのまま同期と資源回収。A2の不足はここまで先送りせずG0で先行調査する |
| G6 integration | 実crate entry、関連bodyすべて、自動Drop、通常APIとsource対応 | 全体translation成功と対象実経路のbody proof。隔離helperだけの成功では未達 |

現在のvtable/clone循環はG0/G6の独立した翻訳障害で、RawBuffer案では自動的には解消しない。
proof-only focused targetを調査に使っても、実crateの義務を消したことにはしない。
実装を広げる前に、必要な翻訳対応が許容される小変更に収まるか判断する。

`advance_unchecked` はpacked offset上限を超えると `promote_to_shared(1)` も呼ぶ。
二handleを作るpromotion(2)の証明をこの分岐へ無条件に流用せず、1登録の初期化と
region/stash移動を別に接続する。同じSharedの連続した二viewをまとめるunsplitは、
split完了後にregion joinとticket返却を使って扱う。freezeは同じcontrol blockをBytes側へ
渡すので、その公開前にread-sharing invariantへの切替が必要となる。

G1からG3を最初の実装単位とし、G0の致命的な不足が判明したら広いAPIへの注釈追加を止める。
初回対象は初期化済み範囲内のmutationを使い、`reserve`やfreezeに依存させない。
その後に境界値と並行性へ進む。freezeではwrite-exclusive領域から別のread-sharing protocolへ
変換する必要があり、OwnedRegionの複製では扱わない。

## 12. 正例・反例と証拠

正例は実allocationを使い、(a) owned splitをhelperから返す、(b) 独立mutation後の値を確認、
(c) joinして新しい値でVec復元、(d) spareがUnknownのままraw破棄、(e) advance後のbase回収、
(f) actual BytesMut両drop順、(g) 3参加者のRelease/Acquire回収、を段階的に証明する。

| 反例群 | 拒否するもの |
|---|---|
| physical binding | 任意Resourceのalloc、整数metadata、同じaddress/異なるprovenanceからのpermission生成 |
| separation | overlap、非空region複製、transfer後の使用、別allocationのjoin、ghostだけの値書換え |
| borrowing/init | 借用のescape、借用中のrecover、Unknown read、mutation前のstale値でのresume |
| recovery | 半分だけの回収、interior free、誤layout/allocator、Recovery二重消費、空bufferの不正dealloc |
| encapsulation | suspended storageへの通常Vec API、ptr::readでのghost ownership複製 |
| protocol | ticket二重消費、登録が残るfinalize、regionが欠けるfinalize、forgetを回収扱いする操作 |
| weak memory | Acquire省略、不十分なSyncView、Relaxed readerによる早期資源使用、plain store経由の不正伝播 |

禁止APIは型/visibility拒否、誤った主張は意図したVC拒否として記録する。
コンパイルエラーをnegative VC成功と数えない。各証拠にsource/config/hash、trusted契約一覧、
translated/body-proved/caller-connected/automatic-Drop-connectedを記録する。

## 13. 配置と完了の定義

候補配置（まだ作らない）:

- `src/raw_buffer.rs`: 全build共通のprivate runtime descriptorと実detach/resume/free操作。
- `src/ownership_proof/raw_vec.rs`: B1–B4のcfg(creusot)契約、physical interpretation。
- `src/ownership_proof/owned_region.rs`: sealed領域wrapperと証明するsplit/join。
- `src/ownership_proof/shared_protocol.rs`: 登録、retired pool、view付き資源とFinalizer。
- `verification/probes/...`: 各gateとnegative controls。

既存 `src/verification.rs` は古いモデルなので再利用せず、実型に接続した新moduleと区別する。
変更はこのbytes crate内から始め、一般的なatomic規則だけ必要に応じてcreusot-stdへ提案する。

第一段階の成功は、実from_vec、promotion、split、mutable access、release_shared、
DropからA/S破棄までがresource modelへ接続されてbody-provedであること。
通常テスト成功、数値helperの証明、algebraだけの証明、TCBを仮定した孤立したcallerの成功は
個別の進捗として記録する。現在はすべてこの設計の未実装部分である。

## 14. 調査した根拠

- bytes checkpointの `src/bytes_mut.rs`: Shared、from_vec、shallow_clone、promotion、
  reserve_inner、From<BytesMut> for Vec、release_shared、shared_v_to_vec/shared_v_to_mut。
- vanilla Creusot 0.13 source `318615be3b8bbc60d1f6d52469ba5c0bdebed4f1` の
  `creusot-std/src/ghost/resource.rs`、`logic/ra/fmap.rs`、`logic/ra/excl.rs`:
  ownedなghost Resource splitと、物理Permのborrowed splitの違い。
- 同 `std/sync/committer.rs` と `std/sync/atomic.rs`: ordering-parametric操作、
  現在明示されているpublication契約とfetch_subの不足。
- 同 `std/sync/view.rs`、`ghost/perm.rs`、`ghost.rs`: AtView/SyncView、non-objective資源。
- 同 `ghost/lifetime_logic.rs`、`ghost/shared.rs`: FullBorrow/EndBorrowとfractional lifetimeの候補。
- 同 `std/mem.rs`: drop/forgetのResolve契約。実deallocationとは別に接続が必要。

Astraによる独立レビューで、physical interpretationの封じ込め、non-objectivity、
忘れられた登録の扱い、RMW message forwardingを優先的な設計条件として確認した。
