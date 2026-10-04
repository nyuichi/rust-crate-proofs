# Adopted runtime ownership verification direction

The user explicitly adopted the following direction on 2026-10-04 (Asia/Tokyo).
It supersedes the earlier helper-only frontier for this task. Proof status must
remain distinct from translation/source connection/ordinary tests.

次の方針でそのまま進めてください。

1. pointer tagging / provenance
================================

pointer taggingについては、新しい実装を考えなくてよいです。

既にあるMiri用のprovenance-friendly branchを`cfg(creusot)`でも使ってください。

概念的には、

    #[cfg(any(miri, creusot))]
    provenance-friendly implementation

    #[cfg(not(any(miri, creusot)))]
    existing optimized implementation

とします。

対象は特に、

    ptr_map
    invalid_ptr
    without_provenance
    KIND_ARC / KIND_VEC / KIND_MASK
    pointer型fieldに詰めているmetadata

です。

区別すべきものは明確に分けてください。

- allocation由来のtagged pointer
  - 元pointerのprovenanceを保持する
  - address bitだけ操作する

- pointer型fieldに入れている整数metadata
  - dereference可能なpointerとして扱わない
  - provenance-lessな値として扱う

Miri用branchで使っている操作をCreusotが扱えない場合のみ、最小限のcreusot-std contractを追加してください。

そのcontractから、

    arbitrary address manipulation
      =>
    dereferenceable pointer

のような強すぎる結論が導けないことをnegative probeで確認してください。


2. recursive Buf / BufMut blocker
=================================

Creusotがrejectしているrecursive trait構造を解消してください。

普通のruntime buildのAPIは変えないでください。

`cfg(creusot)`でのみ、再帰性を生むconvenience methodsをtrait本体から分離して構いません。

候補は例えば、

Buf:

    take
    chain
    reader

BufMut:

    writer
    chain_mut

です。

free function、extension trait、wrapper等に移してください。

重要なのは、translationを通しただけで終わらないことです。

移動した結果body proof可能になったものは、その場で仕様を付けて実際に証明してください。

例えば、

`take`:

    remaining == min(original_remaining, limit)

    advanceによってlimitを超えて消費しない

`chain`:

    logical contents
      ==
    first.remaining_bytes ++ second.remaining_bytes

    firstを消費し終わるまでsecondを消費しない

`reader` / `writer`:

    underlying Buf / BufMut operationとadapterのobservable behaviorが一致

まで、現在のstd modelで表現可能な範囲を証明してください。

既にあるisolated helper proofは必ずruntime callerへ接続してください。


3. PartialOrd / DeepModel ICE
=============================

`Bytes: PartialOrd<T>`周辺のDeepModel normalization ICEは、まず最小再現を特定してください。

rustc / Creusot内部を大規模に直す必要がありそうなら、そこには時間を使わないでください。

`cfg(creusot)`だけgeneric implを具体化して構いません。

必要なら、

    Bytes <-> Bytes
    Bytes <-> BytesMut
    Bytes <-> [u8]
    Bytes <-> &[u8]
    Bytes <-> str
    Bytes <-> &str

およびBytesMut側を個別impl / proof adapterにしてください。

通常buildではupstream generic implをそのまま残してください。

具体化してtranslation可能になったら、その場でbody proofまで進めてください。

証明目標は、

    equality iff logical byte sequences are equal

    ordering == lexicographic byte ordering

です。

既存のcomparison helper proofをruntime callerへ接続してください。


4. real runtime modulesへ移行
=============================

1〜3を解消しながら、`cfg(creusot)`をproof-only replacementから実runtime実装へ移してください。

最終的には、

    actual Bytes
    actual BytesMut
    actual Buf
    actual BufMut
    actual supporting modules

を検証対象にしてください。

既存`verification.rs`の論理modelやlemmaはspecificationとして再利用して構いません。

ただしruntime typeの代替実装として使った結果をruntime coverageには数えないでください。


5. 開通したruntime codeはそのまま証明する
=========================================

1〜4の作業で新しくtranslation可能になったbodyについては、先送りせず順次証明してください。

特に現在isolated helper proofが既にある以下を優先してください。

    slice / cursor operations
    Take
    Limit
    Chain
    Reader
    Writer
    fixed-width reads/writes
    variable-width reads
    comparison impls
    BytesMut packed-capacity helpers
    initialized storage helpers

目標は、

    helper proved

から

    actual caller body proved
    +
    integrated runtime path proved

へ進めることです。


6. BytesMut ownership modelを一気に設計してよい
===============================================

structural blockerをある程度除いたら、actual `BytesMut`に対するownership representation invariantを設計してください。

ここは小さいhelperを増やし続けるのではなく、まとまったモデルを一度作って構いません。

既に実`Box<[u8]>`について、

- permissionを非重複regionへ分割
- regionを独立に変更
- 再回収
- explicit deallocation

まで証明できているので、それを基礎として使ってください。

最初の対象はVec-backedなunique `BytesMut`です。

概念的には、各`BytesMut` handleについて、

    backing allocation A
    allocation base
    current offset
    len
    cap

と、

    readable interval  = [ptr, ptr + len)
    writable interval  = [ptr, ptr + cap)

