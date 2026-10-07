# 元bytesの強い所有権仕様と既存61件の接続判定

対象はbytes 1.11.1、baseline `dfe759ee`。依頼されたstage 1（最終仕様設計）と
stage 2（既存証明との対応監査）の成果である。以下は数学的な契約設計であり、
Creusotに実装済みのpredicate、証明済みのprotocol、新しいtrusted公理ではない。
APIのbody証明を新規に増やしてはいない。元のpublic API・native representationを対象とする。

## 判定

**強い契約の目標は設計できるが、現ツール・現sidecarで共有まで通るarchitectureは未承認。**
既存61件は実memory・内容・constructor field correspondenceの再利用可能な部品。
ただし現在の`original_frozen_valid`から、単独所有であっても最終の共有不変条件は導けない。
実`data`、refcount、vtableとの関係がないためである。さらに現在は全PhysicalRegionと
排他的Box Permを各Bytes sidecarに置くため、これをそのままCloneに複製することはできない。
実shallow_clone_arcのproof cfgは`original_frozen: None`を設定するので、clone先に現read前提は成立しない。

従って「弱いコントラクトを後で強くする」のではなく、以下の強い仕様を目標に固定し、
既存predicateはその**部分射影**として保存する。射影から最終仕様を導く補完証明と、
authorityの保存場所の変更が必要。既存61件のproof treeがそのまま新仕様のproofになるとは言わない。
共有accessと元Cloneの健全なencodingが決まる前に、その配置変更へ投資しない。
既存source-gated Buf::advance/inc_startは0だけであり、off>0のsliceやadvanceの証明も未接続。

## 1. allocationとハンドルの意味

`a`は実allocationの論理identity、`e`はprimitive capabilityのepoch/namespace、
`p`はそのallocation由来のbase pointer、`C`は実capacity、`S:[0,C)→Unknown|Known(u8)`は実slot map。
identityは数値addressとは別物である。B2でVecを再構成しB1で再detachした場合はnamespaceが更新される。
その間の実pointer/内容/割当ての連続性をB1/B2 boundaryで対応付ける必要があり、
古いnamespaceを勝手に新しいnamespaceに読み替えたり、同addressを永続identityにしない。

`Handle(h,a,mode,o,l,w,t)`は実native fieldに対応するaffineな所有権主張。
`o`はbaseからのoffset、`l`は可視長、`w`はmutable handleのcapacity、`t`はfreshな登録ticket。
static/detached emptyケースでは`a`と`t`はabsentであり、heap handleと同じ登録を要求しない。
logical ticketの識別子だけからticket/permissionを作れない。Ghost Seqだけから実allocationを作れない。

共通条件は `0≤o`, `0≤l`, `o+l≤C`。mutableなら `l≤w`, `o+w≤C`。
非空viewのpointerは実baseから同allocation内のpointer arithmeticで導出される。
`contents(h)=S[o..o+l]`、可視slotは全てKnown。
数値address一致やOpaque.ptrの等式だけでprovenance・liveness・所有権を得ない。

空viewは別扱いが必要。元`new_empty_with_ptr`は元領域のpointerを保持していても
`without_provenance`で元allocationのprovenanceを切り、同addressのfake ZSTに対応する
静的empty representationとなり、元allocationの登録を持たない。
zero-length sliceの型有効性は必要だが、空であることから元allocationのlivenessは導かない。
逆にallocationに登録された空viewはticketを保持し、長さ0だからrefcount対象から除かない。

## 2. 最終representation predicate（必要な全ケース）

`ValidBytes`/`ValidBytesMut`は以下の実representationのdisjunctionであり、
未検証ケースを`None`だから有効と認めるpredicateではない。

| 実representation | 必要な強い関係 |
|---|---|
| from_static Bytes | lifetime付き実readonly slice、ptr/len、STATIC_VTABLEの実callback契約。解放ticketなし |
| new_empty_with_ptr Bytes | without_provenance由来のzero-length ZST pointer、len=0、data=null、STATIC_VTABLE。元allocationのlifetimeやticketなし |
| BytesMut KIND_VEC | 実RawAllocation/Recovery、full physical coverage、offset/len/cap、packed KIND・offset・capacity-classの正確な関係 |
| Bytes Shared（bytes.rs） | 実Shared.buf/cap/ref_cntとdata/vtable、登録ticket、read authority、唯一の回収権限 |
| BytesMut Shared（bytes_mut.rs）とそのfrozen Bytes | 実Shared.vec/ref_count/original_capacity_repr、data/vtable、互いに衝突しないmutable capacity領域とreadonly view |
| promotable even/odd Bytes | 実boxed allocationとpacked data、その後のCAS promotionの状態遷移。候補Sharedと実winnerの区別 |
| from_ownerのOwned<T> | 実owner/control blockと借用byte列、owner寿命、refcount、型固有callback/Drop。未検証AsRef実装に普遍的法則を与えない |

