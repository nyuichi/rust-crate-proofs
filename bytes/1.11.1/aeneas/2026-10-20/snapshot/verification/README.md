# bytes の機能検証

対象は `bytes 1.11.1` から作った `../bytes-adapted`、x86_64 Linux、既定の std 構成です。公開版からの変更は関数ポインタ引数のラッパーと二つのジェネリック境界で、差分は `../adapted-source.patch` にあります。公開版全体との意味的同値性は証明していません。serde 機能、no_std 構成、他のターゲットは今回の検証範囲に含めていません。

現在は Lean で補助定理を含む 272 件を検査し、450 個の透明なソース関数のうち 218 個に性質を対応付けています。これは各 API の仕様全体を証明した数ではありません。`cargo elicit` の厳格な監査では 24 件中 22 件を採用し、2 件は opaque 型の公理により未解決です。集計は [results.json](results.json) にあります。前回の全体検査（265 件）は [evidence/final-integrated-all.log](evidence/final-integrated-all.log)、今回の仕様強化に伴う 5 グループの再検査と全 receipt の整合性確認は [evidence/spec-strengthening-result.json](evidence/spec-strengthening-result.json) に記録しています。Rust と生成された関数本体は今回変更していません。

`Take` の残量は内側の残量と制限値の最小値になり、範囲内の advance は制限値を正確に減らします。`Chain` の advance は、境界の前後に応じて二つの内側の状態を更新します。これらの定理は内側の Buf 操作の成功や、明記した内容の更新則を前提にしています。整数の書き込みでは、put_slice が渡された列を追記するという前提の下で、変換されたバイト列と追記後の内容を証明します。

コピー処理は、残量・チャンクの先頭部分・advance が整合するという前提で、複数チャンクをまたいだコピー後の内容と消費量を示しています。比較では、取得できたバイト列について等値判定を特徴づけています。順序比較は標準モデルへの委譲を示すもので、そのモデルの辞書式順序そのものを証明した結果ではありません。

Vec の BufMut::put は、入力のチャンクと advance が整合し、最終サイズが isize::MAX 以下で、reserve が成功して既存内容を保存するという前提で、入力全体を既存のバイト列の末尾に追記することを示しています。追記自体は Aeneas の Vec モデルから証明しています。

今回の強化では、有限スライスから一定幅のチャンクを返す Lean の入力モデルを構成し、その remaining / chunk / advance の法則を証明して、上の Vec 追記定理へ適用しました。コピーにも 1 バイトずつ進む有限スライスのモデルを構成し、出力が元の先頭部分、残りが対応する末尾部分になることを入力が十分という条件だけで示しています。整数書き込みでは、容量内で実際にリストへ追記する Lean モデルに対して、翻訳された `put_u32_le` が正確な 4 バイトを追記することを示しました。これらは抽出された処理本体の具体的なモデルでの証明であり、モデルを特定の Rust Buf / BufMut 実装と同一視する主張ではありません。未使用の辞書メソッドにはエラーを返す定義を与えています。

符号拡張には 1〜8 バイトという入力条件があり、容量表現の復号は 0〜7 を対象にします。これらの具体的な数値処理は、バッファのメモリ表現に関する仮定を必要としません。長さや容量の getter は、翻訳された構造体のフィールドを返すことを示しています。

現在の opaque 境界は `opaque-baseline.json` に固定しています。unsafe、割り当て、参照カウントなど、その境界の内側の実装は証明していません。呼び出し側の条件付き定理は、opaque な呼び出し先の正当性まで示すものではありません。浮動小数点用の補助ファイルは F32/F64 の型だけを導入し、浮動小数点の意味は仮定した変換結果以上に証明していません。

クレート全体の生成 Lean は現在も型検査に失敗します。検証には API ごとの抽出を使い、生成された関数本体を編集せずに証明しています。関数ごとの結果は `coverage/ledger.json`、各定理の振る舞いと前提は `basic/manifest.json`、`adapters/manifest.json`、`extensions/*/manifest.json`、実際に成功した検査は各 receipt と audit.log を参照してください。台帳の body property はその定理の性質を示すもので、API 全体の仕様が網羅されたという意味ではありません。

再検査:

```sh
source /workspace/bytes-aeneas-fresh/env.sh
cd /workspace/bytes-aeneas-fresh/verification
bash verify-all.sh
# 抽出からやり直す場合
bash verify-all.sh --reextract
```

検証用ハーネス `harness/` と `borrowed_iter_harness/` は、通常の抽出先指定では選べない比較と借用イテレータの実装を呼び出すために使っています。bytes 本体のソースと opaque 設定は変えていません。Aeneas の名前衝突対策は `toolchain.json` と `backend/aeneas-prefix-collision.patch` に記録しています。

全体翻訳の `chain` / `chain_mut` の型引数については、暗黙引数を明示引数として出力する問題を切り分けました。修正候補は別のツールコピーでビルドでき、Rust の最小再現例から生成した Lean の型検査も成功しました。既存の検証に使う翻訳器には未統合です。詳細と差分は [backend/chain-binder-probe.md](backend/chain-binder-probe.md) に保存しています。

`cargo elicit` の結果と厳格な axiom 監査は `elicit-integration/` にあります。native Lean の成功と cargo-elicit の採用判定は別に記録しています。

upstream の調査記録は [backend/upstream-2026-10-20/README.md](backend/upstream-2026-10-20/README.md) です。可変参照をジェネリック型へ代入した際の型不一致は既知の制限として open であり、その制限を README に記載した merged PR を実装上の修正と混同していません。

標準モデルの補完では、標準ライブラリ呼び出しの意味を、値や状態を保持する Lean の定義として与える必要があります。バイト列の順序比較なら、先頭から比較し、最初の相違で順序を決め、片方が接頭辞なら短い方を小さくする定義です。比較グループにその具体的な定義と `List.compareLex` との一致証明を追加しましたが、生成された標準スライス比較の公理との接続法則は依然として明示的な前提です。Aeneas の標準モデルとして登録・再抽出するまでは、その公理の実装を証明したとは扱いません。登録後も Rust 標準ライブラリとの対応はモデルの信頼境界であり、そこまで検証するなら Rust 実装の抽出とモデルとの対応証明が別途必要です。

書式化では `Arguments = Unit` のままでは文字列の内容を証明できません。出力バッファと書式指定、引数の表示、書き込みエラーを表すモデルに置き換え、`write_str` や `write_fmt` の状態更新を定義し、その上で bytes の `Debug` / `Display` の出力を証明する必要があります。現行の簡略モデルから期待する表示内容を公理として追加するだけでは、この不足を解消したとは扱いません。
