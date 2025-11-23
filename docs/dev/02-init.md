# 02-init: アウトライナーの土台コンポーネント

## ゴール

- 再利用可能な `<Outliner />` コンポーネントの最小スケルトンを作る。
- 現時点では「1 本のテキストエリア」レベルでよいが、Dioxus 的な props/状態管理の形を決める。
- 今後の機能追加をこのコンポーネントに積み上げていける構造を作る。

## 設計ポリシー（初期）

- `Outliner` は「テキスト全体」と「変更を外に通知するコールバック」を props として受け取れるようにする。
- 将来的には、行ごとのツリー構造（ノード列）を props として扱うが、ここではまだプレーンテキストでよい。
- Dioxus 0.7 の Signal を前提に実装する（AGENTS 参照）。

## 手順

1. modules の整理
   - `src/outliner.rs` を新規作成。
   - `src/main.rs` から `mod outliner;` を宣言。

2. Outliner コンポーネントの定義（プレーン版）

   - `src/outliner.rs` に以下のスケルトンを書く。

     ```rust
     use dioxus::prelude::*;

     #[derive(Clone, PartialEq)]
     pub struct OutlinerProps {
         pub initial_text: String,
         pub on_change: EventHandler<String>,
     }

     #[component]
     pub fn Outliner(props: OutlinerProps) -> Element {
         let mut text = use_signal(|| props.initial_text.clone());

         rsx! {
             textarea {
                 value: "{text}",
                 oninput: move |e| {
                     let new_value = e.value();
                     text.set(new_value.clone());
                     props.on_change.call(new_value);
                 }
             }
         }
     }
     ```

3. App から Outliner を呼び出す

   - `src/main.rs` を次のように変更する。

     ```rust
     use dioxus::prelude::*;
     mod outliner;

     fn main() {
         dioxus::launch(App);
     }

     #[component]
     fn App() -> Element {
         let mut value = use_signal(|| String::from("最初のアウトライン\nここから開発を始める"));

         rsx! {
             div {
                 h1 { "Dioxus Outliner" }
                 outliner::Outliner {
                     initial_text: value(),
                     on_change: move |new_value| {
                         value.set(new_value);
                     }
                 }
             }
         }
     }
     ```

4. ビルド・動作確認
   - `dx serve` を実行し、ブラウザで確認。
   - textarea に入力するとリアルタイムに値が更新されることを確認。
   - コンソールにログを出したい場合は、一時的に `log::info!` や `println!` を使って確認してもよい。

## Definition of Done

- `<Outliner />` コンポーネントが存在し、`App` から再利用可能な形で呼び出せる。
- textarea での入力が画面上で反映される。
- 将来、プレーンテキスト → 行配列 → ノード配列への変換をこのコンポーネントに追加できる構造になっている。
