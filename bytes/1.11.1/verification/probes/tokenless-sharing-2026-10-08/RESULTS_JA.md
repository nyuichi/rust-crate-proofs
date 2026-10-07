# 元Clone(&self)の共有権限更新: 調査・実験結果

2026-10-08（Asia/Tokyo）。baseline 8bed373b、bytes 1.11.1のみ。

## 判定

**現在利用できる仕組みで、元Cloneの共有更新と最後の回収を両立する方法は未確立。**
今回positiveが示すのはtokenless immutable observationだけである。
それを原BytesのClone/refcount/Drop検証として採用しない。production representationも
既存61件のsidecarも変更していない。bytes-specific trusted契約は追加していない。

## 実際に試したコード

同じ実VecをB1で消費し、その実Recovery/PhysicalRegionをstock GhostSharedへ移した。
元のVec所有者は残らず、この実験ではbyte allocationを回収しない。

```rust
let (raw, _, caps) = raw_vec::detach_vec(input);
let (base, _) = raw.bound_ptr_at_zero();
let shared = GhostShared::new(caps);
let alias = ghost!(clone_shared_receiver(&*shared)); // &selfからCopy
let first = unsafe {
    raw_vec::borrow_bound(&base, len, ghost!(&shared.to_ref().1))
}.to_vec();
let second = unsafe {
    raw_vec::borrow_bound(&base, len, ghost!(&alias.to_ref().1))
}.to_vec();
```

postconditionは両方の実読み出し結果が入力の全byte列と等しいこと。
Why3 positive exit0、34 Coma/34 complete proof JSON、null0。
actual_two_readsとclone_shared_receiverのbodyも含む。34はhelper込みのfile countであり
原API完成件数ではない。source-gated original Cloneやconcurrent Send/Syncは含まない。
GhostSharedは消費したTを永久に回収不能にする既存generic TCBであり、
任意lifetimeのreadonly参照を許す。その性質のままread witnessを複製することはできる。

しかし、回収しようとすると拒否される。

```rust
let recovered = ghost!(*shared.to_ref());
raw_vec::deallocate_vec(raw, recovered);
```

実translationはE0507「cannot move out of a shared reference」。Coma0、prover未実行。
残ったlocal GhostSharedからの抽出の診断であり、「最後のclone」を証明した診断ではない。
GhostSharedはCopyでlastnessを追跡しない。`into_inner`もない。解放できるように
新trusted extractionを足すと、stock arbitrary-lifetime readonly借用と矛盾する。
実negativeをnativeで実行していない。

最初のtranslationは既存raw_vecのattribute macro展開にrecursion_limit不足で停止した。
他のgateと同じ512に合わせて解消。prover timeout/depthは変更していない。
この環境設定診断もpositive archiveに保存し、数学的な失敗とは数えない。

## Luna xhighによるstock API監査とAstra review

canonical private creusot-std 0.13.0の該当sourceをhash付きで保存した。

| 実API | できること | 完全な原Cloneへの不足 |
|---|---|---|
| GhostShared | &selfからimmutable witnessを複製 | 中身を所有して回収できずlastnessもない |
| Resource::core(&self) | persistent/idempotent coreだけを複製 | fresh affine登録を生成しない。ExclのcoreはNone。権限を伴うupdate/splitは&mutが必要 |
| PermCell | 実UnsafeCellのinterior update | 更新にはGhost<&mut Perm>が必要。cellのSyncはpermissionの共有更新を保証しない |
| PredCell | 固定predicateを保つCell update | 普通のCell-backed !Sync、ghost mutationではない。bytesのatomic/history/authority対応もない |
| NonAtomicInvariant::open_mut | Tokensなしでprotocol更新 | &mut invariantが必要でCloneの&selfに適合しない。非atomic invariant自体も!Sync |
| AtomicInvariant::open | &selfから共有protocol更新 | explicit Tokensを要求。原Clone signatureにその引数はなく、Clone内生成も禁止 |
| weak atomic Committer | 実eventとordering/historyに対するgeneric効果 | shoot_storeに&mut PermとSyncViewが必要。eventだけでprotocol authorityが出てこない |
| AtView/Objective | 既存payloadを正しいviewで運搬 | authorityを発行したりtokenlessにprotocolを開いたりしない |
| stock Rc/Arc specs | abstract pointeeを保持するClone | bytesのnative control/count/historyを表さない。Rcはcross-thread不可。ghost内Rc cloneも認められない |

「GhostSharedでAtomicInvariantを包む」もopenのTokens前提を消さない。
無関係なfresh Resource::allocでcloneごとのticketを作っても、共有control/countとの
対応を確立できない。unsafe markerでCell/permissionをSync扱いにする方法は採用しない。
D05の旧ticket.split_off through &selfは再実行していない。

## 小さなgeneric extension案の健全性検討

原atomic RMWが返すunique Committerの&mut借用を、namespace Tokensの代わりに使って
protocolを開けないかAstraと検討した。候補は概念的に以下のsignature。

```text
open_at(existing_invariant, &mut committer, FnGhost closure(&mut state))
```

この案は**既存AtomicInvariantへの単純な追加methodとしては拒否**した。
closureが同じinvariantと既存namespace Tokensをcaptureできるため、次の再入が可能。

```text
open_at(inv, committer, |outer_state| {
    inv.open(existing_tokens, |inner_state| {
        // 同じpayloadを二重に開く
    })
})
```

committerがexclusiveでも別のTokens経路はmaskされない。native atomicをghost closure内で
呼ばなくてもstock openはghostなので再入できる。またcallbackがcommitterへアクセスできなければ、
invariant内Permを使ってshoot_storeする接続もできない。これはAstraによるinterface adequacy reviewであり、
新しいtrusted opening lawを実装してproverを騙した実験ではない。

別の専用invariant型で、他のopening入口を一切持たず、各実eventにunforgeable opening capabilityを
結び付け、commit/permission更新まで含める設計は研究候補。ただしexisting methodのbody証明から
導けるwrapperではなく、新generic synchronization ruleになる。Reentrancy、同event二重使用、
weak-memory publication、missing-Acquire、premature recoveryのadequacyが必要。
数行のtrusted宣言では表現できても、後で局所的にtrustedを外せることは未実証なので採用しない。

## 固定する境界と再開条件

* stock immutable sharingは読み出しを支えるが、complete lifecycleの共有authority保存先に採用しない。
* original Cloneのinterface gateはBLOCKED/NOT ESTABLISHED。別buffer/APIへ変更して閉じない。
* 再開には原Clone signatureとSend/Syncを保ち、actual atomic eventへ結び付く健全なgeneric opening/update/recovery機構が必要。
* opening規則はalternate-entry再入・event二重使用・missing-Acquire・早過ぎる回収を拒否し、affine authority conservationを説明できなければならない。
* 新しいtool/generic ruleの裏付けが得られるまで、singleton-only authority改造、GhostSharedへの全投資、同じTokens失敗へのassertion追加は保留。

この結果は「original crateは数学的に検証不可能」を意味しない。現在のtool/改造scopeに
不足があることを具体的に固定する。次のAPI証明へ進めるarchitecture admissionは得ていない。

証拠: evidence/positive-34.tar.gz、evidence/recovery-rejected.tar.gz、audit.json。
archive全member hash、complete proof tree、現helper/production source一致を独立再監査した。
