# 03-model-and-lines: 行モデルとツリー構造の導入

## ゴール

- 仕様書の「インデント構造」「行の入れ替え」「折りたたみ」の前提になるデータ構造を定義する。
- プレーンテキストを「行の配列」にパースし、インデントレベルを管理できるようにする。
- レンダリングはまだ「単純な行リスト」でよい（バレットや折りたたみ UI は後回し）。

## 抽象モデルの方針

- `LineId` を持つ構造体 `Line` を定義する:

  ```rust
  pub type LineId = u64;

  #[derive(Clone, PartialEq)]
  pub struct Line {
      pub id: LineId,
      pub indent: u32,
      pub text: String,
      pub collapsed: bool,
  }
  ```

- ツリー構造は「インデントレベルに基づく親子関係」で表現し、最初は「フラットな Vec<Line>」として扱う。
- ノード入れ替えや折りたたみは「この Vec<Line> 上で、インデントルールに従った並び替え」として実装できるようにする。

## 手順

1. `src/model.rs` を作成

   - `Line` 構造体と `Document` 構造体を定義。

     ```rust
     #[derive(Clone, PartialEq)]
     pub struct Document {
         pub lines: Vec<Line>,
     }
     ```

2. テキスト ⇔ Document 変換関数を定義

    - `String` → `Document`:

       - `\n` で split。
       - 行頭インデントは **タブ、半角スペース、全角スペースのいずれも 1 文字 = 1 インデントレベル** としてカウントする。混在しても文字単位でしきい値を足し込み、結果を `indent` に記録する。
       - インデント文字列を取り除いた残りを `text` とする。コードブロックなどで先頭スペース自体を保持したい場合は、仕様として `text` 側に明示スペースを残すか、ブロック記法で包む。
       - どのプラットフォームでも同じ `Document` が得られるよう、上記ルールは必ず共有ユーティリティで実装する。
    - `Document` → `String`:

       - 各行を `indent` の値だけタブを出力し（1 文字 = 1 レベル）、その後に `text` を連結して join する。全角スペースで始めたい場合は `text` に含めておく。

3. Outliner 内部状態を `Document` ベースに変更

   - `Outliner` コンポーネントの内部状態を `Signal<Document>` に変更。
   - 表示は一旦 `textarea` ではなく `div` + `input` の組み合わせにしてもよいが、初期段階では `textarea` とモデル変換をラップしてもよい。

4. シンプルな行表示 UI

   - `rsx!` 内で `for (index, line) in document.lines.iter().enumerate()` を回し、各行を `div` で表示。
   - この時点では「表示のみ」。編集は 04 以降に実装する。

5. B-01〜B-02 のテスト観点を意識した確認

   - まだショートカットは実装しないが、「内部モデルとして行の概念がある」状態を用意する。
   - 手動で `Document` を作って表示させ、インデントレベルに応じて左余白（`margin-left`）を変えて視覚的に階層が見えることを確認。

## Definition of Done

- `Document` と `Line` 構造体が定義されている。
- `Outliner` は内部的に `Signal<Document>` を持ち、UI に各行を表示できている。
- インデントレベルに応じて視覚的な階層が確認できる。
