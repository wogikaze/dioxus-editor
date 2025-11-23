# 09-undo-tree-and-history: Undo Tree 履歴管理

## ゴール

- Undo/Redo を「単純スタック」ではなく「Undo Tree」として実装する。
- A-02, E-601〜E-603 のテストを通す。

## 実装方針

- `Document` のスナップショットをノードにもつ「履歴ツリー」を用意する。
- Undo → ひとつ前のスナップショットへ移動。
- Undo 後に新しい編集をすると「別ブランチ」が生える形にする（最低限、データ構造として保持）。

## 手順

1. 履歴ノード構造の定義

   ```rust
   pub struct HistoryNode {
       pub id: u64,
       pub document: Document,
       pub parent: Option<u64>,
       pub children: Vec<u64>,
   }

   pub struct History {
       pub nodes: HashMap<u64, HistoryNode>,
       pub current_id: u64,
   }
   ```

2. 編集操作時のスナップショット保存

   - すべての「ユーザー操作」（入力・インデント・移動など）の後に、新しい `HistoryNode` を作成して `History` に追加。
   - 過去に戻っている状態で新編集が発生した場合、現在ノードの `children` に新しいノードをぶら下げる。

3. Undo/Redo 操作

   - `Ctrl+Z`: `current_id` を `parent` に移動。
   - `Ctrl+Shift+Z`: `children` のうち「最新ブランチ」を選んで移動（UI なしなら単純に最後の子を選ぶ）。

4. Document と履歴の同期

   - `Outliner` のメイン状態を `History` にし、`current_document()` のような helper で現在の `Document` を参照する。

5. テスト観点

   - E-601: "A" → Undo → "B" と入力し、履歴上 "A" の状態が別ブランチとして残っていること。
   - E-603: 全選択削除後の Undo で階層構造が完全に戻ること。

## Definition of Done

- `Ctrl+Z` / `Ctrl+Shift+Z` で直前操作の取り消し・やり直しが行える。
- 大きな編集（全削除など）を Undo しても割れずに元の構造に戻る。
