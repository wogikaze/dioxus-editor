use dioxus::prelude::*;

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        main {
            class: "app",
            h1 { "Dioxus Web" }
            p { "Ready to build a reusable UI as a web library." }
        }
    }
}
