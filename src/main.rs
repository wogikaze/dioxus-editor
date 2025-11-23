// main.rs
use dioxus::prelude::*;

// 1行ぶんのデータ
#[derive(Clone, PartialEq)]
struct OutlineNode {
    id: usize,
    indent: u32,
    text: String,
    collapsed: bool,
}

impl OutlineNode {
    fn new(id: usize, indent: u32, text: impl Into<String>) -> Self {
        Self {
            id,
            indent,
            text: text.into(),
            collapsed: false,
        }
    }
}

fn main() {
    // dioxus-cli などを使う想定
    // 例: dioxus create, dioxus serve でこの main.rs をエントリにする
    launch(App);
}

#[component]
fn App() -> Element {
    // 行データ
    let mut nodes: Signal<Vec<OutlineNode>> = use_signal(|| {
        vec![
            OutlineNode::new(0, 0, "アウトライナーへようこそ"),
            OutlineNode::new(1, 1, "Enter: 下に行追加"),
            OutlineNode::new(2, 1, "Tab: インデント"),
            OutlineNode::new(3, 1, "Backspace (空行): 行と子を削除"),
        ]
    });

    // 現在フォーカス中の行 index（非表示行も含む index ベース）
    let mut focused_index: Signal<usize> = use_signal(|| 0);

    // 一意な id を振るためのカウンタ
    let mut next_id: Signal<usize> = use_signal(|| 4);

    // スナップショットを取ってから rsx! に渡す
    let snapshot = nodes.read().clone();

    rsx! {
        div {
            class: "outliner-root",
            style: "max-width: 800px; margin: 24px auto; font-family: system-ui, -apple-system, BlinkMacSystemFont, sans-serif;",

            h1 {
                style: "font-size: 20px; margin-bottom: 12px;",
                "Dioxus Outliner (single-file demo)"
            }

            // エディタ本体
            div {
                style: "border: 1px solid #ccc; border-radius: 8px; padding: 4px 0; background: #fafafa;",

                // snapshot 上の全インデックスを回す
                for (idx, node) in snapshot.iter().enumerate() {
                    if is_visible(&snapshot, idx) {
                        OutlineRow {
                            key: "{node.id}",
                            id: node.id,
                            index: idx,
                            indent: node.indent,
                            has_children: has_children(&snapshot, idx),
                            collapsed: node.collapsed,
                            text: node.text.clone(),
                            // Signals
                            nodes,
                            focused_index,
                            next_id,
                        }
                    }
                }
            }

            // フッター / コントロール
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-top: 8px; font-size: 12px; color: #555;",
                span {
                    "Enter: 下に行追加 / Tab: インデント / Backspace (空行): 行と子を削除"
                }
                button {
                    style: "padding: 4px 8px; border-radius: 4px; border: 1px solid #ccc; background: #fff; cursor: pointer;",
                    onclick: move |_| {
                        // 末尾に新しいルート行を追加してフォーカス
                        let new_index = {
                            // id を採番
                            let new_id = {
                                let mut id_guard = next_id.write();
                                let v = *id_guard;
                                *id_guard += 1;
                                v
                            };
                            let mut vec = nodes.write();
                            let idx = vec.len();
                            vec.push(OutlineNode::new(new_id, 0, ""));
                            idx
                        };
                        focused_index.set(new_index);
                    },
                    "+ 行を追加"
                }
            }
        }
    }
}

