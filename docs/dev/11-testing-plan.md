# 11-testing-plan: 自動テスト戦略

## ゴール

- 包括的テスト仕様書を自動化するための方針を決める。
- 最低限のユニットテスト（モデル・操作ロジック）を Rust 側で書き始める。

## 方針

- モデル操作（インデント・行移動・折りたたみ・Undo Tree）は純粋関数に近い形で切り出し、`cargo test` でユニットテストを書く。
- Dioxus の UI 部分は、最初は E2E テスト（Playwright 等）の候補をメモに留めるに留める。

## 手順

1. モデル操作 API の切り出し
   - `model` モジュールに以下のような関数を定義:
     - `indent_line(doc, index, delta)`
     - `move_line_up(doc, index)`
     - `toggle_collapse(doc, index)`
     - `subtree_range(lines, index)` etc.

2. B/M/F 系テストケースのユニットテスト化
   - 仕様書の B-01〜B-09, M-01〜M-05, F-01〜F-07 の期待結果を参考に、入力 Document → 操作 → 出力 Document を検証するテストを書く。

3. Undo Tree のテスト
   - E-601〜E-603 のパターンをユニットテストで再現。

4. CI への組み込み（任意）
   - `cargo test` を GitHub Actions などから実行できるように設定（このステップでは TODO コメントでもよい）。

## Definition of Done

- `cargo test` でモデルレベルのテストが複数通る状態になっている。
- テスト仕様書との紐付けがコメントやテスト名で分かるようになっている。
