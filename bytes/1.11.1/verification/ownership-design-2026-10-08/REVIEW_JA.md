# レビュー記録

対象: STRONG_SPEC_JA.md / baseline dfe759ee。新body証明・production変更なし。

## Astra architecture review

Astraは最終共有不変条件と既存61件の差、元Clone(&self)/token/weak-memory/Dropの
実装可能性を独立に検討し、draftとaudit.jsonをレビューした。

* 61件はphysical/content部分の証拠で、最終singleton共有predicateを含意しない。
* actual data/count/vtableを証明し、fullauthorityを消費して共有protocolへ入れる必要がある。
* 現sidecar全体や「数行の変更で共有まで」という再利用保証はできない。
* in-flight registrations、borrowed slices、B4のlifetimeとinvariant opening、二つのShared、
  static/Owned/promotionの区別はdraftで適切に扱われている。
* 追加指摘3件を反映: 同期をcaller仮定にしない、native count bounds/abort/underflow、
  static/detached-emptyではallocation/ticketがabsent。

結論: draftにmaterial architecture overclaimは見つからず、full shared architectureは未承認。
local materialization葉を除去できても原Clone/Drop/resource protocolの不足は解消しない。

## Luna xhigh source/evidence audit

既存Luna担当advancedが元Clone/slice/split/unsplit/reserve/freeze/Dropと現sidecarの
read-only監査を行った。個別field/spanと61件のexact-source evidenceを確認した。

* BytesMut Cloneはdeep copy、Bytes Cloneはvtable経由のshallow sharing。
* actual shallow_clone_arcはproof sidecar Noneを返す。現在のread前提はclone先に使えない。
* 現Bytes pointer relationはbase固定、advance/inc_start gateは0固定。nonzero view未証明。
* BytesMut promotionのptr::readはproof-only authorityを複製してよい根拠ではない。
* split系の歴史的coordinator/ticket証明は別cfg/model。局所lemma以外をdrop-in統合としない。
* Dropはgateから除外され、native destructorを通すtestsをformal effectsと数えない。

これらの具体的境界をspec/reuse tableへ反映した。新API/代替bufferは追加していない。

Lunaによるdraft追加レビューも反映: from_staticとwithout_provenance由来empty ZSTを
分離し、native/proof cfgのbranch限定とDrop省略、現sidecarのexact fields、
既に通ったsingleton readと不足するmulti-reader access bridgeを明記した。