// 1行ぶんの UI
#[component]
fn OutlineRow(
    id: usize,
    index: usize,
    indent: u32,
    has_children: bool,
    collapsed: bool,
    text: String,
    nodes: Signal<Vec<OutlineNode>>,
    focused_index: Signal<usize>,
    next_id: Signal<usize>,
) -> Element {
    let is_focused = *focused_index.read() == index;

    // インデント幅
    let indent_px = indent * 18;
    let row_bg = if is_focused { "#e6efff" } else { "transparent" };

    rsx! {
        div {
            key: "{id}",
            class: "outline-row",
            style: "display: flex; align-items: center; padding: 2px 8px; background: {row_bg};",

            // 行全体クリックでフォーカス
            onclick: move |_| {
                focused_index.set(index);
            },

            // 折りたたみトグル
            div {
                style: "width: 20px; text-align: center; cursor: pointer; user-select: none;",
                if has_children {
                    span {
                        onclick: move |evt| {
                            // インプットへのフォーカス移動を防ぐ
                            evt.stop_propagation();
                            let mut vec = nodes.write();
                            if let Some(node) = vec.get_mut(index) {
                                node.collapsed = !node.collapsed;
                            }
                        },
                        if collapsed { "▶" } else { "▼" }
                    }
                } else {
                    span { "" }
                }
            }

            // インデント用スペーサ
            div { style: "width: {indent_px}px;" }

            // テキスト入力
            input {
                r#type: "text",
                value: "{text}",
                autofocus: is_focused,
                style: "flex: 1; border: none; background: transparent; outline: none; padding: 2px 4px; font-size: 14px;",

                onfocus: move |_| {
                    focused_index.set(index);
                },

                oninput: move |evt| {
                    let new_text = evt.value();
                    let mut vec = nodes.write();
                    if let Some(node) = vec.get_mut(index) {
                        node.text = new_text;
                    }
                },

                onkeydown: move |evt| {
                    use dioxus::prelude::Key;

                    match evt.key() {
                        // Enter: 下に新しい行を追加
                        Key::Enter => {
                            evt.prevent_default();
                            let new_idx = {
                                let mut vec = nodes.write();

                                // 新しい id
                                let new_id = {
                                    let mut id_guard = next_id.write();
                                    let v = *id_guard;
                                    *id_guard += 1;
                                    v
                                };

                                insert_after(&mut vec, index, new_id)
                            };
                            focused_index.set(new_idx);
                        }

                        // Tab: インデント (行とその子をまとめて)
                        Key::Tab => {
                            evt.prevent_default();
                            let mut vec = nodes.write();
                            indent_subtree(&mut vec, index, 1);
                        }

                        // Backspace: 空行なら削除 (行＋子孫)
                        Key::Backspace => {
                            let new_focus = {
                                let mut vec = nodes.write();
                                handle_backspace_on_empty(&mut vec, index)
                            };
                            if let Some(idx) = new_focus {
                                focused_index.set(idx);
                            }
                        }

                        _ => {}
                    }
                }
            }
        }
    }
}

// 折りたたみ状態を考慮して、この index の行を表示するか
fn is_visible(nodes: &[OutlineNode], index: usize) -> bool {
    let indent = nodes[index].indent;
    if indent == 0 {
        return true;
    }

    let mut i = index;
    while i > 0 {
        i -= 1;
        if nodes[i].indent < indent {
            // 直近の祖先
            return !nodes[i].collapsed;
        }
    }

    true
}

// この行が子を持っているか
fn has_children(nodes: &[OutlineNode], index: usize) -> bool {
    let indent = nodes[index].indent;
    let i = index + 1;
    while i < nodes.len() {
        let ind = nodes[i].indent;
        if ind <= indent {
            break;
        }
        // 最初に indent が大きい行が来た時点で子がいる
        return true;
        // 到達しないが、明示的に増やしておく
        // i += 1;
    }
    false
}

// index の直後に新しい行を挿入して、その index を返す
fn insert_after(nodes: &mut Vec<OutlineNode>, index: usize, new_id: usize) -> usize {
    let indent = nodes
        .get(index)
        .map(|n| n.indent)
        .unwrap_or(0);

    let insert_pos = (index + 1).min(nodes.len());
    nodes.insert(insert_pos, OutlineNode::new(new_id, indent, ""));
    insert_pos
}

// 行とその「サブツリー」をまとめてインデント/アウトデント
// delta > 0 でインデント、delta < 0 でアウトデント
fn indent_subtree(nodes: &mut Vec<OutlineNode>, index: usize, delta: i32) {
    if index >= nodes.len() {
        return;
    }

    let base_indent = nodes[index].indent as i32;
    let new_indent = (base_indent + delta).max(0) as u32;

    // ルート行をこれ以上アウトデントしない
    if base_indent == 0 && delta < 0 {
        return;
    }

    nodes[index].indent = new_indent;

    // 子孫もまとめてずらす
    let mut i = index + 1;
    while i < nodes.len() {
        let ind = nodes[i].indent as i32;
        if ind <= base_indent {
            break;
        }
        let updated = (ind + delta).max(0) as u32;
        nodes[i].indent = updated;
        i += 1;
    }
}

// Backspace を押したときに、行が空なら削除を行う
// 削除した場合は新しいフォーカス先 index を返す
fn handle_backspace_on_empty(nodes: &mut Vec<OutlineNode>, index: usize) -> Option<usize> {
    if index >= nodes.len() {
        return None;
    }

    if !nodes[index].text.is_empty() {
        // 通常の文字削除に任せる
        return None;
    }

    if nodes.len() == 1 {
        // 最後の 1 行は消さない（必要ならここも消す仕様にしてよい）
        return Some(0);
    }

    let current_indent = nodes[index].indent;

    // この行と、その子孫をまとめて削除
    let start = index;
    let mut end = index + 1;
    while end < nodes.len() && nodes[end].indent > current_indent {
        end += 1;
    }

    nodes.drain(start..end);

    if start == 0 {
        Some(0)
    } else {
        Some(start - 1)
    }
}
