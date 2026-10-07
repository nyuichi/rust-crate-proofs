# 汎用trusted境界による共有更新の再開

対象はbytes 1.11.1。2026-10-08（Asia/Tokyo）のユーザー指示に従い、
不足する汎用同期・資源操作を明示TCBとして置き、その利用側を証明する。
今後の調査手順はAGENTS.mdとGENERIC_TRUSTED_WORKFLOW.mdに保存した。

## 何をtrustedに置いたか

`probes/trusted-atomic-event-2026-10-08/src/event.rs`の新しい汎用境界。

* RawAtomicの生成: 実core::sync::atomic::AtomicUsizeと、対応するmodel/Perm/historyを結ぶ。
* EventAtomicへのbind: actual atomicと一致するprotocolを一度消費し、唯一の隠れたaffine状態Sとして保存する。
* 実Relaxed increment: 同じatomicのmodification-order上のeventで、その同じSを一時的に開く。
  FnGhost callbackはmatching Committerを完了し、public identityとProtocolを復元する。
  復元したSが次のeventへ引き継がれる。毎回別のauthorityを生成する意味ではない。
* Send/Sync marker: stock弱メモリinvariantと同じく、共有時にはSend+Objectiveを要求する。

このpersistent-state/event対応は**新しいgeneric trusted仮定**である。
opaque型やPhantomDataのlayoutから証明した性質とは扱わない。
creusot-stdのAtomicInvariantもpublic/existential mutable-state契約を用いており、
sharedなprivate_state値を観測するgetterを足す必要はない。そうしたpure getterは
干渉による変更を誤ってframeする危険がある。

native命令はself内のAtomicUsizeに対するfetch_add(1, Relaxed)。Mutexを追加せず、
RelaxedをSCに変えず、Acquireのvisibilityも与えない。MAX→0の実wrappingを契約に含む。

## 何をbodyで証明したか

利用側が管理するAuthority/Fragmentの登録処理、source登録の保存、新IDの発行と非重複、
実atomic historyと登録数modulo native幅の対応を証明する。
登録発行関数やrefcount法則そのものにtrustedは付けない。

private診断callerは次の形で、引数にtoken/contextを追加せず呼べる。

```rust
fn duplicate(&self) -> Self {
    let (_, ticket) = self.registry.clone_registration(self.ticket.borrow());
    Self { registry: self.registry, ticket }
}
```

これはbyte bufferの代替APIではなく、原Cloneに必要なshared-receiver資源更新の診断。
元Bytesのvtable呼出しや実Sharedへの接続はまだ行っていない。

## 以前の再入問題への対応

stock AtomicInvariantへtokenless openを追加する案は採用していない。
新しい型はstock invariantへの変換・ghost open・state getter・shared extractionを公開しない。
入口は実native atomic操作を行うordinary methodだけであり、そのFnGhost callbackから
もう一度ordinary methodを呼ぼうとするとghost purityで拒否される。
既存Tokensで同じ隠れたSを開く別経路もない。

stock Mutex例はtrusted guard/invariant操作の分担を参考にしたもので、
この新しい原子操作規則をstockだけから導いたという意味ではない。
比較した実sourceとhashはtrusted-analogue-sources-2026-10-08に保存した。
Astraが実装と契約を設計し、Lunaとrootがstockとの対応・persistent-state解釈をレビューした。

## 適用範囲

この増分はgeneric境界と登録callerの検証。原Bytes::clone全体、byte読出し権限の共有、
Release/Acquireによるphysical payload回収、native refcount overflow-abort、
最後の所有者の解放、vtable、自動Dropの証明ではない。
modulo登録数はgeneric加算の正確さを表すが、zeroから解放権限を与えない。

trusted除去・置換は、同じward/Protocol/callback/ordering契約を提供するStd/tool primitiveに
このmoduleを置き換える境界として管理する。今のcaller成功は、その新primitive自体の
soundness証明ではない。対応する検証器機能が未実装である点もTCBとして明記する。

各positive/negativeの正確なsource、feature、結果はprobe内README/監査receiptを正とする。

## 最終検証と証拠の対応

最終positiveは `positive-restored-8.tar.gz`。現src/lib.rs・src/event.rsと一致し、
8 Coma / 8 complete proof JSON / null 0、native逐次・並行テスト2件も成功した。
これはprobeの結果であり、元crate全体の再検証ではない。

negativeは、異なるatomicへのbind、異なるwardでのcommit、commit省略が各1 VCで失敗。
二重commitは1 VCで失敗、ticketの二重moveはE0382、ghost callbackからの
ordinary操作再入はpurity checkingで拒否された。これらはTCB自体の健全性証明ではない。

`negative-bind-ward-empty.tar.gz` のlib.rsは最終版の直前であり、
その後の変更はclone_registrationのold値と新ticket IDを結ぶpostconditionの追加と、
別feature下の二重commit負例の追加だけ。三つの対象negative関数とevent.rsは変更なし。
このarchiveを最終source完全一致と扱わない。残りの最終negativeとpositiveは現sourceと一致する。
履歴の失敗archiveも保存し、成功証拠と区別した。

[probe README](probes/trusted-atomic-event-2026-10-08/README.md)、
[agent監査](probes/trusted-atomic-event-2026-10-08/evidence/audit.json)、
[root独立監査](probes/trusted-atomic-event-2026-10-08/evidence/root-audit.json)に
実行条件、archive hash、member hash、proof treeとsource対応を記録した。
Std類例の保存source 8件もmanifest.sha256と一致した。
