use dioxus::prelude::*;

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    let mut document = use_signal(|| String::from(""));

    let char_count = use_memo(move || document.read().chars().count());  
    let line_count = use_memo(move || document.read().lines().count());  

    rsx! {
        main {
            style: {
                r#"
                    min-height: 100vh;
                    background: #0f1117;
                    color: #e5e7eb;
                    font-family: 'Inter', system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
                    margin: 0;
                    padding: 24px;
                    box-sizing: border-box;
                "#
            },
            header {
                style: {
                    r#"
                        display: flex;
                        align-items: center;
                        justify-content: space-between;
                        margin-bottom: 16px;
                    "#
                },
                div {
                    h1 { "Dioxus Editor" }
                    p { style: "color: #9ca3af; margin: 4px 0 0;",
                        "Lightweight text editing scaffold"
                    }
                }
                button {
                    style: {
                        r#"
                            background: #2563eb;
                            color: white;
                            border: none;
                            padding: 10px 16px;
                            border-radius: 8px;
                            font-weight: 600;
                            cursor: pointer;
                            box-shadow: 0 10px 25px rgba(37, 99, 235, 0.3);
                        "#
                    },
                    "New Document"
                }
            }
            section {
                style: {
                    r#"
                        display: grid;
                        grid-template-columns: 260px 1fr;
                        gap: 16px;
                        min-height: calc(100vh - 120px);
                    "#
                },
                nav {
                    style: {
                        r#"
                            background: #111827;
                            border: 1px solid #1f2937;
                            border-radius: 12px;
                            padding: 16px;
                            box-shadow: 0 10px 40px rgba(0, 0, 0, 0.25);
                        "#
                    },
                    h2 { style: "margin: 0 0 12px; font-size: 16px;", "Workspace" }
                    ul { style: "list-style: none; padding: 0; margin: 0; display: grid; gap: 8px;",
                        li {
                            style: {
                                r#"
                                    background: #0b1221;
                                    border: 1px solid #1f2937;
                                    border-radius: 10px;
                                    padding: 10px 12px;
                                    display: flex;
                                    align-items: center;
                                    justify-content: space-between;
                                "#
                            },
                            span { "Current draft" }
                            small { style: "color: #9ca3af;", "Unsaved" }
                        }
                        li {
                            style: {
                                r#"
                                    border: 1px dashed #374151;
                                    color: #9ca3af;
                                    border-radius: 10px;
                                    padding: 10px 12px;
                                    text-align: center;
                                "#
                            },
                            "Templates coming soon"
                        }
                    }
                }
                section {
                    style: {
                        r#"
                            background: #0b1221;
                            border: 1px solid #1f2937;
                            border-radius: 12px;
                            padding: 20px;
                            box-shadow: 0 20px 50px rgba(0, 0, 0, 0.35);
                            display: grid;
                            grid-template-rows: auto 1fr auto;
                            gap: 16px;
                        "#
                    },
                    div { style: "display: flex; align-items: center; justify-content: space-between;",
                        h2 { "Editor" }
                        span { style: "color: #9ca3af;", "Autosave disabled" }
                    }
                    textarea {
                        style: {
                            r#"
                                width: 100%;
                                min-height: 320px;
                                background: #0f172a;
                                color: #e5e7eb;
                                border: 1px solid #1f2937;
                                border-radius: 10px;
                                padding: 12px;
                                font-size: 16px;
                                line-height: 1.6;
                                resize: vertical;
                                box-sizing: border-box;
                                outline: none;
                            "#
                        },
                        value: document.read().clone(),
                        placeholder: "Start typing your ideas...",
                        oninput: move |evt| document.set(evt.value()),
                    }
                    section {
                        style: {
                            r#"
                                background: rgba(255, 255, 255, 0.02);
                                border: 1px solid #1f2937;
                                border-radius: 10px;
                                padding: 12px;
                            "#
                        },
                        h3 { style: "margin: 0 0 8px;", "Preview" }
                        p { style: "white-space: pre-wrap; margin: 0; color: #cbd5e1;",
                            "{document.read()}"
                        }
                    }
                    footer {
                        style: {
                            r#"
                                display: flex;
                                align-items: center;
                                gap: 12px;
                                color: #9ca3af;
                                font-size: 14px;
                            "#
                        },
                        span { "Characters: {char_count}" }
                        span { "Lines: {line_count}" }
                    }
                }
            }
        }
    }
}
