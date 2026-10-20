# bytes 1.11.1 — Aeneas 検証の途中成果

これは既存の Creusot 検証とは独立した、2026-10-20 時点の Aeneas / Lean 成果物です。既存のソース・契約・検査結果は変更していません。

コピー後の内容と消費量、Vec への追記、整数書き込み・符号拡張、比較、アダプタの状態更新などについて、補助定理を含む **272 件の定理**を検査しています。呼び出し先の正しい振る舞いを仮定した証明と、具体的な Lean モデルでの証明も含みます。**クレート全体の検証完了を意味しません。** `cargo elicit` の厳格な監査は 24 件中 22 件を採用し、2 件は opaque 型の公理により未解決です。

対象は公開版 bytes 1.11.1 に検証用の小変更を加えた `snapshot/bytes-adapted`、default/std、x86_64 Linux です。公開版との同値性、serde/no_std/他ターゲット、メモリ安全性や並行処理全体は証明していません。unsafe を含む既存の opaque 境界は trusted のままです。全体の Lean 翻訳には未解決の型エラーがあります。

- [証明・前提・未完部分の説明](snapshot/verification/README.md)
- [集計](snapshot/verification/results.json) と [関数ごとの台帳](snapshot/verification/coverage/ledger.json)
- [Rust の変更差分](snapshot/adapted-source.patch) と [出所](snapshot/source-provenance.json)
- [cargo elicit の結果](snapshot/verification/elicit-integration/README.md)
- [翻訳器の既存 issue 調査](snapshot/verification/backend/upstream-2026-10-20/README.md)
- [実験段階の型引数修正](snapshot/verification/backend/chain-binder-probe.md)。この修正候補は現在の検証ツールに未統合です。

## 保存内容の確認

Python 3.11 以降で、保存ファイル、ソース、検査記録のハッシュと件数をオフラインで確認できます。証明器は動かしません。

```sh
python3 check.py
```

`snapshot/` は元のファイルをバイト単位で保存しています。`snapshot-manifest.json` が保存したファイルの一覧です。ビルド出力、依存キャッシュ、ツールの実行ファイルや巨大なチェックアウトは含めていません。検査結果を記録した小さなログ、証明に対応する生成 Lean / LLBC、元の crates.io 配布ファイル、Rust ソース、ツールの修正差分とバージョンは含めています。

今回の保存時には、別ディレクトリで保存済み Lean の 272 定理を再検査し、成功しました。[再検査記録](validation/replay-result.json) と [保存時の確認結果](validation/publication-check.json) も残しています。

## 保存した Lean の再検査

Lean **4.35.0** と、記録された Aeneas コミットの Lean ライブラリ・依存関係が必要です。Aeneas のコミットは `e62dd83d133d9043e7d3628096838b7430f36565` です。ツールの全バージョンは [toolchain.json](snapshot/verification/toolchain.json)、Lean 依存の固定値は [lake-manifest.json](snapshot/verification/lake-manifest.json) にあります。

```sh
# lean と lake を 4.35.0 に設定した環境で実行
python3 replay.py --aeneas-home /path/to/pinned-aeneas
```

`replay.py` は新しい一時ディレクトリへコピーして、保存済みの生成 Lean と証明を検査します。`snapshot/` や既存の Creusot 成果物は書き換えません。出力先は `--output /path/to/new-directory` でも指定できます。Aeneas の `backends/lean/.lake/packages` と構築済みライブラリを利用するため、事前に固定された依存関係でそのライブラリを準備してください。依存関係の自動更新やツールの自動インストールは行いません。

これは **Rust からの新しい抽出ではありません**。新しい検査ログと receipt は出力先へ保存し、過去の成功記録と区別します。

## Rust から再抽出する場合

元の抽出手順・コマンド・生の入力は `snapshot/` に保存しています。元のスクリプトやログ内の `/workspace/bytes-aeneas-fresh` は当時の作業場所です。これらを保存先でそのまま実行すると元の場所へアクセスするので、通常の確認には上の `check.py` / `replay.py` を使ってください。

再抽出には、別の作業ディレクトリで固定コミットの Charon / Aeneas / cargo-elicit と Rust ツールチェーンを準備し、元のスクリプトの作業場所を移す必要があります。Aeneas に適用済みの差分は `snapshot/verification/backend/aeneas-prefix-collision.patch`、cargo-elicit の互換差分は `snapshot/verification/elicit-compat.patch` です。chain/chain_mut の実験的な修正候補は適用済みの差分に含めないでください。

## この追加の分離

追加先は `bytes/1.11.1/aeneas/2026-10-20/` のみです。公開先の既存 `bytes-runtime-verification` ブランチから開始し、既存ファイルの変更・削除・上書きを含めない形で保存しています。この資料の結果を、親ディレクトリにある Creusot 検証の結果へ合算しないでください。
