# bytes 1.11.1：これまでのAPI・検証実験の棚卸し

対象コード：`bytes-runtime-verification`、`6ecb66fb4eea9ebe443d074938a73d0d8054b601`。2026-10-07時点。

**元のbytes crateを完全検証するという目標に対して、現在の成果には「元APIの限定された本体の証明」「再利用候補の補題」「追加した別APIの証明」「未完了の実験」が混在している。別APIの証明を元APIの完了件数として数えてはいけない。** この棚卸しではコード・証拠を削除せず、trustedの追加もしていない。

今回のユーザー指示を判断基準とする。元の実装をなるべく保ち、未解決の性質やCreusotの不足を局所的な一時的契約に隔離することは許容される。その契約の解除時に少ない手数で済み、他の部分を大幅に作り直さなくて済むことが許容条件である。Clone/Drop等では、現時点でその条件を満たす見通しが未確認。以前の「改変APIを最終目標にする」解釈は、この目標を満たさない。既存の決定記録や反例は履歴として残す。

## 全項目を確認するための一覧

| 一覧 | 内容 |
|---|---|
| [API一件ずつの一覧](api-inventory-2026-10-07/MODIFIED_API_AND_ORIGINAL_COUNTERPARTS.md) | 現在の追加実装の275個の`pub* fn`宣言、内部ownership支援の132宣言、trait実装、元`Bytes`・`BytesMut`・`Buf`・`BufMut`の各メソッドとの対応。整数幅・エンディアンの各overloadも個別に記載。公開API以外の論理関数・内部型のメソッドも区別して含む。 |
| [元APIと部品検証の詳細](api-inventory-2026-10-07/ORIGINAL_COMPONENTS.md) | 元APIに近い本体を抽出した各gate、制限、表現変更、メタデータだけのモデル、trusted境界。 |
| [未統合API・失敗した実験の一覧](api-inventory-2026-10-07/EXPERIMENTS.md) | コピー系、参照比較、initialized window、sealed trait、native float、Hash・Debug・Serde・unwind・spawn・termination等。成功・失敗・native確認のみを区別。 |
| [一時trusted境界の検討](api-inventory-2026-10-07/TEMPORARY_BOUNDARIES.md) | Astraによる元実装を保つ方向の評価。解除に必要な条件と、まだ少ない変更で解除できるとは言えない箇所。 |
| [全ソース関数宣言CSV](api-inventory-2026-10-07/source-function-signatures.csv) | 現在の`src`内の名前付き関数宣言1,505件のパス・行・signature。内部関数、モデル、テストを含む索引であり、API数・証明済み数ではない。マクロ展開後の全項目を数えるものでもない。 |
| [過去の部品証明symbol一覧](api-inventory-2026-10-07/historical-component-proof-symbols.csv) | 21部品gateの194 proof artifactごとのsymbol、証拠パス、hash。同じ依存関数が重複するため194 APIではない。 |
| [明示的trusted宣言27件](api-inventory-2026-10-07/explicit-trusted-functions.json) | 現在のcrateソース内の`#[trusted]`の正確な位置。標準ライブラリ契約や検証器自体のTCBは別途必要であり、この27件だけが全TCBではない。 |

詳細付録は識別子を正確に保持するため英語で記載した。以下が日本語での分類・評価である。分類は廃棄の決定ではない。

## 1. 元のbytesに存在したAPIの本体に近い成果

「元に存在した」は名前・責務の由来を表す。元の全入力・全所有状態を検証したという意味ではない。