二つのShared型を同一型・同一destructorにまとめない。後者のVecの長さと実mutable viewの
初期化済み範囲は別で、隠れたprefix/spareのslot mapまで保持する。
`from_owner<T>`のユーザー実装の機能・destructor作用は、実装refinement/適切なgeneric interfaceを
検証できた範囲のみ保証する。元APIに勝手なgeneric boundを追加して完成扱いにしない。

## 3. shared protocolの強い不変条件

実control blockごとに、以下を一つのcompatible resource invariantで保存することを目標にする。

* 実control pointer・buf/cap又はVec、actual atomic locations、各handle dataとvtableの対応。
* 全physical allocation authorityの保存。Recovery、control Boxの回収権限は一つだけ。
* 発行済みのaffine登録ticketと、そのhandle/view又は実行途中の所有者との対応。
* readonly/mutable accessの合法性、未初期化slot、retired range、進行中の借用の保存。
* actual refcountのmodification-order上の値と、**counted registrations**のcardinalityの一致。
* zeroへの遷移が一度だけfinalizerを生成し、回収又はVec等への所有権移動が重複しないこと。
* native usize refcountのbounds、increment後のoverflow-abort判定と実abort branch、
  decrementのunderflow排除。単純な数学Intの増減だけでnative wrappingを無視しない。
  元コードはincrement後にold値を検査するので、全中間状態に適用できない閾値上限を仮定しない。

`count == 今Rust変数として見えているBytesの個数`は採用しない。
cloneのfetch_add後で出力handleの組立て前、promotion初期count=2、変換やDropの途中には
reserved/in-flight registrationsがある。mem::forgetされたhandleのticketも自動的に消えない。
borrowed sliceは新ticketではなく既存handleの寿命に従う。実access権限はticketの存在だけでなく
borrow discipline・readonly sharing・mutable領域の排他関係に依存する。

active mutable capacity windowは他のmutable windowとdisjointであり、他handleからアクセス可能な
readonly rangeとも衝突しない。範囲外領域のframeを保存する。immutable view同士は重複してよい。
単純なPhysicalRegion.splitだけでは重複immutable ownershipを表せない。
BytesMutのshallow_clone/promotionはnative `ptr::read(self)`を使うため、proof sidecarの
affine authorityを同時に二つへ複製しないconsuming transferの対応も必要。
readonly権限の共有と最後の回収を許す健全なgeneric encodingが必要であり、
「global invariantに入れる」という文章だけでその権限を作ったことにはならない。
既存B4は`&PhysicalRegion`借用の寿命にread sliceを結ぶので、invariantを閉じた後にも
借用を返せるか、原`&self` APIを満たすかを明示的に検査する必要がある。

弱いorderingは元コードのまま。Relaxed clone incrementは数の追加を行うが、
それ自体をbyte access permissionのAcquireとみなさない。clone元の既存access/livenessと
実publication edgeを使用する。Release decrementの数値結果だけで他threadのphysical payloadを
回収してはいけない。最後のAcquire load/fenceが、各先行利用/退役のRelease及びrelease sequenceと
必要なhappens-beforeを成立させることが必要。payload搬送はAtView等のsubjective資源、
同期できるmetadataはObjectiveとして区別する。数値SC lawの付け替えは禁止（D02）。

## 4. 最初から要求するAPI契約

以下の`*`は数学的なseparating resource compositionの記法で、Rust演算子ではない。
`Protocol(a)`は操作の前後に保存される共有不変条件、`Frame`は無関係なallocation/resource。
全契約はまずnormal-return安全性。panic/unwind/abort/allocator failureは別の実effect契約が必要。
単独所有というpreconditionを先行lemmaに置くことは、postconditionを弱くすることではない。
ただしlemmaを一般APIそのものの証明とは数えない。