を関連付けてください。

最低限、

    len <= cap

    writable interval is inside the backing allocation

    readable interval is initialized

    logical byte contents
      ==
    bytes in readable interval

    this BytesMut owns unique write permission
    for its writable interval

をrepresentation invariantとして持たせる方向を検討してください。

実際のCreusot `Perm` / `PtrLive` modelに合わせて形は調整して構いません。


7. 最初のend-to-end ownership target
====================================

最初にこの一本をactual runtime codeで通してください。

    Vec<u8>
      ->
    BytesMut
      ->
    split_to または split_off
      ->
    両方のnon-overlapping regionを変更
      ->
    byte contentsを確認
      ->
    両handleをdrop

このpathで、

- actual pointer fields
- len / cap / offset
- allocation
- region permissions
- split
- independent mutation
- drop / resource recovery

を接続してください。

`split_to(at)`なら概念的には、

    permission [0, cap)

を

    permission [0, at)
      *
    permission [at, cap)

へ分割し、それぞれのactual `BytesMut` handleが対応するpermissionを保持することを証明してください。

既存のBox-region permission proofを再利用してください。


8. unique BytesMut APIをできるだけ一気に広げる
===============================================

representation invariantが通ったら、そのinvariantを使ってunique Vec-backed pathのAPIをできるだけまとめて証明してください。

候補:

    len
    capacity
    as_ref / deref
    as_mut
    truncate
    clear
    advance
    split
    split_to
    split_off
    reserve
    resize
    zeroed
    conversion to/from Vec

各APIについてbyte-content semanticsまで可能な限り証明してください。

単なるlength/capacity transitionだけで完了扱いにしないでください。


9. freeze -> Bytesへ接続
========================

unique `BytesMut`が安定したら、

    BytesMut::freeze

を次のtargetにしてください。

freeze後は、

- logical byte contentsが保存される
- mutable permissionは消費される
- `Bytes`側のread-only ownership/resourceへ移行する
- 同じmemoryを二重にmutable accessできない

ことを表現してください。

ここで初めて`Bytes`側のrepresentation invariantを導入して構いません。


10. shared Bytes / refcount model
=================================

次にrefcounted shared storageを扱ってください。

ここでは、単に

    refcount == integer

を証明するだけでは不足です。

少なくとも、

    allocation is alive while logical references exist

    each live shared handle corresponds to ownership of one logical reference

    cloning adds one reference

    dropping removes one reference

    only the last reference may recover the allocation/deallocation resource

を結びつけるモデルが必要です。

必要であればghost token / invariantを導入してください。

actual runtime Orderingは維持してください。

    fetch_add(Relaxed)
    fetch_sub(Release)
    load(Acquire)

等を勝手にSeqCstへ変更しないでください。


11. Vec-backed -> Shared promotion
==================================

refcount modelができた後に、

    Vec-backed
      ->
    Shared-backed

promotionを証明してください。

まずsingle-threaded成功pathを証明して構いません。

その後、

    concurrent shallow_clone_vec
    compare_exchange
    winner / loser

へ進んでください。

証明したい内容は、

- 競合してもallocationがlostしない
- 二重freeしない
- 最終的に全handleが同じShared storageを参照する
- 正しいrefcountになる
- losing threadが作った一時Shared allocationを正しく処理する
- byte contentsが保存される

です。


12. Drop / last-owner deallocation
==================================

最終的に、

    clone
    split
    freeze
    promotion
    release
    Drop

を通じてresource invariantが保存されることを証明してください。

最後のreferenceだけが、

    backing allocation permission
      ->
    deallocation

へ進めることを示してください。

既に証明されている`free_boxed_slice`やexplicit deallocation helperを必ず再利用してください。


進め方
======

1〜3だけ終えて停止する必要はありません。

translation可能になったruntime bodyは随時証明し、そのままownership modelまで進めてください。

ただし、同じ形のownership VCで何度も止まる場合はassertionを追加し続けず、representation invariant自体を見直してください。

特に、

    actual handle
      <->
    allocation / permission / logical byte contents

の対応が不足している状態でhelper lemmaを増やさないでください。


停止条件
========

以下のどれかに達するまで進めてください。

A.
unique Vec-backed BytesMutについて、
split / mutation / dropを含む主要APIがactual runtime bodyで証明された。

B.
freezeまで接続され、Bytesのread-only representationまで証明された。

C.
shared/refcount pathまで進み、clone/drop/last-owner resource recoveryが証明された。

D.
Creusotまたはcreusot-stdに新しい根本的modelが必要で、soundness設計を先に決めないと進めない具体的blockerに到達した。

Dの場合のみ停止して、以下を報告してください。

    exact failing runtime path
    exact missing logical resource/model
    current Perm/PtrLive modelで表せない理由
    最小の必要拡張案
    その拡張がsoundであるために必要な条件
    positive / negative probe案


報告時
======

以下を明確に区別してください。

    translated
    body proved
    helper only proved
    caller connected
    trusted
    integrated crate proof

VC数やhelper数をcompletion percentageには換算しないでください。

普通のtests/no_std checks/negative controlsも継続して実行してください。

`bytes`全体が実runtime codeで証明されるまでは「fully verified」とは報告しないでください。