| 対象API | 元に存在 | これまでの証明範囲 | 元crateへ向かう位置づけ |
|---|---|---|---|
| `BytesMut::new`, `zeroed`, `with_capacity`, `From<&[u8]>`、内部`from_vec` | あり | 抽出したコンストラクタ本体、容量契約、限定された所有状態、明示的解放。 | 元APIの本体検証として再接続する候補。普通の自動Dropまでは含まない。 |
| `BytesMut::{len,is_empty,capacity}` | あり | initialized unique / canonical empty状態のobserver本体。 | 条件付き本体証明。全状態に拡張するには統一した所有不変条件が必要。 |
| `BytesMut::{spare_capacity_mut,AsRef,AsMut}` | あり | 限定状態の借用と現在・終了時のview。 | 借用契約は有用。DerefMutや未初期化領域の公開まで完了したわけではない。 |
| `BytesMut::{split,split_to,split_off}`、内部advance | あり | coordinator/ticketを持つ抽出構成でsplitと明示的cleanup。 | 元の共有領域に近い本体。ただし現在の私有表現変更と証明用cfgへの対応が必要。 |
| `BytesMut::{truncate,clear,set_len,resize,extend_from_slice}` | あり | in-capacity更新、初期化・frame、限定されたunique/shared growth。各メソッドの個別の証拠範囲は詳細付録。 | 元本体候補。すべて同一gateで証明したわけではない。 |
| `BytesMut::{reserve,try_reclaim}` | あり | 限定されたunique/shared/登録状態でのgrow・reclaim。 | allocator、共有領域、実際のatomics、cleanupへの接続が残る。 |
| `BytesMut::{freeze,unsplit}` | あり | 限定freeze/read/recovery、隣接・非隣接・異なるcontrol blockなどの個別gate。 | 元の分岐本体の成果。永続的`Bytes`のCloneと通常Dropは未接続。 |
| `BytesMut`のrelease関数・解放本体 | あり | 明示的な消費呼び出しによるcleanup。 | 解放本体の材料。Rustが挿入する自動Dropの証明ではない。 |
| `Bytes::{len,is_empty}` | あり | メタデータ本体。 | 所有protocolの証明ではない。 |
| `Bytes::{truncate,clear}` | あり | **nonpromotable**なtable状態の本体とcaller。 | promotable分岐、vtable全種類、一般所有状態が残る。 |
| `impl Buf for &[u8]` | あり | `remaining/chunk`、19 checked integer readerと19通常getterを含む具体的slice gate。 | 実在する具体実装の成果。任意の`B: Buf`やfloatには一般化しない。 |
| `BufMut`の`&mut [u8]` / `&mut [MaybeUninit<u8>]`実装 | あり | 選択した`remaining_mut/chunk_mut/advance_mut`。 | unsafe初期化義務とopen traitの実装義務が残る。 |
| `Take/Limit/Chain/Reader/Writer` | あり | 選択したconstructor、projection、`into_inner`等のmetadata。 | generic read/writeや全合成の証明は未完了。 |
| `UninitSlice` | あり | coreのwrite/copy/len/accessorと6種類のrange indexing。`new/uninit/uninit_ref`はtrusted。 | typed castとraw constructorの権限・effect契約を精査する必要がある。 |
| `Vec<u8>`と`BytesMut`の相互比較 | あり | 選択した比較本体とcaller。逆方向`PartialOrd`の実装不具合を修正した。 | 元APIに直接関係する修正・条件付き証明。一般`Bytes`・Stringの比較は含まない。 |

注意：現在の通常実装でも`BytesMut::Shared`の`Vec`を`SharedBuffer { base, capacity }`に置き換えている。さらに証明用cfgではnative `AtomicUsize`を`SequentialCounter`に置き換える。「現在の本体を抽出して証明した」と「上流の元本体を証明した」は異なる。順序付きnative atomicsの並行動作の証明は、sequential counterの証明からは得られない。

## 2. 新設した別APIの成果

**下記の型・API自体は元bytesにない。** 対応する責務があっても、元APIを実装したことにはならない。