| 対象 | 完成時に要求する保証 |
|---|---|
| with_capacity/from_vec | 実容量と全slot map、exact native encoding、live unique ownership。input Vec所有権を消費し内容を保持。無関係なFrame保持 |
| 容量内extend_from_slice | 可視列がold contents++input、lengthが正確に増加、新prefixが実Known、対象外slot/ticket/control状態をframe。growth branchは別証明 |
| freeze | 入力mutable handleを消費し、同内容のValidBytesを発行。選択branchの実allocation/offset対応、actual data/vtable/count、回収責任を保存。KIND_ARCでは既存ticketの移動であり単なるcount=1ではない |
| Bytes::clone(&self) | clone元の内容/view/accessを保持、別fresh登録の返り値も同内容/view。既存sharedならnative increment、promotableなら正しいCAS promotion、staticなら登録追加なし。authorityを複製せずFrame保持 |
| BytesMut::clone(&self) | 元を保持し**deep copy**で新しい実allocationの独立mutable ownershipと同内容を返す。Bytesのshallow Cloneと同契約にしない |
| slice/split/advance | rangeの正確な内容・offset・length。empty/move/clone各branchで実際の登録増減を証明。mutable splitはcapacity領域を排他的に分割。登録数を常に増やすとは仮定しない |
| non-final release | 当該ticketを一度だけ消費し、counted registrationsを一つ減らす。他ticketのread/write validityとcontrol/storage liveness保持。storage/controlを解放しない |
| final release | 最後のticket及び同期済み全authorityを消費。該当storage/controlを実layoutで各一度だけ解放。他allocationをframe。解放後のaccess/再回収ticketなし |
| unsplit/reserve/reclaim | move/adjacent same-control join/copy各armのexact byte列とcapacity/native encoding。reallocationなら実authorityを消費して新storageへ移す。old view/borrowsを残したまま回収しない |
| into_vec/into_mut | unique branchはbytesを移動/再配置し所有権をtransfer、storageを解放しない。copy branchは新allocation+旧ticket退役。controlのみの解放とstorage解放を区別 |
| 自動Drop | 呼出元で元Drop/callbackのticket消費・必要な解放effectsを反映。Drop body単独の証明を自動Drop効果の証明と数えない |

この仕様はunconditional eventual deallocationを要求しない。mem::forget、abort、無限実行は
元APIで可能。正常に所有者を全てreleaseし、所有権transfer/外部owner delegationもなく、
有限lifecycleに対してexactly-onceを要求する。必要な同期はcallerに追加仮定させず、
元のatomic操作・orderingからprotocol/body内で導く証明義務である。
raw allocationのdeallocationはgeneric primitive TCBに置けても、
「refcountが1だからこの権限が全部戻る」はbytes body/protocolの証明対象。

## 5. singleton特殊ケースと既存61件の差

目標`SingletonShared(h)`はsection 3のshared invariantで、counted登録集合が
当該live handleの1 ticketだけ、進行中操作/借用がなく、actual count=1、actual dataが実Sharedを指し、
正しいvtable、full physical/recovery/control ownershipが回収可能な状態。

現在のsidecarは`BoundPtr`、Ghost `(Recovery, PhysicalRegion)`、Shared pointer、
Ghost `Box<Perm<*const Shared>>`を保持する。`original_frozen_valid(h)`は次の部分に限られる。
実ptr/len、Known-prefix、full Recovery/PhysicalRegionのnamespace/capacity、
実Shared Box ward/raw pointer、Shared.buf/cap。これは独立した強いphysical/content witnessであり、
`SingletonShared`の不足をtrue-only atomicsで埋めたことにはならない。

必要な接続は、元constructor bodyでatomic/data/vtableのrelationを新たに証明し、
**既存のauthorityを消費して**実controlに結び付いたprotocolへ移し、fresh handle ticket/read accessを
発行するrefinement theorem。singletonから必要な部分射影を借り出すには、
そのprotocolの健全なopening/access/closing規則が必要。global mutable ghostやtrusted registration
生成で代用しない。現ordinary Cloneではこのopening方式が未確定。

## 6. 再利用判定

