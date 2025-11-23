# 01-setup: プロジェクト環境と最小 Dioxus アプリ

## ゴール

- Dioxus 0.7（web ターゲット）でブラウザ表示できる最小アプリを作る。
- `dx serve` でローカル開発ができる状態にする。
- 今後のアウトライナー実装を `src/main.rs` で進められる状態にする。

## 前提

- Rust と Cargo がインストール済み。
- `curl` が使える（dx インストール用）。

## 手順

1. プロジェクト作成
   - `cargo new dioxus-outliner --bin` を実行。
   - `cd dioxus-outliner` に移動。

2. Cargo.toml の編集
   - `[dependencies]` に Dioxus を追加する。

     ```toml
     [dependencies]
     dioxus = { version = "0.7", features = ["web"] }
     ```

   - dev 用に `dioxus-cli` はグローバルインストールする想定とし、ここでは依存に入れない。

3. Dioxus の最小アプリを作成
   - `src/main.rs` を以下の内容に書き換える。

     ```rust
     use dioxus::prelude::*;

     fn main() {
         dioxus::launch(App);
     }

     #[component]
     fn App() -> Element {
         rsx! {
             "Hello, Outliner!"
         }
     }
     ```

4. dx CLI のインストール
   - `curl -sSL http://dioxus.dev/install.sh | sh` を実行して `dx` を導入。
   - プロジェクト直下で `dx --version` が動くことを確認。

5. ローカルサーバでの動作確認
   - `dx serve` を実行。
   - ブラウザで `http://localhost:8080` を開き、「Hello, Outliner!」が表示されることを確認。

## Definition of Done

- `dx serve` でブラウザにアクセスすると "Hello, Outliner!" と表示される。
- コンパイルエラーが発生していない。
- Cargo.toml に Dioxus 0.7 (web feature) が設定されている。
