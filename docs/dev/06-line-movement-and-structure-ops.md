# 06-line-movement-and-structure-ops: 行の移動・複製・階層操作

## ゴール

- 「行の入れ替え」「子要素を含めた移動」「行複製」など、M-01〜M-05 を実装する。
- Alt+Up/Down で親+子をまとめて移動する挙動を安定させる。

## 実装方針

- `Document.lines` 上で、「1 行だけ」「サブツリー単位」など複数パターンのスライス操作をしやすいヘルパ関数を用意する。
- 「サブツリー」とは「ある行から始まり、後続の行のうち indent がそれより大きいものをすべて含むブロック」。

## 手順

1. サブツリー範囲検出関数

   ```rust
   fn subtree_range(lines: &[Line], start_index: usize) -> std::ops::Range<usize> {
       let base_indent = lines[start_index].indent;
       let mut end = start_index + 1;
       while end < lines.len() && lines[end].indent > base_indent {
           end += 1;
       }
       start_index..end
   }
   ```

2. 行の単純な上下移動 (M-01)

   - Ctrl+Up/Down: 現在行とその一つ上/下の行を swap する。
   - サブツリーは考慮せず「1 行だけ」の移動。

3. サブツリーの上下移動 (M-02)

   - Alt+Up/Down: subtree_range を使って「親+子」を丸ごと取り出し、兄弟ブロック間で挿入する。
   - 移動先の indent は変えない。

4. インデントレベルの移動 (M-03)

   - Ctrl+Left/Right: 対象行（または選択範囲）の indent を ±1 変更する。
   - B 系のインデント操作と同じロジックを共通化する。

5. 行複製 (M-04, M-05)

   - Shift+Alt+Up/Down: 現在行（またはサブツリー）を上下に複製する。
   - Shift+Enter: 現行とは別に、indent=0 の新しい行を末尾か現在行の直下に挿入する（仕様を固定）。

## Definition of Done

- M-01〜M-05 のテストをブラウザ上で実施し、期待通りの順序・階層が維持される。
- 折りたたみ状態に関わらず、データ構造が破綻しない。
