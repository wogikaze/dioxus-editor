# selection: 複数行選択モデルの設計

## ゴール

- 単一キャレットと複数行選択を共通のデータ構造で扱えるようにする。
- B 系 (B-04/B-05/B-06/B-07 など) や M 系の操作が「選択範囲」を意識したロジックで実装できる状態を整える。
- クリップボード/IME/Undo-tree との連携を見越した API を決める。

## 仕様まとめ

- 選択は「アンカー」と「フォーカス」の 2 点で表現する。アンカー <= フォーカス の順序に正規化した `SelectionRange` を常に持つ。
- 各点は `(line_index: usize, column_offset: usize)` で表す。行の境界を跨ぐ場合でも column は「Unicode Scalar 単位」で正規化する。
- 選択が存在しない場合はアンカー=フォーカス に揃え、「キャレットのみ」の状態とみなす。
- 複数行操作 (インデント、削除、複製、移動) は `SelectionRange` から **完全に含まれている行の集合** を先に計算し、同じヘルパーに渡して処理を統一する。
- 折りたたみ中でも選択範囲は Document ベースで保持し、UI で非表示行を飛び越えた場合も同じインデックスを使う。

## データ構造

```rust
#[derive(Clone, PartialEq)]
pub struct CaretPosition {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, PartialEq)]
pub struct SelectionRange {
    pub anchor: CaretPosition,
    pub focus: CaretPosition,
}

impl SelectionRange {
    pub fn normalized(&self) -> (CaretPosition, CaretPosition) { /* anchor <= focus */ }
    pub fn is_collapsed(&self) -> bool { /* anchor == focus */ }
}

```

- `SelectionState` として `Signal<SelectionRange>` を持ち、キーボード/マウスイベントですべて更新する。
- `SelectionState` を `use_context_provider` で配下コンポーネントへ渡し、ハイライトやショートカット判定で共有する。

## 実装ステップ

1. **イベントハンドリング**
   - 行要素 (`input` など) の `onmousedown`, `onmousemove`, `onmouseup` で範囲を更新する。Shift+Click の場合はアンカーを保持し、フォーカスのみ更新する。
   - キー操作 (Shift+Arrow, Ctrl+A, Ctrl+Shift+End など) で Selection を更新できるユーティリティ関数を用意する。

2. **行集合の抽出ヘルパー**

   ```rust
   pub fn selected_line_range(doc: &Document, sel: &SelectionRange) -> Range<usize> {
       let (start, end) = sel.normalized();
       start.line..(end.line + 1)
   }
   ```

   - 行全選択 (Ctrl+A) やドラッグを Document インデックスに変換する共通関数を作る。

3. **編集系との統合**
   - B 系 (インデント/デインデント/行結合) や M 系 (移動/複製) の関数は `SelectionRange` を引数に加え、事前に抽出した行スライスを対象に処理する。
   - Undo-tree へは「操作前後の Document + SelectionRange」をセットで記録し、Undo 後もカーソル位置が復元されるようにする。

4. **アクセシビリティ / IME**
   - IME 合成中は Selection の更新をブロックし、`compositionend` 後に合成結果の長さで `caret` を進める。
   - キーボードフォーカスと Selection がずれた場合に備え、`focus` のみ変更された際も `SelectionRange` を `normalized()` しておく。

5. **UI 表示**
   - `rsx!` で選択範囲をハイライトするため、行ごとに `line_index` を受け取り `SelectionRange` を参照して `selected_start_col`, `selected_end_col` を計算する。
   - 今はプレーン div/input で十分。後続のリッチレンダリングでも同じ Selection API を利用できるよう、描画とは分離する。

## Definition of Done

- SelectionRange を表すデータ構造と更新 API が用意されている。
- Shift+Arrow、Shift+Click、Ctrl+A など基本操作が SelectionState を更新できる。
- B/M/F 系の行編集関数が SelectionRange を前提に設計されている（単一行専用ロジックを排除）。
- Undo/Redo で選択状態が復元される。