| 追加した型・API群 | 元の対応物 | 差と評価 |
|---|---|---|
| `ExclusiveBytes`：生成、長さ、容量、読書き、extend、resize、truncate、clear、reserve等 | 主に`BytesMut` | 普通のVecを一つ所有する別実装。原本の共有所有protocolを経由しない。codecや借用の補題は再利用候補、本体の証明は原本の完了に数えない。 |
| `try_split_to_copy`, `try_split_off_copy`, `try_copy_slice`, `copy_clone`, `append_owner` | split/slice/Clone/unsplit | O(n)のコピーや別allocationを使う。元の共有・O(1)操作と意味・費用が違う。 |
| `ExclusiveBytes::try_reclaim` | `BytesMut::try_reclaim` | 名前は同じだが、現在のVecに十分なspareがあるかを返すだけ。共有領域を回収する元の操作ではない。 |
| `copy_from_slice`, `copy_from_str`, `from_string`, `from_boxed_slice`, `from_iter`, `append_str`, `append_iter`, `default` | 元の各constructor/conversion/collection trait | Vec-backedの生成・収集。元のtrait surface、共有状態、owner転送を全て再現しない。 |
| `ExclusiveBytes`のDeref/AsRef/AsMut/Borrow/Eq/OrdとVec/slice比較 | 元のBytes/BytesMutのtraits | 普通のVecに対する本体証明。元のhandle/vtable経由の借用・比較の本体とは別。 |
| `Cursor`：integer/endian/可変幅reader、advance、copy、prefix | `Buf`・具体slice/cursor | 借用sliceに閉じた別のconcrete API。checked戻り値なども異なる。整数の数学的部分は移植候補。 |
| `LimitedCursor`, `ChainedCursor`とaccessor、scoped reader | `Take/Chain`等 | 具体Cursorのcomposition。open `Buf`や任意実装を扱わない。 |
| integer/endian writer、`LimitedWriter`, `ChainedWriter`とaccessor | `BufMut`、Limit/Chain/Writer | initialized Vecへのappendを扱う。元のuninitialized spareへの書き込みとは別。 |
| concrete `Read/Write/BufRead`、gather/scatter helpers | generic Reader/Writer・vectored APIs | concrete slice/ownerに閉じたI/O。元のgeneric `Buf`/`BufMut` interfaceではない。 |
| sealed `ReadableOwner` / `copy_from_owner` | `Bytes::from_owner<T>` | 4種類だけを認め、元ownerを返しつつ別ownerへコピーする。元の一般ownerをそのまま所有するAPIではない。 |
| `eq_str_copy/cmp_str_copy/eq_string_copy/cmp_string_copy` | str/String比較traits | 名前付きhelper。余分なO(n)コピー・解放がある。元の演算子trait実装ではない。 |
| `OwnedByteCursor`とfinite caller | IntoIterator・Buf iter | concrete有限cursorと明示的close。Iterator/IntoIteratorを実装した証明ではない。 |
| `Float32Bits/Float64Bits`とreader/writer | native f32/f64 APIs | bit wordのまま扱う。実際のfloat変換は未証明。 |
| `stable_digest`, `hex_bytes` | Hash・Debug | 新しいpolynomial digestとhex allocation。元のHasher/Formatter protocolの証明ではない。 |
| scoped share/thaw/callback/range/tree、アドレス回復、spawn-slot wrappers | 共有Bytes・Clone・並行readの責務 | scopeに閉じた登録と明示的retirement。資源補題は候補だが、永続handle・`Clone(&self)`・automatic Dropには未接続。 |
| `close`・`retire`・ticket/coordinator/resource操作 | 元の内部cleanupの責務 | 新しい証明用・明示的操作。内部資源の証明材料にはなるが、元の自動呼び出しとの対応証明が必要。 |

各メソッドとtrait overloadを省略せず列挙したものが[個別API一覧](api-inventory-2026-10-07/MODIFIED_API_AND_ORIGINAL_COUNTERPARTS.md)。ここは分類のための群別表であり、全件数の代わりではない。

## 3. まだ統合していない追加API・失敗した実験

