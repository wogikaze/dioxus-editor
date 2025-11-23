# 04-basic-editing: 行単位の編集と改行・インデント

## ゴール

- B-01〜B-09 の基本編集テスト仕様を満たすこと。
- 行単位のテキスト編集、Enter 改行、Backspace による行結合/デインデント、Tab/Shift+Tab によるインデント操作を実装する。

## 実装方針

- キャレット位置の管理は「(line_index, column_offset)」で管理する。
- キーイベントは「フォーカス中の行」用の `input`/`textarea` にバインドし、ショートカットは `onkeydown` で判定する。
- まずは IME なしのシンプルなケースで実装し、後で E-401〜 の IME 関連を調整する。

## 手順

1. フォーカス中行の管理
   - `Signal<usize>` で `current_line` を保持。
   - 各行 `div` 内に `input` または `contenteditable` 相当の要素を置き、クリック時に `current_line` を更新。

2. テキスト編集（B-01）
   - 各行の `input` の `value` と `oninput` を `Line.text` とバインド。
   - 入力時に `Document` を更新し `Signal` を set。

3. Enter 改行（B-02）
   - `onkeydown` で Enter を検知。
   - 現在行の `text` をカーソル位置で 2 つに分割し、新しい行を現在行の「直後」に挿入。
   - 新しい行の `indent` は元の行と同じ値を引き継ぐ。

4. 行頭 Backspace と行結合（B-03）
   - カーソルが行頭かつ `Backspace` の場合:
     - `indent > 0` のとき: インデントレベルを 1 減らす（B-07）。
     - `indent == 0` かつ前の行が存在する場合: 前行の末尾に現在行の `text` を結合し、現在行を削除。

5. Tab/Shift+Tab によるインデント変更（B-04, B-05, B-06, B-07）
   - Tab: `indent += 1`
   - Shift+Tab: `indent = indent.saturating_sub(1)`
   - 行頭 Space（半角/全角）のときも Tab と同様の動きにする（B-06, B-07）。
   - 「インデント飛び級（B-08）」はあえて禁止せず、任意の数値を許容する。

6. インデント構造の視覚化（B-09）
   - `rsx!` 内で `style: "margin-left: {line.indent * 16}px;"` などを指定し、階層が視覚的に分かるようにする。
   - 後でバレットと折りたたみボタンをここに追加予定。

## Definition of Done

- B-01〜B-09 の手作業テストをブラウザ上で実施し、期待通りの挙動になる。
- 改行、インデント、デインデント、行結合が破綻せずに動く。
