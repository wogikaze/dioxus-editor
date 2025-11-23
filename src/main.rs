use dioxus::prelude::*;
mod model;
mod outliner;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let initial_text = "アウトライン\n 子トピック\n\t更に深いトピック\n　全角スペース始まり";

    rsx! {
        div {
            h1 { "Dioxus Outliner" }
            outliner::Outliner {
                initial_text: initial_text.to_string(),
            }
        }
    }
}