| 対象 | 元に存在したか | 保存状態 |
|---|---|---|
| Vec/slice参照の比較forwarder | 元には対応する比較trait群あり。追加型での実装は別 | private candidateで334 proof filesのpositive。現在HEAD未統合。元の全方向・String・array比較matrixの証明ではない。 |
| `ExclusiveBytes::initialized_window` | この名前はなし。mutable slice accessの責務はあり | private candidate positive。元のspare capacityではなく、既に初期化済み領域を借用する。 |
| `InitializedWriteWindow::{new,commit_prefix,discard,Deref,DerefMut}` | なし | private candidateで358 files positive、native68件。別scratch allocationをzero-fillしてコピーする。原本のzero-copy spare accessとは別。 |
| sealed `VerifiedRead/VerifiedSink`、generic helper | なし。元はopen Buf/BufMut | 4関数のproof filesに未証明VCが残り、さらにprophetic-call frontend block。native2件はformal成功ではない。 |
| 広範囲のString/str DeepModel変更 | 元runtime APIではなく証明モデルの変更 | 翻訳/契約問題。名前付きcopy helperの成功で元trait完了にしない。 |
| native floatのfrom_bits/to_bits | 元のfloat APIに必要 | frontend failure / caller VC未証明。bits wrapperで代用しても元APIは残る。 |
| generic Hasher・Formatter | 元にHashとDebugあり | callable契約不足で未証明。元Bytes/BytesMutにDebugがなかったという記録は誤り。 |
| optional Serde Serialize/Deserialize | 元featureにあり | opaque callable/sequence accessが未証明。native成功のみでは埋まらない。 |
| catch_unwind、raw thread scope、spawn failure | 補助実験。元crateのpanic/concurrency責務に関係 | frontend crash・欠けた契約・callback destructorによる反例。bounded Copy-slot成功とは区別。 |
| arbitrary callback/iterator/allocator/OS callのtermination | 元の一般呼び出しの責務 | finite concrete callerのみ成功。一般totalityは未証明。 |
| consumer replay・最終matrix runner | consumer/検証準備 | API本体証明ではない。未実行の計画・native consumerをformal replayと数えない。 |

失敗履歴と証拠hash、成功した後続版は[実験一覧](api-inventory-2026-10-07/EXPERIMENTS.md)に記載した。反例、翻訳失敗、Why3未証明、nativeだけの確認を混同しない。

## 4. APIではないが、これまで証明した部品・モデル

21部品はhelpers、storage、deallocation、bounded-ops、slice-ops、cursor-ops、byte-codecs、comparison-ops、chain-ops、capacity-ops、slice-read-ops、slice-wide-read-ops、variable-read-ops、initialized-storage、uninit-ops、wide-codecs、endian-ops、signed-wide-ops、region-permissions、provenance-ops、ownership-frontier。

その194 artifactの関数symbolを[CSV](api-inventory-2026-10-07/historical-component-proof-symbols.csv)に全件記載した。算術・codec・borrowed regionの補題は移植候補だが、独立モデルの証明を足して元crateの全体証明にはできない。`OwnedRegion/KernelRA`の純粋ledgerはallocationのnative所有権をそれだけで与えない。`src/verification.rs`の同名`Bytes/BytesMut`もmetadata state-machineであり、元の型ではない。

## 5. 一時trustedとして残せるかという評価

今回新しいtrustedは追加していない。現在の明示的27宣言は、UninitSliceの3 cast、Box alignmentの1操作、raw Vecの13操作、sequential counterの4操作、Vec capacityの1操作、modified weak atomicの5操作。これにprivate stdのcapacity/base/pointer/convert/thread-slot等の契約、Creusotの資源モデル・基礎TCBがある。