| 既存材料 | 再利用できる保証 | 必要な変更/追加、再利用の限界 |
|---|---|---|
| B1/B2/B3/B4 generic contractsとmetadata helpers | 実Vecとの接続、Known slots、borrow、回収primitiveの局所契約 | 最終storage authorityの保存/共有先が未確定。generic TCBは証明済みprotocolではない |
| unique constructor/append bodies、typed-copy reference、bitwise lemmas | 実値・初期化・frame、KIND_VEC/off0判定 | exact capacity-classは現contractに不足。authority accessor変更ならcaller再証明必要。bit lemmasとcopy leafは独立再利用可能 |
| freeze B2→B1、Shared.buf/cap/Box ward | 元bodyのphysical continuityとfield correspondence | descriptor/authorityをprotocolへhandoffする部分を追加。data/count/vtable事後条件を追加して再証明 |
| current as_sliceとactual first-byte caller | full-region witnessによる具体的B4 read値と全byte列のinput一致は既に証明済み | 不足はordinary Clone後のmulti-reader/lifetime-aware shared access bridge。元のslice/byte論理は使えるがgateは再実行必要 |
| original_frozen sidecar | singleton physical witness | final shared representationとして不適合。各handleにfull region/Box Permを持たせない配置へ変更が必要 |
| historical ticket/retirement/thread components | affine登録、部分範囲、Objective metadata運搬の個別補題 | ordinary Bytes::clone(&self)、元ordering、元control/data、arbitrary handle lifecycleは未接続。丸ごと採用しない |
| 61 positive / retarget negative / unique51 regression | baseline exact source/configに対する証拠 | 新sourceのproof treeには流用不可。保持したlemma/interfaceの再証明で使う。negativeは将来のbridgeにも必要 |

結論: **契約の意味と局所補題には再利用価値があるが、今のsidecarを少し足せば共有まで無改造、
という見通しは成立しない。** authority保存場所とaccess boundaryは必ず変更対象。
その変更が小さく済むという保証はまだ得られていない。

61件のextractorはBytesMut/BytesのDropを省略し、live Bytesを返す。現Dropはnative kind/vtableで
dispatchするだけで、sidecarのRecovery/PhysicalRegionを明示consumeする証明はない。
proof cfgのFrom<Vec>はlen<capを要求しlen==cap boxed armを到達不能にし、freezeはKIND_VECのみ選択。
native cfgはboxed/KIND_ARCを扱い、ManuallyDrop/rebuild/元atomic/vtableを保持する。
B1/B2解釈とsource correspondenceを監査していても、native testsだけでproof/native全bodyの
意味等価性又はdestructor effectsを証明したことにはならない。

## 7. admission blockersと固定した次の関門

| 境界 | 現在の証拠 | 再開に必要な具体的前提 |
|---|---|---|
| 元Clone(&self)でshared authorityを更新 | D05、shared-receiver-fraction.logのE0596 | 原signatureとSend/Syncを保つ健全なshared ghost/interior invariant access。global mutable state/偽のticket生成不可 |
| tokenなし元APIでinvariantを開く | active pinned invariantsはexplicit Tokens、生成はmain内一度 | 安全なimplicit token/effect管理又は十分な別generic resource interface。APIへcontext追加は今回採用しない |
| readonly共有とfinal authority回収 | full PhysicalRegionとB4借用は既存61で有効、重複registered readerは別 | lifetime付きreadonly sharingとmutable disjointnessを保つchecked encoding。分割だけ・数字のfractionだけ不可 |
| weak atomicsにauthorityを接続 | D02のRelease-only negative | adequacyを説明できるgeneric weak-memory resource rule。必要なpublication/missing-Acquire negativesも拒否 |
| vtable identity/callback dispatch | D05、static materialization source diagnostics | 実static/callbackを契約へ結ぶtool support。true-only getterはidentityを与えない |
| 自動Dropの呼出元effects | pinned terminatorでDrop→Goto、livenessもdestructor effectsを無視 | effect-preserving generic destructor verification support。explicit cleanupはユーザー許可済みだが自動Drop完成と数えない |

このstage 1–2では上記の失敗を再実行していない。新前提なしにassertionやtimeoutを変えて
試し直さない。active pinned tool sourceとの一致も`audit.json`で確認した。

次のstage 3へ進む前に、元Clone/readonly sharingのgeneric interface admissionが必要。
その候補が現制約内で示せなければ、全shared protocolに使えないsingleton-only authority改造は
追加しない。atomic constructorの初期値契約だけは局所的に追加可能だが、
それをshared architectureの承認と取り違えない。
自動Dropは現在の前提で固定blockedであり、約束した「生成から自動解放まで一本通る」を
そのまま進められるとは言わない。大きなCreusot改造は今回のscope外。

## 証拠と維持

`audit.py`はpositive61、retarget negative（62 trees中null 1）、unique regression51の
archive全member hash、現production source一致を検査する。proverは起動しない。
既存counterexample/tool snapshotのhashとactive pinned tool factを記録する。
`audit.json`はbody proof countの追加ではなく監査receipt。production codeはこの増分で変更しない。
