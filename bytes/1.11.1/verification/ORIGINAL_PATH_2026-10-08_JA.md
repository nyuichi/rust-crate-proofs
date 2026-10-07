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
