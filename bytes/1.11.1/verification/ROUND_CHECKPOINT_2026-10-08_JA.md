# bytes 1.11.1 — Shared共通化と公開Clone接続の前提検証

対象ブランチは `bytes-runtime-verification`。この記録は2026-10-08の増分であり、crate全体または公開APIの完全検証の宣言ではない。

## 今回保存・確認した結果

| 増分 | 検証結果 | 意味と限界 |
| --- | --- | --- |
| original Shared宣言の共通include | source leaf44ファイル/280 prover leaves、null0。native default1011、portable cleanup2、no-default library build | productionとsource leafが同じ3-field宣言を使う。別crateの型が同一になるわけではない。Bytes宣言・全callback本体の共通化は未完了 |
| safe registered dispatch | 4ファイル/9 leaves、null0、native1 test | Std FnExtで呼び出し。関数itemのreificationだけgeneric TCB。unsafe/公開vtableではない |
| unsafe dispatch + 外部権限forward | 6ファイル/12 leaves、null0、native1 test | 間違ったnative/spec組合せは登録条件で失敗。権限はcallback外でそのまま移動 |
| unsafe erased callback内のghost更新 | 6ファイル/19 leaves、null0、native1 test | 同じchecked shim内でExclUpdateしresource IDを保存。generic erasure correspondenceはTCB。atomic eventとの同期は未証明 |
| quota-free registration | 23ファイル/113 leaves、null0、native1 test | 同じ生存中sourceを共有借用して2回登録。source保存、新ticketの妥当性、sourceより大きなID、ticket間のID相違を本体証明。外部CloneQuotaなし |

証明ファイル数を公開APIの件数に換算してはいけない。bytes固有のregistration/refcount/last-owner/destructor法則をtrustedにして通した結果ではない。既存のphysical/native-model/invariant-operation境界、および新しいgeneric dispatch/erasure境界は明示的TCBのままである。

各probeのREADME、immutable archive、root-audit/root-final-audit/root-negative-auditに、実行結果と現在のsource対応を保存した。quota-free最終archiveは `c707fc4f2e807fbeb3b3ad1635977528b86838cb52f22fa31b82a0f637a26994`。171 archive membersと7つの現行入力をrootが確認した。negativeのsnapshotは最後のwrapper postcondition強化より前で、相違は `new_id != source_id` から `new_id > source_id` の1行だけである。

## 公開API経路をまだ完成扱いにできない理由

元の公開 `Bytes::clone` は5-field Vtableのunsafe pointerを呼ぶ。今回のgeneric erasure実験はそのためのcall規則を用意したが、実際のSHARED_VTABLE、data/control field、source ticketをまだ同じchecked shimへ接続していない。

既存C source leafは、retirement、AtView payload recovery、completion creditを持つbounded B Stateを使い、外部CloneQuotaを要求する。新しいregistration Stateは権限の残量から何度でも発行できるが、live prefixが全て存在する前提で、retirementやpublicationを持たない。このまま置き換えると回収の強い契約が失われる。新しいstateへ必要な回収則をaxiomとして追加してはいけない。

さらに、新しいregistrationのcount対応はmoduloである。native countが1であることからlive ticketが1個とは推論できない。Rust Arcの元ソースもpost-fetch_add overflow checkの窓を認めている。保存したtoy schedulerは、十分なpending incrementをabort checkの直前で止める抽象モデルでwrapが起こることを示す。これはnative OSでのuse-after-free実証ではない。既存モデルにはpending operationの正当な物理上限がない。

## 次に進める依存順序

1. 非wrappingなcountの根拠を確定する。任意のbytes-specific thread上限をtrustedにしない。小さなnative代替としてguarded Relaxed CAS/fetch_update案を保存した。成功した1回だけregistration callbackを実行し、失敗/再試行でticketを発行しない。元のpost-increment方式で無制限並行lastnessを推論する手法はD-ABに固定し、前提変更なしに再試行しない。
2. sparse live domain、単調next、pool+live fraction保存、実際のcount、retirement、AtView recoveryを1つのbody-checked stateに統合する。Relaxed RMWは先行release publicationを引き継ぐが、Acquireやcurrent-thread全viewのpublicationを与えない。最終Releaseと実際のAcquireを維持する。
3. 同じtyped Shared.ref_cnt fieldの単一native eventに、このtransitionを接続する。native RMWを呼んでからmodel adapterで別のRMWを行う二重incrementは不可。
4. actual constructor/vtable/public Clone/read/explicit cleanupを接続する。cleanup後に自動Dropが再度実行されない処理も必要。自動Drop、Send/Sync、他representation、他APIは別の未完了項目である。

guarded native案は未適用。hot pathがload/CAS loopになり、retry、個別starvation、overflow時のabort timingが変わる。`shallow_clone_arc`だけでなく`owned_clone`とBytesMut `increment_shared`を含む全increment経路がcount boundを保存する必要がある。詳細は `probes/refcount-overflow-review-2026-10-08/README.md`。

## 証拠と公開の注意

途中の正常系archive（SHA prefix `7ef78f00`）は同じ保存先への上書きで失われ、復元できない。どの完了判定にも使わない。上記current final archiveは別途独立監査済み。今後のsource変更には新しいcapture labelを使い、既存archiveを上書きしない。

CLI認証切れを受けた前回の公開では、接続済みGitHubで同一treeのcommitを作り、force:falseで更新してanonymous fetchで検査した。対応する元のローカルcommitはcheckpoint branchに保存した。その後の`4163ad6e`初回公開時には、自動承認レビューがquota-free最終evidence manifestのblob uploadを「private source/proof artifactsの公開先への明示的許可がない」として一度拒否した。この時点ではtree/commit/refを更新せず、19個のblobだけがrefに接続されないremote objectとして残った。

この拒否をユーザーへ説明した後、ユーザーは「これまでは普通にpushしてたよね？何が違うんだ… 特に違いないならpushして。んで、次何するかもまとめて」と明示し、同一の公開先へ検証済みproof/source incrementを公開するよう再指示した。その指示後に以前拒否されたmanifestを含む全blob uploadが受理され、`4163ad6e`と`2f8eae30`を順に公開した。両treeはローカルtreeと一致し、各branch更新はforce:falseと現在のexpected SHAで行い、最後にanonymous fetchでparent chainを検査した。公開HEADは `732887b5c4bb168d4610374a0153e535c014c302`。ローカルbranchも同一treeのremote commitへconditional update済みで、元のローカルcommitはcheckpoint branchに保持した。全local→remote対応と過去の拒否・解決経緯は `verification/CONNECTOR_PUBLISH_FINAL_2026-10-08.json` に記録した。

## 再開時

旧checkpoint、ARCHITECTURE_DECISIONSのY/Z/AA/AB、各probeのroot receiptを先に読む。source/archive SHAとgit状態を確認する。実行環境の絶対path、CLI認証、toolchain/configは再確認し、保存されたものをそのまま有効とみなさない。Why3は既存共有lock、1 prover/1024MiB、sc-drf無効、elevated executionで実行する。無変更の全crate DeepModel/comparison ICE、直接FnPtr、automatic Drop、missing-Acquire等の凍結済み反例を無条件に再試行しない。
