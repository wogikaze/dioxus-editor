use dioxus::prelude::*;
use uuid::Uuid;

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    let mut document = use_signal(Document::example);

    let visible_nodes = use_memo(move || document.read().visible_nodes());
    let stats = use_memo(move || document.read().stats());
    let stats_value = stats();
    let visible_nodes_value = visible_nodes();
    rsx! {
        document::Stylesheet { href: asset!("/assets/style.css") }
        main { class: "surface",
            header { class: "header",
                div {
                    h1 { "Dioxus Outliner" }
                    p { class: "subhead", "Hierarchical notes with collapsible rows" }
                }
                div { class: "header-actions",
                    button { class: "ghost", "Import" }
                    button { class: "primary", onclick: move |_| document.write().append_root_line(), "New top-level item" }
                }
            }
            section { class: "layout",
                nav { class: "panel",
                    h2 { "Workspace" }
                    p { class: "muted", "Reusable outline primitives" }
                    ul { class: "legend",
                        li { span { class: "dot", style: "background: #60a5fa;" } span { "Editable nodes" } }
                        li { span { class: "dot", style: "background: #f59e0b;" } span { "Collapsible branches" } }
                        li { span { class: "dot", style: "background: #34d399;" } span { "Visible list derived from tree" } }
                    }
                    section { class: "stats",
                        div { "Lines" span { "{stats_value.line_count}" } }
                        div { "Characters" span { "{stats_value.char_count}" } }
                        div { "Collapsed" span { "{stats_value.collapsed_branches}" } }
                    }
                    p { class: "muted", "This step wires the core document model to a visible list and folding controls." }
                }
                section { class: "panel editor",
                    div { class: "panel-head",
                        h2 { "Outline" }
                        span { class: "muted", "Derived rows: {visible_nodes_value.len()}" }
                    }
                    OutlineEditor { document: document.clone(), visible_nodes: visible_nodes_value.clone() }
                    footer { class: "footer",
                        div { class: "pill", "Undo-tree & selection logic stubbed" }
                        div { class: "pill", "Next: keyboard bindings and history" }
                    }
                }
            }
        }
    }
}

#[component]
fn OutlineEditor(document: Signal<Document>, visible_nodes: Vec<VisibleNode>) -> Element {
    rsx! {
        section { class: "outline",
            for node in visible_nodes.into_iter() {
                OutlineRow { node, document: document.clone() }
            }
        }
    }
}

#[component]
fn OutlineRow(node: VisibleNode, mut document: Signal<Document>) -> Element {
    let padding = format!("calc(12px + {} * 20px)", node.indent);
    let toggle_icon = if node.has_children {
        if node.collapsed { "▶" } else { "▼" }
    } else {
        "•"
    };
    let toggle_title = if node.has_children {
        if node.collapsed {
            "Expand children"
        } else {
            "Collapse children"
        }
    } else {
        "Leaf node"
    };

    rsx! {
        div { class: "row",
            div { class: "row-left", style: format!("padding-left: {padding};"),
                button {
                    class: if node.has_children { "toggle" } else { "toggle disabled" },
                    disabled: !node.has_children,
                    title: toggle_title,
                    onclick: move |_| document.write().toggle_collapse(node.id),
                    "{toggle_icon}"
                }
                div { class: "bullet" }
            }
            textarea {
                class: "editor-input",
                value: node.text.clone(),
                oninput: move |evt| document.write().update_text(node.id, evt.value()),
            }
            button {
                class: "ghost",
                onclick: move |_| document.write().insert_after(node.id),
                "+"
            }
        }
    }
}

#[derive(Clone, Debug)]
struct Node {
    id: Uuid,
    indent: usize,
    text: String,
    collapsed: bool,
}

impl Node {
    fn new(indent: usize, text: impl Into<String>, collapsed: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            indent,
            text: text.into(),
            collapsed,
        }
    }
}

#[derive(Clone, Default, Debug)]
struct Document {
    nodes: Vec<Node>,
}

impl Document {
    fn example() -> Self {
        let mut nodes = vec![
            Node::new(0, "Roadmap", false),
            Node::new(1, "Foundation: core model + visible rows", false),
            Node::new(2, "Implement selection and caret stubs", false),
            Node::new(2, "Undo tree data model placeholder", false),
            Node::new(1, "Structure operations", false),
            Node::new(2, "Indent / dedent nodes", false),
            Node::new(2, "Move branches while preserving children", true),
            Node::new(3, "Alt+Up/Down should respect collapsed state", false),
            Node::new(1, "Rendering & notation", false),
            Node::new(2, "Inline styles, tags, and links", false),
            Node::new(2, "Blocks: code, table, LaTeX", false),
            Node::new(1, "Input helpers & search", false),
            Node::new(2, "Bracket completion and IME guards", false),
            Node::new(2, "Command palette & shortcuts", false),
            Node::new(1, "Performance & embedding", false),
            Node::new(2, "Virtualized list for 5k+ rows", false),
            Node::new(2, "Expose embeddable editor component", false),
        ];

        nodes.insert(6, Node::new(3, "Leaf after deep jump", false));
        Self { nodes }
    }

    fn visible_nodes(&self) -> Vec<VisibleNode> {
        let mut rows = Vec::with_capacity(self.nodes.len());
        let mut hidden_below: Option<usize> = None;

        for (index, node) in self.nodes.iter().enumerate() {
            if let Some(depth) = hidden_below {
                if node.indent > depth {
                    continue;
                }

                hidden_below = None;
            }

            let next_indent = self.nodes.get(index + 1).map(|n| n.indent);
            let has_children = next_indent.is_some_and(|n| n > node.indent);

            rows.push(VisibleNode {
                id: node.id,
                indent: node.indent,
                text: node.text.clone(),
                collapsed: node.collapsed,
                has_children,
            });

            if node.collapsed {
                hidden_below = Some(node.indent);
            }
        }

        rows
    }

    fn toggle_collapse(&mut self, id: Uuid) {
        if let Some(node) = self.nodes.iter_mut().find(|node| node.id == id) {
            node.collapsed = !node.collapsed;
        }
    }

    fn update_text(&mut self, id: Uuid, new_text: String) {
        if let Some(node) = self.nodes.iter_mut().find(|node| node.id == id) {
            node.text = new_text;
        }
    }

    fn insert_after(&mut self, id: Uuid) {
        if let Some(position) = self.nodes.iter().position(|node| node.id == id) {
            let indent = self.nodes[position].indent;
            self.nodes
                .insert(position + 1, Node::new(indent, "New line", false));
        }
    }

    fn append_root_line(&mut self) {
        self.nodes.push(Node::new(0, "New root item", false));
    }

    fn stats(&self) -> DocumentStats {
        let mut char_count = 0;
        let mut collapsed_branches = 0;

        for node in &self.nodes {
            char_count += node.text.chars().count();
            if node.collapsed {
                collapsed_branches += 1;
            }
        }

        DocumentStats {
            line_count: self.nodes.len(),
            char_count,
            collapsed_branches,
        }
    }
}

#[derive(Clone, PartialEq)]
struct DocumentStats {
    line_count: usize,
    char_count: usize,
    collapsed_branches: usize,
}

#[derive(Clone, PartialEq)]
struct VisibleNode {
    id: Uuid,
    indent: usize,
    text: String,
    collapsed: bool,
    has_children: bool,
}
