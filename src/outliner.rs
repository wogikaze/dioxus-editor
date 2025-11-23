use crate::model::Document;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct OutlinerProps {
    pub initial_text: String,
}

#[component]
pub fn Outliner(props: OutlinerProps) -> Element {
    let document = use_signal(|| Document::from_text(&props.initial_text));

    rsx! {
        div { class: "outliner",
            for (index, line) in document.read().lines.iter().enumerate() {
                div {
                    key: "line-{line.id}",
                    class: "outliner-line",
                    style: format!("margin-left: {}px;", line.indent * 16),
                    span { class: "line-number", "{index + 1}" }
                    span { class: "line-text", "{line.text}" }
                }
            }
        }
    }
}
