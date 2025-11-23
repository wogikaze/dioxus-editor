# テスト自動化方針書

## 目的とゴール

包括的なテスト仕様の自動化方針を明文化し、Rust 側のモデル・操作ロジックに対するユニットテストを優先実装する。

## テストレベルと対象

ユニットテスト（Rust）: インデント、行移動、折りたたみ、Undo Tree など純粋関数化したモデル操作を cargo test で検証。

統合テスト（将来拡張）: モデル層とストア/イベントハンドラの連携を少数のハッピーパスで確認。

E2E テスト（候補メモ）: Dioxus UI は Playwright 等での自動化候補を列挙するが、当面は実装しない方針。

## テストケース設計方針

仕様書の B-01〜B-09, M-01〜M-05, F-01〜F-07 をユニットテスト化し、「入力 Document → 操作 → 出力 Document」を期待通りに検証する。

Undo Tree は E-601〜E-603 パターンをユニットテストで再現し、履歴の分岐・復元を確認。

テスト名やコメントに仕様 ID を含め、追跡可能性を確保する。

## 優先度とマイルストーン

Milestone 1: モデル操作 API の切り出しと B/M/F 系ユニットテストの実装完了（最小セット）。

Milestone 2: Undo Tree テストの追加とリファクタリング（不変モデルの維持を確認）。

Milestone 3（任意）: CI（GitHub Actions 等）での cargo test 実行設定とレポート収集。

## 実装メモ

model モジュールに indent_line(doc, index, delta), move_line_up(doc, index), toggle_collapse(doc, index), subtree_range(lines, index) などを切り出し、純粋関数としてテスト可能にする。

失敗時の差分が読めるよう、入力・期待・実出力を構造体化するヘルパを用意。

## Definition of Done

cargo test でモデルレベルのテストが複数通り、仕様書 ID との紐付けが明確になっている状態。

上記をそのまま docs/test/ 直下の新規ファイル（例: test-plan.md）として配置することを想定しています。