| 未解決境界 | 元の本体を残す方向 | 解除までの見通し |
|---|---|---|
| raw allocation/pointer/slice/layout | 物理所有・初期化・frameを備えたgeneric primitive契約に隔離 | 契約が正しければ局所的な置換の候補。数値address一致だけで権限を作ってはいけない。 |
| vtable/static/function pointer | 実際のtableとcallbackの対応を明示したdispatch契約 | frontend対応だけでなく全representationのrefinementが必要。 |
| immutable Deref/AsRef | original invariantからの読取権限とviewを契約化 | 原本の全状態への権限対応が必要。 |
| mutable DerefMut/AsMut/chunk_mut | typed exclusive borrow、初期化、終了時writeback、frame | 実際のeffectが消えないことが必要。既知のghost mutation反例がある規則をtrustedにしてはいけない。 |
| open Buf/BufMut | 実装者に明示的な証明義務を課し、各実装をrefine | 任意の下流実装がbyte lawを満たすという無条件trustedは不可。 |
| Send/Sync | validator markerの受理と並行所有定理を別の義務として記録 | marker通過だけでthread safetyの証明にはならない。 |
| native refcount | 実際のRelaxed/Release/Acquireと資源不変条件を保ち、未証明protocolを名前付きの仮定に隔離 | 検討候補であり、局所的解除と契約の妥当性の条件を満たす設計は未確立。条件付きcaller証明が通っても原本protocolは未完了と表示する。既知の不健全なSC規則や架空Acquireは不可。 |
| `Clone(&self)` | actual handleの登録・promotion/CASを含む契約 | **少ない手数でtrustを外せるという見通しはまだ立証されていない。** 共有参照からの資源更新が難所。 |
| automatic Drop / unwind | 実際のdrop eventとcaller effectを結ぶgeneric bridge | **Drop本体をtrustedにするだけでは足りない。** MIR Dropが消える構成ではcallerの解放効果にならず、局所的解除の見通しも未確立。 |

原本を残す方針なら、元の`BytesMut`生成→実際の`freeze`→実際の`Bytes::clone`→thread read→最後の自動破棄の経路を、未解決契約を列挙した条件付きgateとして繋げることが次の設計判断材料になる。ただし今回の作業は棚卸しまでであり、実験は開始していない。

## 証拠の照合と注意点

- 最新379 gateのarchive hashは`54ffedeb6b763a600aad991d1fedfcce55a6ff3e86536241bd2e51e2f970f5b4`。保存されたCargo/src入力39件は現在のHEADと全て一致した。[照合記録](api-inventory-2026-10-07/current-379-source-parity.json)。379はComa/proof file数。元Bytes/BytesMut/Buf/BufMutをcfgで除外した構成なので、元API379件を検証したことにはならない。
- 過去21部品の直接入力33件を照合すると5件は現在のsourceと不一致。capacity-ops、endian-ops、provenance-ops、signed-wide-ops、variable-read-ops。[照合記録](api-inventory-2026-10-07/historical-component-source-parity.json)。保存revisionの証拠として残し、現在HEADのreplay済みとはしない。一致した28件も全依存・cfgの再検証を意味しない。
- legacy reference comparisonで「negative」と名付けたnative checkは実際にはexit 0だった。positive/negativeともコンパイル成功。元のAPIがそのexpressionを拒否したという根拠にしない。
- 0 Coma・0 goalの失敗やfrontend errorをWhy3の反例にしない。bare `-g`が0 goalを選んだhelper確認は成功証拠から撤回済み。
- 原本baseline：upstream VCS `417dccdeff249e0c011327de7d92e0d6fbe7cc43`、crate archive SHA `1e748733b7cbc798e1434b6ac524f0c1ff2ab456fe201501e6497c8417a4fc33`。`e678b774`のflattened sourceを原本APIの照合に使用。古い997行のsurface manifestはcompiler item数であり、検証済みAPI数ではない。

この文書は残す/破棄する判断のための一覧である。新設APIを削除する決定、原本表現を戻す決定、trustedを追加する決定は行っていない。
